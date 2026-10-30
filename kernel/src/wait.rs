//! Bounded P3-08 policy core. The caller serializes observation and registration
//! under its single-CPU exclusion boundary. No pointers, pins, object references,
//! scheduler state or public handles are retained here. This is not yet wired to
//! the guest syscall adapter.
use crate::{
    Result, error, platform,
    scheduler::{CAPACITY, ThreadId},
};

pub const ITEMS: usize = 8;

/// Elapsed HPET epoch, independent of delivered timer interrupts. Counter deltas
/// must be observed at least once per half wrap. Exhaustion fails closed rather
/// than wrapping a deadline into the past. Failed observations leave state intact.
pub struct Clock {
    last: u64,
    ticks: u64,
    period_fs: u64,
}
impl Clock {
    pub fn new(counter: u64, period_fs: u64) -> Result<Self> {
        if period_fs == 0 || period_fs > 100_000_000 {
            return Err(error("clock", "period"));
        }
        Ok(Self {
            last: counter,
            ticks: 0,
            period_fs,
        })
    }
    pub fn resolution_ns(&self) -> u64 {
        self.period_fs.div_ceil(1_000_000)
    }
    pub fn observe(&mut self, counter: u64) -> Result<u64> {
        let delta = platform::delta(counter, self.last, 64)?;
        let ticks = self
            .ticks
            .checked_add(delta)
            .ok_or(error("clock", "overflow"))?;
        let ns = platform::nanos(ticks, self.period_fs)?;
        self.last = counter;
        self.ticks = ticks;
        Ok(ns)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Instant(pub u64);
impl Instant {
    pub fn after(self, duration_ns: u64) -> Result<Self> {
        self.0
            .checked_add(duration_ns)
            .map(Self)
            .ok_or(error("deadline", "overflow"))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Closed(usize),
    Cancelled,
    TimedOut,
    Ready(usize),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Readiness {
    Pending,
    Ready,
    Closed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Pending,
    Completed(Outcome),
}

#[derive(Clone, Copy)]
struct Record {
    thread: ThreadId,
    deadline: Option<Instant>,
    cancelled: bool,
    items: usize,
    state: State,
}
/// One outstanding wait per thread slot, including an uncollected completion.
/// Adapter-owned watch records must retain exact object/handle generations.
/// Completion is non-consuming readiness, not a reservation or IPC commit.
pub struct Waits {
    records: [Option<Record>; CAPACITY],
}
impl Waits {
    pub const EMPTY: Self = Self {
        records: [None; CAPACITY],
    };
    pub fn count(&self) -> usize {
        self.records.iter().flatten().count()
    }
    /// `items` is a trusted snapshot taken in the same exclusion interval as
    /// registration; the adapter must not unlock between those operations.
    /// Input time must come from the same nondecreasing Clock epoch throughout.
    pub fn register(
        &mut self,
        thread: ThreadId,
        deadline: Option<Instant>,
        now: Instant,
        items: &[Readiness],
        cancelled: bool,
    ) -> Result<State> {
        if thread.slot >= CAPACITY || thread.generation == 0 {
            return Err(error("wait", "identity"));
        }
        if items.is_empty() || items.len() > ITEMS {
            return Err(error("wait", "items"));
        }
        if self.records[thread.slot].is_some() {
            return Err(error("wait", "busy"));
        }
        let mut record = Record {
            thread,
            deadline,
            cancelled,
            items: items.len(),
            state: State::Pending,
        };
        settle(&mut record, now, items);
        let state = record.state;
        self.records[thread.slot] = Some(record);
        Ok(state)
    }
    fn record(&mut self, thread: ThreadId) -> Result<&mut Record> {
        self.records
            .get_mut(thread.slot)
            .and_then(Option::as_mut)
            .filter(|r| r.thread == thread)
            .ok_or(error("wait", "stale"))
    }
    /// Request alone does not acknowledge cancellation; observe confirms it.
    /// A committed completion cannot be overwritten by a later request.
    pub fn request_cancel(&mut self, thread: ThreadId) -> Result<()> {
        let r = self.record(thread)?;
        if r.state == State::Pending {
            r.cancelled = true;
        }
        Ok(())
    }
    /// The adapter supplies a fresh snapshot with the original item order and
    /// exact watched identities; it owns duplicate detection and watch storage.
    pub fn observe(
        &mut self,
        thread: ThreadId,
        now: Instant,
        items: &[Readiness],
    ) -> Result<State> {
        let r = self.record(thread)?;
        if items.len() != r.items {
            return Err(error("wait", "items"));
        }
        settle(r, now, items);
        Ok(r.state)
    }
    pub fn take(&mut self, thread: ThreadId) -> Result<Option<Outcome>> {
        let state = self.record(thread)?.state;
        if let State::Completed(outcome) = state {
            self.records[thread.slot] = None;
            Ok(Some(outcome))
        } else {
            Ok(None)
        }
    }
    /// Termination discards even completed waits. The adapter must remove its
    /// watch references before destroying owner/object storage; never wake owner.
    pub fn terminate(&mut self, thread: ThreadId) -> Result<()> {
        self.record(thread)?;
        self.records[thread.slot] = None;
        Ok(())
    }
}
fn settle(r: &mut Record, now: Instant, items: &[Readiness]) {
    if r.state != State::Pending {
        return;
    }
    let outcome = if let Some(i) = items.iter().position(|s| *s == Readiness::Closed) {
        Some(Outcome::Closed(i))
    } else if r.cancelled {
        Some(Outcome::Cancelled)
    } else if r.deadline.is_some_and(|d| now.0 >= d.0) {
        Some(Outcome::TimedOut)
    } else {
        items
            .iter()
            .position(|s| *s == Readiness::Ready)
            .map(Outcome::Ready)
    };
    if let Some(o) = outcome {
        r.state = State::Completed(o);
    }
}

/// Level timer: expiration stays ready until cancel/rearm. Periodic overruns are
/// counted once per observation and coalesced. Exhausting the next deadline
/// disarms after reporting the final expiration instead of wrapping/repeating.
#[derive(Clone, Copy)]
pub struct Timer {
    next: Option<Instant>,
    period: u64,
    ready: bool,
}
impl Timer {
    pub const EMPTY: Self = Self {
        next: None,
        period: 0,
        ready: false,
    };
    pub fn arm(&mut self, deadline: Instant, period_ns: u64) {
        self.next = Some(deadline);
        self.period = period_ns;
        self.ready = false;
    }
    pub fn cancel(&mut self) {
        *self = Self::EMPTY;
    }
    pub fn ready(&self) -> bool {
        self.ready
    }
    pub fn next(&self) -> Option<Instant> {
        self.next
    }
    pub fn observe(&mut self, now: Instant) -> Result<u64> {
        let Some(next) = self.next else {
            return Ok(0);
        };
        if now.0 < next.0 {
            return Ok(0);
        }
        let count = if self.period == 0 {
            1
        } else {
            ((now.0 - next.0) / self.period)
                .checked_add(1)
                .ok_or(error("timer", "overflow"))?
        };
        let following = if self.period == 0 {
            None
        } else {
            self.period
                .checked_mul(count)
                .and_then(|n| next.0.checked_add(n))
                .map(Instant)
        };
        self.ready = true;
        self.next = following;
        Ok(count)
    }
}

#[cfg(test)]
mod tests;
