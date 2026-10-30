use super::*;
fn id(slot: usize, generation: u64) -> ThreadId {
    ThreadId { slot, generation }
}
const P: Readiness = Readiness::Pending;
const R: Readiness = Readiness::Ready;
const C: Readiness = Readiness::Closed;
fn done(o: Outcome) -> State {
    State::Completed(o)
}

#[test]
fn hpet_elapsed_wrap_fractional_resolution_and_failed_observation_are_transactional() {
    let mut c = Clock::new(u64::MAX - 2, 100_001).unwrap();
    assert_eq!(c.resolution_ns(), 1);
    assert_eq!(c.observe(2).unwrap(), 0); // five hardware ticks, not IRQs
    assert_eq!(c.observe(7).unwrap(), 1); // fractional nanoseconds accumulate
    assert!(c.observe(6).is_err());
    assert_eq!(c.observe(17).unwrap(), 2); // rejected sample did not change epoch
    assert!(Clock::new(0, 0).is_err());
    assert!(Clock::new(0, 100_000_001).is_err());
    let mut c = Clock::new(0, 100_000_000).unwrap();
    assert_eq!(c.resolution_ns(), 100);
    assert!(c.observe(1 << 63).is_err());
    assert!(c.observe(u64::MAX / 100 + 1).is_err());
    assert_eq!(c.observe(1).unwrap(), 100);
    assert_eq!(Instant(u64::MAX).after(0).unwrap(), Instant(u64::MAX));
    assert!(Instant(u64::MAX).after(1).is_err());
}

#[test]
fn deadline_uses_elapsed_sample_even_when_timer_interrupt_delivery_is_delayed() {
    let mut clock = Clock::new(0, 100_000_000).unwrap();
    let mut waits = Waits::EMPTY;
    let t = id(0, 1);
    assert_eq!(
        waits
            .register(t, Some(Instant(1000)), Instant(0), &[P], false)
            .unwrap(),
        State::Pending
    );
    // No delivered-tick input exists; the next counter sample is authoritative.
    let now = Instant(clock.observe(50).unwrap());
    assert_eq!(
        waits.observe(t, now, &[R]).unwrap(),
        done(Outcome::TimedOut)
    );
    assert_eq!(waits.take(t).unwrap(), Some(Outcome::TimedOut));
    assert_eq!(waits.count(), 0);
}

#[test]
fn exhaustive_completion_priority_and_immutable_winner() {
    for closed in [false, true] {
        for cancel in [false, true] {
            for expired in [false, true] {
                for ready in [false, true] {
                    let mut w = Waits::EMPTY;
                    let items = [if ready { R } else { P }, if closed { C } else { P }];
                    let expected = if closed {
                        done(Outcome::Closed(1))
                    } else if cancel {
                        done(Outcome::Cancelled)
                    } else if expired {
                        done(Outcome::TimedOut)
                    } else if ready {
                        done(Outcome::Ready(0))
                    } else {
                        State::Pending
                    };
                    let t = id(0, 7);
                    assert_eq!(
                        w.register(
                            t,
                            Some(Instant(10)),
                            Instant(if expired { 10 } else { 9 }),
                            &items,
                            cancel
                        )
                        .unwrap(),
                        expected
                    );
                    if expected != State::Pending {
                        w.request_cancel(t).unwrap();
                        assert_eq!(w.observe(t, Instant(u64::MAX), &[C, C]).unwrap(), expected);
                        assert!(w.take(t).unwrap().is_some());
                        assert!(w.take(t).is_err()); // no second completion
                    } else {
                        w.terminate(t).unwrap();
                    }
                    assert_eq!(w.count(), 0);
                }
            }
        }
    }
}

#[test]
fn cancellation_request_requires_confirmation_and_success_cannot_be_recalled() {
    let mut w = Waits::EMPTY;
    let t = id(0, 1);
    w.register(t, None, Instant(0), &[P], false).unwrap();
    w.request_cancel(t).unwrap();
    assert_eq!(w.take(t).unwrap(), None);
    assert_eq!(
        w.observe(t, Instant(1), &[R]).unwrap(),
        done(Outcome::Cancelled)
    );
    assert_eq!(w.take(t).unwrap(), Some(Outcome::Cancelled));
    w.register(t, None, Instant(2), &[R], false).unwrap();
    w.request_cancel(t).unwrap();
    assert_eq!(w.take(t).unwrap(), Some(Outcome::Ready(0)));
}

