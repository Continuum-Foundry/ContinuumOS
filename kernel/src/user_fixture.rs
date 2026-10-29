//! Explicit experimental flavor policy. No public operation selects return state.
use super::*;
type Value = core::result::Result<u64, u64>;
#[derive(Clone, Copy)]
pub(super) struct Hooks {
    pub entry_time: fn() -> u64,
    pub entry: fn(&mut Frame, u64),
    pub before_operation: fn(),
    pub operation: fn(u64, &Frame) -> Option<Value>,
    pub scheduled: fn() -> bool,
    pub return_to_user: fn(&Frame, u64),
}
const BOUNDARY: Hooks = Hooks {
    entry_time: || 0,
    entry: |f, _| inject_bad_return(f),
    before_operation: || {},
    operation: |_, _| None,
    scheduled: || false,
    return_to_user: |_, _| {},
};
static HOOKS: Local<Hooks> = Local(UnsafeCell::new(BOUNDARY));
pub(super) fn hooks() -> Hooks {
    // Single CPU; registration precedes user entry and never overlaps dispatch.
    unsafe { HOOKS.0.get().read() }
}
pub(super) fn install_boundary() {
    unsafe {
        HOOKS.0.get().write(BOUNDARY);
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
#[cfg(feature = "scheduler-test")]
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
        entry_time: crate::timer::sample_ns,
        entry: |f, entered| {
            scheduling::begin_syscall(entered);
            inject_bad_return(f);
        },
        before_operation: scheduling::check_fp_policy,
        operation,
        scheduled: scheduling::active,
        return_to_user: |f, operation| {
            if scheduling::active() {
                scheduling::syscall_return(f, operation == THREAD_YIELD && f.rax == 0);
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
#[cfg(feature = "scheduler-test")]
pub(super) fn install_scheduler() {
    scheduled::install(scheduled::HOOKS);
}
#[cfg(feature = "capability-test")]
pub(super) fn install_capabilities() {
    // Rights checking precedes scheduler policy, including yield/exit control.
    scheduled::install(Hooks {
        operation: |op, f| capabilities::dispatch(op, f).or_else(|| scheduled::operation(op, f)),
        ..scheduled::HOOKS
    });
}
