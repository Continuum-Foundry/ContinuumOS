use super::*;
use crate::objects::channels::INLINE;
use crate::objects::waiting::{Interest, Watch, WatchSet};
use crate::scheduler::ThreadId;
use crate::wait::{Instant, Outcome, Readiness as Ready, State, Waits};

fn event(c: &Registry, s: SpaceId, h: u64) -> WatchSet {
    c.watch_set(
        s,
        &[Watch {
            handle: h,
            interest: Interest::Signalled,
        }],
    )
    .unwrap()
}

#[test]
fn manual_events_rights_duplicates_stale_watches_and_exact_cleanup() {
    let (mut r, mut b, mut c) = setup();
    let s = r.create().unwrap();
    let (p, _) = c.admit(s).unwrap();
    let baseline = c.inventory();
    let h = c.event_create(s, p, false).unwrap();
    let set = event(&c, s, h);
    assert_eq!(
        c.watch_observe(&set, Instant(0)).unwrap()[0],
        Ready::Pending
    );
    let readonly = c.duplicate(s, h, BASIC).unwrap();
    assert_eq!(c.event_set(s, readonly, true), Err(Error::AccessDenied));
    c.event_set(s, h, true).unwrap();
    assert_eq!(c.watch_observe(&set, Instant(0)).unwrap()[0], Ready::Ready);
    assert_eq!(
        c.watch_observe(&event(&c, s, readonly), Instant(0))
            .unwrap()[0],
        Ready::Ready
    );
    c.close(&mut r, &mut b, s, h).unwrap();
    assert_eq!(c.watch_observe(&set, Instant(0)).unwrap()[0], Ready::Closed);
    let replacement = c.event_create(s, p, true).unwrap();
    assert_ne!(replacement, h);
    assert_eq!(c.watch_observe(&set, Instant(0)).unwrap()[0], Ready::Closed);
    c.close(&mut r, &mut b, s, readonly).unwrap();
    c.close(&mut r, &mut b, s, replacement).unwrap();
    assert_eq!(c.inventory(), baseline);
}

#[test]
fn signal_and_close_around_registration_observe_exact_handles_without_consuming_signal() {
    let (mut r, mut b, mut c) = setup();
    let s = r.create().unwrap();
    let (p, _) = c.admit(s).unwrap();
    let h = c.event_create(s, p, false).unwrap();
    let set = event(&c, s, h);
    let mut waits = Waits::EMPTY;
    for slot in 0..3 {
        let t = ThreadId {
            slot,
            generation: 1,
        };
        let snapshot = c.watch_observe(&set, Instant(0)).unwrap();
        assert_eq!(
            waits
                .register(t, None, Instant(0), &snapshot[..1], false)
                .unwrap(),
            State::Pending
        );
    }
    c.event_set(s, h, true).unwrap();
    for slot in 0..3 {
        let t = ThreadId {
            slot,
            generation: 1,
        };
        let snapshot = c.watch_observe(&set, Instant(1)).unwrap();
        assert_eq!(
            waits.observe(t, Instant(1), &snapshot[..1]).unwrap(),
            State::Completed(Outcome::Ready(0))
        );
        assert_eq!(waits.take(t).unwrap(), Some(Outcome::Ready(0)));
    }
    c.event_set(s, h, false).unwrap();
    let t = ThreadId {
        slot: 0,
        generation: 2,
    };
    let snapshot = c.watch_observe(&set, Instant(2)).unwrap();
    waits
        .register(t, None, Instant(2), &snapshot[..1], false)
        .unwrap();
    c.close(&mut r, &mut b, s, h).unwrap();
    let snapshot = c.watch_observe(&set, Instant(3)).unwrap();
    assert_eq!(
        waits.observe(t, Instant(3), &snapshot[..1]).unwrap(),
        State::Completed(Outcome::Closed(0))
    );
    waits.take(t).unwrap();
    assert_eq!(waits.count(), 0);
}

