//! Bounded single-CPU equal-priority policy. Mechanism lives in user_entry.
use crate::{Result, error};
pub const CAPACITY: usize = 32;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThreadId {
    pub slot: usize,
    pub generation: u64,
}
pub struct Queue {
    slots: [Option<ThreadId>; CAPACITY],
    head: usize,
    len: usize,
}
impl Queue {
    pub const EMPTY: Self = Self {
        slots: [None; CAPACITY],
        head: 0,
        len: 0,
    };
    pub fn push(&mut self, id: ThreadId, priority: u8) -> Result<()> {
        if priority != 0 {
            return Err(error("scheduler", "priority_unsupported"));
        }
        if id.slot >= CAPACITY || id.generation == 0 {
            return Err(error("scheduler", "identity"));
        }
        if self.slots.iter().flatten().any(|v| v.slot == id.slot) {
            return Err(error("scheduler", "duplicate"));
        }
        if self.len == CAPACITY {
            return Err(error("scheduler", "quota"));
        }
        self.slots[(self.head + self.len) % CAPACITY] = Some(id);
        self.len += 1;
        Ok(())
    }
    pub fn pop(&mut self) -> Option<ThreadId> {
        if self.len == 0 {
            return None;
        }
        let id = self.slots[self.head].take();
        self.head = (self.head + 1) % CAPACITY;
        self.len -= 1;
        id
    }
}
/// No XSAVE-managed state is admitted. XCR0 is irrelevant with OSXSAVE clear.
pub fn cpu_supported(edx: u32, cr4: u64, required: u64) -> bool {
    edx & ((7 << 24) | 1) == ((7 << 24) | 1) && cr4 & (1 << 18) == 0 && required & !7 == 0
}
/// A trusted compute ceiling cannot erase an exit or fault from that dispatch.
pub fn compute_budget_result(result: Option<u64>, dispatches: u64, limit: u64) -> Option<u64> {
    result.or_else(|| (dispatches >= limit).then_some(0x300))
}
#[repr(C, align(16))]
#[derive(Clone, Copy)]
pub struct FxState(pub [u8; 512]);
impl FxState {
    pub fn supported_controls(&self) -> bool {
        let mxcsr = u32::from_le_bytes(self.0[24..28].try_into().unwrap());
        // TCG does not deliver unmasked SSE arithmetic traps. Keep that control
        // mode unsupported; all rounding modes and masked sticky flags survive.
        mxcsr & 0x1f80 == 0x1f80
    }
    pub const fn fresh() -> Self {
        let mut b = [0; 512];
        b[0] = 0x7f;
        b[1] = 3;
        b[24] = 0x80;
        b[25] = 0x1f;
        Self(b)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fault_and_exit_take_precedence_at_compute_budget_boundary() {
        for status in [0, 94, 99, 0x200, 0x400] {
            for dispatches in [15, 16, 17, u64::MAX] {
                assert_eq!(
                    compute_budget_result(Some(status), dispatches, 16),
                    Some(status)
                );
            }
        }
        assert_eq!(compute_budget_result(None, 15, 16), None);
        assert_eq!(compute_budget_result(None, 16, 16), Some(0x300));
        assert_eq!(compute_budget_result(None, 17, 16), Some(0x300));
    }
    #[test]
    fn full_queue_round_robin_reuse_and_priority_denial() {
        let mut q = Queue::EMPTY;
        for slot in 0..CAPACITY {
            q.push(
                ThreadId {
                    slot,
                    generation: 1,
                },
                0,
            )
            .unwrap();
        }
        for _ in 0..64 {
            for slot in 0..CAPACITY {
                let id = q.pop().unwrap();
                assert_eq!(id.slot, slot);
                assert!(q.push(id, 1).is_err());
                q.push(id, 0).unwrap();
                assert!(
                    q.push(
                        ThreadId {
                            generation: 2,
                            ..id
                        },
                        0
                    )
                    .is_err()
                );
            }
        }
        for slot in 0..CAPACITY {
            assert_eq!(q.pop().unwrap().slot, slot);
        }
        assert_eq!(q.pop(), None);
        q.push(
            ThreadId {
                slot: 0,
                generation: 2,
            },
            0,
        )
        .unwrap();
        assert_eq!(q.pop().unwrap().generation, 2);
        assert!(
            q.push(
                ThreadId {
                    slot: 32,
                    generation: 1
                },
                0
            )
            .is_err()
        );
    }
    #[test]
    fn cpu_gate_and_pristine_state() {
        let mask = (7 << 24) | 1;
        assert!(cpu_supported(mask, 0, 7));
        for bit in [0, 24, 25, 26] {
            assert!(!cpu_supported(mask & !(1 << bit), 0, 7));
        }
        assert!(!cpu_supported(mask, 1 << 18, 7));
        assert!(!cpu_supported(mask, 0, 8));
        let f = FxState::fresh();
        assert_eq!(&f.0[..2], &[0x7f, 3]);
        assert!(f.0[32..].iter().all(|v| *v == 0));
        assert_eq!(core::mem::align_of::<FxState>(), 16);
    }
    #[test]
    fn rejects_each_unmasked_sse_control_without_narrowing_rounding_or_sticky_flags() {
        for rounding in 0..4 {
            for flags in 0..64 {
                let mut f = FxState::fresh();
                f.0[24..28].copy_from_slice(&(0x1f80u32 | (rounding << 13) | flags).to_le_bytes());
                assert!(f.supported_controls());
                for bit in 7..13 {
                    let mut bad = f;
                    let value = u32::from_le_bytes(bad.0[24..28].try_into().unwrap()) & !(1 << bit);
                    bad.0[24..28].copy_from_slice(&value.to_le_bytes());
                    assert!(!bad.supported_controls());
                }
            }
        }
    }
}
