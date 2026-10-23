use crate::{input::*, layout::*, scene::*, semantics::*, *};
use alloc::{format, vec};
use editor_model::{Action, Entry, Model, Selection};

fn node(id: &str, kind: Kind, rect: [f32; 4]) -> Node {
    Node {
        id: id.into(),
        kind,
        children: vec![],
        rect: Rect(rect),
        clip: false,
        opacity: 1.0,
        text: String::new(),
        action: String::new(),
        resource: String::new(),
        semantic_id: format!("sem.{id}"),
        role: "group".into(),
        name: id.into(),
        focusable: matches!(
            kind,
            Kind::Editor | Kind::Field | Kind::List | Kind::Toggle | Kind::Button
        ),
    }
}
fn tree() -> Tree {
    let mut root = node("desktop", Kind::Column, [0.0, 0.0, 400.0, 300.0]);
    root.children = vec![
        "editor".into(),
        "search".into(),
        "files-window".into(),
        "settings".into(),
    ];
    let mut settings = node("settings", Kind::Column, [200.0, 20.0, 180.0, 90.0]);
    settings.clip = true;
    settings.children = vec!["reduced-motion".into()];
    let mut toggle = node("reduced-motion", Kind::Toggle, [210.0, 40.0, 160.0, 28.0]);
    toggle.action = "settings.motion@1".into();
    Tree {
        nodes: vec![
            root,
            node("editor", Kind::Editor, [10.0, 10.0, 180.0, 100.0]),
            node("search", Kind::Field, [10.0, 120.0, 180.0, 24.0]),
            node("files-window", Kind::List, [10.0, 160.0, 180.0, 120.0]),
            settings,
            toggle,
        ],
        root: "desktop".into(),
        revision: 1,
        generation: 1,
    }
}
fn tokens() -> Tokens {
    Tokens {
        colors: [[0, 0, 0, 255]; 7],
        spacing: 4.0,
        radius: 8.0,
        control_radius: 4.0,
        body_size: 14,
        editor_size: 14,
        title_size: 18,
        tint: [20, 24, 32, 192],
        fallback: [32, 38, 50, 255],
        blur_radius: 8,
        passes: 3,
        reduced_effects: false,
        reduced_motion: false,
    }
}
fn controller() -> Controller {
    Controller::new(Model::new(1, 1, "draft", "", Selection::new(5, 5)).unwrap()).unwrap()
}
fn event(sequence: u64, revision: u64, target: &str, kind: EventKind) -> Event {
    Event {
        sequence,
        time_us: sequence,
        revision,
        target: target.into(),
        kind,
    }
}

