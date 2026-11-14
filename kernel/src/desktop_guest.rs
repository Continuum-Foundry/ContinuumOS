//! Trusted desktop runtime adapter; acceptance faults are selected only at boot.
use super::*;
use continuum_kernel::desktop as d;
static GRAPH: Local<d::Graph> = Local(UnsafeCell::new(d::Graph::EMPTY));
static ROLES: Local<[Option<ThreadId>; 7]> = Local(UnsafeCell::new([None; 7]));
static READY: Local<bool> = Local(UnsafeCell::new(false));
static VERIFIED: Local<[Option<VerifiedImage<'static>>; 3]> =
    Local(UnsafeCell::new([const { None }; 3]));
#[derive(Clone, Copy)]
struct Restart {
    due: u64,
    mode: u64,
    remaining: u32,
}
fn runnable_owners() -> u32 {
    unsafe {
        (&*TASKS.0.get())
            .iter()
            .enumerate()
            .fold(0, |mask, (slot, t)| {
                if let Some(t) = t {
                    let id = ThreadId {
                        slot,
                        generation: t.current.id.generation,
                    };
                    if t.result.is_none() && !waiting::blocked(id) {
                        return mask | (1 << slot);
                    }
                }
                mask
            })
    }
}
pub(crate) fn ready() -> bool {
    unsafe { *READY.0.get() }
}
static RESIDENCE_WITHOUT_WAIT: Local<[u64; 32]> = Local(UnsafeCell::new([0; 32]));
pub(crate) fn cooperated() {
    unsafe {
        (&mut *RESIDENCE_WITHOUT_WAIT.0.get())[*INDEX.0.get()] = 0;
    }
}
fn time() -> u64 {
    crate::timer::elapsed().expect("desktop elapsed clock")
}
fn bound(kind: &'static str, role: usize, observed: u64, limit: u64) -> Result<()> {
    if observed > limit {
        record(
            "m1.desktop_bound",
            "desktop",
            "ok",
            format_args!(
                "kind={kind} role={} observed_ns={observed} limit_ns={limit} passed=false",
                d::ROLES[role]
            ),
        );
        return Err(error("desktop", kind));
    }
    Ok(())
}
fn prepare_phase(role: usize, phase: &str, begin: u64) {
    #[cfg(feature = "desktop-test")]
    record(
        "m1.desktop_prepare_phase",
        "desktop",
        "ok",
        format_args!(
            "role={} phase={phase} cumulative_ns={}",
            d::ROLES[role],
            time() - begin
        ),
    );
    #[cfg(not(feature = "desktop-test"))]
    let _ = (role, phase, begin);
}
fn spawn_role(queue: &mut Queue, role: usize, mode: u64, fail: u64) -> Result<ThreadId> {
    let prepare_begin = time();
    unsafe {
        (&*GRAPH.0.get()).check(role, time())?;
    }
    let slot = unsafe { (&*TASKS.0.get()).iter().position(Option::is_none) }
        .ok_or(error("desktop", "slots"))?;
    let context = d::Graph::context(role)?;
    let module = role % 3;
    let id = runtime_memory::with(|r, b| unsafe {
        (&mut *LAUNCHER.0.get()).launch_verified_context(
            r,
            b,
            (&*VERIFIED.0.get())[module]
                .as_ref()
                .expect("verified immutable boot image"),
            context,
            EXIT_TRAMPOLINE,
            false,
        )
    })?;
    let space = unsafe { (&*LAUNCHER.0.get()).get(id)?.space };
    prepare_phase(role, "image", prepare_begin);
    let mut admitted = false;
    let prepared = (|| {
        if fail == 1 {
            return Err(error("desktop", "injected-image"));
        }
        let (process, thread, timer) = capabilities::registry(|c| {
            let (p, t) = c.admit(space).map_err(|_| error("desktop", "handles"))?;
            admitted = true;
            c.constrain(space).map_err(|_| error("desktop", "quota"))?;
            let timer = c
                .timer_create(space, p)
                .map_err(|_| error("desktop", "timer"))?;
            // CREATE/DUPLICATE/TRANSFER bootstrap authority never reaches the
            // payload. Keep only exit/yield control and timer query/control.
            let mut grants = [p, t, timer];
            for (handle, rights) in grants.iter_mut().zip([256, 256, 257]) {
                let old = *handle;
                *handle = c
                    .duplicate(space, old, rights)
                    .map_err(|_| error("desktop", "attenuation"))?;
                runtime_memory::with(|r, b| c.close(r, b, space, old))
                    .map_err(|_| error("desktop", "close_bootstrap"))?;
            }
            Ok::<_, continuum_kernel::Error>((grants[0], grants[1], grants[2]))
        })?;
        prepare_phase(role, "grants", prepare_begin);
        if fail == 2 {
            return Err(error("desktop", "injected-grants"));
        }
        runtime_memory::with(|r, b| {
            // The page-table builder reconstructs the complete inactive bank on
            // each map. Map the tiny config before the large heap so config
            // publication cannot rebuild the same 48/64-MiB heap a second time.
            let a = r.allocate(b, space, PAGE, PAGE)?.unwrap();
            for (i, v) in [role as u64, d::HEAPS[role], process, thread, timer, mode]
                .into_iter()
                .enumerate()
            {
                r.write_staging(b, space, a, i as u64 * 8, &v.to_le_bytes())?;
            }
            r.map(b, space, a, d::CONFIG, 0, PAGE, Access::Read)?;
            prepare_phase(role, "config", prepare_begin);
            let heap = r.allocate(b, space, d::HEAPS[role], PAGE)?.unwrap();
            prepare_phase(role, "heap_allocate", prepare_begin);
            r.map(
                b,
                space,
                heap,
                d::HEAP_BASE,
                0,
                d::HEAPS[role],
                Access::Write,
            )?;
            prepare_phase(role, "heap_map", prepare_begin);
            if fail == 3 {
                return Err(error("desktop", "injected-heap"));
            }
            // All large backing is admitted once. Public calls cannot create or
            // replace it; repeated resource work consumes no kernel credits.
            r.constrain(space, d::HEAPS[role] + d::OVERHEAD, 16, 16)?;
            let p = unsafe { (&*LAUNCHER.0.get()).get(id)? };
            Ok((
                r.root(space)?.0,
                r.kernel_stack(space)?.end(),
                p.entry,
                p.stack,
            ))
        })
    })();
    let (root, stack, entry, sp) = match prepared {
        Ok(v) => v,
        Err(e) => {
            if admitted {
                capabilities::remove(space)?;
            }
            runtime_memory::with(|r, b| unsafe { (&mut *LAUNCHER.0.get()).destroy(r, b, id) })?;
            return Err(e);
        }
    };
    let thread = ThreadId {
        slot,
        generation: id.generation,
    };
    // Queue capacity is 32, while the graph has only seven owners. No public
    // operation adds tasks. All fallible backing work precedes publication.
    if let Err(e) = queue.push(thread, 0) {
        capabilities::remove(space)?;
        runtime_memory::with(|r, b| unsafe { (&mut *LAUNCHER.0.get()).destroy(r, b, id) })?;
        return Err(e);
    }
    unsafe {
        (&mut *GRAPH.0.get())
            .started(role, time())
            .expect("checked graph publication");
        (&mut *ROLES.0.get())[role] = Some(thread);
        (&mut *RESIDENCE_WITHOUT_WAIT.0.get())[slot] = 0;
        (&mut *TASKS.0.get())[slot] = Some(Task {
            current: Current {
                id,
                space,
                context,
                case: 100 + role as u64,
            },
            root,
            stack,
            frame: Frame {
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
            },
            fp: FxState::fresh(),
            heap: [None; 32],
            dispatches: 0,
            preemptions: 0,
            progress: 0,
            ready_ns: time(),
            elapsed_ns: 0,
            runnable_ns: 0,
            result: None,
            vector: 0,
        });
    }
    record(
        "m1.desktop_launch",
        "desktop",
        "ok",
        format_args!(
            "role={} process={} generation={} principal={} authority_session={} grants=private-memory heap={} image_stack_tables_cap={} units={} mode={mode}",
            d::ROLES[role],
            id.slot,
            id.generation,
            context.principal,
            context.session,
            d::HEAPS[role],
            d::OVERHEAD,
            unsafe { (&*GRAPH.0.get()).consumed() }
        ),
    );
    record(
        "m1.desktop_prepare",
        "desktop",
        "ok",
        format_args!(
            "role={} elapsed_ns={} units={}",
            d::ROLES[role],
            time() - prepare_begin,
            unsafe { (&*GRAPH.0.get()).consumed() }
        ),
    );
    Ok(thread)
}
fn reap(role: usize, id: ThreadId, result: u64) -> Result<()> {
    let cleanup_begin = time();
    let t = unsafe { (&mut *TASKS.0.get())[id.slot].take() }.ok_or(error("desktop", "reap"))?;
    need(t.current.id.generation == id.generation)?;
    crate::display_guest::owner_dead(t.current.id.slot as u64 + 1, id.generation)?;
    #[cfg(feature = "device-test")]
    crate::device_guest::owner_dead(t.current.id.slot as u64 + 1, id.generation)?;
    waiting::terminate(id);
    capabilities::remove(t.current.space)?;
    runtime_memory::with(|r, b| unsafe { (&mut *LAUNCHER.0.get()).destroy(r, b, t.current.id) })?;
    unsafe {
        (&mut *ROLES.0.get())[role] = None;
        (&mut *GRAPH.0.get()).stopped(role, time())?;
    }
    record(
        "m1.desktop_reap",
        "desktop",
        "ok",
        format_args!(
            "role={} process={} generation={} result={result} units={} refunded=false",
            d::ROLES[role],
            t.current.id.slot,
            id.generation,
            unsafe { (&*GRAPH.0.get()).consumed() }
        ),
    );
    record(
        "m1.desktop_cleanup",
        "desktop",
        "ok",
        format_args!(
            "role={} elapsed_ns={}",
            d::ROLES[role],
            time() - cleanup_begin
        ),
    );
    bound("cleanup", role, time() - cleanup_begin, 320_000_000)?;
    Ok(())
}
fn start_graph(queue: &mut Queue, fail_role: Option<usize>) -> Result<()> {
    for role in 0..7 {
        let mode = (fixture::hooks().desktop_mode)(role, 0);
        if let Err(e) = spawn_role(
            queue,
            role,
            mode,
            if fail_role == Some(role) { 3 } else { 0 },
        ) {
            // No user has entered yet. Retire every earlier prepared owner and
            // keep its spent lifetime units; recovery credits are never used.
            while let Some(id) = queue.pop() {
                let owner = unsafe { (&*TASKS.0.get())[id.slot].unwrap().current.case - 100 };
                reap(owner as usize, id, 0x201)?;
            }
            return Err(e);
        }
    }
    Ok(())
}
pub(crate) fn run() -> Result<()> {
    unsafe {
        asm!("cli");
        let root: u64;
        asm!("mov {}, cr3",out(reg)root);
        KERNEL_ROOT.0.get().write(root);
    }
    install()?;
    need(cpu_supported(core::arch::x86_64::__cpuid(1).edx, 0, 7))?;
    unsafe {
        let cr4: u64;
        let cr0: u64;
        asm!("mov {}, cr4",out(reg)cr4);
        asm!("mov {}, cr0",out(reg)cr0);
        asm!("mov cr0, {}",in(reg)cr0|(1<<5));
        asm!("mov cr4, {}",in(reg)cr4 & !((1<<18)|(1<<16)|(1<<22)|(1<<23)));
    }
    fixture::install_desktop();
    for module in 0..3 {
        let image = VerifiedImage::verify(TrustedImage {
            bytes: crate::paging::module(module)?,
            digest: IMAGES[module].0,
            size: IMAGES[module].1,
        })?;
        unsafe {
            (&mut *VERIFIED.0.get())[module] = Some(image);
        }
    }
    capabilities::enable();
    waiting::enable();
    #[cfg(feature = "desktop-test")]
    let empty = runtime_memory::with(|r, _| r.inventory());
    #[cfg(feature = "desktop-test")]
    let objects = capabilities::inventory();
    #[cfg(feature = "desktop-test")]
    let frames = crate::paging::runtime_frames();
    // Reserve is withheld from this graph for its entire lifetime, even after
    // owners exit. Physical availability is checked before admitting any image.
    let available = crate::paging::runtime_available_bytes();
    d::Graph::preflight(available)?;
    crate::paging::runtime_quota((d::Graph::required() / PAGE) as usize)?;
    let mut queue = Queue::EMPTY;
    #[cfg(feature = "desktop-test")]
    {
        need(d::Graph::preflight(d::Graph::required() + d::RESERVE - 1).is_err())?;
        for fail in 1..=3 {
            need(spawn_role(&mut queue, 0, 1, fail).is_err())?;
            need(
                runtime_memory::with(|r, _| r.inventory()) == empty
                    && capabilities::inventory() == objects
                    && crate::paging::runtime_frames() == frames,
            )?;
            need(unsafe { (&*GRAPH.0.get()).consumed() } == 0 && queue.pop().is_none())?;
            record(
                "m1.desktop_unwind",
                "desktop",
                "ok",
                format_args!(
                    "phase={fail} frames={frames} units=0 reserve={} authority=none",
                    d::RESERVE
                ),
            );
        }
        need(start_graph(&mut queue, Some(3)).is_err())?;
        need(
            runtime_memory::with(|r, _| r.inventory()) == empty
                && capabilities::inventory() == objects
                && crate::paging::runtime_frames() == frames,
        )?;
        need(unsafe { (&*GRAPH.0.get()).consumed() } == 3)?;
        record(
            "m1.desktop_abort",
            "desktop",
            "ok",
            format_args!(
                "prepared=3 failed_role=shell frames=0 units=3 refunded=false reserve={}",
                d::RESERVE
            ),
        );
        let after = time() + d::RETRY_NS;
        while time() < after {
            unsafe {
                asm!("sti", "hlt", "cli");
            }
        }
    }
    start_graph(&mut queue, None)?;
    unsafe {
        for t in (&mut *TASKS.0.get()).iter_mut().flatten() {
            t.ready_ns = time();
        }
    }
    let peak = runtime_memory::with(|r, _| r.inventory());
    let caps = capabilities::inventory();
    record(
        "m1.desktop_cap_inventory",
        "desktop",
        "ok",
        format_args!(
            "owners={} objects={} handles={} mappings={} backing={} queues=0 device_grants=0 process_rights=256 thread_rights=256 timer_rights=257",
            caps.owners, caps.objects, caps.handles, caps.mappings, caps.backing_bytes
        ),
    );
    record(
        "m1.desktop_inventory",
        "desktop",
        "ok",
        format_args!(
            "phase=admitted spaces={} allocations={} committed={} tables={} fixed_tables={} kernel_stacks={} frames={} reserve={} units={}",
            peak.spaces,
            peak.allocations,
            peak.committed,
            peak.page_tables,
            peak.table_storage_bytes,
            peak.kernel_stack_bytes,
            crate::paging::runtime_frames(),
            d::RESERVE,
            unsafe { (&*GRAPH.0.get()).consumed() }
        ),
    );
    #[cfg(not(feature = "desktop-test"))]
    let mut entered = 0u8;
    #[cfg(feature = "desktop-test")]
    let start = time();
    let mut control = 0u64;
    let mut max_ready_ns = 0u64;
    #[cfg(feature = "desktop-test")]
    let mut editor_ends = 0;
    // Only scalar policy survives a scheduler iteration. No memory/registry
    // borrow, pin or unpublished image survives user dispatch.
    let mut restarts: [Option<Restart>; 7] = [None; 7];
    loop {
        #[cfg(feature = "desktop-test")]
        bound("case_elapsed", 0, time() - start, 24_999_999_999)?;
        waiting::poll(&mut queue)?;
        // Restart backoff never busy-waits over healthy ready owners. If an
        // admission cannot obtain a complete ready-owner round in a bounded interval,
        // quarantine it rather than monopolizing the CPU or retrying forever.
        for role in 0..7 {
            if restarts[role].is_some_and(|r| time().saturating_sub(r.due) > 320_000_000) {
                restarts[role] = None;
                unsafe {
                    (&mut *GRAPH.0.get()).quarantine(role)?;
                }
                record(
                    "m1.desktop_start_denied",
                    "desktop",
                    "ok",
                    format_args!(
                        "role={} field=desktop reason=admission_pressure quarantined=true reserve={}",
                        d::ROLES[role],
                        d::RESERVE
                    ),
                );
            }
        }
        if let Some(role) =
            (0..7).find(|&role| restarts[role].is_some_and(|r| r.remaining == 0 && time() >= r.due))
        {
            let restart = restarts[role].take().unwrap();
            if let Err(e) = spawn_role(&mut queue, role, restart.mode, 0) {
                unsafe {
                    (&mut *GRAPH.0.get()).quarantine(role)?;
                }
                record(
                    "m1.desktop_start_denied",
                    "desktop",
                    "ok",
                    format_args!(
                        "role={} field={} reason={} quarantined=true reserve={}",
                        d::ROLES[role],
                        e.field,
                        e.reason,
                        d::RESERVE
                    ),
                );
                #[cfg(feature = "desktop-test")]
                return Err(e);
            }
            continue;
        }
        let Some(id) = queue.pop() else {
            #[cfg(feature = "desktop-test")]
            if unsafe { (&*TASKS.0.get()).iter().all(Option::is_none) }
                && restarts.iter().all(Option::is_none)
            {
                break;
            }
            waiting::idle()?;
            continue;
        };
        let (root, frame, fp, stack, c, heap) = unsafe {
            let t = (&mut *TASKS.0.get())[id.slot].as_mut().unwrap();
            need(t.current.id.generation == id.generation)?;
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
        let role = (c.case - 100) as usize;
        let ready = waiting::ready_time(id)
            .unwrap_or_else(|| unsafe { (&*TASKS.0.get())[id.slot].unwrap().ready_ns });
        let delay = time()
            .checked_sub(ready)
            .ok_or(error("desktop", "ready_clock"))?;
        max_ready_ns = max_ready_ns.max(delay);
        bound("ready_delay", role, delay, 320_000_000)?;
        waiting::resume(id)?;
        if role == 0 {
            control += 1;
        }
        unsafe {
            CURRENT.0.get().write(Some(c));
            INDEX.0.get().write(id.slot);
            KERNEL_SP.0.get().write(stack);
            HEAP.0.get().write(heap);
            PENDING.0.get().write(false);
            BEGIN.0.get().write(0);
        }
        crate::exceptions::user_stack(stack);
        let residence = time();
        unsafe {
            if valid_return(&*frame) {
                ENABLED.0.get().write(true);
                task_resume(root, frame, fp);
                ENABLED.0.get().write(false);
            } else {
                (&mut *TASKS.0.get())[id.slot].as_mut().unwrap().result = Some(0x200);
            }
        }
        let mut result = unsafe { (&*TASKS.0.get())[id.slot].unwrap().result };
        for restart in restarts.iter_mut().flatten() {
            restart.remaining &= !(1 << id.slot);
        }
        #[cfg(not(feature = "desktop-test"))]
        if result.is_none() && entered != 127 {
            if entered & (1 << role) == 0 {
                record(
                    "m1.desktop_enter",
                    "desktop",
                    "ok",
                    format_args!(
                        "role={} process={} generation={} cpl=3 acceptance_observers=false",
                        d::ROLES[role],
                        c.id.slot,
                        c.id.generation
                    ),
                );
            }
            entered |= 1 << role;
            if entered == 127 {
                record(
                    "m1.boot_complete",
                    "boot",
                    "ok",
                    format_args!("reclaimed_bytes=0"),
                );
                record(
                    "m1.ready",
                    "ready",
                    "ok",
                    format_args!(
                        "baseline=boot-memory-timer reclaimed_bytes=0 roles=7 acceptance_observers=false"
                    ),
                );
                unsafe {
                    *READY.0.get() = true;
                }
            }
        }
        let budget = unsafe { &mut (&mut *RESIDENCE_WITHOUT_WAIT.0.get())[id.slot] };
        *budget = budget
            .checked_add(time() - residence)
            .ok_or(error("desktop", "residence_overflow"))?;
        if *budget >= 1_000_000_000 && result.is_none() {
            result = Some(0x300);
            record(
                "m1.desktop_watchdog",
                "desktop",
                "ok",
                format_args!(
                    "role={} residence_without_wait_ns={} limit_ns=1000000000",
                    d::ROLES[role],
                    *budget
                ),
            );
        }
        #[cfg(feature = "desktop-test")]
        {
            // Healthy control remains scheduled until all fault incarnations end.
            if result.is_none()
                && role == 0
                && (fixture::hooks().desktop_stop_control)(control, editor_ends as u64)
            {
                result = Some(0);
            }
            bound("case_elapsed", 0, time() - start, 24_999_999_999)?;
        }
        if let Some(result) = result {
            reap(role, id, result)?;
            #[cfg(feature = "desktop-test")]
            need(
                (fixture::hooks().expected_result)(
                    if role == 4 { editor_ends as u64 } else { 0 },
                    role,
                ) == Some(result),
            )?;
            #[cfg(feature = "desktop-test")]
            if role == 4 {
                editor_ends += 1;
                if editor_ends < 3 {
                    restarts[4] = Some(Restart {
                        due: time() + d::RETRY_NS,
                        mode: (fixture::hooks().desktop_mode)(4, editor_ends as u64),
                        remaining: runnable_owners(),
                    });
                } else {
                    need(unsafe { (&*GRAPH.0.get()).check(4, u64::MAX).is_err() })?;
                }
            }
        } else if !waiting::blocked(id) {
            unsafe {
                (&mut *TASKS.0.get())[id.slot].as_mut().unwrap().ready_ns = time();
            }
            queue.push(id, 0)?;
        }
        #[cfg(not(feature = "desktop-test"))]
        if result.is_some() {
            // Bounded retries with quarantine. Existing services retain their
            // mappings; a dead owner is fully reaped before any slot is reused.
            if unsafe { (&*GRAPH.0.get()).state(role) } != d::State::Quarantined {
                restarts[role] = Some(Restart {
                    due: time() + d::RETRY_NS,
                    mode: 0,
                    remaining: runnable_owners(),
                });
            }
        }
    }
    #[cfg(feature = "desktop-test")]
    {
        need(control > 10 && editor_ends == 3)?;
        need(
            runtime_memory::with(|r, _| r.inventory()) == empty
                && capabilities::inventory() == objects
                && crate::paging::runtime_frames() == frames,
        )?;
        waiting::settled()?;
        record(
            "m1.desktop_settled",
            "desktop",
            "ok",
            format_args!(
                "spaces=0 allocations=0 objects=0 handles=0 frames={frames} units=12 control={control} editor_ends=3 heap_cycles_per_owner=100 hidden_launches=0 reserve={} observers=false ready_max_ns={max_ready_ns} ready_limit_ns=320000000 elapsed_ns={}",
                d::RESERVE,
                time() - start
            ),
        );
    }
    #[cfg(feature = "desktop-test")]
    return Ok(());
}
