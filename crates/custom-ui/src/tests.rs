use super::*;
use crate::{paint::*, scene::*, surface::*};
use alloc::{string::ToString, vec};

#[test]
fn leases_prevent_writes_and_delayed_release() {
    let mut s = Surface::new(3, 2, 16, 1).unwrap();
    let first = s.acquire(7).unwrap();
    s.write(first).unwrap().fill(3);
    s.commit(first, &[[0, 0, 3, 2]]).unwrap();
    assert_eq!(s.write(first), Err(Error::Lease));
    assert_eq!(s.acquire(7), Err(Error::Exhausted));
    s.release(first).unwrap();
    let next = s.acquire(7).unwrap();
    assert!(next.serial > first.serial);
    assert_eq!(s.release(first), Err(Error::Lease));
    assert_eq!(s.release(next), Err(Error::Lease));
    assert_eq!(s.commit(next, &[[2, 0, 2, 1]]), Err(Error::Bounds));
    s.commit(next, &[[0, 0, 3, 2]]).unwrap();
    s.release(next).unwrap();
}
#[test]
fn fractional_clip_and_padding_are_exact() {
    let mut bytes = vec![0u8; 40 * 8];
    let mut c = Canvas::new(&mut bytes, 8, 8, 40, 150).unwrap();
    c.fill(
        Rect([0.0, 0.0, 8.0, 8.0]),
        Rect([1.0, 1.0, 2.0, 2.0]),
        0.0,
        [255, 0, 0, 128],
        255,
    );
    for y in 0..8 {
        for x in 0..8 {
            let at = y * 40 + x * 4;
            assert_eq!(
                &bytes[at..at + 4],
                if (1..5).contains(&x) && (1..5).contains(&y) {
                    &[128, 0, 0, 128]
                } else {
                    &[0, 0, 0, 0]
                }
            );
        }
        assert_eq!(&bytes[y * 40 + 32..y * 40 + 40], &[0; 8]);
    }
}
#[test]
fn blur_matches_independent_slow_nonconstant_oracle() {
    let (w, h) = (9, 7);
    let mut p: Vec<[u8; 4]> = (0..w * h)
        .map(|n| {
            [
                (n * 31 % 256) as u8,
                (n * 7 % 256) as u8,
                (n * 13 % 256) as u8,
                255,
            ]
        })
        .collect();
    let mut expected = p.clone();
    for _ in 0..3 {
        for axis in 0..2 {
            let old = expected.clone();
            for y in 0..h {
                for x in 0..w {
                    let mut sum = [0u32; 4];
                    for d in -2..=2 {
                        let xx = if axis == 0 {
                            (x as i32 + d).clamp(0, w as i32 - 1) as usize
                        } else {
                            x
                        };
                        let yy = if axis == 1 {
                            (y as i32 + d).clamp(0, h as i32 - 1) as usize
                        } else {
                            y
                        };
                        for k in 0..4 {
                            sum[k] += old[yy * w + xx][k] as u32
                        }
                    }
                    expected[y * w + x] = sum.map(|n| ((n + 2) / 5) as u8);
                }
            }
        }
    }
    blur(&mut p, w, h, 2, 3).unwrap();
    assert_eq!(p, expected);
}
#[test]
fn random_access_list_matches_sequential_generator() {
    let mut state = 20261020u32;
    for row in 0..10000 {
        state = state.wrapping_mul(1664525).wrapping_add(1013904223);
        assert_eq!(lcg_at(20261020, row + 1), state)
    }
    for count in [10000, 1_000_000, 1_000_000_000] {
        assert_eq!(realize(count - 20, count, 328.0, 24.0).unwrap().len(), 15)
    }
    assert_eq!(realize(0, 10000, 2000.0, 24.0), Err(Error::Exhausted));
}
#[test]
fn invalid_graphs_reject_without_recursing_forever() {
    let root = Node {
        id: "desktop".to_string(),
        kind: Kind::Overlay,
        children: vec![],
        rect: Rect([0.0, 0.0, 640.0, 360.0]),
        clip: false,
        opacity: 1.0,
        text: "".into(),
        action: "".into(),
        resource: "".into(),
        semantic_id: "sem.desktop".into(),
        role: "group".into(),
        name: "desktop".into(),
        focusable: false,
    };
    let mut tree = Tree {
        nodes: vec![root],
        root: "desktop".into(),
        revision: 4,
        generation: 7,
    };
    tree.validate().unwrap();
    tree.nodes[0].children.push("desktop".into());
    assert_eq!(tree.validate(), Err(Error::Invalid));
    tree.nodes[0].children.clear();
    tree.nodes[0].rect.0[2] = f32::INFINITY;
    assert_eq!(tree.validate(), Err(Error::Bounds));
}
#[test]
fn malformed_images_and_surface_geometry_reject() {
    for (w, h, stride, count) in [
        (0, 1, 4, 1),
        (4097, 1, 16388, 1),
        (1, 1, 3, 1),
        (1, 1, 4, 4),
    ] {
        assert!(Surface::new(w, h, stride, count).is_err())
    }
    let mut pixels = vec![0; 16];
    let mut canvas = Canvas::new(&mut pixels, 2, 2, 8, 100).unwrap();
    assert_eq!(
        canvas.image(
            Rect([0.0, 0.0, 2.0, 2.0]),
            Rect([0.0, 0.0, 2.0, 2.0]),
            1,
            1,
            &[[255, 0, 0, 0]]
        ),
        Err(Error::Resource)
    );
}
#[test]
fn compiled_behavior_feature_is_explicit() {
    use crate::input::*;
    use editor_model::{Model, Selection};
    let model = Model::new(1, 11, "Draft: unsaved\n", "", Selection::new(7, 14)).unwrap();
    let mut controller = Controller::new(model).unwrap();
    let editor = Node {
        id: "editor".into(),
        kind: Kind::Editor,
        children: vec![],
        rect: Rect([0.0, 0.0, 100.0, 100.0]),
        clip: true,
        opacity: 1.0,
        text: "".into(),
        action: "edit.insert@1".into(),
        resource: "document.unsaved".into(),
        semantic_id: "sem.editor".into(),
        role: "textbox".into(),
        name: "Editor".into(),
        focusable: true,
    };
    let tree = Tree {
        nodes: vec![editor],
        root: "editor".into(),
        revision: 4,
        generation: 7,
    };
    controller
        .event(
            &tree,
            Event {
                sequence: 1,
                time_us: 1,
                revision: 4,
                target: "editor".into(),
                kind: EventKind::Text("notes".into()),
            },
        )
        .unwrap();
    assert_eq!(
        controller.provider.model().document().text(),
        if cfg!(feature = "behavior-v2") {
            "Draft: NOTES\n"
        } else {
            "Draft: notes\n"
        }
    );
}

