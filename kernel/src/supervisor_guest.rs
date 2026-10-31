//! Single-CPU adapter. Dynamic tasks are prepared on the caller's kernel stack,
//! but publication/entry and all destruction occur at quiescent scheduler points.
use super::*;
use continuum_kernel::{
    objects::channels::Sender,
    supervisor::{self as policy, Exit, State, Tree},
};
static TREE: Local<Tree> = Local(UnsafeCell::new(Tree::EMPTY));
static TOKENS: Local<[u64; N]> = Local(UnsafeCell::new([0; N]));
static CALL_COUNTS: Local<[u64; N]> = Local(UnsafeCell::new([0; N]));
fn tree<R>(f: impl FnOnce(&mut Tree) -> R) -> R {
    f(unsafe { &mut *TREE.0.get() })
}
fn token() -> u64 {
    unsafe { (*TOKENS.0.get())[*INDEX.0.get()] }
}
fn status(e: policy::Error) -> u64 {
    use policy::Error::*;
    match e {
        Stale => STALEEPOCH,
        Denied => ACCESSDENIED,
        Quota | Quarantined => QUOTAEXCEEDED,
        State => INVALIDARGUMENT,
        WouldBlock | Backoff => WOULDBLOCK,
    }
}
fn trace(action: &str, token: u64, parent: u64, code: u64, vector: u64) {
    record(
        "m1.supervisor",
        "supervisor",
        "ok",
        format_args!(
            "action={action} token={token} parent={parent} code={code} vector={vector} closed={}",
            tree(|t| t.closed())
        ),
    );
}
fn prepare(module: usize, mode: u64, context: Context) -> Result<usize> {
    prepare_task(module, mode, context, true)
}
pub(super) fn prepare_task(
    module: usize,
    mode: u64,
    context: Context,
    bounded: bool,
) -> Result<usize> {
    let slot = unsafe { (&*TASKS.0.get()).iter().position(Option::is_none) }
        .ok_or(error("supervisor", "quota"))?;
    let id = runtime_memory::with(|r, b| unsafe {
        (&mut *LAUNCHER.0.get()).launch_context(
            r,
            b,
            TrustedImage {
                bytes: crate::paging::module(module)?,
                digest: IMAGES[module].0,
                size: IMAGES[module].1,
            },
            context,
            EXIT_TRAMPOLINE,
            true,
        )
    })?;
    let (space, entry, sp) = unsafe {
        let p = (&*LAUNCHER.0.get()).get(id)?;
        (p.space, p.entry, p.stack)
    };
    let mut admitted = false;
    let result = (|| {
        let grants = capabilities::registry(|c| {
            let grants = c.admit_sender(
                space,
                Sender {
                    process: id.slot as u64,
                    generation: id.generation,
                    principal: context.principal,
                    session: context.session,
                    service: context.service,
                },
            )?;
            admitted = true;
            if bounded {
                c.constrain(space)?;
            }
            Ok::<_, continuum_kernel::objects::Error>(grants)
        })
        .map_err(|_| error("supervisor", "objects"))?;
        let (root, stack) = runtime_memory::with(|r, b| {
            let a = r.allocate(b, space, PAGE, PAGE)?.unwrap();
            for (i, v) in [mode, 0, module as u64 + 1, grants.0, grants.1, 0, 0, 0]
                .into_iter()
                .enumerate()
            {
                r.write_staging(b, space, a, i as u64 * 8, &v.to_le_bytes())?;
            }
            r.map(b, space, a, 0x70000000, 0, PAGE, Access::Write)?;
            Ok::<_, continuum_kernel::Error>((r.root(space)?.0, r.kernel_stack(space)?.end()))
        })?;
        Ok((root, stack))
    })();
    let (root, stack) = match result {
        Ok(v) => v,
        Err(e) => {
            if admitted {
                capabilities::remove(space)?;
            }
            runtime_memory::with(|r, b| unsafe { (&mut *LAUNCHER.0.get()).destroy(r, b, id) })?;
            return Err(e);
        }
    };
    let frame = Frame {
        r15: 0,
        r14: 0,
        r13: 0,
        r12: 0,
        r11: 0,
        r10: 0,
        r9: 0,
        r8: 0,
        rdi: STARTUP,
        rsi: 0,
        rbp: 0,
        rbx: 0,
        rdx: 0,
        rcx: 0,
        rax: 0,
        ip: entry,
        cs: USER_CS,
        flags: 0x202,
        sp,
        ss: USER_SS,
    };
    unsafe {
        (&mut *TASKS.0.get())[slot] = Some(Task {
            current: Current {
                id,
                space,
                context,
                case: mode,
            },
            root,
            stack,
            frame,
            fp: FxState::fresh(),
            heap: [None; 32],
            dispatches: 0,
            preemptions: 0,
            progress: 0,
            ready_ns: 0,
            elapsed_ns: 0,
            runnable_ns: 0,
            result: None,
            vector: 0,
        });
        (*CALL_COUNTS.0.get())[slot] = 0;
    }
    Ok(slot)
}
fn publish(slot: usize, token: u64) -> Result<()> {
    // A scalar entry argument, never a pointer or privileged return register.
    unsafe {
        (*TOKENS.0.get())[slot] = token;
        (&mut *TASKS.0.get())[slot].as_mut().unwrap().frame.rsi = token;
        (&mut *TASKS.0.get())[slot].as_mut().unwrap().frame.rdx =
            tree(|t| t.parent(token)).unwrap();
    }
    let t = unsafe { (&*TASKS.0.get())[slot].unwrap() };
    record(
        "m1.supervisor_admit",
        "supervisor",
        "ok",
        format_args!(
            "token={token} parent={} process={} generation={} thread={slot} principal={} authority_session={} service={} grants={} memory={} mode={} private_handles=true",
            tree(|t| t.parent(token)).unwrap(),
            t.current.id.slot,
            t.current.id.generation,
            t.current.context.principal,
            t.current.context.session,
            t.current.context.service,
            t.current.context.grant_ceiling,
            t.current.context.commitment_ceiling,
            t.current.case
        ),
    );
    Ok(())
}
/// Registered profile observer: no return IP, selector or privileged flag changes.
pub(crate) fn observe_state(target: u64, state: u64) -> u64 {
    if state != 1 {
        return state;
    }
    let slot = unsafe { (&*TOKENS.0.get()).iter().position(|t| *t == target) };
    if let Some(slot) = slot {
        if let Some(task) = unsafe { (&*TASKS.0.get())[slot] } {
            if waiting::blocked(ThreadId {
                slot,
                generation: task.current.id.generation,
            }) {
                return 4;
            }
        }
    }
    state
}
pub(crate) fn dispatch(op: u64, f: &Frame) -> Option<core::result::Result<u64, u64>> {
    let caller = token();
    unsafe {
        let count = &mut (*CALL_COUNTS.0.get())[*INDEX.0.get()];
        *count += 1;
        if *count > 2048 {
            finish("call-budget", 0x301, 0);
        }
    }
    if op == DIAGNOSTIC_LOG {
        return tree(|t| t.log(caller, now())).err().map(|e| Err(status(e)));
    }
    if op == MEMORY_ALLOCATE {
        return Some(if f.rdi > 65536 {
            Err(QUOTAEXCEEDED)
        } else {
            allocate(f.rdi, f.rsi)
        });
    }
    if op == MEMORY_FREE {
        return Some(free(f.rdi));
    }
    if !(27..=33).contains(&op) {
        return None;
    }
    let result = (|| {
        match op {
            27 => {
                if f.rsi != 96 {
                    return Err(INVALIDARGUMENT);
                }
                let mut bytes = [0u8; 96];
                copy(f.rdi, &mut bytes, false)?;
                let w = core::array::from_fn::<_, 12, _>(|i| {
                    u64::from_le_bytes(bytes[i * 8..i * 8 + 8].try_into().unwrap())
                });
                if !(1..=2).contains(&w[0]) || w[11] != 0 {
                    return Err(INVALIDARGUMENT);
                }
                let context = Context {
                    principal: w[2],
                    session: w[3],
                    service: w[4] != 0,
                    grant_ceiling: w[5],
                    commitment_ceiling: w[6],
                };
                record(
                    "m1.supervisor_request",
                    "supervisor",
                    "ok",
                    format_args!(
                        "caller={caller} image={} mode={} principal={} authority_session={} service={} grants={} memory={} units={} threads={} priority={} inherited={}",
                        w[0], w[1], w[2], w[3], w[4], w[5], w[6], w[7], w[8], w[9], w[10]
                    ),
                );
                tree(|t| t.check_child(caller, context, w[7], w[8], w[9], w[10]))
                    .map_err(status)?;
                let slot = prepare(w[0] as usize, w[1], context).map_err(|_| OUTOFMEMORY)?;
                // All fallible resources committed; single CPU holds IF clear.
                let child =
                    tree(|t| t.child(caller, context, w[7])).expect("prechecked publication");
                publish(slot, child).expect("scalar publication");
                trace("create", child, caller, w[7], 0);
                Ok(child)
            }
            28 => {
                tree(|t| t.start(caller, f.rdi)).map_err(status)?;
                trace("start", f.rdi, caller, 0, 0);
                Ok(0)
            }
            29 => {
                tree(|t| t.terminate(caller, f.rdi)).map_err(status)?;
                trace("terminate", f.rdi, caller, 768, 0);
                Ok(0)
            }
            30 => {
                let e = tree(|t| t.join(caller, f.rdi)).map_err(status)?;
                trace("join", f.rdi, caller, e.code, e.vector);
                Ok(e.code)
            }
            31 => {
                let parent = tree(|t| t.parent(f.rdi)).map_err(status)?;
                if f.rdi != caller && caller != tree(|t| t.root()) && parent != caller {
                    return Err(ACCESSDENIED);
                }
                let state = tree(|t| t.state(f.rdi)).map_err(status)?;
                let value = match state {
                    State::Prepared => 0,
                    State::Running => 1,
                    State::Stopping => 2,
                    State::Reaped => 3,
                };
                Ok((fixture::hooks().process_state)(f.rdi, value))
            }
            32 => {
                tree(|t| t.restart(caller, now())).map_err(status)?;
                trace("restart", caller, 0, 0, 0);
                Ok(0)
            }
            33 => tree(|t| t.first_child(caller, f.rdi)).map_err(status),
            _ => unreachable!(),
        }
    })();
    let (status, value) = match result {
        Ok(v) => (0, v),
        Err(e) => (e, 0),
    };
    record(
        "m1.supervisor_call",
        "supervisor",
        "ok",
        format_args!(
            "caller={caller} operation={op} target={} status={status} value={value}",
            f.rdi
        ),
    );
    Some(result)
}
fn reap(slot: usize, code: u64, vector: u64) -> Result<()> {
    let task = unsafe { (&*TASKS.0.get())[slot].unwrap() };
    let token = unsafe { (*TOKENS.0.get())[slot] };
    let thread = ThreadId {
        slot,
        generation: task.current.id.generation,
    };
    let parked = waiting::blocked(thread);
    waiting::terminate(thread); // discard wait/continuation before any backing
    unsafe {
        *CURRENT.0.get() = None;
    }
    crate::exceptions::user_stack(crate::exceptions::stacks()[0] + 69632);
    capabilities::remove(task.current.space)?; // drain queued transfers, maps/tables
    runtime_memory::with(|r, b| unsafe {
        (&mut *LAUNCHER.0.get()).destroy(r, b, task.current.id)
    })?;
    unsafe {
        (&mut *TASKS.0.get())[slot] = None;
        (*TOKENS.0.get())[slot] = 0;
    }
    tree(|t| t.reaped(token, Exit { code, vector })).map_err(|_| error("supervisor", "reap"))?;
    trace("reap", token, 0, code, vector);
    record(
        "m1.supervisor_end",
        "supervisor",
        "ok",
        format_args!(
            "token={token} process={} generation={} thread={slot} mode={} code={code} vector={vector} preemptions={} dispatches={} parked={parked}",
            task.current.id.slot,
            task.current.id.generation,
            task.current.case,
            task.preemptions,
            task.dispatches
        ),
    );
    Ok(())
}
pub(crate) fn run() -> Result<()> {
    fixture::install_supervisor();
    unsafe {
        asm!("cli");
        let root: u64;
        asm!("mov {}, cr3",out(reg)root);
        *KERNEL_ROOT.0.get() = root;
    }
    need(cpu_supported(core::arch::x86_64::__cpuid(1).edx, 0, 7))?;
    install()?;
    unsafe {
        let cr0: u64;
        let cr4: u64;
        asm!("mov {}, cr0","mov {}, cr4",out(reg)cr0,out(reg)cr4);
        asm!("mov cr0, {}",in(reg)cr0|(1<<5));
        asm!("mov cr4, {}",in(reg)cr4&!((1<<18)|(1<<16)|(1<<22)|(1<<23)));
    }
    capabilities::enable();
    capabilities::enable_channels();
    waiting::enable();
    let empty = runtime_memory::with(|r, _| r.inventory());
    let objects = capabilities::inventory();
    let frames = crate::paging::runtime_frames();
    let fixed = empty.metadata_bytes
        + empty.table_storage_bytes
        + 16 * 73728
        + core::mem::size_of::<[Option<Task>; N]>()
        + core::mem::size_of::<Tree>()
        + core::mem::size_of::<continuum_kernel::objects::Registry>()
        + core::mem::size_of::<Launcher<16>>()
        + waiting::fixed_bytes()
        + 8192;
    need(fixed <= 16 * 1024 * 1024)?;
    for wave in 0..4 {
        let context = Context {
            commitment_ceiling: policy::MEMORY,
            session: 3003 + wave,
            ..Binding::Service.context()
        };
        let slot = prepare(0, 500 + wave, context)?;
        let root = tree(|t| t.bootstrap(context)).map_err(|_| error("supervisor", "bootstrap"))?;
        publish(slot, root)?;
        tree(|t| t.start_root()).unwrap();
        trace("bootstrap", root, 0, wave, 0);
        let mut turns = 0;
        let mut next = 0;
        loop {
            // A completed parent first closes admission and marks every orphan.
            for i in 0..N {
                let done = unsafe { (&*TASKS.0.get())[i].is_some_and(|t| t.result.is_some()) };
                if done {
                    let token = unsafe { (*TOKENS.0.get())[i] };
                    if token == tree(|t| t.root()) {
                        let result = unsafe { (&*TASKS.0.get())[i].unwrap().result.unwrap() };
                        need(result == if wave == 3 { 262 } else { 0 })?;
                    }
                    tree(|t| t.stop(token)).unwrap();
                    if token == tree(|t| t.root()) {
                        let denied = tree(|t| t.check_child(token, context, 1, 1, 0, 0));
                        need(denied == Err(policy::Error::Denied))?;
                        trace("admission-closed", token, 0, ACCESSDENIED, 0);
                    }
                }
            }
            for i in 0..N {
                let token = unsafe { (*TOKENS.0.get())[i] };
                if token != 0 && tree(|t| t.state(token)) == Ok(State::Stopping) {
                    let task = unsafe { (&*TASKS.0.get())[i].unwrap() };
                    reap(i, task.result.unwrap_or(768), task.vector)?;
                }
            }
            if unsafe { (&*TASKS.0.get()).iter().all(Option::is_none) } {
                break;
            }
            // Poll uses FIFO completion, then this bounded round-robin scan uses
            // no queue references that could outlive a task's generation.
            let mut wakes = Queue::EMPTY;
            waiting::poll(&mut wakes)?;
            while wakes.pop().is_some() {} // completion frames already updated
            let chosen = (0..N).map(|d| (next + d) % N).find(|i| {
                let token = unsafe { (*TOKENS.0.get())[*i] };
                if token == 0 || tree(|t| t.state(token)) != Ok(State::Running) {
                    return false;
                }
                let generation = unsafe { (&*TASKS.0.get())[*i].unwrap().current.id.generation };
                !waiting::blocked(ThreadId {
                    slot: *i,
                    generation,
                })
            });
            let Some(i) = chosen else {
                waiting::idle()?;
                continue;
            };
            next = (i + 1) % N;
            turns += 1;
            need(turns < 4096)?;
            let (root, frame, fp, stack, c, heap) = unsafe {
                let t = (&mut *TASKS.0.get())[i].as_mut().unwrap();
                t.dispatches += 1;
                (
                    t.root,
                    &t.frame as *const Frame,
                    &t.fp as *const FxState,
                    t.stack,
                    t.current,
                    t.heap,
                )
            };
            unsafe {
                *CURRENT.0.get() = Some(c);
                *INDEX.0.get() = i;
                *KERNEL_SP.0.get() = stack;
                *HEAP.0.get() = heap;
                *PENDING.0.get() = false;
                *BEGIN.0.get() = 0;
                for msr in [0xc0000100, 0xc0000101, 0xc0000102] {
                    wrmsr(msr, 0);
                }
            }
            crate::exceptions::user_stack(stack);
            need(valid_return(unsafe { &*frame }))?;
            waiting::resume(ThreadId {
                slot: i,
                generation: c.id.generation,
            })?;
            record(
                "m1.supervisor_dispatch",
                "supervisor",
                "ok",
                format_args!(
                    "token={} process={} generation={} thread={i} mode={} validated=true",
                    unsafe { (*TOKENS.0.get())[i] },
                    c.id.slot,
                    c.id.generation,
                    c.case
                ),
            );
            unsafe {
                *ENABLED.0.get() = true;
                task_resume(root, frame, fp);
                *ENABLED.0.get() = false;
            }
        }
        let m = runtime_memory::with(|r, _| r.inventory());
        let o = capabilities::inventory();
        need(m == empty && o == objects && crate::paging::runtime_frames() == frames)?;
        waiting::settled()?;
        tree(|t| t.clear_epoch()).map_err(|_| error("supervisor", "epoch"))?;
        record(
            "m1.supervisor_settled",
            "supervisor",
            "ok",
            format_args!(
                "wave={wave} turns={turns} objects={} handles={} mappings={} committed={} resident={} spaces={} allocations={} tables={} stacks={} pins={} frames={frames} nodes={}",
                o.objects,
                o.handles,
                o.mappings,
                m.committed,
                m.resident,
                m.spaces,
                m.allocations,
                m.page_tables,
                m.kernel_stack_bytes,
                m.active_pins,
                tree(|t| t.live())
            ),
        );
    }
    record(
        "m1.supervisor_ready",
        "supervisor",
        "ok",
        format_args!(
            "waves=4 settled=true user_execution=true fail_closed=true fixed_bytes={fixed}"
        ),
    );
    unsafe {
        asm!("sti");
    }
    Ok(())
}
