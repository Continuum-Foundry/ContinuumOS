//! Trusted P4-02 profile. This is internal policy, not an extension of SDK v1.
use crate::{Result, error, user_image::Context};
pub const MIB: u64 = 1024 * 1024;
pub const HEAP_BASE: u64 = 0x4000_0000;
pub const CONFIG: u64 = 0x7000_0000;
pub const LIFETIME: u64 = 24;
pub const RESERVE: u64 = 16 * MIB;
pub const ACTIVE: u64 = 384 * MIB;
pub const OVERHEAD: u64 = 2 * MIB; // image, stacks, both table banks, config and metadata
pub const ROLES: [&str; 7] = [
    "supervisor",
    "compositor",
    "input",
    "shell",
    "editor",
    "update",
    "policy",
];
pub const HEAPS: [u64; 7] = [
    2 * MIB,
    48 * MIB,
    2 * MIB,
    16 * MIB,
    64 * MIB,
    32 * MIB,
    4 * MIB,
];
pub const RETRY_NS: u64 = 20_000_000;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Vacant,
    Running,
    Stopped,
    Quarantined,
}
pub struct Graph {
    states: [State; 7],
    attempts: [u8; 7],
    after: [u64; 7],
    consumed: u64,
}
impl Graph {
    pub const EMPTY: Self = Self {
        states: [State::Vacant; 7],
        attempts: [0; 7],
        after: [0; 7],
        consumed: 0,
    };
    pub fn required() -> u64 {
        HEAPS.iter().sum::<u64>() + 7 * OVERHEAD
    }
    pub fn preflight(available: u64) -> Result<()> {
        if Self::required() > ACTIVE || available < Self::required() + RESERVE {
            return Err(error("desktop", "capacity"));
        }
        Ok(())
    }
    pub fn context(role: usize) -> Result<Context> {
        let bytes = *HEAPS.get(role).ok_or(error("desktop", "role"))?;
        Ok(Context {
            principal: 400 + role as u64,
            session: 1001,
            service: role != 3 && role != 4,
            grant_ceiling: 1,
            commitment_ceiling: bytes + OVERHEAD,
        })
    }
    pub fn check(&self, role: usize, now: u64) -> Result<()> {
        let state = *self.states.get(role).ok_or(error("desktop", "role"))?;
        if state == State::Quarantined || self.attempts[role] >= 3 {
            return Err(error("desktop", "quarantined"));
        }
        // Seven initial roles plus two prepaid retries each = 21, leaving three
        // lifetime units and nine slots for trusted recovery; no caller can spend them.
        if self.consumed >= LIFETIME - 3 || state == State::Running {
            return Err(error("desktop", "lifetime"));
        }
        if now < self.after[role] {
            return Err(error("desktop", "backoff"));
        }
        Ok(())
    }
    /// Commit only after image, heap, grants and queue admission all succeed.
    pub fn started(&mut self, role: usize, now: u64) -> Result<()> {
        self.check(role, now)?;
        self.states[role] = State::Running;
        self.attempts[role] += 1;
        self.consumed += 1;
        Ok(())
    }
    pub fn stopped(&mut self, role: usize, now: u64) -> Result<()> {
        if self.states.get(role) != Some(&State::Running) {
            return Err(error("desktop", "state"));
        }
        self.after[role] = now
            .checked_add(RETRY_NS)
            .ok_or(error("desktop", "overflow"))?;
        self.states[role] = if self.attempts[role] == 3 {
            State::Quarantined
        } else {
            State::Stopped
        };
        Ok(())
    }
    pub fn consumed(&self) -> u64 {
        self.consumed
    }
    pub fn quarantine(&mut self, role: usize) -> Result<()> {
        let state = self.states.get_mut(role).ok_or(error("desktop", "role"))?;
        if *state == State::Running {
            return Err(error("desktop", "state"));
        }
        *state = State::Quarantined;
        Ok(())
    }
    pub fn state(&self, role: usize) -> State {
        self.states[role]
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reserved_capacity_and_failed_start_do_not_spend_credits() {
        assert!(Graph::preflight(Graph::required() + RESERVE - 1).is_err());
        assert!(Graph::preflight(Graph::required() + RESERVE).is_ok());
        let g = Graph::EMPTY;
        assert!(g.check(7, 0).is_err());
        assert_eq!(g.consumed(), 0);
        for role in 0..7 {
            let c = Graph::context(role).unwrap();
            assert_eq!(c.grant_ceiling, 1);
            assert_eq!(c.commitment_ceiling, HEAPS[role] + OVERHEAD);
        }
    }
    #[test]
    fn every_role_has_two_restarts_with_no_refund_and_recovery_is_unspendable() {
        let mut g = Graph::EMPTY;
        for attempt in 0..3 {
            let time = attempt * RETRY_NS;
            for role in 0..7 {
                g.started(role, time).unwrap();
                assert!(g.check(role, time).is_err());
                g.stopped(role, time).unwrap();
                assert!(g.check(role, time).is_err());
            }
        }
        assert_eq!(g.consumed(), 21);
        for role in 0..7 {
            assert_eq!(g.state(role), State::Quarantined);
            assert!(g.check(role, u64::MAX).is_err());
        }
    }
}
