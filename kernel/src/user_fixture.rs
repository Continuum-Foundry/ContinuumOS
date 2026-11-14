//! Explicit experimental flavor policy. No public operation selects return state.
use super::*;
type Value = core::result::Result<u64, u64>;
#[derive(Clone, Copy)]
pub(super) struct Hooks {
    // Read-only policy query; callers may consult it before diagnostic work.
    pub record_event: fn(&str) -> bool,
    pub entry_time: fn() -> u64,
    pub entry: fn(&mut Frame, u64),
    pub before_operation: fn(),
    pub operation: fn(u64, &Frame) -> Option<Value>,
    pub scheduled: fn() -> bool,
    pub return_to_user: fn(&Frame, u64),
    pub copy_observer: fn(bool),
    pub wait_boundary: fn(u64, u64),
    pub supervise: fn(&mut continuum_kernel::scheduler::Queue, u64) -> Result<()>,
    pub expected_result: fn(u64, usize) -> Option<u64>,
    pub object_waves: Option<u64>,
    pub process_state: fn(u64, u64) -> u64,
    pub display_copy: fn(u64, u64, u64, u64, u64, u64, u64),
    pub desktop_mode: fn(usize, u64) -> u64,
    pub desktop_stop_control: fn(u64, u64) -> bool,
}
const BOUNDARY: Hooks = Hooks {
    record_event: |_| true,
    entry_time: || 0,
    entry: |f, _| inject_bad_return(f),
    before_operation: || {},
    operation: |_, _| None,
    scheduled: || false,
    return_to_user: |_, _| {},
    copy_observer: |_| {},
    wait_boundary: |_, _| {},
    supervise: |_, _| Ok(()),
    expected_result: |_, _| None,
    object_waves: None,
    process_state: |_, state| state,
    display_copy: |_, _, _, _, _, _, _| {},
    desktop_mode: |_, _| 0,
    desktop_stop_control: |_, _| false,
};
static HOOKS: Local<Hooks> = Local(UnsafeCell::new(BOUNDARY));
#[cfg(feature = "desktop-test")]
static DESKTOP_OPERATION: Local<u64> = Local(UnsafeCell::new(0));
pub(super) fn hooks() -> Hooks {
    // Single CPU; registration precedes user entry and never overlaps dispatch.
    unsafe { HOOKS.0.get().read() }
}
pub(super) fn install_boundary() {
    unsafe {
        HOOKS.0.get().write(BOUNDARY);
    }
}
#[cfg(feature = "desktop-runtime")]
pub(super) fn install_desktop() {
    // Runtime dispatch only. No progress observer, injected returns, IRQ probe,
    // process-state override or acceptance callback is installed.
    let hooks = Hooks {
        // Ordinary steady-state waits are quiet; faults, lifecycle failures and
        // serial recovery remain observable after the readiness checkpoint.
        record_event: |event| {
            !scheduling::desktop::ready()
                || !matches!(
                    event,
                    "m1.wait_call"
                        | "m1.wait_park"
                        | "m1.wait_wake"
                        | "m1.wait_resume"
                        | "m1.wait_complete"
                        | "m1.user_log"
                        | "m1.cap_call"
                )
        },
        entry_time: crate::timer::sample_ns,
        entry: |_, entered| scheduling::begin_syscall(entered),
        before_operation: scheduling::check_fp_policy,
        scheduled: scheduling::active,
        operation: |op, f| {
            let c = current();
            if let Some(result) = crate::display_guest::call(
                op,
                c.case - 100,
                continuum_kernel::device::Owner {
                    process: c.id.slot as u64 + 1,
                    generation: c.id.generation,
                },
                f.rdi,
                f.rsi,
                f.rdx,
                f.r10,
                |a, b| copy(a, b, false),
            ) {
                return Some(result.map_err(|_| ACCESSDENIED));
            }
            #[cfg(feature = "device-test")]
            {
                let c = current();
                if let Some(result) = crate::device_guest::call(
                    op,
                    c.case - 100,
                    c.id.slot as u64 + 1,
                    c.id.generation,
                    f.rdi,
                    f.rsi,
                    f.rdx,
                ) {
                    return Some(result.map_err(|_| ACCESSDENIED));
                }
            }
            match op {
                ABI_QUERY | DIAGNOSTIC_LOG => None,
                PROCESS_EXIT | THREAD_YIELD => capabilities::dispatch(op, f).or_else(|| {
                    if op == THREAD_YIELD {
                        Some(Ok(0))
                    } else {
                        None
                    }
                }),
                CLOCK_NOW | CLOCK_RESOLUTION | TIMER_ARM | WAIT_MANY => waiting::dispatch(op, f),
                MEMORY_ALLOCATE | MEMORY_FREE => Some(Err(UNSUPPORTED)),
                OBJECT_INFO..=CONTEXT_LIVE => Some(Err(ACCESSDENIED)),
                _ => Some(Err(UNSUPPORTED)),
            }
        },
        return_to_user: |f, operation| {
            waiting::notify().expect("desktop wait boundary");
            if operation == THREAD_YIELD || waiting::pending_current() {
                scheduling::desktop::cooperated();
            }
            scheduling::syscall_return(f, operation == THREAD_YIELD || waiting::pending_current());
        },
        ..BOUNDARY
    };
    #[cfg(feature = "desktop-test")]
    let hooks = Hooks {
        entry: |f, entered| {
            unsafe {
                *DESKTOP_OPERATION.0.get() = f.rax;
            }
            scheduling::begin_syscall(entered);
        },
        // Preserve each wait result (including TimedOut), actual resume/latency,
        // lifecycle, authority and heap outcome. Do not format redundant copies
        // of the same wait transition or routine clock/arm confirmations.
        record_event: |event| {
            !matches!(event, "m1.wait_park" | "m1.wait_complete" | "m1.wait_wake")
                && !(event == "m1.wait_call"
                    && matches!(unsafe { *DESKTOP_OPERATION.0.get() }, CLOCK_NOW | TIMER_ARM))
        },
        desktop_mode: |_, attempt| 1 + attempt,
        desktop_stop_control: |control, ends| control > 10 && ends == 3,
        expected_result: |attempt, role| {
            Some(if role == 4 {
                [42, 90, 0x300][attempt as usize]
            } else if role == 5 {
                91
            } else {
                0
            })
        },
        ..hooks
    };
    #[cfg(feature = "display-test")]
    let hooks = Hooks {
        display_copy: |process, generation, g, width, height, placement, copy_ns| {
            if placement >> 32 == height - 1 {
                record(
                    "m1.display_pattern",
                    "display",
                    "ok",
                    format_args!(
                        "process={process} generation={generation} output_generation={g} rows={height} width={width} source_released=true copy_ns={copy_ns} scanout_observed=false"
                    ),
                );
            }
        },
        ..hooks
    };
    unsafe {
        HOOKS.0.get().write(hooks);
    }
}
fn inject_bad_return(f: &mut Frame) {
    // Trusted fixture selection, independent of the writable guest case word.
    // The ordinary return validator must reject both negative-test mutations.
    match current().case {
        12 => f.ip = 0x800000000000,
        13 => f.flags |= 0x3000,
        _ => {}
    }
}
#[cfg(any(feature = "scheduler-test", feature = "desktop-runtime"))]
mod scheduled {
    use super::*;
    // Fixture control page is mapped RW before entry and survives until reap.
    // Sampling occurs with the current owner root at capture and finish, exactly
    // where task progress was observed before introducing this policy seam.
    const PROGRESS: u64 = 0x70000008;
    fn observe_progress() -> u64 {
        unsafe { (PROGRESS as *const u64).read_volatile() }
    }
    pub(super) const HOOKS: Hooks = Hooks {
        record_event: |_| true,
        entry_time: crate::timer::sample_ns,
        entry: |f, entered| {
            scheduling::begin_syscall(entered);
            inject_bad_return(f);
        },
        before_operation: scheduling::check_fp_policy,
        copy_observer: scheduling::copy_irq,
        wait_boundary: |_, _| {},
        supervise: |_, _| Ok(()),
        expected_result: |_, _| None,
        object_waves: None,
        process_state: |_, state| state,
        display_copy: |_, _, _, _, _, _, _| {},
        desktop_mode: |_, _| 0,
        desktop_stop_control: |_, _| false,
        operation,
        scheduled: scheduling::active,
        return_to_user: |f, operation| {
            if scheduling::active() {
                #[cfg(any(feature = "wait-test", feature = "desktop-runtime"))]
                waiting::notify().expect("wait notification boundary");
                let voluntary = operation == THREAD_YIELD && f.rax == 0;
                #[cfg(any(feature = "wait-test", feature = "desktop-runtime"))]
                let voluntary = voluntary || waiting::pending_current();
                scheduling::syscall_return(f, voluntary);
            }
        },
    };
    pub(super) fn install(hooks: Hooks) {
        scheduling::observe_progress(observe_progress);
        unsafe {
            super::HOOKS.0.get().write(hooks);
        }
    }
    pub(super) fn operation(op: u64, f: &Frame) -> Option<Value> {
        if !scheduling::active() {
            return None;
        }
        match op {
            MEMORY_ALLOCATE if f.rdi > 65536 => Some(Err(QUOTAEXCEEDED)),
            THREAD_YIELD => Some(Ok(0)),
            PROCESS_EXIT => {
                scheduling::copy_irq(false);
                None
            }
            _ => None,
        }
    }
}
#[cfg(any(feature = "scheduler-test", feature = "desktop-runtime"))]
pub(super) fn install_scheduler() {
    scheduled::install(scheduled::HOOKS);
}
#[cfg(any(feature = "capability-test", feature = "desktop-runtime"))]
pub(super) fn install_capabilities() {
    // Rights checking precedes scheduler policy, including yield/exit control.
    let hooks = Hooks {
        operation: |op, f| {
            #[cfg(any(feature = "wait-test", feature = "desktop-runtime"))]
            if let Some(value) = waiting::dispatch(op, f) {
                return Some(value);
            }
            capabilities::dispatch(op, f).or_else(|| scheduled::operation(op, f))
        },
        ..scheduled::HOOKS
    };
    #[cfg(any(feature = "wait-test", feature = "desktop-runtime"))]
    let hooks = if waiting::enabled() {
        Hooks {
            copy_observer: |_| {},
            wait_boundary: wait_boundary_probe,
            supervise: wait_supervision_probe,
            expected_result: |wave, slot| {
                if wave == 4 && slot == 1 {
                    Some(0x300)
                } else {
                    None
                }
            },
            object_waves: Some(6),
            ..hooks
        }
    } else {
        hooks
    };
    scheduled::install(hooks);
}
#[cfg(feature = "supervisor-test")]
pub(super) fn install_supervisor() {
    scheduled::install(Hooks {
        process_state: scheduling::supervision::observe_state,
        copy_observer: |_| {},
        entry: |_, entered| scheduling::begin_syscall(entered),
        operation: |op, f| {
            scheduling::supervision::dispatch(op, f)
                .or_else(|| waiting::dispatch(op, f))
                .or_else(|| capabilities::dispatch(op, f))
                .or_else(|| scheduled::operation(op, f))
        },
        ..scheduled::HOOKS
    });
}
#[cfg(any(feature = "wait-test", feature = "desktop-runtime"))]
fn wait_supervision_probe(queue: &mut continuum_kernel::scheduler::Queue, wave: u64) -> Result<()> {
    if wave == 4 {
        if let Some(id) = waiting::blocked_thread(1) {
            if scheduling::observed_progress(id) == Some(900) {
                scheduling::kill_waiting(id, queue)?;
            }
        }
    }
    Ok(())
}
#[cfg(any(feature = "wait-test", feature = "desktop-runtime"))]
fn wait_boundary_probe(phase: u64, watched: u64) {
    const CONTROL: u64 = 0x70000008;
    let selector = unsafe { (CONTROL as *const u64).read_volatile() };
    if selector % 10 != phase || !(700..=752).contains(&selector) {
        return;
    }
    unsafe {
        (CONTROL as *mut u64).write_volatile(0);
    }
    let space = current().space;
    match selector / 10 {
        70 => capabilities::registry(|c| c.event_set(space, watched, true)).unwrap(),
        71 => {
            let sender = unsafe { ((CONTROL + 8 * 6) as *const u64).read_volatile() };
            capabilities::registry(|c| c.channel_send(space, sender, b"boundary", &[])).unwrap();
        }
        72 => {
            runtime_memory::with(|r, b| capabilities::registry(|c| c.close(r, b, space, watched)))
                .unwrap()
        }
        73 => {
            let cancel = unsafe { ((CONTROL + 8 * 6) as *const u64).read_volatile() };
            capabilities::registry(|c| {
                c.event_set(space, watched, true)?;
                c.event_set(space, cancel, true)
            })
            .unwrap();
        }
        74 => {
            let before = crate::timer::elapsed().unwrap();
            let ticks = crate::timer::uptime().unwrap().0;
            let mut completed = false;
            for _ in 0..20_000_000 {
                if crate::timer::elapsed().unwrap() - before >= 30_000_000 {
                    completed = true;
                    break;
                }
                core::hint::spin_loop();
            }
            assert!(completed && crate::timer::uptime().unwrap().0 == ticks);
            let after = crate::timer::elapsed().unwrap();
            record(
                "m1.wait_delay",
                "waits",
                "ok",
                format_args!(
                    "process={} generation={} before_ns={before} after_ns={after} ticks_before={ticks} ticks_after={ticks} irq_off=true",
                    current().id.slot,
                    current().id.generation
                ),
            );
        }
        75 => {
            capabilities::registry(|c| c.event_set(space, watched, true)).unwrap();
            waiting::notify().unwrap();
            capabilities::registry(|c| c.event_set(space, watched, false)).unwrap();
        }
        _ => unreachable!(),
    }
    record(
        "m1.wait_boundary",
        "waits",
        "ok",
        format_args!(
            "process={} generation={} phase={phase} action={}",
            current().id.slot,
            current().id.generation,
            selector / 10
        ),
    );
}