#[test]
fn wait_admission_stale_identity_and_termination_settle_exactly() {
    // Keep policy metadata bounded independently of object/adapter storage.
    assert!(core::mem::size_of::<Waits>() <= 4096);
    let mut w = Waits::EMPTY;
    assert!(w.register(id(0, 0), None, Instant(0), &[P], false).is_err());
    assert!(
        w.register(id(CAPACITY, 1), None, Instant(0), &[P], false)
            .is_err()
    );
    assert!(w.register(id(0, 1), None, Instant(0), &[], false).is_err());
    assert!(
        w.register(id(0, 1), None, Instant(0), &[P; ITEMS + 1], false)
            .is_err()
    );
    for slot in 0..CAPACITY {
        w.register(id(slot, 1), None, Instant(0), &[P; ITEMS], false)
            .unwrap();
    }
    assert_eq!(w.count(), CAPACITY);
    assert!(w.register(id(0, 2), None, Instant(0), &[P], false).is_err());
    assert!(w.observe(id(0, 1), Instant(0), &[R]).is_err());
    assert_eq!(w.take(id(0, 1)).unwrap(), None);
    for slot in 0..CAPACITY {
        assert!(w.terminate(id(slot, 2)).is_err());
        w.terminate(id(slot, 1)).unwrap();
        w.register(id(slot, 2), None, Instant(0), &[R], false)
            .unwrap();
        assert!(w.request_cancel(id(slot, 1)).is_err());
        assert_eq!(w.take(id(slot, 2)).unwrap(), Some(Outcome::Ready(0)));
    }
    assert_eq!(w.count(), 0);
}

#[test]
fn level_readiness_wakes_multiple_records_and_uses_input_order() {
    let mut w = Waits::EMPTY;
    for slot in 0..CAPACITY {
        w.register(id(slot, 1), None, Instant(0), &[P, P], false)
            .unwrap();
    }
    for slot in 0..CAPACITY {
        assert_eq!(
            w.observe(id(slot, 1), Instant(1), &[R, R]).unwrap(),
            done(Outcome::Ready(0))
        );
        assert_eq!(w.take(id(slot, 1)).unwrap(), Some(Outcome::Ready(0)));
    }
    assert_eq!(w.count(), 0);
}

#[test]
fn periodic_timer_coalesces_from_original_schedule_cancel_and_rearm_clear_level() {
    let mut t = Timer::EMPTY;
    assert_eq!(t.observe(Instant(u64::MAX)).unwrap(), 0);
    t.arm(Instant(10), 3);
    assert_eq!(t.observe(Instant(9)).unwrap(), 0);
    assert!(!t.ready());
    assert_eq!(t.observe(Instant(20)).unwrap(), 4); // 10, 13, 16, 19
    assert_eq!(t.next(), Some(Instant(22)));
    assert!(t.ready());
    assert_eq!(t.observe(Instant(20)).unwrap(), 0);
    assert_eq!(t.observe(Instant(22)).unwrap(), 1);
    t.arm(Instant(30), 0);
    assert!(!t.ready());
    assert_eq!(t.observe(Instant(31)).unwrap(), 1);
    assert_eq!(t.observe(Instant(100)).unwrap(), 0);
    assert!(t.ready());
    t.cancel();
    assert!(!t.ready());
    assert_eq!(t.next(), None);
}

#[test]
fn periodic_exhaustion_disarms_and_count_overflow_is_transactional() {
    let mut t = Timer::EMPTY;
    t.arm(Instant(u64::MAX - 1), 2);
    assert_eq!(t.observe(Instant(u64::MAX)).unwrap(), 1);
    assert!(t.ready());
    assert_eq!(t.next(), None);
    assert_eq!(t.observe(Instant(u64::MAX)).unwrap(), 0);
    t.arm(Instant(0), 1);
    assert!(t.observe(Instant(u64::MAX)).is_err());
    assert_eq!(t.next(), Some(Instant(0)));
    assert!(!t.ready());
    assert_eq!(t.observe(Instant(0)).unwrap(), 1);
}

#[test]
fn pending_waits_complete_at_boundary_zero_deadline_and_cycles_release_all_records() {
    let mut w = Waits::EMPTY;
    // Ready before registration completes immediately; during/after registration
    // the next serialized observation completes the pending record. This tests
    // policy only; the guest adapter still must enforce the exclusion interval.
    assert_eq!(
        w.register(id(0, 1), None, Instant(0), &[R], false).unwrap(),
        done(Outcome::Ready(0))
    );
    w.take(id(0, 1)).unwrap();
    assert_eq!(
        w.register(id(0, 1), Some(Instant(0)), Instant(0), &[R], false)
            .unwrap(),
        done(Outcome::TimedOut)
    );
    w.take(id(0, 1)).unwrap();
    for participants in [2, 3] {
        for slot in 0..participants {
            w.register(id(slot, 1), Some(Instant(10)), Instant(1), &[P], false)
                .unwrap();
        }
        assert_eq!(w.count(), participants);
        w.request_cancel(id(0, 1)).unwrap();
        assert_eq!(
            w.observe(id(0, 1), Instant(9), &[P]).unwrap(),
            done(Outcome::Cancelled)
        );
        w.take(id(0, 1)).unwrap();
        w.terminate(id(1, 1)).unwrap();
        if participants == 3 {
            assert_eq!(
                w.observe(id(2, 1), Instant(11), &[P]).unwrap(),
                done(Outcome::TimedOut)
            );
            w.take(id(2, 1)).unwrap();
        }
        // A separate record remains admissible after the synthetic cycle.
        w.register(id(3, 1), None, Instant(11), &[P], false)
            .unwrap();
        assert_eq!(
            w.observe(id(3, 1), Instant(12), &[R]).unwrap(),
            done(Outcome::Ready(0))
        );
        w.take(id(3, 1)).unwrap();
        assert_eq!(w.count(), 0);
    }
}
