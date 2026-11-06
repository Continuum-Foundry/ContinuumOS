//! Test oracle for a four-record, elapsed-second diagnostic quota.
//! A syscall may fall on either side of its surrounding clock observations.
pub struct RateWindow {
    epoch: u64,
    counts: u8,
    last: u64,
}
impl RateWindow {
    pub fn new(now: u64) -> Self {
        Self {
            epoch: now / 1_000_000_000,
            counts: 1,
            last: now,
        }
    }
    pub fn observe(&mut self, before: u64, after: u64, status: u64) -> Result<bool, ()> {
        if before < self.last || after < before {
            return Err(());
        }
        let before_epoch = before / 1_000_000_000;
        let after_epoch = after / 1_000_000_000;
        let counts = if before_epoch == self.epoch {
            self.counts
        } else {
            1
        };
        let next = match status {
            0 => (counts & 0b1111) << 1,
            11 => counts & 0b10000,
            _ => return Err(()),
        };
        // An accepted call can also have occurred after a boundary, where the
        // quota was empty. A quota rejection cannot occur in an empty epoch.
        let possible = if before_epoch != after_epoch && status == 0 {
            next | 0b10
        } else {
            next
        };
        if possible == 0 {
            return Err(());
        }
        let witnessed = before_epoch == after_epoch && status == 11 && counts == 0b10000;
        self.counts = if before_epoch == after_epoch {
            possible
        } else {
            // Calls in the old epoch leave zero records in the observed new
            // epoch; calls in the new epoch leave one accepted record.
            (if next != 0 { 1 } else { 0 }) | (if status == 0 { 2 } else { 0 })
        };
        self.epoch = after_epoch;
        self.last = after;
        Ok(witnessed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn four_then_quota_and_next_epoch_reset() {
        let mut r = RateWindow::new(0);
        for n in 0..4 {
            assert_eq!(r.observe(n * 2, n * 2 + 1, 0), Ok(false));
        }
        assert_eq!(r.observe(8, 9, 11), Ok(true));
        assert_eq!(r.observe(1_000_000_001, 1_000_000_002, 0), Ok(false));
    }
    #[test]
    fn rejects_early_quota_fifth_success_bad_status_and_clock() {
        assert_eq!(RateWindow::new(0).observe(1, 2, 11), Err(()));
        let mut r = RateWindow::new(0);
        for n in 0..4 {
            r.observe(n * 2, n * 2 + 1, 0).unwrap();
        }
        assert_eq!(r.observe(8, 9, 0), Err(()));
        assert_eq!(RateWindow::new(0).observe(2, 1, 0), Err(()));
        assert_eq!(RateWindow::new(5).observe(1, 2, 0), Err(()));
        assert_eq!(RateWindow::new(0).observe(1, 2, 7), Err(()));
    }
    #[test]
    fn boundary_between_or_inside_calls_does_not_fake_a_failure() {
        for inside in [false, true] {
            let mut r = RateWindow::new(999_999_980);
            for n in 0..4 {
                r.observe(999_999_980 + n * 2, 999_999_981 + n * 2, 0)
                    .unwrap();
            }
            if inside {
                r.observe(999_999_999, 1_000_000_001, 0).unwrap();
            } else {
                r.observe(1_000_000_000, 1_000_000_001, 0).unwrap();
            }
            for n in 0..3 {
                r.observe(1_000_000_002 + n * 2, 1_000_000_003 + n * 2, 0)
                    .unwrap();
            }
            r.observe(1_000_000_010, 1_000_000_011, 11).unwrap();
            assert_eq!(r.observe(1_000_000_012, 1_000_000_013, 11), Ok(true));
        }
    }
}
