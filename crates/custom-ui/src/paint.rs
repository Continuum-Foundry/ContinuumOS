use crate::{
    scene::{Kind, Scene, Tokens, realize},
    *,
};
use alloc::{format, vec};
use editor_model::Model;
use text_core::{Config, Engine};

pub fn premultiply(c: [u8; 4]) -> [u8; 4] {
    let mut out = c;
    for i in 0..3 {
        out[i] = ((c[i] as u32 * c[3] as u32 + 127) / 255) as u8
    }
    out
}
pub fn over(src: [u8; 4], dst: [u8; 4]) -> [u8; 4] {
    let mut out = [0; 4];
    for i in 0..4 {
        out[i] =
            (src[i] as u32 + (dst[i] as u32 * (255 - src[3] as u32) + 127) / 255).min(255) as u8
    }
    out
}
fn opacity(c: [u8; 4], alpha: u8) -> [u8; 4] {
    c.map(|v| ((v as u32 * alpha as u32 + 127) / 255) as u8)
}
pub struct Canvas<'a> {
    pub bytes: &'a mut [u8],
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub scale: u16,
    pub touched: u64,
    /// Selected path: 8x8 coverage for rounded shapes, unchanged legacy oracle mode.
    pub smooth_edges: bool,
    pub caret: Option<text_core::VisualCaret>,
    pub hide_caret: bool,
    /// Optional host monotonic instrumentation; absent in portable execution.
    pub clock: Option<fn() -> u64>,
    pub cache_layouts: bool,
    scissor: Option<[u32; 4]>,
    rounded_clips: Vec<(Rect, f32)>,
    deferred_clips: Vec<(Rect, f32)>,
}
impl<'a> Canvas<'a> {
    pub fn new(
        bytes: &'a mut [u8],
        width: u32,
        height: u32,
        stride: u32,
        scale: u16,
    ) -> Result<Self> {
        if ![100, 150, 200].contains(&scale)
            || width == 0
            || width > 4096
            || height == 0
            || height > 4096
            || stride < width * 4
            || !stride.is_multiple_of(4)
            || bytes.len() != stride as usize * height as usize
        {
            return Err(Error::Bounds);
        }
        Ok(Self {
            bytes,
            width,
            height,
            stride,
            scale,
            touched: 0,
            smooth_edges: false,
            caret: None,
            hide_caret: false,
            clock: None,
            cache_layouts: false,
            scissor: None,
            rounded_clips: Vec::new(),
            deferred_clips: Vec::new(),
        })
    }
    fn pixel(&mut self, x: u32, y: u32, c: [u8; 4]) {
        self.covered_pixel(x, y, c, u64::MAX);
    }
    fn mask(&self, rect: Rect, radius: f32, x: u32, y: u32) -> u64 {
        let scale = self.scale as f32 / 100.0;
        let [a, b, w, h] = rect.0;
        let r = radius.min(w / 2.0).min(h / 2.0);
        let left = x as f32 / scale;
        let top = y as f32 / scale;
        let right = (x + 1) as f32 / scale;
        let bottom = (y + 1) as f32 / scale;
        if right <= a || bottom <= b || left >= a + w || top >= b + h {
            return 0;
        }
        if left >= a
            && right <= a + w
            && top >= b
            && bottom <= b + h
            && (r == 0.0
                || (left >= a + r && right <= a + w - r)
                || (top >= b + r && bottom <= b + h - r))
        {
            return u64::MAX;
        }
        let mut mask = 0;
        for sy in 0..8 {
            for sx in 0..8 {
                let px = (x as f32 + (sx as f32 + 0.5) / 8.0) / scale;
                let py = (y as f32 + (sy as f32 + 0.5) / 8.0) / scale;
                let cx = px.clamp(a + r, a + w - r);
                let cy = py.clamp(b + r, b + h - r);
                if rect.contains(px, py) && (px - cx) * (px - cx) + (py - cy) * (py - cy) <= r * r {
                    mask |= 1 << (sy * 8 + sx);
                }
            }
        }
        mask
    }
    fn covered_pixel(&mut self, x: u32, y: u32, mut c: [u8; 4], mut mask: u64) {
        if self
            .scissor
            .is_some_and(|[l, t, w, h]| x < l || y < t || x >= l + w || y >= t + h)
        {
            return;
        }
        if self.smooth_edges {
            for (rect, radius) in &self.rounded_clips {
                mask &= self.mask(*rect, *radius, x, y);
            }
            if mask == 0 {
                return;
            }
            let coverage = mask.count_ones();
            c = c.map(|v| ((v as u32 * coverage + 32) / 64) as u8);
        } else {
            let scale = self.scale as f32 / 100.0;
            let px = (x as f32 + 0.5) / scale;
            let py = (y as f32 + 0.5) / scale;
            if self.rounded_clips.iter().any(|(rect, r)| {
                let [x, y, w, h] = rect.0;
                let radius = r.min(w / 2.0).min(h / 2.0);
                let cx = px.clamp(x + radius, x + w - radius);
                let cy = py.clamp(y + radius, y + h - radius);
                !rect.contains(px, py)
                    || (px - cx) * (px - cx) + (py - cy) * (py - cy) > radius * radius
            }) {
                return;
            }
        }
        let at = y as usize * self.stride as usize + x as usize * 4;
        let dst = self.bytes[at..at + 4].try_into().unwrap();
        self.bytes[at..at + 4].copy_from_slice(&over(c, dst));
        self.touched += 1;
    }
    pub fn fill(&mut self, rect: Rect, clip: Rect, radius: f32, color: [u8; 4], alpha: u8) {
        let [left, top, right, bottom] =
            rect.intersect(clip)
                .pixels(self.scale, self.width, self.height);
        let s = self.scale as f32 / 100.0;
        let [x, y, w, h] = rect.0;
        let r = radius.min(w / 2.0).min(h / 2.0);
        let c = opacity(premultiply(color), alpha);
        for py in top..bottom {
            for px in left..right {
                let a = (px as f32 + 0.5) / s;
                let b = (py as f32 + 0.5) / s;
                let cx = a.clamp(x + r, x + w - r);
                let cy = b.clamp(y + r, y + h - r);
                if self.smooth_edges && r > 0.0 {
                    let mask = self.mask(rect, r, px, py);
                    self.covered_pixel(px, py, c, mask);
                    continue;
                }
                if r == 0.0 || (a - cx) * (a - cx) + (b - cy) * (b - cy) <= r * r {
                    self.pixel(px, py, c)
                }
            }
        }
    }
    pub fn image(
        &mut self,
        rect: Rect,
        clip: Rect,
        width: u32,
        height: u32,
        pixels: &[[u8; 4]],
    ) -> Result<()> {
        rect.validate()?;
        clip.validate()?;
        if width == 0
            || height == 0
            || width > 256
            || height > 256
            || pixels.len() != width as usize * height as usize
            || pixels.iter().any(|c| c[..3].iter().any(|v| *v > c[3]))
        {
            return Err(Error::Resource);
        }
        let [l, t, r, b] = rect
            .intersect(clip)
            .pixels(self.scale, self.width, self.height);
        let s = self.scale as f32 / 100.0;
        for y in t..b {
            for x in l..r {
                let ix = (((x as f32 + 0.5) / s - rect.0[0]) / rect.0[2] * width as f32) as u32;
                let iy = (((y as f32 + 0.5) / s - rect.0[1]) / rect.0[3] * height as f32) as u32;
                self.pixel(
                    x,
                    y,
                    pixels[(iy.min(height - 1) * width + ix.min(width - 1)) as usize],
                );
            }
        }
        Ok(())
    }
    pub fn text(
        &mut self,
        engine: &mut Engine<'_>,
        text: &str,
        rect: Rect,
        clip: Rect,
        size: u16,
        mono: bool,
        color: [u8; 4],
        selection: Option<core::ops::Range<usize>>,
        accent: [u8; 4],
    ) -> Result<(usize, usize)> {
        let width = (rect.0[2] * self.scale as f32 / 100.0 + 0.5) as u16;
        if width == 0 {
            return Ok((0, 0));
        }
        let cfg = Config {
            mono,
            size,
            scale: self.scale,
            width: width.min(4096),
            ligatures: true,
        };
        let layout = if self.cache_layouts {
            engine.layout_cached(text, cfg)
        } else {
            engine.layout(text, cfg)
        }
        .map_err(|_| Error::Text)?;
        let scale = self.scale as f32 / 100.0;
        let clip = rect.intersect(clip);
        let [l, t, r, b] = clip.pixels(self.scale, self.width, self.height);
        let line_height = cfg.pixels() * 1.5;
        for (row, line) in layout.lines.iter().enumerate() {
            let baseline = rect.0[1] * scale + cfg.pixels() + row as f32 * line_height;
            if baseline - cfg.pixels() > b as f32 {
                break;
            }
            if let Some(ref range) = selection {
                let a = range.start.max(line.bytes.start);
                let z = range.end.min(line.bytes.end);
                if a < z {
                    for interval in line.selection(a..z).map_err(|_| Error::Text)? {
                        self.fill(
                            Rect([
                                rect.0[0] + interval[0] / scale,
                                rect.0[1] + row as f32 * line_height / scale,
                                (interval[1] - interval[0]) / scale,
                                line_height / scale,
                            ]),
                            clip,
                            0.0,
                            accent,
                            100,
                        );
                    }
                }
                if range.start == range.end
                    && !self.hide_caret
                    && self
                        .caret
                        .is_none_or(|c| c.line == row || c.byte != range.start)
                {
                    if let Some(c) = line.carets.iter().find(|c| c.byte == range.start) {
                        let x = self
                            .caret
                            .filter(|c| c.byte == range.start && c.line == row)
                            .map(|c| c.x)
                            .unwrap_or(c.x);
                        self.fill(
                            Rect([
                                rect.0[0] + x / scale,
                                rect.0[1] + row as f32 * line_height / scale,
                                1.0 / scale,
                                line_height / scale,
                            ]),
                            clip,
                            0.0,
                            color,
                            255,
                        );
                    }
                }
            }
            for g in &line.glyphs {
                let bitmap = engine
                    .rasterize(g.font, g.id, cfg)
                    .map_err(|_| Error::Text)?;
                let round = |v: f32| {
                    if v < 0.0 {
                        (v - 0.5) as i32
                    } else {
                        (v + 0.5) as i32
                    }
                };
                let left = round(rect.0[0] * scale + g.x + g.offset[0]) + bitmap.xmin;
                let top = (baseline + 0.5) as i32
                    - bitmap.height as i32
                    - bitmap.ymin
                    - round(g.offset[1]);
                for y in 0..bitmap.height {
                    let dy = top + y as i32;
                    if dy < t as i32 || dy >= b as i32 {
                        continue;
                    }
                    for x in 0..bitmap.width {
                        let dx = left + x as i32;
                        if dx >= l as i32 && dx < r as i32 {
                            let coverage = bitmap.pixels[y * bitmap.width + x];
                            self.pixel(dx as u32, dy as u32, opacity(premultiply(color), coverage));
                        }
                    }
                }
            }
        }
        Ok((layout.missing.len(), layout.shape_work))
    }
    /// Full backdrop support retained around the panel. Sliding integer box sums,
    /// three horizontal/vertical passes with clamp-to-surface-edge sampling.
    pub fn backdrop(&mut self, rect: Rect, clip: Rect, t: &Tokens, alpha: u8) -> Result<usize> {
        let radius = (t.blur_radius * self.scale as u32).div_ceil(100);
        let support = radius * t.passes;
        let logical_support = support as f32 * 100.0 / self.scale as f32;
        let [l, top, r, b] =
            rect.expand(logical_support)
                .pixels(self.scale, self.width, self.height);
        let width = (r - l) as usize;
        let height = (b - top) as usize;
        if width == 0 || height == 0 {
            return Ok(0);
        }
        let mut pixels = vec![[0; 4]; width * height];
        for y in 0..height {
            for x in 0..width {
                let at = (y + top as usize) * self.stride as usize + (x + l as usize) * 4;
                pixels[y * width + x].copy_from_slice(&self.bytes[at..at + 4]);
            }
        }
        blur(
            &mut pixels,
            width,
            height,
            radius as usize,
            t.passes as usize,
        )?;
        let [a, c, d, e] = rect
            .intersect(clip)
            .pixels(self.scale, self.width, self.height);
        let s = self.scale as f32 / 100.0;
        let [x, y, w, h] = rect.0;
        let rad = t.radius.min(w / 2.0).min(h / 2.0);
        for py in c..e {
            for px in a..d {
                let q = (px as f32 + 0.5) / s;
                let z = (py as f32 + 0.5) / s;
                let cx = q.clamp(x + rad, x + w - rad);
                let cy = z.clamp(y + rad, y + h - rad);
                if self.smooth_edges {
                    let filtered = pixels[(py - top) as usize * width + (px - l) as usize];
                    let mask = self.mask(rect, rad, px, py);
                    self.covered_pixel(
                        px,
                        py,
                        opacity(over(premultiply(t.tint), filtered), alpha),
                        mask,
                    );
                } else if (q - cx) * (q - cx) + (z - cy) * (z - cy) <= rad * rad {
                    let filtered = pixels[(py - top) as usize * width + (px - l) as usize];
                    self.pixel(px, py, opacity(over(premultiply(t.tint), filtered), alpha));
                }
            }
        }
        Ok(width * height * 8)
    }
}
pub fn blur(
    pixels: &mut [[u8; 4]],
    width: usize,
    height: usize,
    radius: usize,
    passes: usize,
) -> Result<()> {
    if width == 0
        || height == 0
        || width.checked_mul(height) != Some(pixels.len())
        || pixels.len() > 4096 * 4096
        || radius > 64
        || passes > 3
    {
        return Err(Error::Bounds);
    }
    let mut scratch = vec![[0; 4]; pixels.len()];
    let div = (2 * radius + 1) as u32;
    for _ in 0..passes {
        for axis in 0..2 {
            let (lines, len) = if axis == 0 {
                (height, width)
            } else {
                (width, height)
            };
            for line in 0..lines {
                let index = |at: usize| {
                    if axis == 0 {
                        line * width + at
                    } else {
                        at * width + line
                    }
                };
                let mut sum = [0u32; 4];
                for j in 0..2 * radius + 1 {
                    let p = pixels
                        [index((j as isize - radius as isize).clamp(0, len as isize - 1) as usize)];
                    for k in 0..4 {
                        sum[k] += p[k] as u32
                    }
                }
                for i in 0..len {
                    scratch[index(i)] = sum.map(|v| ((v + div / 2) / div) as u8);
                    let old = pixels[index(i.saturating_sub(radius))];
                    let new = pixels[index((i + radius + 1).min(len - 1))];
                    for k in 0..4 {
                        sum[k] = sum[k] - old[k] as u32 + new[k] as u32;
                    }
                }
            }
            pixels.copy_from_slice(&scratch);
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Default)]
pub struct Counters {
    pub composition_ns: u64,
    pub nodes: u64,
    pub touched: u64,
    pub shape_work: u64,
    pub missing: u64,
    pub realized: u64,
    pub scratch_peak: usize,
    pub glyph_cache_entries: usize,
    pub glyph_cache_bytes: usize,
}
pub fn render(
    scene: &mut Scene,
    m: &Model,
    preedit: Option<&str>,
    engine: &mut Engine<'_>,
    canvas: &mut Canvas<'_>,
) -> Result<Counters> {
    render_region(scene, m, preedit, engine, canvas, None)
}

/// Repaint an integer pixel region from the background in painter order. Callers
/// must use full redraw for backdrop reads; see retained::Frame.
pub(crate) fn render_region(
    scene: &mut Scene,
    m: &Model,
    preedit: Option<&str>,
    engine: &mut Engine<'_>,
    canvas: &mut Canvas<'_>,
    region: Option<[u32; 4]>,
) -> Result<Counters> {
    canvas.scissor = region;
    scene.layout();
    canvas.smooth_edges |= scene.selected;
    canvas.caret = scene.visual_caret;
    canvas.hide_caret = scene.caret_hidden || (scene.selected && m.session().focus != "editor");
    if scene.selected {
        // Conservative full-surface groups plus blur ping-pong. Reject pressure
        // before clearing the unpublished buffer; allocator-wide OOM is separate.
        fn groups(s: &Scene, id: &str) -> Result<usize> {
            let n = s.tree.node(id)?;
            let mut depth = 0;
            for c in &n.children {
                depth = depth.max(groups(s, c)?);
            }
            Ok(depth + usize::from(n.clip || s.motion.opacity(&n.id, n.opacity) < 1.0))
        }
        if canvas
            .bytes
            .len()
            .checked_mul(groups(scene, &scene.tree.root)? * 2 + 2)
            .is_none_or(|v| v > 64 * 1024 * 1024)
        {
            return Err(Error::Exhausted);
        }
    }
    let mut c = Counters::default();
    let t = &scene.tokens;
    let s = canvas.scale as f32 / 100.0;
    let surface = Rect([0.0, 0.0, canvas.width as f32 / s, canvas.height as f32 / s]);
    let clip = if let Some([x, y, w, h]) = region {
        for row in y..y + h {
            let start = row as usize * canvas.stride as usize + x as usize * 4;
            canvas.bytes[start..start + w as usize * 4].fill(0);
        }
        Rect([x as f32 / s, y as f32 / s, w as f32 / s, h as f32 / s])
    } else {
        canvas.bytes.fill(0); // Padding is initialized before publication.
        surface
    };
    canvas.fill(surface, clip, 0.0, t.colors[0], 255);
    fn walk(
        scene: &Scene,
        id: &str,
        m: &Model,
        preedit: Option<&str>,
        engine: &mut Engine<'_>,
        cv: &mut Canvas<'_>,
        clip: Rect,
        c: &mut Counters,
    ) -> Result<()> {
        let n = scene.tree.node(id)?;
        c.nodes += 1;
        let t = &scene.tokens;
        let clip = if n.clip { clip.intersect(n.rect) } else { clip };
        let group_alpha = (scene.motion.opacity(&n.id, n.opacity) * 255.0 + 0.5) as u8;
        let reduced_launcher =
            n.id == "launcher" && (t.reduced_effects || m.preferences().reduced_effects);
        let deferred_clip = cv.smooth_edges && n.clip;
        let radius = if n.focusable {
            t.control_radius
        } else {
            t.radius
        };
        let composition_start = cv.clock.map(|clock| clock());
        let before = if (group_alpha < 255 && !reduced_launcher) || deferred_clip {
            Some(cv.bytes.to_vec())
        } else {
            None
        };
        if let Some(start) = composition_start {
            c.composition_ns += (cv.clock.unwrap())().saturating_sub(start);
        }
        let opacity = if before.is_some() { 255 } else { group_alpha };
        if deferred_clip {
            cv.deferred_clips.push((n.rect, radius));
        }
        if n.clip && !deferred_clip {
            cv.rounded_clips.push((
                n.rect,
                if deferred_clip {
                    0.0
                } else if n.focusable {
                    t.control_radius
                } else {
                    t.radius
                },
            ));
        }
        if n.id == "launcher" {
            if t.reduced_effects || m.preferences().reduced_effects {
                cv.fill(
                    n.rect,
                    clip,
                    if deferred_clip { 0.0 } else { t.radius },
                    t.fallback,
                    255,
                )
            } else {
                let mut backdrop = t.clone();
                if deferred_clip {
                    backdrop.radius = 0.0;
                }
                c.scratch_peak = c
                    .scratch_peak
                    .max(cv.backdrop(n.rect, clip, &backdrop, opacity)?)
            }
        } else if n.id != "desktop" {
            let color = if matches!(n.kind, Kind::Field | Kind::Editor) {
                t.colors[0]
            } else {
                t.colors[1]
            };
            cv.fill(
                n.rect,
                clip,
                if deferred_clip {
                    0.0
                } else if n.focusable {
                    t.control_radius
                } else {
                    t.radius
                },
                color,
                opacity,
            );
        }
        if n.focusable && m.session().focus == n.id {
            cv.fill(
                n.rect,
                clip,
                if deferred_clip { 0.0 } else { t.control_radius },
                t.colors[5],
                255,
            );
            cv.fill(
                Rect([
                    n.rect.0[0] + 1.0,
                    n.rect.0[1] + 1.0,
                    (n.rect.0[2] - 2.0).max(0.0),
                    (n.rect.0[3] - 2.0).max(0.0),
                ]),
                clip,
                (t.control_radius - 1.0).max(0.0),
                t.colors[0],
                255,
            );
        }
        let pad = t.spacing;
        let [x, y, w, h] = n.rect.0;
        let content = Rect([
            x + pad,
            y + 2.0,
            (w - 2.0 * pad).max(0.0),
            (h - 4.0).max(0.0),
        ]);
        if n.kind == Kind::List {
            let rows = realize(m.session().file_scroll, 10000, h, 24.0)?;
            c.realized += rows.len() as u64;
            for row in rows {
                let index = row - m.session().file_scroll;
                let rect = Rect([x + pad, y + index as f32 * 24.0, w - pad * 2.0, 24.0]);
                if row == m.session().selected_file {
                    cv.fill(rect, clip, 0.0, t.colors[4], 80);
                    if cv.smooth_edges {
                        cv.fill(
                            Rect([x, y + index as f32 * 24.0, 3.0, 24.0]),
                            clip,
                            0.0,
                            t.colors[2],
                            255,
                        );
                    }
                }
                let state = lcg_at(20261020, row + 1);
                let label = format!("File {row:05} — {state:08x}.txt");
                let (missing, work) = cv.text(
                    engine,
                    &label,
                    rect,
                    clip,
                    t.body_size,
                    false,
                    t.colors[2],
                    None,
                    t.colors[4],
                )?;
                c.missing += missing as u64;
                c.shape_work += work as u64;
            }
        } else {
            let text = match n.kind {
                Kind::Editor => m.document().text(),
                Kind::Field => {
                    if m.document().query().is_empty() {
                        &n.text
                    } else {
                        m.document().query()
                    }
                }
                _ => &n.text,
            };
            if !text.is_empty() {
                let selection = if n.kind == Kind::Editor {
                    let sel = m.document().selection();
                    Some(sel.anchor.min(sel.caret) as usize..sel.anchor.max(sel.caret) as usize)
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
                let (missing, work) = cv.text(
                    engine,
                    text,
                    content,
                    clip,
                    size,
                    n.kind == Kind::Editor,
                    t.colors[2],
                    selection,
                    t.colors[4],
                )?;
                c.missing += missing as u64;
                c.shape_work += work as u64;
            }
            if n.kind == Kind::Editor {
                if let Some(text) = preedit {
                    let (missing, work) = cv.text(
                        engine,
                        text,
                        Rect([x + pad, y + 32.0, w - 2.0 * pad, 28.0]),
                        clip,
                        t.editor_size,
                        true,
                        t.colors[4],
                        None,
                        t.colors[4],
                    )?;
                    c.missing += missing as u64;
                    c.shape_work += work as u64;
                }
            }
            if n.kind == Kind::Toggle {
                let checked = if n.id == "reduced-effects" {
                    m.preferences().reduced_effects
                } else {
                    m.preferences().reduced_motion
                };
                cv.fill(
                    Rect([x + w - 18.0, y + 8.0, 10.0, 10.0]),
                    clip,
                    2.0,
                    if checked { t.colors[4] } else { t.colors[3] },
                    255,
                );
                if cv.smooth_edges && checked {
                    cv.fill(
                        Rect([x + w - 16.0, y + 11.0, 2.0, 4.0]),
                        clip,
                        0.0,
                        t.colors[0],
                        255,
                    );
                    cv.fill(
                        Rect([x + w - 14.0, y + 13.0, 4.0, 2.0]),
                        clip,
                        0.0,
                        t.colors[0],
                        255,
                    );
                }
            }
            if n.resource == "icon.missing" {
                cv.image(
                    Rect([x + w - 16.0, y, 16.0, 16.0]),
                    clip,
                    2,
                    2,
                    &[
                        [255, 0, 255, 255],
                        [0, 0, 0, 255],
                        [0, 0, 0, 255],
                        [255, 0, 255, 255],
                    ],
                )?;
            }
        }
        for child in &n.children {
            walk(scene, child, m, preedit, engine, cv, clip, c)?;
        }
        if n.clip && !deferred_clip {
            cv.rounded_clips.pop();
        }
        if deferred_clip {
            cv.deferred_clips.pop();
        }
        if let Some(before) = before {
            let composition_start = cv.clock.map(|clock| clock());
            c.scratch_peak = c.scratch_peak.max(before.len() + c.scratch_peak);
            let masks = if deferred_clip {
                // One mask for the entire isolated clip, avoiding repeated edge
                // coverage when several descendants touch the same rounded edge.
                let [l, top, r, b] = n.rect.intersect(clip).pixels(cv.scale, cv.width, cv.height);
                let mut masks = vec![0u8; cv.width as usize * cv.height as usize];
                for y in top..b {
                    for x in l..r {
                        let mut ancestor = u64::MAX;
                        for (rect, radius) in &cv.deferred_clips {
                            ancestor &= cv.mask(*rect, *radius, x, y);
                        }
                        let count = ancestor.count_ones();
                        let covered = (cv.mask(n.rect, radius, x, y) & ancestor).count_ones();
                        masks[y as usize * cv.width as usize + x as usize] = if count == 0 {
                            0
                        } else {
                            ((covered * 255 + count / 2) / count) as u8
                        };
                    }
                }
                Some(masks)
            } else {
                None
            };
            let effective_alpha = if reduced_launcher { 255 } else { group_alpha };
            let stride = cv.stride as usize / 4;
            let width = cv.width as usize;
            for (index, (after, old)) in cv
                .bytes
                .chunks_exact_mut(4)
                .zip(before.chunks_exact(4))
                .enumerate()
            {
                if after != old {
                    let alpha = if let Some(masks) = &masks {
                        let x = index % stride;
                        let y = index / stride;
                        if x >= width {
                            0
                        } else {
                            ((masks[y * width + x] as u32 * effective_alpha as u32 + 127) / 255)
                                as u8
                        }
                    } else {
                        group_alpha
                    };
                    let source: [u8; 4] = (&*after).try_into().unwrap();
                    let destination: [u8; 4] = old.try_into().unwrap();
                    after.copy_from_slice(&over(crate::paint::opacity(source, alpha), destination));
                }
            }
            if let Some(start) = composition_start {
                c.composition_ns += (cv.clock.unwrap())().saturating_sub(start);
            }
        }
        Ok(())
    }
    walk(
        scene,
        &scene.tree.root,
        m,
        preedit,
        engine,
        canvas,
        clip,
        &mut c,
    )?;
    c.touched = canvas.touched;
    let (entries, bytes) = engine.cache_usage();
    c.glyph_cache_entries = entries;
    c.glyph_cache_bytes = bytes;
    scene.paint_passes += 1;
    scene.dirty.paint = false;
    Ok(c)
}
/// Jump-ahead LCG: fixed logarithmic work for any virtual row, no materialized list.
pub fn lcg_at(seed: u32, mut delta: u32) -> u32 {
    let (mut mult, mut add, mut acc_mult, mut acc_add) = (1664525u32, 1013904223u32, 1u32, 0u32);
    while delta > 0 {
        if delta & 1 != 0 {
            acc_mult = acc_mult.wrapping_mul(mult);
            acc_add = acc_add.wrapping_mul(mult).wrapping_add(add)
        }
        add = mult.wrapping_add(1).wrapping_mul(add);
        mult = mult.wrapping_mul(mult);
        delta >>= 1;
    }
    acc_mult.wrapping_mul(seed).wrapping_add(acc_add)
}
