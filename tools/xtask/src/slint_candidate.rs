//! P2-06: real Slint AOT primitives/software renderer, with an explicit P2-04
//! text-image adapter. No system font discovery or native-text pass is implied.
use crate::{Result, custom, hash, root, write_json};
use custom_ui::{
    Rect,
    input::Controller,
    paint::{Canvas, Counters},
    scene::{Kind, Scene},
};
use serde_json::json;
use slint::platform::{
    Platform, WindowAdapter, WindowEvent,
    software_renderer::{MinimalSoftwareWindow, RepaintBufferType},
};
use slint::{ComponentHandle, ModelRc, Rgb8Pixel, Rgba8Pixel, SharedPixelBuffer};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};

slint::slint! {
    export struct Layer {
        x: float, y: float, w: float, h: float,
        cx: float, cy: float, cw: float, ch: float,
        radius: float, background: color, alpha: float,
        image: image, identity: string, active: bool,
    }
    export component CandidateWindow inherits Window {
        in property <[Layer]> layers;
        in property <color> surface;
        in property <bool> native-probe: false;
        if root.native-probe: Text { text: "ffi العربية हिन्दी"; }
        callback activated(string);
        background: surface;
        for layer in layers: Rectangle {
            x: layer.cx * 1px; y: layer.cy * 1px;
            width: layer.cw * 1px; height: layer.ch * 1px;
            clip: true;
            Rectangle {
                x: (layer.x - layer.cx) * 1px; y: (layer.y - layer.cy) * 1px;
                width: layer.w * 1px; height: layer.h * 1px;
                border-radius: layer.radius * 1px;
                background: layer.background;
                opacity: layer.alpha;
                Image { source: layer.image; width: parent.width; height: parent.height; image-fit: fill; }
                TouchArea { enabled: layer.active; clicked => { root.activated(layer.identity); } }
            }
        }
    }
}
struct Backend {
    next: Rc<RefCell<Option<Rc<MinimalSoftwareWindow>>>>,
    time: Rc<Cell<u64>>,
}
impl Platform for Backend {
    fn create_window_adapter(
        &self,
    ) -> std::result::Result<Rc<dyn WindowAdapter>, slint::PlatformError> {
        self.next
            .borrow()
            .as_ref()
            .cloned()
            .map(|w| w as Rc<dyn WindowAdapter>)
            .ok_or(slint::PlatformError::NoPlatform)
    }
    fn duration_since_start(&self) -> Duration {
        Duration::from_millis(self.time.get())
    }
    fn cursor_flash_cycle(&self) -> Duration {
        Duration::ZERO
    }
}
struct Runtime {
    next: Rc<RefCell<Option<Rc<MinimalSoftwareWindow>>>>,
    time: Rc<Cell<u64>>,
}
thread_local! { static RUNTIME: RefCell<Option<Runtime>> = const { RefCell::new(None) }; }
fn window(
    width: u32,
    height: u32,
    scale: u16,
) -> Result<(Rc<MinimalSoftwareWindow>, CandidateWindow)> {
    RUNTIME.with(|r| {
        let mut r = r.borrow_mut();
        if r.is_none() {
            let next = Rc::new(RefCell::new(None));
            let time = Rc::new(Cell::new(0));
            slint::platform::set_platform(Box::new(Backend {
                next: next.clone(),
                time: time.clone(),
            }))
            .map_err(|e| e.to_string())?;
            *r = Some(Runtime { next, time });
        }
        let r = r.as_ref().unwrap();
        let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
        *r.next.borrow_mut() = Some(window.clone());
        let component = CandidateWindow::new().map_err(|e| e.to_string())?;
        // Clear platform's temporary strong reference after construction.
        r.next.borrow_mut().take();
        window.dispatch_event(WindowEvent::ScaleFactorChanged {
            scale_factor: scale as f32 / 100.0,
        });
        window.set_size(slint::PhysicalSize::new(width, height));
        component.show().map_err(|e| e.to_string())?;
        r.time.set(r.time.get() + 1);
        slint::platform::update_timers_and_animations();
        Ok((window, component))
    })
}
fn color(c: [u8; 4]) -> slint::Color {
    slint::Color::from_argb_u8(c[3], c[0], c[1], c[2])
}
fn layer(rect: Rect, clip: Rect, bg: [u8; 4], radius: f32, alpha: f32) -> Layer {
    let [x, y, w, h] = rect.0;
    let [cx, cy, cw, ch] = rect.intersect(clip).0;
    Layer {
        x,
        y,
        w,
        h,
        cx,
        cy,
        cw,
        ch,
        background: color(bg),
        radius,
        alpha,
        ..Default::default()
    }
}
fn image(bytes: &[u8], width: u32, height: u32) -> slint::Image {
    // Canvas text images are premultiplied. Tell Slint explicitly; no hidden unpremultiply.
    slint::Image::from_rgba8_premultiplied(SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
        bytes, width, height,
    ))
}
fn raster(
    window: &MinimalSoftwareWindow,
    ui: &CandidateWindow,
    layers: Vec<Layer>,
    surface: [u8; 4],
    out: &mut [u8],
    width: u32,
    height: u32,
    stride: u32,
) -> Result<()> {
    ui.set_surface(color(surface));
    ui.set_layers(ModelRc::new(slint::VecModel::from(layers)));
    let mut rgb = vec![Rgb8Pixel::default(); width as usize * height as usize];
    window.request_redraw();
    if !window.draw_if_needed(|r| {
        r.render(&mut rgb, width as usize);
    }) {
        return Err("Slint did not render requested frame".into());
    }
    out.fill(0);
    for (row, target) in rgb
        .chunks_exact(width as usize)
        .zip(out.chunks_exact_mut(stride as usize))
    {
        for (p, dst) in row.iter().zip(target.chunks_exact_mut(4)) {
            dst.copy_from_slice(&[p.r, p.g, p.b, 255]);
        }
    }
    Ok(())
}
/// Route a text-only observation through Slint's actual Image item and rasterizer.
pub(crate) fn text_surface(canvas: &mut Canvas<'_>) -> custom_ui::Result<()> {
    let expected = hash(canvas.bytes);
    let (w, ui) = window(canvas.width, canvas.height, canvas.scale)
        .map_err(|_| custom_ui::Error::Resource)?;
    let s = canvas.scale as f32 / 100.0;
    let rect = Rect([0.0, 0.0, canvas.width as f32 / s, canvas.height as f32 / s]);
    let mut l = layer(rect, rect, [0; 4], 0.0, 1.0);
    l.image = image(canvas.bytes, canvas.width, canvas.height);
    raster(
        &w,
        &ui,
        vec![l],
        [0, 0, 0, 255],
        canvas.bytes,
        canvas.width,
        canvas.height,
        canvas.stride,
    )
    .map_err(|_| custom_ui::Error::Resource)?;
    if hash(canvas.bytes) != expected {
        return Err(custom_ui::Error::Resource);
    }
    Ok(())
}
struct Builder<'a, 'b> {
    layers: Vec<Layer>,
    engine: &'a mut text_core::Engine<'b>,
    scale: u16,
    counters: Counters,
}
impl Builder<'_, '_> {
    fn text(
        &mut self,
        text: &str,
        rect: Rect,
        clip: Rect,
        size: u16,
        mono: bool,
        fg: [u8; 4],
        selection: Option<std::ops::Range<usize>>,
        accent: [u8; 4],
    ) -> custom_ui::Result<()> {
        if text.is_empty() || rect.0[2] <= 0.0 || rect.0[3] <= 0.0 {
            return Ok(());
        }
        let s = self.scale as f32 / 100.0;
        let width = (rect.0[2] * s).ceil() as u32;
        let height = (rect.0[3] * s).ceil() as u32;
        if width > 4096 || height > 4096 {
            return Err(custom_ui::Error::Bounds);
        }
        let byte_count = width as usize * height as usize * 4;
        let staged = self
            .counters
            .scratch_peak
            .checked_add(byte_count)
            .ok_or(custom_ui::Error::Exhausted)?;
        // Bound the bridge-owned retained text-image payload before allocating it.
        // Slint's internal storage, model/font allocations and RGB output are separate.
        if staged > 16 * 1024 * 1024 {
            return Err(custom_ui::Error::Exhausted);
        }
        let mut bytes = vec![0; byte_count];
        let mut cv = Canvas::new(&mut bytes, width, height, width * 4, self.scale)?;
        let local = Rect([0.0, 0.0, rect.0[2], rect.0[3]]);
        let (missing, work) = cv.text(
            self.engine,
            text,
            local,
            local,
            size,
            mono,
            fg,
            selection,
            accent,
        )?;
        self.counters.shape_work += work as u64;
        self.counters.missing += missing as u64;
        self.counters.scratch_peak = staged;
        let mut l = layer(rect, clip, [0; 4], 0.0, 1.0);
        l.image = image(&bytes, width, height);
        self.layers.push(l);
        Ok(())
    }
    fn walk(
        &mut self,
        scene: &Scene,
        c: &Controller,
        id: &str,
        clip: Rect,
    ) -> custom_ui::Result<()> {
        let n = scene.tree.node(id)?;
        let t = &scene.tokens;
        let m = c.provider.model();
        self.counters.nodes += 1;
        let clip = if n.clip { clip.intersect(n.rect) } else { clip };
        if id != "desktop" {
            let bg = if id == "launcher" {
                t.fallback
            } else if matches!(n.kind, Kind::Field | Kind::Editor) {
                t.colors[0]
            } else {
                t.colors[1]
            };
            // Launcher always uses the required functional opaque fallback. Native blur and
            // isolated group opacity are unsupported findings, never labeled equivalent.
            let mut l = layer(
                n.rect,
                clip,
                bg,
                if n.focusable {
                    t.control_radius
                } else {
                    t.radius
                },
                1.0,
            );
            l.identity = n.id.clone().into();
            l.active = n.focusable;
            self.layers.push(l);
        }
        if n.focusable && m.session().focus == id {
            self.layers
                .push(layer(n.rect, clip, t.colors[5], t.control_radius, 1.0));
            let [x, y, w, h] = n.rect.0;
            self.layers.push(layer(
                Rect([x + 1.0, y + 1.0, (w - 2.0).max(0.0), (h - 2.0).max(0.0)]),
                clip,
                t.colors[0],
                (t.control_radius - 1.0).max(0.0),
                1.0,
            ));
        }
        let [x, y, w, h] = n.rect.0;
        let pad = t.spacing;
        let content = Rect([
            x + pad,
            y + 2.0,
            (w - 2.0 * pad).max(0.0),
            (h - 4.0).max(0.0),
        ]);
        if n.kind == Kind::List {
            let rows = custom_ui::scene::realize(m.session().file_scroll, 10000, h, 24.0)?;
            self.counters.realized += rows.len() as u64;
            for row in rows {
                let rect = Rect([
                    x + pad,
                    y + (row - m.session().file_scroll) as f32 * 24.0,
                    w - 2.0 * pad,
                    24.0,
                ]);
                if row == m.session().selected_file {
                    let mut col = t.colors[4];
                    col[3] = 80;
                    self.layers.push(layer(rect, clip, col, 0.0, 1.0));
                }
                let seed = custom_ui::paint::lcg_at(20261020, row + 1);
                self.text(
                    &format!("File {row:05} — {seed:08x}.txt"),
                    rect,
                    clip,
                    t.body_size,
                    false,
                    t.colors[2],
                    None,
                    t.colors[4],
                )?;
            }
        } else {
            let text = match n.kind {
                Kind::Editor => m.document().text(),
                Kind::Field if !m.document().query().is_empty() => m.document().query(),
                _ => &n.text,
            };
            let selection = if n.kind == Kind::Editor {
                let s = m.document().selection();
                Some(s.anchor.min(s.caret) as usize..s.anchor.max(s.caret) as usize)
            } else {
                None
            };
            let size = if n.kind == Kind::Editor {
                t.editor_size
            } else if !n.focusable && !n.children.is_empty() {
                t.title_size
            } else {
                t.body_size
            };
            self.text(
                text,
                content,
                clip,
                size,
                n.kind == Kind::Editor,
                t.colors[2],
                selection,
                t.colors[4],
            )?;
            if n.kind == Kind::Editor {
                if let Some(p) = &c.composition {
                    self.text(
                        p.preedit(),
                        Rect([x + pad, y + 32.0, w - 2.0 * pad, 28.0]),
                        clip,
                        t.editor_size,
                        true,
                        t.colors[4],
                        None,
                        t.colors[4],
                    )?;
                }
            }
            if n.kind == Kind::Toggle {
                let checked = if id == "reduced-effects" {
                    m.preferences().reduced_effects
                } else {
                    m.preferences().reduced_motion
                };
                self.layers.push(layer(
                    Rect([x + w - 18.0, y + 8.0, 10.0, 10.0]),
                    clip,
                    if checked { t.colors[4] } else { t.colors[3] },
                    2.0,
                    1.0,
                ));
            }
            if n.resource == "icon.missing" {
                let mut l = layer(Rect([x + w - 16.0, y, 16.0, 16.0]), clip, [0; 4], 0.0, 1.0);
                l.image = image(
                    &[
                        255, 0, 255, 255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 0, 255, 255,
                    ],
                    2,
                    2,
                );
                self.layers.push(l);
            }
        }
        for child in &n.children {
            self.walk(scene, c, child, clip)?;
        }
        Ok(())
    }
}
pub(crate) fn paint(
    scene: &mut Scene,
    c: &Controller,
    engine: &mut text_core::Engine<'_>,
    canvas: &mut Canvas<'_>,
) -> custom_ui::Result<Counters> {
    scene.layout();
    let s = canvas.scale as f32 / 100.0;
    let clip = Rect([0.0, 0.0, canvas.width as f32 / s, canvas.height as f32 / s]);
    let mut builder = Builder {
        layers: Vec::new(),
        engine,
        scale: canvas.scale,
        counters: Counters::default(),
    };
    builder.walk(scene, c, &scene.tree.root, clip)?;
    if builder.layers.len() > 1024 {
        return Err(custom_ui::Error::Exhausted);
    }
    let (w, ui) = window(canvas.width, canvas.height, canvas.scale)
        .map_err(|_| custom_ui::Error::Resource)?;
    raster(
        &w,
        &ui,
        builder.layers,
        scene.tokens.colors[0],
        canvas.bytes,
        canvas.width,
        canvas.height,
        canvas.stride,
    )
    .map_err(|_| custom_ui::Error::Resource)?;
    let (entries, bytes) = builder.engine.cache_usage();
    builder.counters.glyph_cache_entries = entries;
    builder.counters.glyph_cache_bytes = bytes;
    builder.counters.touched = canvas.width as u64 * canvas.height as u64;
    // Recreated AOT component + full redraw is intentional first-candidate instrumentation.
    builder.counters.scratch_peak += canvas.width as usize * canvas.height as usize * 3;
    scene.paint_passes += 1;
    scene.dirty.paint = false;
    Ok(builder.counters)
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
