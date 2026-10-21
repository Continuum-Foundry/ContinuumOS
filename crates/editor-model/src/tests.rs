use super::*;
use alloc::vec;
fn provider() -> Provider {
    let mut p = Provider::new(
        Model::new(1, 11, "Draft: unsaved\n", "", Selection::new(7, 14)).unwrap(),
        7,
    )
    .unwrap();
    p.grant(1, Rights::ALL).unwrap();
    p
}
fn request(p: &Provider, id: u64, action: Action) -> Request {
    Request {
        version: 1,
        caller: 1,
        id,
        epoch: 7,
        document: 1,
        expected_document_revision: p.model().document().revision(),
        expected_model_revision: p.model().revision(),
        deadline: 100,
        context: None,
        action,
    }
}
fn run(p: &mut Provider, id: u64, action: Action) -> Outcome {
    let r = request(p, id, action);
    p.execute(r, 1, false)
}
#[test]
fn undo_replacement_preserves_direction_and_dirty_history() {
    let mut p = provider();
    assert_eq!(
        run(&mut p, 1, Action::Select(Selection::new(14, 7))).status,
        Ok(true)
    );
    run(&mut p, 2, Action::Insert("notes".into()));
    assert_eq!(p.model.document.text(), "Draft: notes\n");
    run(&mut p, 3, Action::Undo);
    assert_eq!(p.model.document.selection(), Selection::new(14, 7));
    assert_eq!(p.model.document.text(), "Draft: unsaved\n");
    assert!(p.model.document.dirty());
    run(&mut p, 4, Action::Redo);
    assert_eq!(p.model.document.revision(), 14);
    run(&mut p, 5, Action::Undo);
    run(&mut p, 6, Action::Delete);
    assert_eq!(p.model.document.text(), "Draft: \n");
    assert_eq!(p.model.document.redo_len(), 0);
}
#[test]
fn dirty_returns_to_baseline_and_revision_never_rewinds() {
    let mut p =
        Provider::new(Model::new(1, 1, "a", "a", Selection::new(1, 1)).unwrap(), 7).unwrap();
    p.grant(1, Rights::ALL).unwrap();
    run(&mut p, 1, Action::Insert("b".into()));
    assert!(p.model.document.dirty());
    run(&mut p, 2, Action::Undo);
    assert!(!p.model.document.dirty());
    assert_eq!(p.model.document.revision(), 3);
}
#[test]
fn rejects_utf8_and_grapheme_interiors_atomically() {
    for text in ["é", "e\u{301}", "👩🏽‍💻", "\r\n", "🇺🇸", "नमस्ते"] {
        let mut p =
            Provider::new(Model::new(1, 1, text, "", Selection::new(0, 0)).unwrap(), 7).unwrap();
        p.grant(1, Rights::ALL).unwrap();
        let before = p.model.clone();
        assert_eq!(
            run(&mut p, 1, Action::Select(Selection::new(1, 1))).status,
            Err(Error::Invalid)
        );
        assert_eq!(p.model, before);
    }
}
#[test]
fn insertion_joining_grapheme_snaps_caret_right() {
    let mut p = Provider::new(
        Model::new(1, 1, "👩💻", "", Selection::new(4, 4)).unwrap(),
        7,
    )
    .unwrap();
    p.grant(1, Rights::ALL).unwrap();
    assert_eq!(
        run(&mut p, 1, Action::Insert("\u{200d}".into())).status,
        Ok(true)
    );
    assert_eq!(p.model.document.selection(), Selection::new(11, 11));
}
#[test]
fn invalid_revision_rights_epoch_versions_and_caller_leave_model_intact() {
    let mut p = provider();
    let before = p.model.clone();
    for (n, error) in [
        (0, Error::Unsupported),
        (1, Error::Conflict),
        (2, Error::Conflict),
        (3, Error::Denied),
        (4, Error::Conflict),
    ] {
        let mut r = request(&p, n + 1, Action::Insert("x".into()));
        match n {
            0 => r.version = 2,
            1 => r.expected_document_revision = 10,
            2 => r.epoch = 8,
            3 => r.caller = 99,
            _ => r.expected_model_revision = 2,
        }
        assert_eq!(p.execute(r, 1, false).status, Err(error));
        assert_eq!(p.model, before);
    }
    p.grant(
        1,
        Rights {
            edit: false,
            preferences: true,
            session: false,
        },
    )
    .unwrap();
    assert_eq!(run(&mut p, 10, Action::Undo).status, Err(Error::Denied));
    assert_eq!(
        run(&mut p, 11, Action::Unknown("save.file".into())).status,
        Err(Error::Unsupported)
    );
    assert_eq!(p.model, before);
}
#[test]
fn retries_are_exact_bounded_and_never_repeat_evicted_effects() {
    let mut p = provider();
    let r = request(&p, 1, Action::Insert("x".into()));
    let committed = p.execute(r.clone(), 1, false);
    assert_eq!(p.execute(r.clone(), 1000, true), committed); // Too late to cancel a committed edit.
    let mut changed = r.clone();
    changed.action = Action::Delete;
    assert_eq!(
        p.execute(changed, 1, false).status,
        Err(Error::DuplicateMismatch)
    );
    for id in 2..=34 {
        run(&mut p, id, Action::Motion(id % 2 == 0));
    }
    let before = p.model.clone();
    assert_eq!(p.execute(r, 1, false).status, Err(Error::RetryExpired));
    assert_eq!(p.model, before);
    p.grant(
        1,
        Rights {
            edit: false,
            preferences: false,
            session: false,
        },
    )
    .unwrap();
    let mut r = request(&p, 34, Action::Motion(true));
    r.expected_model_revision = 0;
    assert_eq!(p.execute(r, 1, false).status, Err(Error::Denied));
}
#[test]
fn cancellation_deadline_and_noop_are_deterministic() {
    let mut p = provider();
    let before = p.model.clone();
    let r = request(&p, 1, Action::Delete);
    assert_eq!(p.execute(r, 1, true).status, Err(Error::Cancelled));
    let r = request(&p, 2, Action::Delete);
    assert_eq!(p.execute(r, 100, false).status, Err(Error::Deadline));
    assert_eq!(p.model, before);
    let r = request(&p, 3, Action::Motion(false));
    assert_eq!(p.execute(r.clone(), 1, false).status, Ok(false));
    assert_eq!(p.execute(r, 200, true).status, Ok(false));
    assert_eq!(p.model, before);
}
fn context() -> Context {
    Context {
        id: 3,
        caller: 1,
        provider_epoch: 7,
        document: 1,
        source_revision: 11,
        provenance: "synthetic.owner".into(),
        purpose: Effect::Document,
        expires: 10,
        size: 7,
        selection: Selection::new(7, 14),
        permitted_action: "edit.insert".into(),
    }
}
#[test]
fn contexts_check_owner_scope_revision_expiry_and_revocation() {
    let mut p = provider();
    p.issue_context(context()).unwrap();
    assert_eq!(p.resolve_context(1, 3, 1), Ok("unsaved"));
    assert_eq!(p.resolve_context(2, 3, 1), Err(Error::Denied));
    assert_eq!(p.resolve_context(1, 3, 10), Err(Error::Expired));
    let before = p.model.clone();
    let mut r = request(&p, 1, Action::Insert("x".into()));
    r.context = Some(99);
    assert_eq!(p.execute(r, 1, false).status, Err(Error::Denied));
    let mut r = request(&p, 2, Action::Insert("x".into()));
    r.context = Some(3);
    assert_eq!(p.execute(r, 10, false).status, Err(Error::Expired));
    let mut r = request(&p, 3, Action::Delete);
    r.context = Some(3);
    assert_eq!(p.execute(r, 1, false).status, Err(Error::Denied));
    assert_eq!(p.model, before);
    run(&mut p, 4, Action::Insert("x".into()));
    let mut r = request(&p, 5, Action::Insert("y".into()));
    r.context = Some(3);
    assert_eq!(p.execute(r, 1, false).status, Err(Error::Conflict));
    assert_eq!(p.resolve_context(1, 3, 1), Err(Error::Conflict));
    p.revoke_context(3);
    assert_eq!(p.resolve_context(1, 3, 1), Err(Error::Denied));
}
#[test]
fn rejects_fabricated_and_oversize_context_descriptors() {
    let mut p = provider();
    for n in 0..5 {
        let mut c = context();
        match n {
            0 => c.provider_epoch = 8,
            1 => c.source_revision = 10,
            2 => c.size = 4097,
            3 => c.caller = 99,
            _ => c.provenance = String::new(),
        };
        assert!(p.issue_context(c).is_err());
    }
}
#[test]
fn snapshot_preserves_history_preferences_session_and_unsaved_state() {
    let mut p = provider();
    for (n, a) in [
        Action::Insert("notes".into()),
        Action::Search("notes".into()),
        Action::Scroll(31),
        Action::Theme(Theme::HighContrast),
        Action::TextSize(20),
        Action::Motion(true),
        Action::Effects(true),
        Action::FileScroll(100),
        Action::FileSelect(101),
        Action::Focus("search".into()),
        Action::Window([2, 3, 400, 300]),
        Action::Undo,
    ]
    .into_iter()
    .enumerate()
    {
        assert!(run(&mut p, n as u64 + 1, a).status.is_ok());
    }
    let bytes = p.model.encode().unwrap();
    let imported = Model::decode(&bytes).unwrap();
    assert_eq!(p.model, imported);
    p.restore(&bytes).unwrap();
    let mut q = Provider::new(imported, 7).unwrap();
    q.grant(1, Rights::ALL).unwrap();
    assert_eq!(run(&mut p, 20, Action::Redo), run(&mut q, 20, Action::Redo));
    assert_eq!(p.model, q.model);
}
#[test]
fn every_truncation_and_invalid_header_preserve_active_state() {
    let mut p = provider();
    let before = p.model.clone();
    let bytes = p.model.encode().unwrap();
    for n in 0..bytes.len() {
        assert!(p.restore(&bytes[..n]).is_err());
        if n >= 17 {
            let mut truncated = bytes[..n].to_vec();
            truncated[13..17].copy_from_slice(&(n as u32).to_le_bytes());
            assert!(p.restore(&truncated).is_err());
        }
        assert_eq!(p.model, before);
    }
    for at in [0, 8, 10, 13, 41] {
        let mut b = bytes.clone();
        b[at] = 255;
        assert!(p.restore(&b).is_err());
        assert_eq!(p.model, before);
    }
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(p.restore(&trailing).is_err());
    assert_eq!(p.restore(&vec![0; MAX_SNAPSHOT + 1]), Err(Error::Bounds));
    assert_eq!(p.model, before);
}

