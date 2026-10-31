use super::*;
fn who(n: u64) -> Identity {
    Identity {
        process: n,
        generation: 1,
        principal: 101,
        session: n + 100,
        service: n != 3,
    }
}
fn setup() -> Authority {
    let mut a = Authority::new(who(1), 100);
    a.policy.restore(who(1)).unwrap();
    for n in [3, 4, 5] {
        a.policy
            .lease_session(who(1), who(n), 50_000_000_000, 0)
            .unwrap();
        assert_eq!(
            a.policy
                .assign(who(1), who(n), SCOPE, policy::WRITE, 50_000_000_000, 0),
            Ok(n - 2)
        );
    }
    a.grant_writer(who(1), who(4), 2, 0).unwrap();
    a.publish(who(1), who(4), 100).unwrap();
    a
}
fn req(id: u64) -> Request {
    Request {
        caller: who(3),
        id,
        service: SERVICE,
        protocol: PROTOCOL,
        major: 1,
        features: 0,
        epoch: 100,
        policy_epoch: 100,
        grant: 1,
        deadline: 40_000_000_000,
        delta: 7,
    }
}
#[test]
fn route_admin_and_fence_order() {
    let mut a = setup();
    assert_eq!(a.fence(who(3)), Err(policy::DENIED));
    assert_eq!(a.publish(who(3), who(4), 100), Err(policy::DENIED));
    assert_eq!(a.grant_writer(who(3), who(5), 3, 0), Err(policy::DENIED));
    assert_eq!(a.grant_writer(who(1), who(5), 3, 0), Err(policy::DENIED));
    a.fence(who(1)).unwrap();
    assert_eq!(a.writer, None);
    assert_eq!(
        a.admit(
            who(3),
            Request {
                epoch: 101,
                ..req(1)
            },
            0
        ),
        Err(UNAVAILABLE)
    );
    assert_eq!(a.publish(who(1), who(5), 101), Err(policy::STALE));
    a.grant_writer(who(1), who(5), 3, 0).unwrap();
    assert_eq!(
        a.admit(
            who(3),
            Request {
                epoch: 101,
                ..req(1)
            },
            0
        ),
        Err(UNAVAILABLE)
    );
    a.publish(who(1), who(5), 101).unwrap();
}
#[test]
fn retained_old_writer_fails_after_new_grant() {
    let mut a = setup();
    a.admit(who(3), req(1), 0).unwrap();
    a.forward(req(1), 0).unwrap();
    a.fence(who(1)).unwrap();
    a.grant_writer(who(1), who(5), 3, 0).unwrap();
    a.publish(who(1), who(5), 101).unwrap();
    assert_eq!(
        a.commit(who(4), 2, req(1), 0).unwrap().status,
        policy::STALE
    );
    assert_eq!((a.value, a.revision), (0, 0));
}
#[test]
fn duplicate_and_reordered_are_bounded() {
    let mut a = setup();
    a.admit(who(3), req(2), 0).unwrap();
    assert_eq!(a.admit(who(3), req(2), 0), Err(PENDING));
    assert_eq!(a.admit(who(3), req(1), 0), Err(policy::STALE));
    a.forward(req(2), 0).unwrap();
    let r = a.commit(who(4), 2, req(2), 0).unwrap();
    assert_eq!(a.admit(who(3), req(2), 0), Ok(Some(r)));
    assert_eq!(
        a.admit(who(3), Request { delta: 8, ..req(2) }, 0),
        Err(policy::DENIED)
    );
    assert_eq!((a.value, a.revision), (7, 1));
    assert_eq!(a.admit(who(3), req(2), RETENTION), Err(policy::STALE));
}
#[test]
fn saturation_reserves_results() {
    let mut a = setup();
    for id in 1..=4 {
        a.admit(who(3), req(id), 0).unwrap();
    }
    assert_eq!(a.queued(), 4);
    assert_eq!(a.admit(who(3), req(5), 0), Err(policy::QUOTA));
    for id in 1..=4 {
        a.cancel(who(3), req(id), 0).unwrap();
    }
    for id in 5..=8 {
        a.admit(who(3), req(id), 0).unwrap();
        a.cancel(who(3), req(id), 0).unwrap();
    }
    assert_eq!(a.admit(who(3), req(9), 0), Err(policy::QUOTA));
    assert_eq!(a.admit(who(3), req(9), RETENTION), Ok(None));
}
#[test]
fn commit_rechecks_policy_and_session() {
    for revoke in [false, true] {
        let mut a = setup();
        a.admit(who(3), req(1), 0).unwrap();
        a.forward(req(1), 0).unwrap();
        if revoke {
            a.policy.expire_session(who(1), who(3).session).unwrap();
        } else {
            a.policy.invalidate(who(1)).unwrap();
        }
        assert_ne!(a.commit(who(4), 2, req(1), 0).unwrap().status, 0);
        assert_eq!(a.value, 0);
    }
}
#[test]
fn service_grant_and_forward_binding_checked() {
    let mut a = setup();
    a.admit(who(3), req(1), 0).unwrap();
    a.forward(req(1), 0).unwrap();
    assert_eq!(
        a.commit(who(5), 3, req(1), 0).unwrap().status,
        policy::DENIED
    );
    assert_eq!(a.value, 0);
    let mut a = setup();
    a.admit(who(3), req(1), 0).unwrap();
    assert_eq!(
        a.commit(who(4), 2, req(1), 0).unwrap().status,
        policy::DENIED
    );
}
#[test]
fn cancel_does_not_undo_committed_effect() {
    let mut a = setup();
    a.admit(who(3), req(1), 0).unwrap();
    assert_eq!(a.cancel(who(3), req(1), 0).unwrap().status, CANCELLED);
    assert_eq!(a.commit(who(4), 2, req(1), 0), Err(policy::STALE));
    a.admit(who(3), req(2), 0).unwrap();
    a.forward(req(2), 0).unwrap();
    let r = a.commit(who(4), 2, req(2), 0).unwrap();
    assert_eq!(a.cancel(who(3), req(2), 0), Ok(r));
    assert_eq!(a.value, 7);
}
#[test]
fn reply_correlation_and_epoch() {
    let mut a = setup();
    a.admit(who(3), req(1), 0).unwrap();
    a.forward(req(1), 0).unwrap();
    let r = a.commit(who(4), 2, req(1), 0).unwrap();
    a.reply(who(3), req(1), r, 0).unwrap();
    assert_eq!(a.reply(who(3), req(1), r, RETENTION), Err(policy::DENIED));
    assert_eq!(a.reply(who(5), req(1), r, 0), Err(policy::DENIED));
    assert_eq!(a.reply(who(3), req(2), r, 0), Err(policy::STALE));
    a.fence(who(1)).unwrap();
    assert_eq!(a.reply(who(3), req(1), r, 0), Err(policy::STALE));
}
#[test]
fn incompatible_and_forged_caller() {
    let mut a = setup();
    for r in [
        Request { major: 2, ..req(1) },
        Request {
            features: 1,
            ..req(1)
        },
        Request {
            service: 999,
            ..req(1)
        },
        Request {
            protocol: 0,
            ..req(1)
        },
    ] {
        assert_eq!(a.admit(who(3), r, 0), Err(INCOMPATIBLE));
    }
    assert_eq!(a.admit(who(5), req(1), 0), Err(policy::DENIED));
    assert_eq!(a.queued(), 0);
}
#[test]
fn expiry_and_overflow_leave_resource_unchanged() {
    let mut a = setup();
    a.admit(who(3), req(1), 0).unwrap();
    a.forward(req(1), 0).unwrap();
    assert_eq!(
        a.commit(who(4), 2, req(1), 40_000_000_000).unwrap().status,
        policy::EXPIRED
    );
    a.value = u64::MAX;
    a.admit(who(3), req(2), 0).unwrap();
    a.forward(req(2), 0).unwrap();
    assert_eq!(
        a.commit(who(4), 2, req(2), 0).unwrap().status,
        policy::QUOTA
    );
    assert_eq!(a.revision, 0);
}
#[test]
fn fresh_incarnation_has_no_authority() {
    let a = Authority::new(who(1), 200);
    assert_eq!(a.writer, None);
    let mut a = a;
    assert_eq!(a.admit(who(3), req(1), 0), Err(policy::DENIED));
    assert_eq!(a.grant_writer(who(1), who(4), 2, 0), Err(policy::DENIED));
}
