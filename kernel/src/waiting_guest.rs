//! Single-CPU wait adapter. All calls hold IF clear. Watches contain only copied
//! scalar handles; no user span, memory pin or registry borrow survives parking.
use super::*;
use continuum_kernel::{
    objects::waiting::{Interest, Watch, WatchSet},
    scheduler::{Queue, ThreadId},
    wait::{ITEMS, Instant, Outcome, Readiness, State, Waits},
};
#[derive(Clone, Copy)]
struct Blocked {
    id: ThreadId,
    set: WatchSet,
    cancel: Option<WatchSet>,
    since: u64,
    sequence: u64,
}
static BLOCKED: Local<[Option<Blocked>; 32]> = Local(UnsafeCell::new([None; 32]));
static WAITS: Local<Waits> = Local(UnsafeCell::new(Waits::EMPTY));
static ENABLED: Local<bool> = Local(UnsafeCell::new(false));
static HALTS: Local<u64> = Local(UnsafeCell::new(0));
static WAKES: Local<u64> = Local(UnsafeCell::new(0));
static SEQUENCE: Local<u64> = Local(UnsafeCell::new(1));
static READY_AT: Local<[Option<u64>; 32]> = Local(UnsafeCell::new([None; 32]));
pub(super) fn fixed_bytes() -> usize {
    core::mem::size_of::<[Option<Blocked>; 32]>()
        + core::mem::size_of::<Waits>()
        + core::mem::size_of::<[Option<u64>; 32]>()
        + 256
}
pub(super) fn clock_probe() -> Result<()> {
    let start = crate::timer::elapsed()?;
    for _ in 0..100 {
        core::hint::black_box(crate::timer::elapsed()?);
    }
    let observer = crate::timer::elapsed()? - start;
    let before_ticks = crate::timer::uptime()?.0;
    let before = crate::timer::elapsed()?;
    let target = before
        .checked_add(30_000_000)
        .ok_or(error("clock", "overflow"))?;
    let mut reached = false;
    for _ in 0..20_000_000 {
        if crate::timer::elapsed()? >= target {
            reached = true;
            break;
        }
        core::hint::spin_loop();
    }
    need(reached)?;
    let after = crate::timer::elapsed()?;
    let after_ticks = crate::timer::uptime()?.0;
    need(after - before >= 30_000_000 && before_ticks == after_ticks)?;
    record(
        "m1.wait_clock",
        "waits",
        "ok",
        format_args!(
            "source=hpet resolution_ns={} before_ns={before} after_ns={after} ticks_before={before_ticks} ticks_after={after_ticks} observer_samples=100 observer_total_ns={observer} irq_off=true",
            crate::timer::elapsed_resolution()
        ),
    );
    Ok(())
}
static ATTEMPT_DEADLINE: Local<Option<u64>> = Local(UnsafeCell::new(None));
pub(super) fn check_attempt_deadline() -> core::result::Result<(), u64> {
    if let Some(deadline) = unsafe { *ATTEMPT_DEADLINE.0.get() } {
        if now()?.0 >= deadline {
            return Err(TIMEDOUT);
        }
    }
    Ok(())
}
pub(super) fn enabled() -> bool {
    unsafe { *ENABLED.0.get() }
}
pub(crate) fn enable() {
    unsafe {
        *ENABLED.0.get() = true;
    }
}
fn now() -> core::result::Result<Instant, u64> {
    crate::timer::elapsed()
        .map(Instant)
        .map_err(|_| INTERNALFAILURE)
}
fn words<const N: usize>(address: u64) -> core::result::Result<[u64; N], u64> {
    let mut out = [0; N];
    for (i, v) in out.iter_mut().enumerate() {
        let at = address.checked_add(i as u64 * 8).ok_or(INVALIDARGUMENT)?;
        let mut bytes = [0; 8];
        copy(at, &mut bytes, false)?;
        *v = u64::from_le_bytes(bytes);
    }
    Ok(out)
}
fn cancel_set(space: SpaceId, h: u64) -> core::result::Result<Option<WatchSet>, u64> {
    if h == 0 {
        return Ok(None);
    }
    capabilities::registry(|c| {
        if c.info(space, h)? >> 32 != continuum_kernel::objects::Kind::Event as u64 {
            return Err(continuum_kernel::objects::Error::WrongType);
        }
        c.watch_set(
            space,
            &[Watch {
                handle: h,
                interest: Interest::Signalled,
            }],
        )
    })
    .map(Some)
    .map_err(capabilities::status)
}
fn cancellation(set: &Option<WatchSet>, time: Instant) -> core::result::Result<bool, u64> {
    let Some(set) = set else {
        return Ok(false);
    };
    let state =
        capabilities::registry(|c| c.watch_observe(set, time)).map_err(capabilities::status)?[0];
    match state {
        Readiness::Pending => Ok(false),
        Readiness::Ready => Ok(true),
        Readiness::Closed => Err(BADHANDLE),
    }
}
fn outcome(o: Outcome) -> (u64, u64) {
    match o {
        Outcome::Closed(_) => (PEERCLOSED, 0),
        Outcome::Cancelled => (CANCELLED, 0),
        Outcome::TimedOut => (TIMEDOUT, 0),
        Outcome::Ready(i) => (0, i as u64),
    }
}
pub(super) fn blocked(id: ThreadId) -> bool {
    unsafe { (&*BLOCKED.0.get())[id.slot] }.is_some_and(|b| b.id == id)
}
pub(super) fn blocked_thread(slot: usize) -> Option<ThreadId> {
    unsafe {
        (&*BLOCKED.0.get())
            .get(slot)
            .copied()
            .flatten()
            .map(|b| b.id)
    }
}
pub(super) fn pending_current() -> bool {
    let c = current();
    let id = scheduling::current_thread();
    blocked(id) && c.id.generation == id.generation
}
pub(super) fn terminate(id: ThreadId) {
    unsafe {
        (&mut *READY_AT.0.get())[id.slot] = None;
    }
    if blocked(id) {
        unsafe {
            (&mut *WAITS.0.get()).terminate(id).unwrap();
            (&mut *BLOCKED.0.get())[id.slot] = None;
        }
    }
}
/// Latch completion before a mutating caller can issue another operation. A
/// later reset, drain or cancellation cannot erase a registered notification.
/// Runnable queue admission remains a supervisor operation.
pub(super) fn notify() -> Result<()> {
    if !enabled() {
        return Ok(());
    }
    let time = crate::timer::elapsed().map(Instant)?;
    let mut previous = 0;
    for _ in 0..32 {
        let next = unsafe {
            (&*BLOCKED.0.get())
                .iter()
                .flatten()
                .filter(|b| b.sequence > previous)
                .min_by_key(|b| b.sequence)
                .copied()
        };
        let Some(b) = next else {
            break;
        };
        previous = b.sequence;
        let slot = b.id.slot;
        if unsafe { (&*READY_AT.0.get())[slot].is_some() } {
            continue;
        }
        let mut observations = capabilities::registry(|c| c.watch_observe(&b.set, time))
            .map_err(|_| error("wait", "observe"))?;
        let cancelled = match cancellation(&b.cancel, time) {
            Ok(v) => v,
            Err(BADHANDLE) => {
                observations[0] = Readiness::Closed;
                false
            }
            Err(_) => return Err(error("wait", "cancel_observe")),
        };
        let waits = unsafe { &mut *WAITS.0.get() };
        if cancelled {
            waits.request_cancel(b.id)?;
        }
        if let State::Completed(o) = waits.observe(b.id, time, &observations[..b.set.len()])? {
            unsafe {
                (&mut *READY_AT.0.get())[slot] = Some(time.0);
            }
            let (status, value) = outcome(o);
            record(
                "m1.wait_complete",
                "waits",
                "ok",
                format_args!(
                    "thread={slot} generation={} status={status} value={value} since_ns={} now_ns={} elapsed_ns={}",
                    b.id.generation,
                    b.since,
                    time.0,
                    time.0 - b.since
                ),
            );
        }
    }
    Ok(())
}
pub(super) fn poll(queue: &mut Queue) -> Result<()> {
    notify()?;
    let mut previous = 0;
    for _ in 0..32 {
        let next = unsafe {
            (&*BLOCKED.0.get())
                .iter()
                .flatten()
                .filter(|b| b.sequence > previous)
                .min_by_key(|b| b.sequence)
                .copied()
        };
        let Some(b) = next else {
            break;
        };
        previous = b.sequence;
        if let Some(o) = unsafe { (&mut *WAITS.0.get()).take(b.id) }? {
            let time = unsafe { (&*READY_AT.0.get())[b.id.slot] }
                .ok_or(error("wait", "missing_completion_time"))?;
            unsafe {
                (&mut *BLOCKED.0.get())[b.id.slot] = None;
                *WAKES.0.get() += 1;
            }
            let (status, value) = outcome(o);
            scheduling::complete_wait(b.id, status, value)?;
            queue.push(b.id, 0)?;
            record(
                "m1.wait_wake",
                "waits",
                "ok",
                format_args!(
                    "thread={} generation={} status={status} value={value} since_ns={} now_ns={time} elapsed_ns={}",
                    b.id.slot,
                    b.id.generation,
                    b.since,
                    time - b.since
                ),
            );
        }
    }
    Ok(())
}
pub(super) fn pending() -> bool {
    unsafe { (&*BLOCKED.0.get()).iter().any(Option::is_some) }
}
pub(super) fn idle() -> Result<()> {
    // No Rust references/pins are live here. Single CPU IF-clear check followed
    // immediately by STI/HLT closes the enqueue-vs-sleep interrupt window.
    need(runtime_memory::quiescent() && capabilities::quiescent())?;
    need(runtime_memory::with(|r, _| r.inventory().active_pins) == 0)?;
    unsafe {
        *HALTS.0.get() += 1;
        asm!("sti", "hlt", "cli");
    }
    Ok(())
}
pub(super) fn settled() -> Result<()> {
    need(!pending() && unsafe { (&*WAITS.0.get()).count() == 0 })?;
    need(unsafe { (&*READY_AT.0.get()).iter().all(Option::is_none) })?;
    record(
        "m1.wait_settled",
        "waits",
        "ok",
        format_args!(
            "records=0 watches=0 halts={} wakes={}",
            unsafe { *HALTS.0.get() },
            unsafe { *WAKES.0.get() }
        ),
    );
    Ok(())
}
pub(super) fn resume(id: ThreadId) -> Result<()> {
    let ready = unsafe { (&mut *READY_AT.0.get())[id.slot].take() };
    if let Some(ready) = ready {
        let resumed = crate::timer::elapsed()?;
        record(
            "m1.wait_resume",
            "waits",
            "ok",
            format_args!(
                "thread={} generation={} ready_ns={ready} resume_ns={resumed} wake_latency_ns={}",
                id.slot,
                id.generation,
                resumed - ready
            ),
        );
    }
    Ok(())
}
pub(super) fn dispatch(op: u64, f: &Frame) -> Option<core::result::Result<u64, u64>> {
    if !enabled() || !(CLOCK_NOW..=CHANNEL_RECEIVE_UNTIL).contains(&op) {
        return None;
    }
    let space = current().space;
    let mut declared_deadline = 0;
    let result = (|| {
        let time = now()?;
        match op {
            CLOCK_NOW => Ok(time.0),
            CLOCK_RESOLUTION => Ok(crate::timer::elapsed_resolution()),
            EVENT_CREATE => {
                if f.rsi > 1 {
                    return Err(INVALIDARGUMENT);
                }
                capabilities::registry(|c| c.event_create(space, f.rdi, f.rsi == 1))
                    .map_err(capabilities::status)
            }
            EVENT_SET => {
                if f.rsi > 1 {
                    return Err(INVALIDARGUMENT);
                }
                capabilities::registry(|c| c.event_set(space, f.rdi, f.rsi == 1))
                    .map(|_| 0)
                    .map_err(capabilities::status)
            }
            TIMER_CREATE => capabilities::registry(|c| c.timer_create(space, f.rdi))
                .map_err(capabilities::status),
            TIMER_CANCEL => capabilities::registry(|c| c.timer_cancel(space, f.rdi))
                .map(|_| 0)
                .map_err(capabilities::status),
            TIMER_ARM => {
                if f.rdx != 24 {
                    return Err(INVALIDARGUMENT);
                }
                let w = words::<3>(f.rsi)?;
                if w[2] != 0 || w[0] == u64::MAX {
                    return Err(INVALIDARGUMENT);
                }
                capabilities::registry(|c| c.timer_arm(space, f.rdi, Instant(w[0]), w[1]))
                    .map(|_| 0)
                    .map_err(capabilities::status)
            }
            WAIT_MANY => {
                if f.rsi != 32 {
                    return Err(INVALIDARGUMENT);
                }
                let w = words::<4>(f.rdi)?;
                declared_deadline = w[2];
                if w[1] == 0 || w[1] > ITEMS as u64 {
                    return Err(INVALIDARGUMENT);
                }
                let mut watches = [Watch {
                    handle: 0,
                    interest: Interest::Signalled,
                }; ITEMS];
                for i in 0..w[1] as usize {
                    let at = w[0].checked_add(i as u64 * 32).ok_or(INVALIDARGUMENT)?;
                    let d = words::<4>(at)?;
                    let interest = match d[1] {
                        0 if d[2] == 0 && d[3] == 0 => Interest::Signalled,
                        1 if d[2] == 0 && d[3] == 0 => Interest::Exited,
                        2 if d[2] == 0 && d[3] == 0 => Interest::Readable,
                        3 => Interest::Writable {
                            bytes: usize::try_from(d[2]).map_err(|_| INVALIDARGUMENT)?,
                            attachments: usize::try_from(d[3]).map_err(|_| INVALIDARGUMENT)?,
                        },
                        _ => return Err(INVALIDARGUMENT),
                    };
                    watches[i] = Watch {
                        handle: d[0],
                        interest,
                    };
                }
                let set = capabilities::registry(|c| c.watch_set(space, &watches[..w[1] as usize]))
                    .map_err(capabilities::status)?;
                let cancel = cancel_set(space, w[3])?;
                (fixture::hooks().wait_boundary)(0, watches[0].handle);
                let fresh = now()?; // all input copies/pins ended before sampling
                let _initial = capabilities::registry(|c| c.watch_observe(&set, fresh))
                    .map_err(capabilities::status)?;
                (fixture::hooks().wait_boundary)(1, watches[0].handle);
                let fresh = now()?;
                let observations = capabilities::registry(|c| c.watch_observe(&set, fresh))
                    .map_err(capabilities::status)?;
                let cancelled = cancellation(&cancel, fresh)?;
                let id = scheduling::current_thread();
                let state = unsafe {
                    (&mut *WAITS.0.get()).register(
                        id,
                        if w[2] == u64::MAX {
                            None
                        } else {
                            Some(Instant(w[2]))
                        },
                        fresh,
                        &observations[..set.len()],
                        cancelled,
                    )
                }
                .map_err(|_| QUOTAEXCEEDED)?;
                if let State::Completed(o) = state {
                    (fixture::hooks().wait_boundary)(2, watches[0].handle);
                    unsafe {
                        (&mut *WAITS.0.get()).take(id).unwrap();
                    }
                    let (status, value) = outcome(o);
                    if status == 0 { Ok(value) } else { Err(status) }
                } else {
                    let sequence = unsafe { *SEQUENCE.0.get() };
                    if sequence == 0 {
                        unsafe {
                            (&mut *WAITS.0.get()).terminate(id).unwrap();
                        }
                        return Err(QUOTAEXCEEDED);
                    }
                    unsafe {
                        *SEQUENCE.0.get() = sequence.checked_add(1).unwrap_or(0);
                    }
                    need(runtime_memory::with(|r, _| r.inventory().active_pins) == 0)
                        .map_err(|_| INTERNALFAILURE)?;
                    unsafe {
                        (&mut *BLOCKED.0.get())[id.slot] = Some(Blocked {
                            id,
                            set,
                            cancel,
                            since: fresh.0,
                            sequence,
                        });
                    }
                    record(
                        "m1.wait_park",
                        "waits",
                        "ok",
                        format_args!(
                            "thread={} generation={} items={} deadline_ns={} cancel={} now_ns={}",
                            id.slot,
                            id.generation,
                            set.len(),
                            w[2],
                            w[3] != 0,
                            fresh.0
                        ),
                    );
                    (fixture::hooks().wait_boundary)(2, watches[0].handle);
                    Ok(0) // provisional scalar replaced only on confirmed wake
                }
            }
            CHANNEL_SEND_UNTIL | CHANNEL_RECEIVE_UNTIL => {
                // One atomic attempt. Blocking SDK composition retries after a
                // readiness wait with this same absolute deadline/cancel token.
                if f.rdx != 24 {
                    return Err(INVALIDARGUMENT);
                }
                let w = words::<3>(f.rsi)?;
                declared_deadline = w[1];
                let interest = if op == CHANNEL_SEND_UNTIL {
                    Interest::Writable {
                        bytes: 0,
                        attachments: 0,
                    }
                } else {
                    Interest::Readable
                };
                let endpoint = capabilities::registry(|c| {
                    c.watch_set(
                        space,
                        &[Watch {
                            handle: f.rdi,
                            interest,
                        }],
                    )
                })
                .map_err(capabilities::status)?;
                let observation_time = now()?;
                if capabilities::registry(|c| c.watch_observe(&endpoint, observation_time))
                    .map_err(capabilities::status)?[0]
                    == Readiness::Closed
                {
                    return Err(PEERCLOSED);
                }
                let cancel = cancel_set(space, w[2])?;
                if cancellation(&cancel, now()?)? {
                    return Err(CANCELLED);
                }
                if w[1] != u64::MAX && now()?.0 >= w[1] {
                    return Err(TIMEDOUT);
                }
                let mut attempt = *f;
                attempt.rsi = w[0];
                attempt.rdx = if op == CHANNEL_SEND_UNTIL { 32 } else { 48 };
                unsafe {
                    *ATTEMPT_DEADLINE.0.get() = if w[1] == u64::MAX { None } else { Some(w[1]) };
                }
                let value = capabilities::dispatch(
                    if op == CHANNEL_SEND_UNTIL {
                        CHANNEL_SEND
                    } else {
                        CHANNEL_RECEIVE
                    },
                    &attempt,
                )
                .unwrap();
                let value = if value == Err(WOULDBLOCK) {
                    check_attempt_deadline().and(value)
                } else {
                    value
                };
                unsafe {
                    *ATTEMPT_DEADLINE.0.get() = None;
                }
                value
            }
            _ => Err(UNSUPPORTED),
        }
    })();
    let (status, value) = match result {
        Ok(v) => (0, v),
        Err(e) => (e, 0),
    };
    let pending = op == WAIT_MANY && pending_current();
    let observed_ns = crate::timer::elapsed().unwrap_or(0);
    record(
        "m1.wait_call",
        "waits",
        "ok",
        format_args!(
            "process={} generation={} operation={op} status={status} value={value} pending={pending} deadline_ns={declared_deadline} observed_ns={observed_ns}",
            current().id.slot,
            current().id.generation
        ),
    );
    Some(result)
}
