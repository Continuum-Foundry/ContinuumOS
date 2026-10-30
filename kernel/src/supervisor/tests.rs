use super::*;
use crate::user_image::Binding;
fn setup() -> (Tree, u64, Context) {
    let mut t = Tree::EMPTY;
    let r = t.bootstrap(Binding::Service.context()).unwrap();
    t.start_root().unwrap();
    let c = Context {
        service: false,
        grant_ceiling: 1,
        commitment_ceiling: MEMORY,
        ..Binding::Service.context()
    };
    (t, r, c)
}
#[test]
fn hierarchical_precharge_cannot_be_reused_after_exit_or_delegation() {
    let (mut t, r, c) = setup();
    let p = t.child(r, c, 7).unwrap();
    t.start(r, p).unwrap();
    for _ in 0..6 {
        t.child(p, c, 1).unwrap();
    }
    assert_eq!(t.child(p, c, 1), Err(Error::Quota));
    assert_eq!(t.child(r, c, 1), Err(Error::Quota));
    t.stop(p).unwrap();
    for token in t
        .tokens()
        .into_iter()
        .filter(|v| *v != 0 && *v != r && *v != p)
    {
        t.reaped(
            token,
            Exit {
                code: 768,
                vector: 0,
            },
        )
        .unwrap();
        t.join(r, token).unwrap();
    }
    t.reaped(
        p,
        Exit {
            code: 768,
            vector: 0,
        },
    )
    .unwrap();
    t.join(r, p).unwrap();
    assert_eq!(t.child(r, c, 1), Err(Error::Quota));
}
#[test]
fn identities_grants_priority_threads_and_implicit_inheritance_reject() {
    let (t, r, c) = setup();
    for bad in [
        Context {
            principal: 202,
            ..c
        },
        Context { session: 1, ..c },
        Context { service: true, ..c },
        Context {
            grant_ceiling: 4,
            ..c
        },
        Context {
            commitment_ceiling: MEMORY + 1,
            ..c
        },
    ] {
        assert_eq!(t.check_child(r, bad, 1, 1, 0, 0), Err(Error::Denied));
    }
    assert_eq!(t.check_child(r, c, 1, 2, 0, 0), Err(Error::Quota));
    assert_eq!(t.check_child(r, c, 1, 1, 1, 0), Err(Error::Denied));
    assert_eq!(t.check_child(r, c, 1, 1, 0, 1), Err(Error::Denied));
}
#[test]
fn lifecycle_join_reuse_and_orphans_preserve_generations() {
    let (mut t, r, c) = setup();
    let p = t.child(r, c, 2).unwrap();
    assert_eq!(t.join(r, p), Err(Error::WouldBlock));
    t.start(r, p).unwrap();
    assert_eq!(t.start(r, p), Err(Error::State));
    let child = t.child(p, c, 1).unwrap();
    t.stop(p).unwrap();
    assert_eq!(t.state(child), Ok(State::Stopping));
    t.reaped(p, Exit { code: 1, vector: 0 }).unwrap();
    assert_eq!(t.join(r, p), Err(Error::WouldBlock));
    t.reaped(
        child,
        Exit {
            code: 768,
            vector: 0,
        },
    )
    .unwrap();
    t.join(r, child).unwrap();
    assert_eq!(t.join(r, p).unwrap().code, 1);
    let fresh = t.child(r, c, 1).unwrap();
    assert_ne!(fresh, p);
    assert_eq!(t.terminate(r, p), Err(Error::Stale));
    assert_eq!(t.state(fresh), Ok(State::Prepared));
}
#[test]
fn root_failure_closes_admission_and_epoch_reset_never_reuses_tokens() {
    let (mut t, r, c) = setup();
    let p = t.child(r, c, 1).unwrap();
    t.stop(r).unwrap();
    assert!(t.closed());
    assert_eq!(t.child(r, c, 1), Err(Error::Denied));
    assert_eq!(t.state(p), Ok(State::Stopping));
    assert_eq!(t.clear_epoch(), Err(Error::State));
    t.reaped(
        r,
        Exit {
            code: 262,
            vector: 6,
        },
    )
    .unwrap();
    t.reaped(
        p,
        Exit {
            code: 768,
            vector: 0,
        },
    )
    .unwrap();
    t.clear_epoch().unwrap();
    assert!(t.bootstrap(Binding::Service.context()).unwrap() > p);
}
#[test]
fn restart_backoff_quarantine_and_diagnostic_flood_are_bounded() {
    let (mut t, r, _) = setup();
    t.restart(r, 0).unwrap();
    assert_eq!(t.restart(r, 9_999_999), Err(Error::Backoff));
    t.restart(r, 10_000_000).unwrap();
    assert_eq!(t.restart(r, u64::MAX), Err(Error::Quarantined));
    for _ in 0..4 {
        t.log(r, 1).unwrap();
    }
    assert_eq!(t.log(r, 2), Err(Error::Quota));
    t.log(r, 1_000_000_000).unwrap();
}