#[test]
fn watch_validation_is_bounded_typed_and_requires_query_and_direction_rights() {
    let (mut r, _, mut c) = setup();
    let s = r.create().unwrap();
    let (p, _) = c.admit(s).unwrap();
    let e = c.event_create(s, p, false).unwrap();
    let w = Watch {
        handle: e,
        interest: Interest::Signalled,
    };
    let baseline = c.inventory();
    assert!(c.watch_set(s, &[]).is_err());
    assert!(c.watch_set(s, &[w; crate::wait::ITEMS + 1]).is_err());
    assert!(c.watch_set(s, &[w, w]).is_err());
    assert!(
        c.watch_set(
            s,
            &[Watch {
                handle: e,
                interest: Interest::Readable
            }]
        )
        .is_err()
    );
    assert!(c.watch_set(s, &[Watch { handle: 0, ..w }]).is_err());
    assert_eq!(c.inventory(), baseline);
    let no_query = c.duplicate(s, e, CLOSE).unwrap();
    assert!(
        c.watch_set(
            s,
            &[Watch {
                handle: no_query,
                ..w
            }]
        )
        .is_err()
    );
    let [tx, _] = super::channels::pair(&mut c, s, p);
    let read_only = c.duplicate(s, tx, QUERY | CLOSE | RECEIVE).unwrap();
    assert!(
        c.watch_set(
            s,
            &[Watch {
                handle: read_only,
                interest: Interest::Writable {
                    bytes: 1,
                    attachments: 0
                }
            }]
        )
        .is_err()
    );
    assert!(
        c.watch_set(
            s,
            &[Watch {
                handle: tx,
                interest: Interest::Writable {
                    bytes: INLINE + 1,
                    attachments: 0
                }
            }]
        )
        .is_err()
    );
}

#[test]
fn timer_readiness_elapsed_cancel_rearm_and_closed_handle_are_distinct() {
    let (mut r, mut b, mut c) = setup();
    let s = r.create().unwrap();
    let (p, _) = c.admit(s).unwrap();
    let h = c.timer_create(s, p).unwrap();
    let set = event(&c, s, h);
    c.timer_arm(s, h, Instant(100), 10).unwrap();
    assert_eq!(
        c.watch_observe(&set, Instant(99)).unwrap()[0],
        Ready::Pending
    );
    assert_eq!(
        c.watch_observe(&set, Instant(199)).unwrap()[0],
        Ready::Ready
    );
    c.timer_cancel(s, h).unwrap();
    assert_eq!(
        c.watch_observe(&set, Instant(200)).unwrap()[0],
        Ready::Pending
    );
    c.timer_arm(s, h, Instant(201), 0).unwrap();
    assert_eq!(
        c.watch_observe(&set, Instant(201)).unwrap()[0],
        Ready::Ready
    );
    c.close(&mut r, &mut b, s, h).unwrap();
    assert_eq!(
        c.watch_observe(&set, Instant(202)).unwrap()[0],
        Ready::Closed
    );
}

#[test]
fn channel_pressure_and_peer_closure_observe_without_consuming_bytes_or_authority() {
    use super::channels::{pair, receive, sender};
    let (mut r, mut b, mut c) = setup();
    let s = r.create().unwrap();
    let (p, _) = c.admit_sender(s, sender(0)).unwrap();
    let [tx, rx] = pair(&mut c, s, p);
    let write = c
        .watch_set(
            s,
            &[Watch {
                handle: tx,
                interest: Interest::Writable {
                    bytes: INLINE,
                    attachments: 0,
                },
            }],
        )
        .unwrap();
    let read = c
        .watch_set(
            s,
            &[Watch {
                handle: rx,
                interest: Interest::Readable,
            }],
        )
        .unwrap();
    assert_eq!(
        c.watch_observe(&read, Instant(0)).unwrap()[0],
        Ready::Pending
    );
    for _ in 0..4 {
        c.channel_send(s, tx, &[1; INLINE], &[]).unwrap();
    }
    assert_eq!(
        c.watch_observe(&write, Instant(0)).unwrap()[0],
        Ready::Pending
    );
    let inventory = c.inventory();
    let usage = c.channel_usage();
    for _ in 0..3 {
        assert_eq!(c.watch_observe(&read, Instant(0)).unwrap()[0], Ready::Ready);
    }
    assert_eq!(c.inventory(), inventory);
    assert_eq!(c.channel_usage(), usage);
    receive(&mut c, s, rx);
    assert_eq!(
        c.watch_observe(&write, Instant(0)).unwrap()[0],
        Ready::Ready
    );
    c.close(&mut r, &mut b, s, tx).unwrap();
    assert_eq!(c.watch_observe(&read, Instant(0)).unwrap()[0], Ready::Ready);
    for _ in 0..3 {
        receive(&mut c, s, rx);
    }
    assert_eq!(
        c.watch_observe(&read, Instant(0)).unwrap()[0],
        Ready::Closed
    );
    c.close(&mut r, &mut b, s, rx).unwrap();
    assert_eq!(c.channel_usage().pairs, 0);
}

