use super::*;
fn id(n: u64) -> Identity {
    Identity {
        process: n,
        generation: 1,
        principal: 101,
        session: 1001,
        service: n == 0,
    }
}
const S: Scope = Scope {
    kind: FILE,
    resource: 1,
    owner: 101,
};
fn req(b: &Broker, op: u64) -> Request {
    Request {
        scope: S,
        operation: op,
        revision: b.revision(),
        epoch: b.epoch(),
        deadline: 100,
    }
}
fn setup() -> (Broker, u64) {
    let mut b = Broker::new(id(0), 1);
    b.restore(id(0)).unwrap();
    b.lease_session(id(0), id(1), 100, 0).unwrap();
    let h = b
        .assign(id(0), id(1), S, READ | WRITE | DELEGATE, 100, 0)
        .unwrap();
    (b, h)
}
#[test]
fn explicit_subject_scope_and_untrusted_administration() {
    let (b, h) = setup();
    assert_eq!(b.preview(id(2), h, req(&b, READ), 0), Err(DENIED));
    let mut b = b;
    assert_eq!(b.assign(id(1), id(1), S, READ, 100, 0), Err(DENIED));
    for scope in [
        Scope { resource: 2, ..S },
        Scope { owner: 202, ..S },
        Scope { kind: DISK, ..S },
    ] {
        assert_eq!(
            b.preview(
                id(1),
                h,
                Request {
                    scope,
                    ..req(&b, READ)
                },
                0
            ),
            Err(DENIED)
        );
    }
    assert_eq!(
        b.preview(
            Identity {
                generation: 2,
                ..id(1)
            },
            h,
            req(&b, READ),
            0
        ),
        Err(DENIED)
    );
}
#[test]
fn delegation_never_widens() {
    let (mut b, h) = setup();
    let child = b.delegate(id(1), h, id(2), S, READ, 90, 0).unwrap();
    assert!(b.preview(id(2), child, req(&b, READ), 0).is_ok());
    assert_eq!(b.preview(id(2), child, req(&b, WRITE), 0), Err(DENIED));
    assert_eq!(
        b.delegate(id(2), child, id(3), S, WRITE, 90, 0),
        Err(DENIED)
    );
    assert_eq!(b.delegate(id(1), h, id(2), S, READ, 101, 0), Err(DENIED));
}
#[test]
fn preview_rechecked_and_commit_is_irreversible() {
    let (mut b, h) = setup();
    let r = req(&b, WRITE);
    b.preview(id(1), h, r, 0).unwrap();
    b.invalidate(id(0)).unwrap();
    assert!(b.execute(id(1), h, r, 1).is_err());
    assert_eq!(b.revision(), 1);
    let h = b.assign(id(0), id(1), S, WRITE, 100, 1).unwrap();
    let r = req(&b, WRITE);
    assert_eq!(b.execute(id(1), h, r, 1), Ok(2));
    b.expire_session(id(0), 1001).unwrap();
    assert_eq!(b.execute(id(1), h, r, 2), Err(EXPIRED));
    assert_eq!(b.revision(), 2);
}
#[test]
fn expiry_deadline_revision_and_epoch() {
    let (mut b, h) = setup();
    let r = req(&b, WRITE);
    assert_eq!(
        b.execute(id(1), h, Request { epoch: 2, ..r }, 0),
        Err(STALE)
    );
    assert_eq!(
        b.execute(id(1), h, Request { revision: 2, ..r }, 0),
        Err(CONFLICT)
    );
    assert_eq!(
        b.execute(id(1), h, Request { deadline: 1, ..r }, 1),
        Err(EXPIRED)
    );
    assert_eq!(b.execute(id(1), h, r, 100), Err(EXPIRED));
    assert_eq!(b.revision(), 1);
}
#[test]
fn forwarding_intersection_exact_ticket_and_replay() {
    let (mut b, h) = setup();
    let service = Identity {
        service: true,
        ..id(3)
    };
    let sh = b.assign(id(0), service, S, READ | WRITE, 100, 0).unwrap();
    let r = req(&b, READ);
    let t = b.ticket(id(1), h, service, r, 0).unwrap();
    assert_eq!(b.forwarded(service, sh, t, req(&b, WRITE), 0), Err(DENIED));
    assert_eq!(b.forwarded(id(2), sh, t, r, 0), Err(DENIED));
    assert!(b.forwarded(service, sh, t, r, 0).is_ok());
    assert_eq!(b.forwarded(service, sh, t, r, 0), Err(DENIED));
    let ro = b.assign(id(0), id(2), S, READ, 100, 0).unwrap();
    assert_eq!(b.ticket(id(2), ro, service, req(&b, WRITE), 0), Err(DENIED));
    let t = b.ticket(id(1), h, service, req(&b, WRITE), 0).unwrap();
    let sr = b.assign(id(0), service, S, READ, 100, 0).unwrap();
    assert_eq!(b.forwarded(service, sr, t, req(&b, WRITE), 0), Err(DENIED));
}
#[test]
fn restart_and_reserved_scopes() {
    let (mut b, h) = setup();
    let r = req(&b, READ);
    b = Broker::new(id(0), 2);
    assert_eq!(b.preview(id(1), h, r, 0), Err(DENIED));
    assert_eq!(b.restore(id(1)), Err(DENIED));
    b.restore(id(0)).unwrap();
    b.lease_session(id(0), id(1), 100, 0).unwrap();
    assert_eq!(b.preview(id(1), h, r, 0), Err(DENIED));
    let s = Scope {
        kind: DISK,
        resource: 1,
        owner: 101,
    };
    let h = b.assign(id(0), id(1), s, ADMIN, 100, 0).unwrap();
    assert!(
        b.execute(
            id(1),
            h,
            Request {
                scope: s,
                operation: ADMIN,
                ..req(&b, READ)
            },
            0
        )
        .is_ok()
    );
    for kind in [ACCOUNT, PROCESS, DIAGNOSTIC, SERVICE] {
        assert_eq!(
            b.execute(
                id(1),
                h,
                Request {
                    scope: Scope { kind, ..s },
                    operation: ADMIN,
                    ..req(&b, READ)
                },
                0
            ),
            Err(DENIED)
        );
    }
}
#[test]
fn bounded_failures_preserve_policy() {
    let (mut b, _) = setup();
    for _ in 1..32 {
        b.assign(id(0), id(1), S, READ, 100, 0).unwrap();
    }
    assert_eq!(b.assign(id(0), id(1), S, READ, 100, 0), Err(QUOTA));
    for s in 1..=16 {
        b.expire_session(id(0), s).unwrap();
    }
    assert_eq!(b.expire_session(id(0), 17), Err(QUOTA));
}

#[test]
fn session_lease_expires_at_use_and_cannot_be_extended() {
    let (mut b, h) = setup();
    b.lease_session(id(0), id(1), 10, 0).unwrap();
    assert_eq!(b.lease_session(id(0), id(1), 11, 0), Err(DENIED));
    assert!(b.preview(id(1), h, req(&b, READ), 9).is_ok());
    assert_eq!(b.execute(id(1), h, req(&b, READ), 10), Err(EXPIRED));
    assert_eq!(b.assign(id(0), id(1), S, READ, 100, 10), Err(EXPIRED));
}
