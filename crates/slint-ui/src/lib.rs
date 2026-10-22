//! Shared AOT, text-image and software platform path; no host services.
#![no_std]
extern crate alloc;
use alloc::{boxed::Box, format, rc::Rc, vec, vec::Vec};
use core::{
    cell::{Cell, RefCell},
    time::Duration,
};
use custom_ui::{Error, Result};
use custom_ui::{
    Rect,
    input::Controller,
    paint::{Canvas, Counters},
    scene::{Kind, Scene},
};
use slint::platform::{
    Platform, WindowAdapter, WindowEvent,
    software_renderer::{MinimalSoftwareWindow, RepaintBufferType},
};
use slint::{ComponentHandle, ModelRc, Rgb8Pixel, Rgba8Pixel, SharedPixelBuffer};

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
    ) -> core::result::Result<Rc<dyn WindowAdapter>, slint::PlatformError> {
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
/// One platform per process, on exactly one execution thread, never an IRQ.
/// Caller supplies monotonic time; this adapter provides no clock or event loop.
pub struct Runtime {
    next: Rc<RefCell<Option<Rc<MinimalSoftwareWindow>>>>,
    time: Rc<Cell<u64>>,
}
impl Runtime {
    pub fn new() -> Result<Self> {
        let next = Rc::new(RefCell::new(None));
        let time = Rc::new(Cell::new(0));
        slint::platform::set_platform(Box::new(Backend {
            next: next.clone(),
            time: time.clone(),
        }))
        .map_err(|_| Error::Resource)?;
        Ok(Self { next, time })
    }
    pub fn advance_to(&self, milliseconds: u64) -> Result<()> {
        if milliseconds < self.time.get() {
            return Err(Error::Bounds);
        }
        self.time.set(milliseconds);
        slint::platform::update_timers_and_animations();
        Ok(())
    }
    pub fn window(
        &self,
        width: u32,
        height: u32,
        scale: u16,
    ) -> Result<(Rc<MinimalSoftwareWindow>, CandidateWindow)> {
        if width == 0
            || height == 0
            || width > 4096
            || height > 4096
            || ![100, 150, 200].contains(&scale)
        {
            return Err(Error::Bounds);
        }
        let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
        *self.next.borrow_mut() = Some(window.clone());
        let component = CandidateWindow::new().map_err(|_| Error::Resource);
        self.next.borrow_mut().take();
        let component = component?;
        window.dispatch_event(WindowEvent::ScaleFactorChanged {
            scale_factor: scale as f32 / 100.0,
        });
        window.set_size(slint::PhysicalSize::new(width, height));
        component.show().map_err(|_| Error::Resource)?;
        self.advance_to(self.time.get().checked_add(1).ok_or(Error::Exhausted)?)?;
        Ok((window, component))
    }
}
fn color(c: [u8; 4]) -> slint::Color {
    slint::Color::from_argb_u8(c[3], c[0], c[1], c[2])
}
pub fn layer(rect: Rect, clip: Rect, bg: [u8; 4], radius: f32, alpha: f32) -> Layer {
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
pub fn raster(
    window: &MinimalSoftwareWindow,
    ui: &CandidateWindow,
    layers: Vec<Layer>,
    surface: [u8; 4],
    out: &mut [u8],
    width: u32,
    height: u32,
    stride: u32,
) -> Result<()> {
    if width == 0
        || width > 4096
        || height == 0
        || height > 4096
        || stride < width * 4
        || !stride.is_multiple_of(4)
        || out.len() != stride as usize * height as usize
    {
        return Err(Error::Bounds);
    }
    ui.set_surface(color(surface));
    ui.set_layers(ModelRc::new(slint::VecModel::from(layers)));
    let mut rgb = vec![Rgb8Pixel::default(); width as usize * height as usize];
    window.request_redraw();
    if !window.draw_if_needed(|r| {
        r.render(&mut rgb, width as usize);
    }) {
        return Err(Error::Resource);
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
pub fn text_surface(runtime: &Runtime, canvas: &mut Canvas<'_>) -> custom_ui::Result<()> {
    let expected = canvas.bytes.to_vec();
    let (w, ui) = runtime
        .window(canvas.width, canvas.height, canvas.scale)
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
    if canvas.bytes != expected {
        return Err(custom_ui::Error::Resource);
    }
    Ok(())
}
pub struct Builder<'a, 'b> {
    pub layers: Vec<Layer>,
    pub engine: &'a mut text_core::Engine<'b>,
    pub scale: u16,
    pub counters: Counters,
}
impl Builder<'_, '_> {
    pub fn text(
        &mut self,
        text: &str,
        rect: Rect,
        clip: Rect,
        size: u16,
        mono: bool,
        fg: [u8; 4],
        selection: Option<core::ops::Range<usize>>,
        accent: [u8; 4],
    ) -> custom_ui::Result<()> {
        rect.validate()?;
        clip.validate()?;
        if text.is_empty() || rect.0[2] <= 0.0 || rect.0[3] <= 0.0 {
            return Ok(());
        }
        let s = self.scale as f32 / 100.0;
        // Positive finite scene extents: same ceil policy without std float methods.
        let ceil = |v: f32| {
            let n = v as u32;
            n + u32::from((n as f32) < v)
        };
        let width = ceil(rect.0[2] * s);
        let height = ceil(rect.0[3] * s);
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
pub fn paint(
    runtime: &Runtime,
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
    let (w, ui) = runtime
        .window(canvas.width, canvas.height, canvas.scale)
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
