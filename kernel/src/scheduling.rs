//! P3-05 mechanism: one owning thread per admitted process; kernel continuations
//! run to completion. Never suspend a Rust borrow, pin, or kernel call stack.
use super::*;
use continuum_kernel::scheduler::{FxState, Queue, ThreadId, cpu_supported};
const N: usize = 32;
#[derive(Clone, Copy)]
struct Task {
    current: Current,
    root: u64,
    stack: u64,
    frame: Frame,
    fp: FxState,
    heap: [Option<Heap>; 32],
    dispatches: u64,
    preemptions: u64,
    progress: u64,
    ready_ns: u64,
    elapsed_ns: u64,
    runnable_ns: u64,
    result: Option<u64>,
    vector: u64,
}
static PROGRESS_OBSERVER: Local<Option<fn() -> u64>> = Local(UnsafeCell::new(None));
pub(super) fn observe_progress(observer: fn() -> u64) {
    assert!(!active());
    unsafe {
        PROGRESS_OBSERVER.0.get().write(Some(observer));
    }
}
fn progress_sample() -> u64 {
    unsafe { PROGRESS_OBSERVER.0.get().read() }.map_or(0, |observe| observe())
}
static TASKS: Local<[Option<Task>; N]> = Local(UnsafeCell::new([None; N]));
static ENABLED: Local<bool> = Local(UnsafeCell::new(false));
static INDEX: Local<usize> = Local(UnsafeCell::new(0));
static PENDING: Local<bool> = Local(UnsafeCell::new(false));
static BEGIN: Local<u64> = Local(UnsafeCell::new(0));
static MAX_MASKED: Local<u64> = Local(UnsafeCell::new(0));
static NESTED: Local<u64> = Local(UnsafeCell::new(0));
static MAX_WAIT: Local<u64> = Local(UnsafeCell::new(0));
static PIN_IRQS: Local<u64> = Local(UnsafeCell::new(0));
global_asm!(include_str!("scheduler-entry.S"), continuation=sym CONTINUATION,
    kernel_root=sym KERNEL_ROOT, kernel_sp=sym KERNEL_SP);