#[test]
fn retained_process_watch_reports_exit_then_closed_and_does_not_bind_reused_owner() {
    let (mut r, mut b, mut c) = setup();
    let a = r.create().unwrap();
    let observer = r.create().unwrap();
    let (p, _) = c.admit(a).unwrap();
    c.admit(observer).unwrap();
    // Trusted test launcher grants QUERY only. No public process delegation is
    // introduced by this test; supervision still owns that authority gate.
    let (_, _, original, _) = c.resolve(a, p, Some(Kind::Process), QUERY).unwrap();
    let table = c.table(observer).unwrap();
    let slot = c.room(table).unwrap();
    let h = c.install(table, slot, original.object, QUERY | CLOSE);
    let set = c
        .watch_set(
            observer,
            &[Watch {
                handle: h,
                interest: Interest::Exited,
            }],
        )
        .unwrap();
    assert_eq!(
        c.watch_observe(&set, Instant(0)).unwrap()[0],
        Ready::Pending
    );
    c.remove(&mut r, &mut b, a).unwrap();
    r.destroy(&mut b, a).unwrap();
    assert_eq!(c.watch_observe(&set, Instant(1)).unwrap()[0], Ready::Ready);
    let replacement = r.create().unwrap();
    assert_ne!(a, replacement);
    c.admit(replacement).unwrap();
    assert_eq!(c.watch_observe(&set, Instant(2)).unwrap()[0], Ready::Ready);
    c.close(&mut r, &mut b, observer, h).unwrap();
    assert_eq!(c.watch_observe(&set, Instant(3)).unwrap()[0], Ready::Closed);
}

#[test]
fn queued_event_and_timer_survive_creator_death_with_attenuated_authority() {
    use super::channels::{pair, receive, sender};
    use crate::objects::channels::{Attachment, ENDPOINT_RIGHTS, Mode};
    let (mut r, mut b, mut c) = setup();
    let baseline = c.inventory();
    let a = r.create().unwrap();
    let z = r.create().unwrap();
    let (p, _) = c.admit_sender(a, sender(0)).unwrap();
    c.admit_sender(z, sender(1)).unwrap();
    let [tx, local_rx] = pair(&mut c, a, p);
    let rx = c.grant_endpoint(a, local_rx, z, ENDPOINT_RIGHTS).unwrap();
    c.close(&mut r, &mut b, a, local_rx).unwrap();
    let e = c.event_create(a, p, true).unwrap();
    let t = c.timer_create(a, p).unwrap();
    c.timer_arm(a, t, Instant(100), 0).unwrap();
    c.channel_send(
        a,
        tx,
        b"wait-objects",
        &[
            Attachment {
                handle: e,
                rights: crate::objects::waiting::EVENT_RIGHTS,
                mode: Mode::Move,
            },
            Attachment {
                handle: t,
                rights: QUERY | CLOSE,
                mode: Mode::Duplicate,
            },
        ],
    )
    .unwrap();
    c.remove(&mut r, &mut b, a).unwrap();
    r.destroy(&mut b, a).unwrap();
    let (bytes, handles, _) = receive(&mut c, z, rx);
    assert_eq!(bytes, b"wait-objects");
    assert_eq!(handles.len(), 2);
    let event_set = event(&c, z, handles[0]);
    let timer_set = event(&c, z, handles[1]);
    assert_eq!(
        c.watch_observe(&event_set, Instant(99)).unwrap()[0],
        Ready::Ready
    );
    assert_eq!(
        c.watch_observe(&timer_set, Instant(99)).unwrap()[0],
        Ready::Pending
    );
    assert_eq!(c.timer_cancel(z, handles[1]), Err(Error::AccessDenied));
    assert_eq!(
        c.watch_observe(&timer_set, Instant(101)).unwrap()[0],
        Ready::Ready
    );
    c.event_set(z, handles[0], false).unwrap();
    assert_eq!(
        c.watch_observe(&event_set, Instant(101)).unwrap()[0],
        Ready::Pending
    );
    c.remove(&mut r, &mut b, z).unwrap();
    r.destroy(&mut b, z).unwrap();
    assert_eq!(c.inventory(), baseline);
    assert_eq!(c.channel_usage().pairs, 0);
}