#[test]
fn flow_measures_content_and_arranges_fixed_fill_and_overlay() {
    let mut t = tree();
    t.nodes
        .retain(|n| ["desktop", "editor", "search"].contains(&n.id.as_str()));
    t.nodes[0].children = vec!["editor".into(), "search".into()];
    let mut p = Plan::anchored(&t).unwrap();
    for r in &mut p.rules {
        r.placement = Placement::Flow;
    }
    p.rules[0].padding = 10.0;
    p.rules[0].gap = 6.0;
    p.rule_mut("editor").unwrap().size = [Size::Fill, Size::Fixed(100.0)];
    p.rule_mut("search").unwrap().size = [Size::Fill, Size::Fill];
    let (out, work) = p.arrange(&t, Rect([20.0, 30.0, 300.0, 200.0])).unwrap();
    assert_eq!(
        out.node("editor").unwrap().rect,
        Rect([30.0, 40.0, 280.0, 100.0])
    );
    assert_eq!(
        out.node("search").unwrap().rect,
        Rect([30.0, 146.0, 280.0, 74.0])
    );
    assert_eq!(
        work,
        Work {
            measured: 3,
            arranged: 3,
            depth: 2
        }
    );
    t.nodes[0].kind = Kind::Row;
    p.rule_mut("editor").unwrap().size = [Size::Fixed(80.0), Size::Fill];
    p.rule_mut("search").unwrap().size = [Size::Fill, Size::Fill];
    let (out, _) = p.arrange(&t, Rect([0.0, 0.0, 300.0, 200.0])).unwrap();
    assert_eq!(
        out.node("search").unwrap().rect,
        Rect([96.0, 10.0, 194.0, 180.0])
    );
    t.nodes[0].kind = Kind::Overlay;
    let (out, _) = p.arrange(&t, Rect([0.0, 0.0, 300.0, 200.0])).unwrap();
    assert_eq!(
        out.node("search").unwrap().rect,
        Rect([10.0, 10.0, 280.0, 180.0])
    );
    t.nodes[0].kind = Kind::Column;
    p.rule_mut("search").unwrap().size = [Size::Fixed(80.0), Size::Fixed(24.0)];
    p.rule_mut("editor").unwrap().size = [Size::Content, Size::Content];
    let (out, _) = p.arrange(&t, Rect([0.0, 0.0, 300.0, 200.0])).unwrap();
    assert_eq!(out.node("editor").unwrap().rect.0[2..], [180.0, 100.0]);
}
#[test]
fn scroll_transforms_hit_and_semantics_share_arranged_bounds() {
    let mut t = tree();
    t.nodes[4].kind = Kind::Scroll;
    let mut p = Plan::anchored(&t).unwrap();
    p.rule_mut("settings").unwrap().scroll = [0.0, 12.0];
    let (out, _) = p.arrange(&t, Rect([0.0, 0.0, 400.0, 300.0])).unwrap();
    assert_eq!(
        out.node("reduced-motion").unwrap().rect,
        Rect([210.0, 28.0, 160.0, 28.0])
    );
    let point = logical_pointer(330.0, 48.0, 150).unwrap();
    assert_eq!(
        out.hit_rounded(point[0], point[1], &tokens()).unwrap().id,
        "reduced-motion"
    );
    let s = Snapshot::build(&out, controller().provider.model()).unwrap();
    let n = s
        .items
        .iter()
        .find(|n| n.id == "sem.reduced-motion")
        .unwrap();
    assert_eq!(n.parent.as_deref(), Some("sem.settings"));
    assert_eq!(n.bounds, Rect([210.0, 28.0, 160.0, 28.0]));
    assert_ne!(
        out.hit_rounded(200.1, 20.1, &tokens()).unwrap().id,
        "settings"
    );
}
#[test]
fn layout_rejects_pressure_cycles_depth_and_nonfinite_atomically() {
    let t = tree();
    let mut scene = Scene::new(t.clone(), tokens()).unwrap();
    let mut p = Plan::anchored(&t).unwrap();
    p.rules[1].gap = f32::NAN;
    assert!(scene.arrange(&t, &p, t.nodes[0].rect).is_err());
    assert_eq!(scene.tree.nodes, t.nodes);
    assert_eq!(scene.measured_nodes, 0);
    let mut p = Plan::anchored(&t).unwrap();
    p.rule_mut("files-window").unwrap().size[1] = Size::Fixed(2000.0);
    assert_eq!(
        p.arrange(&t, t.nodes[0].rect).unwrap_err(),
        Error::Exhausted
    );
    let mut bad = t.clone();
    bad.nodes[1].children.push("desktop".into());
    assert!(p.arrange(&bad, t.nodes[0].rect).is_err());
    let mut deep = Tree {
        nodes: vec![],
        root: "n0".into(),
        revision: 1,
        generation: 1,
    };
    for i in 0..33 {
        let mut n = node(&format!("n{i}"), Kind::Column, [0.0, 0.0, 10.0, 10.0]);
        if i < 32 {
            n.children.push(format!("n{}", i + 1));
        }
        deep.nodes.push(n);
    }
    assert!(Plan::anchored(&deep).is_err());
    let mut p = Plan::anchored(&t).unwrap();
    p.rules[1] = p.rules[0].clone();
    assert_eq!(
        p.arrange(&t, t.nodes[0].rect).unwrap_err(),
        Error::Duplicate
    );
}
#[test]
fn publication_classifies_paint_semantic_and_layout_changes() {
    let mut s = Scene::new(tree(), tokens()).unwrap();
    s.dirty = Dirty::default();
    let mut next = s.tree.clone();
    next.revision += 1;
    let mut color = s.tokens.clone();
    color.colors[0] = [255; 4];
    s.publish(next, color).unwrap();
    assert_eq!(
        s.dirty,
        Dirty {
            layout: false,
            paint: true,
            semantics: false
        }
    );
    s.dirty = Dirty::default();
    let mut next = s.tree.clone();
    next.revision += 1;
    next.nodes[1].name = "renamed".into();
    s.publish(next, s.tokens.clone()).unwrap();
    assert_eq!(
        s.dirty,
        Dirty {
            layout: false,
            paint: true,
            semantics: true
        }
    );
    s.dirty = Dirty::default();
    let mut next = s.tree.clone();
    next.revision += 1;
    next.nodes[1].rect.0[2] += 1.0;
    s.publish(next, s.tokens.clone()).unwrap();
    assert_eq!(s.dirty, Dirty::ALL);
}
#[test]
fn releases_across_revisions_and_moved_capture_never_activate() {
    let mut c = controller();
    let mut t = tree();
    c.event(
        &t,
        event(
            1,
            1,
            "reduced-motion",
            EventKind::Pointer {
                x: 220.0,
                y: 50.0,
                button: 1,
                down: true,
            },
        ),
    )
    .unwrap();
    t.revision = 2;
    t.nodes[5].rect.0[0] += 2.0;
    c.reconcile(&t).unwrap();
    c.event(
        &t,
        event(
            2,
            1,
            "reduced-motion",
            EventKind::Pointer {
                x: 220.0,
                y: 50.0,
                button: 1,
                down: false,
            },
        ),
    )
    .unwrap();
    assert!(!c.provider.model().preferences().reduced_motion);
    assert!(c.capture.is_empty() && c.buttons.is_empty());
    c.invoke(Action::Focus("editor".into()), Entry::Shortcut)
        .unwrap();
    c.event(
        &t,
        event(
            3,
            2,
            "editor",
            EventKind::Key {
                usage: 229,
                down: true,
            },
        ),
    )
    .unwrap();
    c.event(
        &t,
        event(
            4,
            1,
            "removed",
            EventKind::Key {
                usage: 229,
                down: false,
            },
        ),
    )
    .unwrap();
    assert!(c.keys.is_empty());
    let before = c.provider.model().encode().unwrap();
    assert_eq!(
        c.event(&t, event(5, 1, "editor", EventKind::Text("x".into()))),
        Err(Error::Revision)
    );
    assert_eq!(before, c.provider.model().encode().unwrap());
    c.event(&t, event(5, 2, "editor", EventKind::Text("x".into())))
        .unwrap();
}
#[test]
fn overflow_and_bad_queue_cancel_transients_preserving_accepted_edits() {
    let mut c = controller();
    let t = tree();
    c.invoke(Action::Insert(" accepted".into()), Entry::Button)
        .unwrap();
    let before = c.provider.model().encode().unwrap();
    for i in 1..=256 {
        c.enqueue(event(
            i,
            1,
            "editor",
            EventKind::Key {
                usage: 225,
                down: true,
            },
        ))
        .unwrap();
    }
    assert_eq!(
        c.enqueue(event(257, 1, "editor", EventKind::Reset)),
        Err(Error::Exhausted)
    );
    assert_eq!(c.dropped, 257);
    assert_eq!(before, c.provider.model().encode().unwrap());
    c.event(
        &t,
        event(
            258,
            1,
            "editor",
            EventKind::Key {
                usage: 225,
                down: false,
            },
        ),
    )
    .unwrap();
    c.enqueue(event(259, 0, "editor", EventKind::Text("discard".into())))
        .unwrap();
    c.enqueue(event(260, 1, "editor", EventKind::Text("discard".into())))
        .unwrap();
    assert_eq!(c.drain(&t), Err(Error::Revision));
    assert_eq!(c.dropped, 259);
    assert_eq!(before, c.provider.model().encode().unwrap());
    c.event(&t, event(261, 1, "editor", EventKind::Text("!".into())))
        .unwrap();
    assert!(c.pending.is_empty());
}
#[test]
fn modal_focus_traps_reverse_tab_and_escape_restores_owner() {
    let t = tree();
    let mut c = controller();
    c.open_modal(&t, "settings").unwrap();
    assert_eq!(c.provider.model().session().focus, "reduced-motion");
    c.event(
        &t,
        event(
            1,
            1,
            "reduced-motion",
            EventKind::Key {
                usage: 229,
                down: true,
            },
        ),
    )
    .unwrap();
    c.event(
        &t,
        event(
            2,
            1,
            "reduced-motion",
            EventKind::Key {
                usage: 43,
                down: true,
            },
        ),
    )
    .unwrap();
    assert_eq!(c.provider.model().session().focus, "reduced-motion");
    assert!(c.semantic(&t, 1, "sem.editor", Command::Focus).is_err());
    c.event(
        &t,
        event(
            3,
            1,
            "reduced-motion",
            EventKind::Key {
                usage: 41,
                down: true,
            },
        ),
    )
    .unwrap();
    assert_eq!(c.provider.model().session().focus, "editor");
    assert!(c.modal.is_empty() && c.keys.is_empty());
}
#[test]
fn composition_and_shortcuts_are_separate_from_committed_text() {
    let t = tree();
    let mut c = controller();
    let before = c.provider.model().encode().unwrap();
    c.event(
        &t,
        event(
            1,
            1,
            "editor",
            EventKind::Composition {
                phase: "start".into(),
                text: "preedit".into(),
            },
        ),
    )
    .unwrap();
    c.event(
        &t,
        event(
            2,
            1,
            "editor",
            EventKind::Key {
                usage: 42,
                down: true,
            },
        ),
    )
    .unwrap();
    assert_eq!(before, c.provider.model().encode().unwrap());
    c.event(
        &t,
        event(
            3,
            1,
            "editor",
            EventKind::Composition {
                phase: "cancel".into(),
                text: "".into(),
            },
        ),
    )
    .unwrap();
    c.event(
        &t,
        event(
            4,
            1,
            "editor",
            EventKind::Key {
                usage: 228,
                down: true,
            },
        ),
    )
    .unwrap();
    assert_eq!(
        c.event(&t, event(5, 1, "editor", EventKind::Text("z".into()))),
        Err(Error::Input)
    );
    c.event(
        &t,
        event(
            6,
            1,
            "editor",
            EventKind::Key {
                usage: 228,
                down: false,
            },
        ),
    )
    .unwrap();
    c.event(
        &t,
        event(
            7,
            1,
            "editor",
            EventKind::Composition {
                phase: "start".into(),
                text: "新".into(),
            },
        ),
    )
    .unwrap();
    c.event(
        &t,
        event(
            8,
            1,
            "editor",
            EventKind::Composition {
                phase: "commit".into(),
                text: "新".into(),
            },
        ),
    )
    .unwrap();
    assert_eq!(c.provider.model().document().text(), "draft新");
    assert!(c.composition.is_none());
}
#[test]
fn semantic_virtual_navigation_notifications_and_executor_parity() {
    let t = tree();
    let mut a = controller();
    let mut b = controller();
    let old = Snapshot::build(&t, a.provider.model()).unwrap();
    a.semantic(&t, 1, "sem.reduced-motion", Command::Activate)
        .unwrap();
    b.activate(&t, "reduced-motion", Entry::Button).unwrap();
    assert_eq!(a.provider.model().encode(), b.provider.model().encode());
    let next = Snapshot::build(&t, a.provider.model()).unwrap();
    assert!(
        next.diff(&old)
            .contains(&Notice::State("sem.reduced-motion".into()))
    );
    a.semantic(&t, 1, "sem.files-window", Command::SelectRow(9999))
        .unwrap();
    let next = Snapshot::build(&t, a.provider.model()).unwrap();
    let row = next
        .items
        .iter()
        .find(|n| n.position == Some(10000))
        .unwrap();
    assert!(row.selected);
    assert_eq!(row.count, 10000);
    assert_eq!(a.provider.model().session().file_scroll, 9995);
    assert!(
        a.semantic(&t, 0, "sem.files-window", Command::Navigate(-1))
            .is_err()
    );
    a.semantic(&t, 1, "virtual:sem.files-window:9998", Command::Activate)
        .unwrap();
    assert_eq!(a.provider.model().session().selected_file, 9998);
    assert!(
        a.semantic(&t, 1, "virtual:sem.files-window:0", Command::Activate)
            .is_err()
    );
}