#[cfg(feature = "policy-test")]
pub(super) fn install_policy() {
    scheduled::install(Hooks {
        copy_observer: |_| {},
        entry: |_, entered| scheduling::begin_syscall(entered),
        operation: |op, f| {
            waiting::dispatch(op, f)
                .or_else(|| capabilities::dispatch(op, f))
                .or_else(|| scheduled::operation(op, f))
        },
        ..scheduled::HOOKS
    });
}

#[cfg(feature = "route-test")]
static ROUTE_OPERATION: Local<u64> = Local(UnsafeCell::new(0));
#[cfg(feature = "route-test")]
pub(super) fn install_routes() {
    install_policy();
    scheduled::install(Hooks {
        entry: |f, entered| {
            unsafe {
                *ROUTE_OPERATION.0.get() = f.rax;
            }
            scheduling::begin_syscall(entered);
        },
        // Bound this story's declared diagnostic profile without changing any
        // dispatch, wakeup, timeout or resource behavior. Other flavors retain
        // their full historical trace. Sender, clock/deadline calls, lifecycle,
        // faults, authority decisions and settlements remain recorded.
        record_event: |event| {
            !(event == "m1.wait_call" && unsafe { *ROUTE_OPERATION.0.get() } == CLOCK_NOW)
                && !matches!(
                    event,
                    "m1.wait_complete"
                        | "m1.wait_wake"
                        | "m1.wait_resume"
                        | "m1.wait_park"
                        | "m1.ipc_call"
                        | "m1.cap_call"
                )
        },
        ..hooks()
    });
}

#[cfg(feature = "stress-test")]
pub(super) fn install_stress() {
    install_policy();
    scheduled::install(Hooks {
        entry: |f, entered| scheduling::routes::entry(f.rax, f.rdi, entered),
        copy_observer: scheduling::routes::copy_observer,
        wait_boundary: |phase, watched| {
            if current().case == 902
                && scheduling::routes::race_pending()
                && (phase == 1 || phase == 2)
            {
                capabilities::registry(|c| c.event_set(current().space, watched, phase == 1))
                    .unwrap();
                if phase == 2 {
                    scheduling::routes::race_complete();
                }
            }
        },
        return_to_user: scheduling::routes::return_observer,
        record_event: |event| {
            !(event == "m1.user_log"
                && scheduling::routes::diagnostic_benchmark()
                && !cfg!(feature = "stress-trace"))
                && !matches!(
                    event,
                    "m1.wait_call"
                        | "m1.wait_complete"
                        | "m1.wait_wake"
                        | "m1.wait_resume"
                        | "m1.wait_park"
                        | "m1.ipc_call"
                        | "m1.ipc_sender"
                        | "m1.cap_call"
                        | "m1.user_call"
                )
        },
        ..hooks()
    });
}
