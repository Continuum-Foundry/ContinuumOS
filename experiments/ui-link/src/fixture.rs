//! Small standalone scene uses the same candidate implementations as hosted runs.
use alloc::{vec, vec::Vec};
use custom_ui::{
    Rect, Result,
    scene::{Kind, Node, Scene, Tokens, Tree},
};
pub fn scene(seed: u64) -> Result<Scene> {
    let nodes: Vec<Node> = [
        ("desktop", Kind::Overlay, [0., 0., 640., 360.]),
        ("editor", Kind::Editor, [8., 8., 420., 328.]),
        ("files", Kind::List, [440., 8., 180., 280.]),
    ]
    .into_iter()
    .map(|(id, kind, rect)| Node {
        id: id.into(),
        kind,
        children: if id == "desktop" {
            vec!["editor".into(), "files".into()]
        } else {
            vec![]
        },
        rect: Rect(rect),
        clip: true,
        opacity: 1.,
        text: "".into(),
        action: if id == "editor" {
            "edit.insert@1".into()
        } else {
            "".into()
        },
        resource: "".into(),
        semantic_id: alloc::format!("sem.{id}"),
        role: if id == "editor" {
            "textbox".into()
        } else {
            "group".into()
        },
        name: id.into(),
        focusable: id == "editor",
    })
    .collect();
    Scene::new(
        Tree {
            nodes,
            root: "desktop".into(),
            revision: 1,
            generation: 1,
        },
        Tokens {
            colors: [
                [seed as u8, 20, 30, 255],
                [30, 40, 50, 255],
                [240, 240, 240, 255],
                [150, 150, 150, 255],
                [50, 128, 250, 255],
                [80, 160, 255, 255],
                [255, 40, 40, 255],
            ],
            spacing: 8.,
            radius: 8.,
            control_radius: 4.,
            body_size: 14,
            editor_size: 16,
            title_size: 18,
            tint: [20, 30, 40, 200],
            fallback: [20, 30, 40, 255],
            blur_radius: 8,
            passes: 3,
            reduced_effects: true,
            reduced_motion: true,
        },
    )
}
pub fn fonts() -> [&'static [u8]; 6] {
    [
        include_bytes!(concat!(env!("CONTINUUM_FONT_DIR"), "/NotoSans-Regular.ttf")),
        include_bytes!(concat!(
            env!("CONTINUUM_FONT_DIR"),
            "/NotoSansArabic-Regular.ttf"
        )),
        include_bytes!(concat!(
            env!("CONTINUUM_FONT_DIR"),
            "/NotoSansHebrew-Regular.ttf"
        )),
        include_bytes!(concat!(
            env!("CONTINUUM_FONT_DIR"),
            "/NotoSansDevanagari-Regular.ttf"
        )),
        include_bytes!(concat!(
            env!("CONTINUUM_FONT_DIR"),
            "/NotoSansCJKsc-Regular.otf"
        )),
        include_bytes!(concat!(
            env!("CONTINUUM_FONT_DIR"),
            "/NotoSansMono-Regular.ttf"
        )),
    ]
}