#[test]
fn unsupported_text_and_explicit_queue_reset_preserve_document() {
    let t = tree();
    let mut c = controller();
    let before = c.provider.model().encode().unwrap();
    assert_eq!(
        c.invoke(Action::Insert("\t".into()), Entry::Semantic),
        Err(Error::Text)
    );
    assert_eq!(
        c.invoke(Action::Insert("\n".repeat(256)), Entry::Button),
        Err(Error::Text)
    );
    assert_eq!(before, c.provider.model().encode().unwrap());
    c.enqueue(event(1, 1, "", EventKind::Reset)).unwrap();
    c.enqueue(event(2, 1, "editor", EventKind::Text("discard".into())))
        .unwrap();
    c.drain(&t).unwrap();
    assert_eq!(c.dropped, 1);
    assert_eq!(before, c.provider.model().encode().unwrap());
    c.event(&t, event(3, 1, "editor", EventKind::Text("!".into())))
        .unwrap();
}

#[test]
fn cached_semantics_skip_paint_only_publications_and_retire_modal_hierarchy() {
    let mut s = Scene::new(tree(), tokens()).unwrap();
    let c = controller();
    s.semantics(c.provider.model());
    assert_eq!(s.semantic_passes, 1);
    s.layout();
    let mut next = s.tree.clone();
    next.revision += 1;
    let mut colors = s.tokens.clone();
    colors.colors[0] = [255; 4];
    s.publish(next, colors).unwrap();
    s.semantics(c.provider.model());
    assert_eq!(s.semantic_passes, 1);
    assert!(!s.layout());
    let old = Snapshot::build(&s.tree, c.provider.model()).unwrap();
    s.retire("settings").unwrap();
    let next = Snapshot::build(&s.tree, c.provider.model()).unwrap();
    assert!(
        next.diff(&old)
            .contains(&Notice::Removed("sem.reduced-motion".into()))
    );
    assert!(s.tree.node("settings").is_err());
}
#[test]
fn motion_interrupts_at_visible_value_and_deadlines_stop_when_settled() {
    let mut m = motion::Motion::default();
    m.transition("panel", 0.0, 1.0, 0, false).unwrap();
    m.tick(60_000).unwrap();
    assert_eq!(m.opacity("panel", 0.0), 0.5);
    m.transition("panel", 0.0, 0.0, 60_000, false).unwrap();
    assert_eq!(m.opacity("panel", 0.0), 0.5);
    m.tick(120_000).unwrap();
    assert_eq!(m.opacity("panel", 0.0), 0.25);
    m.tick(180_000).unwrap();
    assert_eq!(m.next_deadline(), None);
    m.transition("panel", 0.0, 1.0, 180_000, true).unwrap();
    assert_eq!(m.opacity("panel", 0.0), 1.0);
    assert_eq!(m.next_deadline(), None);
    m.timers(true, true, true, 180_000, false).unwrap();
    assert_eq!(m.next_deadline(), Some(196_667));
    let tick = m.tick(1_180_000).unwrap();
    assert!(tick.paint && tick.clock && tick.drag);
    assert!(m.caret_visible);
    m.timers(true, false, false, 1_180_000, true).unwrap();
    assert_eq!(m.next_deadline(), None);
    for i in 0..63 {
        m.transition(&format!("id{i}"), 0.0, 1.0, 1_180_000, false)
            .unwrap();
    }
    assert_eq!(
        m.transition("overflow", 0.0, 1.0, 1_180_000, false),
        Err(Error::Exhausted)
    );
    assert!(m.tick(1).is_err());
}
