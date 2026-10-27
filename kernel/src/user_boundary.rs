//! Pure validation shared with host tests. Experimental P3-04, no ABI freeze.
use crate::owned_memory::USER_END;
pub const USER_CS: u64 = 0x33;
pub const USER_SS: u64 = 0x3b;
pub const ARITHMETIC: u64 = 0x8d5; // OF/SF/ZF/AF/PF/CF
pub const USER_FLAGS: u64 = ARITHMETIC | 0x202 | 0x400; // IF, bit 1, DF
pub fn return_state(ip: u64, sp: u64, cs: u64, ss: u64, flags: u64) -> bool {
    (0x10000..USER_END).contains(&ip)
        && (0x10000..USER_END).contains(&sp)
        && cs == USER_CS
        && ss == USER_SS
        && flags & !USER_FLAGS == 0
        && flags & 0x202 == 0x202
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_privilege_noncanonical_and_reserved_flags() {
        assert!(return_state(0x400000, 0x7ffefff8, USER_CS, USER_SS, 0x602));
        for v in [0, 0xffff, USER_END, 0x800000000000, u64::MAX] {
            assert!(!return_state(v, 0x7ffefff8, USER_CS, USER_SS, 0x202));
            assert!(!return_state(0x400000, v, USER_CS, USER_SS, 0x202));
        }
        for bit in [8, 12, 13, 14, 16, 17, 18, 19, 20, 21, 63] {
            assert!(!return_state(
                0x400000,
                0x7ffefff8,
                USER_CS,
                USER_SS,
                0x202 | (1 << bit)
            ));
        }
        assert!(!return_state(0x400000, 0x7ffefff8, 8, USER_SS, 0x202));
        assert!(!return_state(0x400000, 0x7ffefff8, USER_CS, 16, 0x202));
        assert!(!return_state(0x400000, 0x7ffefff8, USER_CS, USER_SS, 2));
    }
}