unsafe extern "C" {
    fn task_resume(root: u64, frame: *const Frame, fp: *const FxState);
    fn task_save_fp(fp: *mut FxState);
}
pub(crate) fn active() -> bool {
    unsafe { ENABLED.0.get().read() }
}
#[cfg(feature = "wait-test")]
pub(super) fn current_thread() -> ThreadId {
    unsafe {
        let slot = *INDEX.0.get();
        ThreadId {
            slot,
            generation: (&*TASKS.0.get())[slot]
                .as_ref()
                .unwrap()
                .current
                .id
                .generation,
        }
    }
}
#[cfg(feature = "wait-test")]
pub(super) fn observed_progress(id: ThreadId) -> Option<u64> {
    unsafe {
        (&*TASKS.0.get())
            .get(id.slot)
            .and_then(Option::as_ref)
            .filter(|t| t.current.id.generation == id.generation)
            .map(|t| t.progress)
    }
}
#[cfg(feature = "wait-test")]
pub(super) fn complete_wait(id: ThreadId, status: u64, value: u64) -> Result<()> {
    let t = unsafe { (&mut *TASKS.0.get())[id.slot].as_mut() }.ok_or(error("wait", "thread"))?;
    need(t.current.id.generation == id.generation && t.result.is_none())?;
    t.frame.rax = status;
    t.frame.rdx = value;
    Ok(())
}
#[cfg(feature = "wait-test")]
pub(super) fn kill_waiting(id: ThreadId, queue: &mut Queue) -> Result<()> {
    need(waiting::blocked(id))?;
    waiting::terminate(id);
    let t = unsafe { (&mut *TASKS.0.get())[id.slot].as_mut() }.ok_or(error("wait", "thread"))?;
    need(t.current.id.generation == id.generation && t.result.is_none())?;
    t.result = Some(0x300);
    record(
        "m1.wait_kill",
        "waits",
        "ok",
        format_args!(
            "thread={} process={} generation={} principal={} result=768",
            id.slot, t.current.id.slot, id.generation, t.current.context.principal
        ),
    );
    queue.push(id, 0)
}
fn now() -> u64 {
    crate::timer::sample_ns()
}
fn measure_masked() {
    let begin = unsafe { BEGIN.0.get().read() };
    if begin != 0 {
        let delta = now().checked_sub(begin).expect("HPET monotonic");
        unsafe {
            *MAX_MASKED.0.get() = (*MAX_MASKED.0.get()).max(delta);
        }
    }
}
pub(crate) fn begin_syscall(entered: u64) {
    if active() {
        unsafe {
            BEGIN.0.get().write(entered);
        }
    }
}
pub(crate) fn copy_irq(pinned: bool) {
    if active() {
        // Test-only interrupt delivery with a live pin/borrow: IRQ may only set
        // pending. This is deliberately not a rescheduling point.
        unsafe {
            asm!("sti", "hlt", "cli");
            assert!(*PENDING.0.get());
            if pinned {
                *PIN_IRQS.0.get() += 1;
            }
        }
    }
}
pub(crate) fn tick(cs: u64, raw: *mut u8) {
    if !active() {
        return;
    }
    if cs == USER_CS {
        unsafe {
            BEGIN.0.get().write(now());
        }
        capture(unsafe { &*(raw as *const Frame) }, true);
    } else {
        unsafe {
            PENDING.0.get().write(true);
            *NESTED.0.get() += 1;
        }
    }
}
fn capture(f: &Frame, timer: bool) -> ! {
    assert!(runtime_memory::quiescent());
    #[cfg(feature = "capability-test")]
    assert!(capabilities::quiescent());
    // IF clear; no memory/registry borrow, pin or lock held here. FXSAVE writes
    // only a trusted aligned supervisor buffer, never user-supplied bytes.
    unsafe {
        let t = (&mut *TASKS.0.get())[*INDEX.0.get()].as_mut().unwrap();
        t.frame = *f;
        // Hardware interrupt frames can contain RF (including entry-boundary
        // delivery under TCG). Debug resumption is unsupported. Normalize only
        // this hardware-owned bit; syscall frames retain strict validation.
        if timer {
            t.frame.flags &= !(1 << 16);
        }
        // Map/free may publish the other page-table bank during this dispatch.
        // Resume the current root, never the bank retired by an earlier syscall.
        asm!("mov {}, cr3", out(reg) t.root);
        task_save_fp(&mut t.fp);
        if !t.fp.supported_controls() {
            t.result = Some(0x400);
        }
        t.heap = HEAP.0.get().read();
        t.preemptions += u64::from(timer);
        t.progress = progress_sample();
    }
    measure_masked();
    unsafe { user_leave() }
}
pub(crate) fn syscall_return(f: &Frame, voluntary: bool) {
    if voluntary || unsafe { PENDING.0.get().read() } {
        if current().case == 8 {
            finish("kill-at-return", 0x300, 0);
        }
        capture(f, false);
    }
    measure_masked();
    // Last instruction in the Rust return path: no kernel FP instructions.
    unsafe {
        asm!("clts", options(nostack));
    }
}
pub(crate) fn check_fp_policy() {
    if active() {
        let supported = unsafe {
            let t = (&mut *TASKS.0.get())[*INDEX.0.get()].as_mut().unwrap();
            task_save_fp(&mut t.fp);
            t.fp.supported_controls()
        };
        if !supported {
            finish("fp-control", 0x400, 0);
        }
    }
}
pub(crate) fn finish(_reason: &'static str, result: u64, vector: u64) -> ! {
    unsafe {
        let t = (&mut *TASKS.0.get())[*INDEX.0.get()].as_mut().unwrap();
        task_save_fp(&mut t.fp);
        t.result = Some(result);
        t.vector = vector;
        t.progress = progress_sample();
    }
    measure_masked();
    unsafe { user_leave() }
}
fn spawn(queue: &mut Queue, mode: u64, module: usize, required: u64) -> Result<ThreadId> {
    let cr4: u64;
    unsafe {
        asm!("mov {}, cr4", out(reg) cr4);
    }
    if !cpu_supported(core::arch::x86_64::__cpuid(1).edx, cr4, required) {
        return Err(error("scheduler", "cpu_features"));
    }
    let slot = unsafe { (&*TASKS.0.get()).iter().position(Option::is_none) }
        .ok_or(error("scheduler", "quota"))?;
    let binding = [Binding::UserA, Binding::UserB, Binding::Service][module];
    #[cfg(feature = "capability-test")]
    let binding = if capabilities::enabled() && module == 2 {
        Binding::UserA
    } else {
        binding
    };
    let id = runtime_memory::with(|r, b| unsafe {
        (&mut *LAUNCHER.0.get()).bootstrap_return(
            r,
            b,
            TrustedImage {
                bytes: crate::paging::module(module)?,
                digest: IMAGES[module].0,
                size: IMAGES[module].1,
            },
            binding,
            EXIT_TRAMPOLINE,
        )
    })?;
    let (space, context, entry, sp) = unsafe {
        let p = (&*LAUNCHER.0.get()).get(id)?;
        (p.space, p.context, p.entry, p.stack)
    };
    #[cfg(feature = "capability-test")]
    let grants = if capabilities::enabled() {
        Some(capabilities::admit(space, module, id, context)?)
    } else {
        None
    };
    let (root, stack) = runtime_memory::with(|r, b| {
        let a = r.allocate(b, space, PAGE, PAGE)?.unwrap();
        r.write_staging(b, space, a, 0, &mode.to_le_bytes())?;
        r.write_staging(b, space, a, 16, &(module as u64 + 1).to_le_bytes())?;
        #[cfg(feature = "capability-test")]
        if let Some(grants) = grants {
            for (i, v) in grants.into_iter().enumerate() {
                r.write_staging(b, space, a, 24 + i as u64 * 8, &v.to_le_bytes())?;
            }
        }
        r.map(b, space, a, 0x70000000, 0, PAGE, Access::Write)?;
        Ok::<_, continuum_kernel::Error>((r.root(space)?.0, r.kernel_stack(space)?.end()))
    })?;
    let thread = ThreadId {
        slot,
        generation: id.generation,
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
    }
    queue.push(thread, 0)?;
    record(
        "m1.task_start",
        "scheduler",
        "ok",
        format_args!(
            "thread={slot} generation={} process={} principal={} mode={mode} root={root:#x} stack={stack:#x} fresh=true",
            thread.generation, id.slot, context.principal
        ),
    );
    Ok(thread)
}
pub(crate) fn run(trace: bool) -> Result<()> {
    fixture::install_scheduler();
    unsafe {
        asm!("cli");
        let root: u64;
        asm!("mov {}, cr3",out(reg) root);
        KERNEL_ROOT.0.get().write(root);
    }
    let edx = core::arch::x86_64::__cpuid(1).edx;
    if !cpu_supported(edx, 0, 7) {
        record(
            "m1.scheduler_cpu_rejected",
            "scheduler",
            "ok",
            format_args!("cpuid_edx={edx:#x} reason=cpu-features user_entries=0"),
        );
        return Ok(());
    }
    install()?;
    // Do not inherit an OSXSAVE/FSGSBASE policy from firmware. TLS and extended
    // state instructions are not in the admitted image contract.
    unsafe {
        let cr4: u64;
        let cr0: u64;
        asm!("mov {}, cr4",out(reg) cr4);
        asm!("mov {}, cr0",out(reg) cr0);
        asm!("mov cr0, {}",in(reg) cr0 | (1<<5)); // native x87 #MF, no legacy IRQ13
        asm!("mov cr4, {}",in(reg) cr4 & !((1<<18)|(1<<16)|(1<<22)|(1<<23)));
    }
    let empty = runtime_memory::with(|r, _| r.inventory());
    let frames = crate::paging::runtime_frames();
    let mut queue = Queue::EMPTY;
    need(spawn(&mut queue, 0, 0, 8).is_err())?;
    need(runtime_memory::with(|r, _| r.inventory()) == empty)?;
    let cr0: u64;
    let cr4: u64;
    unsafe {
        asm!("mov {}, cr0","mov {}, cr4",out(reg) cr0,out(reg) cr4);
    }
    let cpu = core::arch::x86_64::__cpuid(1);
    for bit in [0, 24, 25, 26] {
        need(!cpu_supported(cpu.edx & !(1 << bit), cr4, 7))?;
    }
    let extra = core::mem::size_of::<[Option<Task>; N]>()
        + core::mem::size_of::<Queue>()
        + core::mem::size_of::<Launcher<16>>()
        + core::mem::size_of::<[Option<Heap>; 32]>()
        + core::mem::size_of::<Option<Current>>()
        + 256;
    let fixed = empty.metadata_bytes + empty.table_storage_bytes + 16 * 73728 + extra;
    need(fixed <= 16 * 1024 * 1024)?;
    record(
        "m1.scheduler_cpu",
        "scheduler",
        "ok",
        format_args!(
            "cpuid_edx={:#x} cpuid_ecx={:#x} cr0={cr0:#x} cr4={cr4:#x} missing_feature_denials=4 required_feature_denials=1 fixed_bytes={fixed} extra_bytes={extra}",
            cpu.edx, cpu.ecx
        ),
    );
    record(
        "m1.scheduler_policy",
        "scheduler",
        "ok",
        format_args!(
            "slots=32 owners=16 threads_per_owner=1 priority=equal quantum_ns=5000000 fairness_quanta=64 fp=x87-sse-sse2 xsave=false avx=false unsupported_denied=true trace={trace}"
        ),
    );
    let observer_start = now();
    for _ in 0..100 {
        core::hint::black_box(now());
    }
    let observer_ns = now() - observer_start;
    let mut switches = 0u64;
    let mut masked_upper = 0;
    for (wave, mode) in [1, 2, 3, 4, 5, 6, 7, 8, 9, 10].into_iter().enumerate() {
        spawn(&mut queue, 0, 0, 7)?;
        spawn(&mut queue, 0, 1, 7)?;
        spawn(&mut queue, mode, 2, 7)?;
        let ready = now();
        let mut supervisor_begin = ready;
        unsafe {
            for t in (&mut *TASKS.0.get()).iter_mut().flatten() {
                t.ready_ns = ready;
            }
        }
        while let Some(id) = queue.pop() {
            let (root, frame, fp, stack, c, heap, wait) = unsafe {
                let t = (&mut *TASKS.0.get())[id.slot].as_mut().unwrap();
                need(t.current.id.generation == id.generation)?;
                t.dispatches += 1;
                let wait = now() - t.ready_ns;
                t.runnable_ns += wait;
                *MAX_WAIT.0.get() = (*MAX_WAIT.0.get()).max(wait);
                (
                    t.root,
                    &t.frame as *const Frame,
                    &t.fp as *const FxState,
                    t.stack,
                    t.current,
                    t.heap,
                    wait,
                )
            };
            unsafe {
                CURRENT.0.get().write(Some(c));
                INDEX.0.get().write(id.slot);
                KERNEL_SP.0.get().write(stack);
                HEAP.0.get().write(heap);
                PENDING.0.get().write(false);
                BEGIN.0.get().write(0);
                for msr in [0xc0000100, 0xc0000101, 0xc0000102] {
                    wrmsr(msr, 0);
                }
            }
            crate::exceptions::user_stack(stack);
            let start = now();
            masked_upper = masked_upper.max(start - supervisor_begin);
            unsafe {
                // Validate the complete saved IRET frame outside hard IRQ, with
                // the trusted current binding. No mapping mutation until entry.
                if valid_return(&*frame) {
                    ENABLED.0.get().write(true);
                    task_resume(root, frame, fp);
                    ENABLED.0.get().write(false);
                } else {
                    let f = &*frame;
                    record(
                        "m1.user_bad_return",
                        "user-boundary",
                        "ok",
                        format_args!(
                            "case={} ip={:#x} sp={:#x} cs={:#x} ss={:#x} flags={:#x} operation={}",
                            c.case, f.ip, f.sp, f.cs, f.ss, f.flags, f.rax
                        ),
                    );
                    (&mut *TASKS.0.get())[id.slot].as_mut().unwrap().result = Some(0x200);
                }
            }
            let end = now();
            // Each full residence includes all entry/exit assembly and bounds
            // every masked subinterval, conservatively including user execution.
            masked_upper = masked_upper.max(end - start);
            supervisor_begin = end;
            switches += 1;
            let (done, result, dispatches, preemptions, progress, vector, elapsed, runnable, mxcsr) = unsafe {
                let t = (&mut *TASKS.0.get())[id.slot].as_mut().unwrap();
                t.elapsed_ns += end - start;
                if t.current.case == 0 {
                    t.result = continuum_kernel::scheduler::compute_budget_result(
                        t.result,
                        t.dispatches,
                        16,
                    );
                }
                t.ready_ns = end;
                (
                    t.result.is_some(),
                    t.result.unwrap_or(u64::MAX),
                    t.dispatches,
                    t.preemptions,
                    t.progress,
                    t.vector,
                    t.elapsed_ns,
                    t.runnable_ns,
                    u32::from_le_bytes(t.fp.0[24..28].try_into().unwrap()),
                )
            };
            if trace {
                let shown_result = if done { result } else { 0 };
                record(
                    "m1.task_switch",
                    "scheduler",
                    "ok",
                    format_args!(
                        "wave={wave} thread={} dispatches={dispatches} preemptions={preemptions} progress={progress} elapsed_ns={} wait_ns={wait} done={done} result={shown_result}",
                        id.slot,
                        end - start
                    ),
                );
            }
            if done {
                let expected = match c.case {
                    0 | 8 => 0x300,
                    2 => 0x110,
                    3 => 0x400,
                    4 => 0x106,
                    5 => 0x10d,
                    10 => 0x200,
                    _ => 0,
                };
                need(result == expected)?;
                if c.case == 0 {
                    need(preemptions >= 15 && progress > 0)?;
                }
                record(
                    "m1.task_end",
                    "scheduler",
                    "ok",
                    format_args!(
                        "wave={wave} thread={} generation={} mode={} result={result} vector={vector} dispatches={dispatches} preemptions={preemptions} progress={progress} elapsed_ns={elapsed} runnable_ns={runnable} mxcsr={mxcsr:#x}",
                        id.slot, id.generation, c.case
                    ),
                );
                unsafe {
                    CURRENT.0.get().write(None);
                    HEAP.0.get().write([None; 32]);
                }
                crate::exceptions::user_stack(crate::exceptions::stacks()[0] + 69632);
                runtime_memory::with(|r, b| unsafe {
                    (&mut *LAUNCHER.0.get()).destroy(r, b, c.id)
                })?;
                unsafe {
                    (&mut *TASKS.0.get())[id.slot] = None;
                }
            } else {
                queue.push(id, 0)?;
            }
        }
        let inventory = runtime_memory::with(|r, _| r.inventory());
        need(inventory == empty && crate::paging::runtime_frames() == frames)?;
        record(
            "m1.task_settlement",
            "scheduler",
            "ok",
            format_args!(
                "wave={wave} settled=true frames={frames} spaces={} allocations={} committed={} resident={} mappings={} tables={} stacks={} pins={}",
                inventory.spaces,
                inventory.allocations,
                inventory.committed,
                inventory.resident,
                inventory.mappings,
                inventory.page_tables,
                inventory.kernel_stack_bytes,
                inventory.active_pins
            ),
        );
        masked_upper = masked_upper.max(now() - supervisor_begin);
    }
    let before = crate::timer::uptime()?.0;
    unsafe {
        asm!("sti", "hlt", "cli");
    }
    need(crate::timer::uptime()?.0 > before)?;
    let (masked, wait, nested) =
        unsafe { (*MAX_MASKED.0.get(), *MAX_WAIT.0.get(), *NESTED.0.get()) };
    need(masked_upper < 320_000_000 && masked < 320_000_000 && wait < 320_000_000 && nested > 0)?;
    record(
        "m1.scheduler_ready",
        "scheduler",
        "ok",
        format_args!(
            "waves=10 tasks=30 switches={switches} settled=true idle=true max_masked_upper_ns={masked_upper} max_body_ns={masked} max_wait_ns={wait} nested_irqs={nested} pin_irqs={} observer_samples=100 observer_total_ns={observer_ns} trace={trace}",
            unsafe { *PIN_IRQS.0.get() }
        ),
    );
    unsafe {
        asm!("sti");
    }
    Ok(())
}

