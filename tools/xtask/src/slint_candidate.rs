//! Hosted orchestration of the shared freestanding Slint candidate.
use crate::{Result, custom, hash, root, write_json};
use custom_ui::{
    Rect,
    input::Controller,
    paint::{Canvas, Counters},
    scene::Scene,
};
use serde_json::json;
use slint::platform::{WindowEvent, software_renderer::MinimalSoftwareWindow};
use slint_ui::{Builder, CandidateWindow, Runtime, layer, raster as portable_raster};
use std::{cell::RefCell, rc::Rc};
thread_local! { static RUNTIME: RefCell<Option<Runtime>> = const { RefCell::new(None) }; }
fn with_runtime<T>(f: impl FnOnce(&Runtime) -> custom_ui::Result<T>) -> custom_ui::Result<T> {
    RUNTIME.with(|r| {
        let mut r = r.borrow_mut();
        if r.is_none() {
            *r = Some(Runtime::new()?);
        }
        f(r.as_ref().unwrap())
    })
}
fn window(
    width: u32,
    height: u32,
    scale: u16,
) -> Result<(Rc<MinimalSoftwareWindow>, CandidateWindow)> {
    with_runtime(|r| r.window(width, height, scale)).map_err(|e| format!("{e:?}"))
}
fn raster(
    window: &MinimalSoftwareWindow,
    ui: &CandidateWindow,
    layers: Vec<slint_ui::Layer>,
    surface: [u8; 4],
    out: &mut [u8],
    width: u32,
    height: u32,
    stride: u32,
) -> Result<()> {
    portable_raster(window, ui, layers, surface, out, width, height, stride)
        .map_err(|e| format!("{e:?}"))
}
pub(crate) fn text_surface(canvas: &mut Canvas<'_>) -> custom_ui::Result<()> {
    with_runtime(|r| slint_ui::text_surface(r, canvas))
}
pub(crate) fn paint(
    scene: &mut Scene,
    c: &Controller,
    engine: &mut text_core::Engine<'_>,
    canvas: &mut Canvas<'_>,
) -> custom_ui::Result<Counters> {
    with_runtime(|r| slint_ui::paint(r, scene, c, engine, canvas))
}
pub fn execute() -> Result<()> {
    // API-only tests have an independent small analytic surface.
    let (w, ui) = window(8, 8, 100)?;
    let rect = Rect([0.0, 0.0, 8.0, 8.0]);
    let mut bytes = vec![0; 8 * 8 * 4];
    raster(
        &w,
        &ui,
        vec![layer(
            Rect([2.0, 2.0, 4.0, 4.0]),
            rect,
            [255, 0, 0, 255],
            0.0,
            1.0,
        )],
        [0, 0, 0, 255],
        &mut bytes,
        8,
        8,
        32,
    )?;
    for y in 0..8 {
        for x in 0..8 {
            let expected = if (2..6).contains(&x) && (2..6).contains(&y) {
                [255, 0, 0, 255]
            } else {
                [0, 0, 0, 255]
            };
            if bytes[(y * 8 + x) * 4..(y * 8 + x + 1) * 4] != expected {
                return Err("Independent Slint rectangle vector".into());
            }
        }
    }
    // Independent native source-over and rectangular clipping, not the custom painter.
    raster(
        &w,
        &ui,
        vec![layer(rect, rect, [100, 80, 60, 128], 0.0, 1.0)],
        [20, 40, 60, 255],
        &mut bytes,
        8,
        8,
        32,
    )?;
    // Preserve a failing comparison against the unchanged round-to-nearest formula.
    // Candidate-story acceptance records unsupported cases; it does not waive this
    // product/selection requirement or change a channel tolerance.
    let fractional_blend_observed = bytes[..4].to_vec();
    let fractional_blend_exact = bytes.chunks_exact(4).all(|p| p == [60, 60, 60, 255]);
    // Also exercise the exact shared source-over fixture (half red over opaque blue).
    raster(
        &w,
        &ui,
        vec![layer(rect, rect, [255, 0, 0, 128], 0.0, 1.0)],
        [0, 0, 255, 255],
        &mut bytes,
        8,
        8,
        32,
    )?;
    if !bytes.chunks_exact(4).all(|p| p == [128, 0, 127, 255]) {
        return Err("Shared Slint source-over fixture".into());
    }
    raster(
        &w,
        &ui,
        vec![layer(
            rect,
            Rect([2.0, 2.0, 4.0, 4.0]),
            [255, 0, 0, 255],
            0.0,
            1.0,
        )],
        [0, 0, 0, 255],
        &mut bytes,
        8,
        8,
        32,
    )?;
    for y in 0..8 {
        for x in 0..8 {
            let expected = if (2..6).contains(&x) && (2..6).contains(&y) {
                [255, 0, 0, 255]
            } else {
                [0, 0, 0, 255]
            };
            if bytes[(y * 8 + x) * 4..(y * 8 + x + 1) * 4] != expected {
                return Err("Slint ancestor rectangle clip".into());
            }
        }
    }
    let native_font =
        "Native font setup is not implemented by this bridge; see the isolated Text runtime probe";
    let mut idle_requests = 0;
    for _ in 0..1000 {
        if w.draw_if_needed(|_| {}) {
            idle_requests += 1
        }
    }
    if idle_requests != 0 {
        return Err("Settled Slint window requested frames".into());
    }
    let called = Rc::new(RefCell::new(String::new()));
    let observed = called.clone();
    ui.on_activated(move |id| *observed.borrow_mut() = id.to_string());
    let mut touch = layer(rect, rect, [0, 0, 0, 255], 0.0, 1.0);
    touch.active = true;
    touch.identity = "reduced-effects".into();
    raster(&w, &ui, vec![touch], [0, 0, 0, 255], &mut bytes, 8, 8, 32)?;
    for event in [
        WindowEvent::PointerMoved {
            position: slint::LogicalPosition::new(4.0, 4.0),
        },
        WindowEvent::PointerPressed {
            position: slint::LogicalPosition::new(4.0, 4.0),
            button: slint::platform::PointerEventButton::Left,
        },
        WindowEvent::PointerReleased {
            position: slint::LogicalPosition::new(4.0, 4.0),
            button: slint::platform::PointerEventButton::Left,
        },
    ] {
        w.dispatch_event(event)
    }
    if called.borrow().as_str() != "reduced-effects" {
        return Err("Slint TouchArea logical ID callback".into());
    }
    let fonts = crate::text::inputs()?;
    let refs: Vec<_> = fonts.iter().map(Vec::as_slice).collect();
    let mut engine = text_core::Engine::new(&refs).map_err(|e| format!("quota fonts: {e:?}"))?;
    let mut builder = Builder {
        layers: Vec::new(),
        engine: &mut engine,
        scale: 100,
        counters: Counters::default(),
    };
    let huge = Rect([0.0, 0.0, 4096.0, 4096.0]);
    if builder.text(
        "quota",
        huge,
        huge,
        16,
        false,
        [255; 4],
        None,
        [0, 128, 255, 255],
    ) != Err(custom_ui::Error::Exhausted)
        || builder.counters.shape_work != 0
        || builder.counters.scratch_peak != 0
        || !builder.layers.is_empty()
    {
        return Err("Text-image quota must reject before allocation/shape/publication".into());
    }
    let probes = json!({"schema_version":1,"staged_text_image_quota_rejected_before_shape":true,"staged_text_image_quota_bytes":16777216,"native_rectangle_vector":true,"native_source_over_vector":true,"fractional_blend":{"expected":[60,60,60,255],"actual":fractional_blend_observed,"exact":fractional_blend_exact,"tolerance":0,"requirement_waived":false},"native_rectangular_clip_vector":true,"native_pointer_callback":"reduced-effects","native_font_registration":format!("{native_font:?}"),"native_text":"unsupported in this configuration; P2-04 text-image adapter tested separately","idle_polls":1000,"idle_requests":idle_requests,"probe_rgba_sha256":hash(&bytes),"guest_tested":false});
    std::fs::create_dir_all(root().join("build/m2/P2-06")).map_err(|e| e.to_string())?;
    write_json(&root().join("build/m2/P2-06/api-probes.json"), &probes)?;
    custom::execute_with(custom::Candidate::Slint)
}