#[test]
fn snapshot_mutations_are_rejected_or_canonical_and_bounded() {
    let bytes = provider().model.encode().unwrap();
    for at in 0..bytes.len() {
        for byte in [0, 1, 127, 128, 255] {
            let mut mutated = bytes.clone();
            mutated[at] = byte;
            if let Ok(model) = Model::decode(&mutated) {
                assert_eq!(model.encode().unwrap(), mutated);
            }
        }
    }
}

#[test]
fn authority_capacity_and_request_namespaces_remain_bounded() {
    let mut p = provider();
    for caller in 2..=AUTHORITIES as u64 {
        p.grant(caller, Rights::ALL).unwrap();
    }
    assert_eq!(p.grant(17, Rights::ALL), Err(Error::Bounds));
    let a = request(&p, 1, Action::Motion(true));
    assert_eq!(p.execute(a, 1, false).status, Ok(true));
    let mut b = request(&p, 1, Action::Motion(false));
    b.caller = 2;
    assert_eq!(p.execute(b, 1, false).status, Ok(true));
    for id in 1..=AUTHORITIES as u64 {
        let mut c = context();
        c.id = id;
        p.issue_context(c).unwrap();
    }
    let mut c = context();
    c.id = 17;
    assert_eq!(p.issue_context(c), Err(Error::Bounds));
}
#[test]
fn snapshot_rejects_invalid_utf8_lengths_bools_enums_history_and_anchor() {
    let p = provider();
    let bytes = p.model.encode().unwrap();
    // Header 17 + model/id/document revisions 24 = text length at byte 41.
    for (at, value) in [(45, 255), (60, 255), (41, 255)] {
        let mut b = bytes.clone();
        b[at] = value;
        assert!(Model::decode(&b).is_err());
    }
    // Use a tiny independently located no-history snapshot for enum/bool corruption.
    let mut m = Model::new(1, 1, "", "", Selection::new(0, 0)).unwrap();
    for (at, value) in [(73, 3), (76, 2), (77, 2), (57, 33)] {
        let mut b = m.encode().unwrap();
        b[at] = value;
        assert!(Model::decode(&b).is_err(), "offset {at}");
    }
    m.document.revision = u64::MAX; // Arithmetic exhaustion cannot leak history/edits.
    let mut p = Provider::new(m, 7).unwrap();
    p.grant(1, Rights::ALL).unwrap();
    let before = p.model.clone();
    assert_eq!(
        run(&mut p, 1, Action::Insert("x".into())).status,
        Err(Error::Exhausted)
    );
    assert_eq!(p.model, before);
}
#[test]
fn bounds_and_history_limits_are_enforced_without_truncating_text() {
    let mut p = provider();
    let before = p.model.clone();
    for (id, a) in [
        Action::Insert("x".repeat(MAX_QUERY + 1)),
        Action::TextSize(7),
        Action::Scroll(1_000_001),
        Action::FileSelect(10000),
        Action::Window([0, 0, 0, 100]),
    ]
    .into_iter()
    .enumerate()
    {
        assert!(run(&mut p, id as u64 + 1, a).status.is_err());
        assert_eq!(p.model, before);
    }
    for id in 10..50 {
        run(&mut p, id, Action::Insert("x".into()));
    }
    assert_eq!(p.model.document.undo_len(), HISTORY);
    let mut p = Provider::new(
        Model::new(
            1,
            1,
            &"x".repeat(MAX_TEXT),
            "",
            Selection::new(MAX_TEXT as u32, MAX_TEXT as u32),
        )
        .unwrap(),
        7,
    )
    .unwrap();
    p.grant(1, Rights::ALL).unwrap();
    let before = p.model.clone();
    assert_eq!(
        run(&mut p, 1, Action::Insert("x".into())).status,
        Err(Error::Bounds)
    );
    assert_eq!(p.model, before);
}
#[test]
fn old_snapshots_cannot_rewind_live_state_or_duplicate_tracking() {
    let mut p = provider();
    let old = p.model.encode().unwrap();
    let r = request(&p, 1, Action::Insert("x".into()));
    let result = p.execute(r.clone(), 1, false);
    let fresh = p.model.encode().unwrap();
    assert_eq!(p.restore(&old), Err(Error::Conflict));
    p.restore(&fresh).unwrap();
    assert_eq!(p.execute(r, 1, false), result);
}
#[test]
fn all_entries_have_identical_authority_and_state() {
    let mut baseline = None;
    for entry in [
        Entry::Button,
        Entry::Shortcut,
        Entry::Semantic,
        Entry::Automation,
    ] {
        let mut p = provider();
        let mut outcomes = Vec::new();
        for (i, a) in [
            Action::Insert("notes".into()),
            Action::Undo,
            Action::Redo,
            Action::Search("notes".into()),
            Action::Motion(true),
        ]
        .into_iter()
        .enumerate()
        {
            let r = request(&p, i as u64 + 1, a);
            outcomes.push(Adapter::new(&mut p, entry).invoke(r, 1, false));
        }
        p.grant(
            1,
            Rights {
                edit: false,
                preferences: false,
                session: false,
            },
        )
        .unwrap();
        let r = request(&p, 10, Action::Delete);
        outcomes.push(Adapter::new(&mut p, entry).invoke(r, 1, false));
        let result = (p.model.encode().unwrap(), outcomes);
        if let Some(b) = &baseline {
            assert_eq!(&result, b);
        } else {
            baseline = Some(result);
        }
    }
}