#[cfg(feature = "capability-test")]
pub(crate) fn run_objects(channels: bool) -> Result<()> {
    if channels {
        capabilities::enable_channels();
    }
    fixture::install_capabilities();
    unsafe {
        asm!("cli");
        let root: u64;
        asm!("mov {}, cr3",out(reg) root);
        *KERNEL_ROOT.0.get() = root;
    }
    need(cpu_supported(core::arch::x86_64::__cpuid(1).edx, 0, 7))?;
    install()?;
    unsafe {
        let cr0: u64;
        let cr4: u64;
        asm!("mov {}, cr0","mov {}, cr4",out(reg) cr0,out(reg) cr4);
        asm!("mov cr0, {}",in(reg) cr0 | (1<<5));
        asm!("mov cr4, {}",in(reg) cr4 & !((1<<18)|(1<<16)|(1<<22)|(1<<23)));
    }
    capabilities::enable();
    #[cfg(feature = "wait-test")]
    if waiting::enabled() {
        waiting::clock_probe()?;
    }
    let empty = runtime_memory::with(|r, _| r.inventory());
    let objects = capabilities::inventory();
    let frames = crate::paging::runtime_frames();
    let fixed = empty.metadata_bytes
        + empty.table_storage_bytes
        + 16 * 73728
        + core::mem::size_of::<[Option<Task>; N]>()
        + core::mem::size_of::<continuum_kernel::objects::Registry>()
        + core::mem::size_of::<Launcher<16>>()
        + core::mem::size_of::<Queue>()
        + 8192;
    #[cfg(feature = "wait-test")]
    let fixed = fixed + waiting::fixed_bytes();
    need(fixed <= 16 * 1024 * 1024)?;
    let mut count = 0;
    let waves = if channels { 2 } else { 6 };
    let waves = fixture::hooks().object_waves.unwrap_or(waves);
    for wave in 0..waves {
        let mut queue = Queue::EMPTY;
        for module in 0..3 {
            spawn(
                &mut queue,
                if channels { 200 + wave } else { 100 + wave },
                module,
                7,
            )?;
        }
        let i = capabilities::inventory();
        record(
            "m1.cap_inventory",
            "capabilities",
            "ok",
            format_args!(
                "wave={wave} phase=admitted owners={} objects={} handles={} mappings={} backing={} mapped={} fixed_bytes={fixed}",
                i.owners, i.objects, i.handles, i.mappings, i.backing_bytes, i.mapped_bytes
            ),
        );
        loop {
            (fixture::hooks().supervise)(&mut queue, wave)?;
            #[cfg(feature = "wait-test")]
            waiting::poll(&mut queue)?;
            let Some(id) = queue.pop() else {
                #[cfg(feature = "wait-test")]
                if waiting::enabled() && waiting::pending() {
                    waiting::idle()?;
                    continue;
                }
                break;
            };
            count += 1;
            need(count < if channels { 2048 } else { 512 })?;
            let (root, frame, fp, stack, c) = unsafe {
                let t = (&mut *TASKS.0.get())[id.slot].as_mut().unwrap();
                need(t.current.id.generation == id.generation)?;
                t.dispatches += 1;
                (
                    t.root,
                    &t.frame as *const Frame,
                    &t.fp as *const FxState,
                    t.stack,
                    t.current,
                )
            };
            unsafe {
                *CURRENT.0.get() = Some(c);
                *INDEX.0.get() = id.slot;
                *KERNEL_SP.0.get() = stack;
                *HEAP.0.get() = [None; 32];
                *PENDING.0.get() = false;
                *BEGIN.0.get() = 0;
                for msr in [0xc0000100, 0xc0000101, 0xc0000102] {
                    wrmsr(msr, 0);
                }
            }
            crate::exceptions::user_stack(stack);
            need(valid_return(unsafe { &*frame }))?;
            #[cfg(feature = "wait-test")]
            if waiting::enabled() {
                waiting::resume(id)?;
            }
            unsafe {
                if (&*TASKS.0.get())[id.slot]
                    .as_ref()
                    .unwrap()
                    .result
                    .is_none()
                {
                    *ENABLED.0.get() = true;
                    task_resume(root, frame, fp);
                    *ENABLED.0.get() = false;
                }
            }
            let (result, preemptions, dispatches) = unsafe {
                let t = (&*TASKS.0.get())[id.slot].as_ref().unwrap();
                (t.result, t.preemptions, t.dispatches)
            };
            if let Some(value) = result {
                #[cfg(feature = "wait-test")]
                waiting::terminate(id);
                let expected = if !channels && id.slot == 1 && wave > 0 {
                    0x10e
                } else {
                    0
                };
                let expected =
                    (fixture::hooks().expected_result)(wave, id.slot).unwrap_or(expected);
                record(
                    "m1.cap_reap",
                    "capabilities",
                    "ok",
                    format_args!(
                        "wave={wave} task={} process={} generation={} principal={} authority_session={} result={value} expected={expected} preemptions={preemptions} dispatches={dispatches}",
                        id.slot, c.id.slot, c.id.generation, c.context.principal, c.context.session
                    ),
                );
                need(value == expected)?;
                unsafe {
                    *CURRENT.0.get() = None;
                }
                crate::exceptions::user_stack(crate::exceptions::stacks()[0] + 69632);
                capabilities::remove(c.space)?;
                runtime_memory::with(|r, b| unsafe {
                    (&mut *LAUNCHER.0.get()).destroy(r, b, c.id)
                })?;
                unsafe {
                    (&mut *TASKS.0.get())[id.slot] = None;
                }
                let i = capabilities::inventory();
                let m = runtime_memory::with(|r, _| r.inventory());
                record(
                    "m1.cap_inventory",
                    "capabilities",
                    "ok",
                    format_args!(
                        "wave={wave} phase=reap task={} owners={} objects={} handles={} mappings={} backing={} mapped={} frames={} committed={} resident={} pins={}",
                        id.slot,
                        i.owners,
                        i.objects,
                        i.handles,
                        i.mappings,
                        i.backing_bytes,
                        i.mapped_bytes,
                        crate::paging::runtime_frames(),
                        m.committed,
                        m.resident,
                        m.active_pins
                    ),
                );
            } else {
                #[cfg(feature = "wait-test")]
                if waiting::blocked(id) {
                    continue;
                }
                queue.push(id, 0)?;
            }
        }
        let m = runtime_memory::with(|r, _| r.inventory());
        let i = capabilities::inventory();
        need(m == empty && i == objects && crate::paging::runtime_frames() == frames)?;
        #[cfg(feature = "wait-test")]
        if waiting::enabled() {
            waiting::settled()?;
        }
        record(
            "m1.cap_inventory",
            "capabilities",
            "ok",
            format_args!(
                "wave={wave} phase=settled owners={} objects={} handles={} mappings={} backing={} mapped={} frames={frames} spaces={} allocations={} committed={} resident={} tables={} stacks={} pins={}",
                i.owners,
                i.objects,
                i.handles,
                i.mappings,
                i.backing_bytes,
                i.mapped_bytes,
                m.spaces,
                m.allocations,
                m.committed,
                m.resident,
                m.page_tables,
                m.kernel_stack_bytes,
                m.active_pins
            ),
        );
    }
    record(
        "m1.cap_ready",
        "capabilities",
        "ok",
        format_args!(
            "waves={waves} tasks={} settled=true switches={count} nested_irqs={} pin_irqs={} fixed_bytes={fixed} user_execution=true",
            waves * 3,
            unsafe { *NESTED.0.get() },
            unsafe { *PIN_IRQS.0.get() }
        ),
    );
    unsafe {
        asm!("sti");
    }
    Ok(())
}