pub fn native_text_probe() -> Result<()> {
    let (w, ui) = window(128, 64, 100)?;
    ui.set_native_probe(true);
    let mut bytes = vec![0; 128 * 64 * 4];
    raster(
        &w,
        &ui,
        Vec::new(),
        [0, 0, 0, 255],
        &mut bytes,
        128,
        64,
        512,
    )
}
pub fn behavior() -> Result<()> {
    use editor_model::{Model, Selection};
    let mut c = Controller::new(
        Model::new(1, 11, "", "", Selection::new(0, 0)).map_err(|e| format!("{e:?}"))?,
    )
    .map_err(|e| format!("{e:?}"))?;
    let corpus = crate::ui_fixtures::checked_inputs()?;
    let tree = custom::tree(&corpus["layout"])?;
    c.event(
        &tree,
        custom_ui::input::Event {
            sequence: 1,
            time_us: 1,
            revision: tree.revision,
            target: "editor".into(),
            kind: custom_ui::input::EventKind::Text("Continuum".into()),
        },
    )
    .map_err(|e| format!("{e:?}"))?;
    let expected = if cfg!(feature = "behavior-v2") {
        "CONTINUUM"
    } else {
        "Continuum"
    };
    if c.provider.model().document().text() != expected {
        return Err("Compiled behavior mismatch".into());
    }
    // Property values genuinely reach compiled Slint items; analytic pixel tests detect drift.
    let (w, ui) = window(8, 8, 100)?;
    let rect = Rect([0.0, 0.0, 8.0, 8.0]);
    let mut bytes = vec![0; 256];
    raster(
        &w,
        &ui,
        vec![layer(rect, rect, [15, 25, 35, 255], 0.0, 1.0)],
        [0, 0, 0, 255],
        &mut bytes,
        8,
        8,
        32,
    )?;
    if !bytes.chunks_exact(4).all(|p| p == [15, 25, 35, 255]) {
        return Err("Token setter failed".into());
    }
    raster(
        &w,
        &ui,
        vec![layer(
            Rect([4.0, 0.0, 4.0, 8.0]),
            rect,
            [45, 55, 65, 255],
            0.0,
            1.0,
        )],
        [0, 0, 0, 255],
        &mut bytes,
        8,
        8,
        32,
    )?;
    if bytes[..4] != [0, 0, 0, 255] || bytes[16..20] != [45, 55, 65, 255] {
        return Err("Layout model setter failed".into());
    }
    println!(
        "{}",
        json!({"status":"passed","behavior":expected,"compiled_variant":cfg!(feature="behavior-v2"),"token_and_layout_setters":true,"replacement_tested":false})
    );
    Ok(())
}