#[test]
fn candidate_rejects_text_beyond_render_budget_before_model_change() {
    use crate::input::Controller;
    use editor_model::{Action, Entry, Model, Selection};
    let text = "x".repeat(text_core::MAX_TEXT);
    let mut c =
        Controller::new(Model::new(1, 11, &text, "", Selection::new(4096, 4096)).unwrap()).unwrap();
    let before = c.provider.model().encode().unwrap();
    assert_eq!(
        c.invoke(Action::Insert("y".into()), Entry::Button),
        Err(Error::Bounds)
    );
    assert_eq!(before, c.provider.model().encode().unwrap());
}

#[test]
fn unpublished_frame_abandonment_preserves_lease_ownership() {
    let mut s = Surface::new(2, 2, 8, 1).unwrap();
    let a = s.acquire(7).unwrap();
    s.write(a).unwrap().fill(123);
    s.abandon(a).unwrap();
    let b = s.acquire(7).unwrap();
    assert!(b.serial > a.serial);
    assert_eq!(s.abandon(a), Err(Error::Lease));
    s.write(b).unwrap().fill(0);
    s.commit(b, &[[0, 0, 2, 2]]).unwrap();
    assert_eq!(s.abandon(b), Err(Error::Lease));
    assert_eq!(s.acquire(7), Err(Error::Exhausted));
    s.release(b).unwrap();
}

#[test]
fn fully_offscreen_backdrop_is_an_empty_paint() {
    let mut pixels = vec![0; 64];
    let mut canvas = Canvas::new(&mut pixels, 4, 4, 16, 100).unwrap();
    let tokens = Tokens {
        colors: [[0; 4]; 7],
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
    };
    assert_eq!(
        canvas.backdrop(
            Rect([-1000.0, -1000.0, 100.0, 100.0]),
            Rect([0.0, 0.0, 4.0, 4.0]),
            &tokens,
            255
        ),
        Ok(0)
    );
    assert_eq!(pixels, vec![0; 64]);
}
