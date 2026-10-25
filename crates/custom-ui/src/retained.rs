//! Bounded retained software frame. Internal values, not a surface/wire ABI.
use crate::{
    paint,
    scene::{Kind, Scene, Tokens, Tree},
    *,
};
use editor_model::Model;

pub const MAX_REGIONS: usize = 32;

/// Physical integer rectangles, merged before painting. Fragmentation or coverage
/// above 60% conservatively falls back to the entire surface.
pub struct Damage {
    pub regions: Vec<[u32; 4]>,
    pub fallback: bool,
    width: u32,
    height: u32,
    changes: usize,
}
impl Damage {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            regions: Vec::new(),
            fallback: false,
            width,
            height,
            changes: 0,
        }
    }
    pub fn full(&mut self) {
        self.regions.clear();
        self.regions.push([0, 0, self.width, self.height]);
        self.fallback = true;
    }
    pub fn add(&mut self, rect: Rect, scale: u16) {
        if self.fallback {
            return;
        }
        let [l, t, r, b] = rect.pixels(scale, self.width, self.height);
        if l == r || t == b {
            return;
        }
        self.changes += 1;
        if self.changes > MAX_REGIONS {
            self.full();
            return;
        }
        let mut area = [l, t, r - l, b - t];
        let mut i = 0;
        while i < self.regions.len() {
            let q = self.regions[i];
            if area[0] <= q[0] + q[2]
                && q[0] <= area[0] + area[2]
                && area[1] <= q[1] + q[3]
                && q[1] <= area[1] + area[3]
            {
                let x = area[0].min(q[0]);
                let y = area[1].min(q[1]);
                area = [
                    x,
                    y,
                    (area[0] + area[2]).max(q[0] + q[2]) - x,
                    (area[1] + area[3]).max(q[1] + q[3]) - y,
                ];
                self.regions.swap_remove(i);
                i = 0;
            } else {
                i += 1;
            }
        }
        self.regions.push(area);
        if self.pixels() * 100 > self.width as u64 * self.height as u64 * 60 {
            self.full();
        }
    }
    pub fn pixels(&self) -> u64 {
        self.regions.iter().map(|r| r[2] as u64 * r[3] as u64).sum()
    }
}

// Primitives with deliberate overscan may extend outside the authored node.
// Ancestor clips can reduce this bound, but never enlarge it.
fn paint_bounds(n: &crate::scene::Node) -> Rect {
    let [x, y, w, h] = n.rect.0;
    let mut bounds = n.rect;
    if n.kind == Kind::List {
        bounds = bounds.union(Rect([x, y, w, h + 48.0]));
    }
    if n.kind == Kind::Editor {
        bounds = bounds.union(Rect([x, y + 32.0, w, 28.0]));
    }
    if n.kind == Kind::Toggle {
        bounds = bounds.union(Rect([x + w - 18.0, y + 8.0, 10.0, 10.0]));
    }
    if n.resource == "icon.missing" {
        bounds = bounds.union(Rect([x + w - 16.0, y, 16.0, 16.0]));
    }
    bounds.expand(1.0)
}

#[derive(Debug)]
struct Previous {
    width: u32,
    height: u32,
    tree: Tree,
    tokens: Tokens,
    model: Model,
    preedit: Option<String>,
    opacity: Vec<f32>,
    caret: Option<text_core::VisualCaret>,
    hidden: bool,
    selected: bool,
    scale: u16,
}
#[derive(Debug)]
pub struct Frame {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
    previous: Option<Previous>,
    pub paints: u64,
    pub clock: Option<fn() -> u64>,
    unsubmitted: bool,
}
impl Frame {
    pub fn new(width: u32, height: u32) -> Result<Self> {
        if width == 0
            || height == 0
            || width > 4096
            || height > 4096
            || width as usize * height as usize * 4 > 16 * 1024 * 1024
        {
            return Err(Error::Bounds);
        }
        Ok(Self {
            pixels: alloc::vec![0; width as usize*height as usize*4],
            width,
            height,
            previous: None,
            paints: 0,
            clock: None,
            unsubmitted: false,
        })
    }
    pub fn invalidate(&mut self) {
        self.previous = None;
    }
    /// The pixels were fully validated before publication, but the consumer
    /// still owns the previous generation. The next submission must be full.
    pub fn mark_unsubmitted(&mut self) {
        self.unsubmitted = true;
    }
    pub fn bytes(&self) -> usize {
        self.pixels.capacity()
    }
    pub fn draw(
        &mut self,
        scene: &mut Scene,
        model: &Model,
        preedit: Option<&str>,
        engine: &mut text_core::Engine<'_>,
        scale: u16,
    ) -> Result<(Damage, paint::Counters)> {
        if ![100, 150, 200].contains(&scale)
            || self.width == 0
            || self.height == 0
            || self.width > 4096
            || self.height > 4096
            || self.pixels.len() != self.width as usize * self.height as usize * 4
            || self.pixels.len() > 16 * 1024 * 1024
        {
            return Err(Error::Bounds);
        }
        let mut damage = Damage::new(self.width, self.height);
        let unsubmitted = core::mem::take(&mut self.unsubmitted);
        let mut add_subtree = |tree: &Tree, id: &str| {
            for n in &tree.nodes {
                if tree.descendant(&n.id, id) {
                    damage.add(paint_bounds(n), scale);
                }
            }
        };
        if let Some(old) = &self.previous {
            if old.width != self.width
                || old.height != self.height
                || old.tokens != scene.tokens
                || old.scale != scale
                || old.selected != scene.selected
                || old.tree.root != scene.tree.root
                || (unsubmitted
                    && (old.tree.generation != scene.tree.generation
                        || old.tree.revision != scene.tree.revision
                        || old.tree.nodes != scene.tree.nodes
                        || old.model != *model
                        || old.preedit.as_deref() != preedit
                        || old.caret != scene.visual_caret
                        || old.hidden != scene.caret_hidden
                        || scene.tree.nodes.iter().enumerate().any(|(i, n)| {
                            scene.motion.opacity(&n.id, n.opacity) != old.opacity[i]
                        })))
            {
                damage.full();
            } else {
                // Include both old/new descendants for movement, clipping, reorder,
                // removal and parent opacity changes. Never trust client damage.
                for (i, n) in old.tree.nodes.iter().enumerate() {
                    if scene.tree.node(&n.id).map_or(true, |q| {
                        q != n || scene.motion.opacity(&q.id, q.opacity) != old.opacity[i]
                    }) {
                        add_subtree(&old.tree, &n.id);
                        if scene.tree.node(&n.id).is_ok() {
                            add_subtree(&scene.tree, &n.id);
                        }
                    }
                }
                for n in &scene.tree.nodes {
                    if old.tree.node(&n.id).is_err() {
                        add_subtree(&scene.tree, &n.id);
                    }
                }
                if old.model != *model
                    || old.preedit.as_deref() != preedit
                    || old.caret != scene.visual_caret
                    || old.hidden != scene.caret_hidden
                {
                    let caret_only = old.model == *model
                        && old.preedit.is_none()
                        && preedit.is_none()
                        && old.caret == scene.visual_caret
                        && old.tree.nodes == scene.tree.nodes
                        && old.hidden != scene.caret_hidden;
                    for tree in [&old.tree, &scene.tree] {
                        for n in &tree.nodes {
                            if (n.kind == Kind::Editor
                                && (old.model.document() != model.document()
                                    || old.preedit.as_deref() != preedit
                                    || old.caret != scene.visual_caret
                                    || old.hidden != scene.caret_hidden))
                                || (n.kind == Kind::Field
                                    && old.model.document().query() != model.document().query())
                                || (n.kind == Kind::List
                                    && (old.model.session().file_scroll
                                        != model.session().file_scroll
                                        || old.model.session().selected_file
                                            != model.session().selected_file))
                                || (n.kind == Kind::Toggle
                                    && old.model.preferences() != model.preferences())
                                || (old.model.session().focus != model.session().focus
                                    && (n.id == old.model.session().focus
                                        || n.id == model.session().focus))
                            {
                                if caret_only && n.kind == Kind::Editor {
                                    for rect in paint::caret_damage(
                                        n,
                                        &scene.tokens,
                                        model,
                                        scene.visual_caret,
                                        engine,
                                        scale,
                                    )? {
                                        damage.add(rect, scale);
                                    }
                                } else {
                                    damage.add(paint_bounds(n), scale);
                                }
                            }
                        }
                    }
                }
                // A dirty flag with no modeled difference may be a clock or other
                // explicit invalidation. It must not silently lose that repaint.
                if scene.dirty.paint && damage.regions.is_empty() {
                    damage.full();
                }
            }
        } else {
            damage.full();
        }
        if self.previous.as_ref().is_some_and(|old| {
            old.model.preferences().reduced_effects != model.preferences().reduced_effects
        }) {
            damage.full();
        }
        if unsubmitted {
            if damage.regions.is_empty() {
                damage.full();
                return Ok((
                    damage,
                    paint::Counters {
                        reused_frame: true,
                        ..Default::default()
                    },
                ));
            }
            // Intervening edits/ticks/scale changes invalidate staged pixels.
            // Render and submit fully so an old consumer cannot retain stale areas.
            damage.full();
        }
        // Partial redraw is valid only outside every backdrop source footprint.
        // Any damage touching the exact clamped multi-pass support still uses
        // full redraw, which reconstructs lower layers before reading them.
        if !damage.regions.is_empty()
            && !scene.tokens.reduced_effects
            && !model.preferences().reduced_effects
        {
            let radius = (scene.tokens.blur_radius * scale as u32).div_ceil(100);
            let support = (radius * scene.tokens.passes) as f32 * 100.0 / scale as f32;
            for n in scene.tree.nodes.iter().filter(|n| n.id == "launcher") {
                let [l, t, r, b] = n
                    .rect
                    .expand(support)
                    .pixels(scale, self.width, self.height);
                if damage
                    .regions
                    .iter()
                    .any(|q| q[0] < r && q[0] + q[2] > l && q[1] < b && q[1] + q[3] > t)
                {
                    damage.full();
                    break;
                }
            }
        }
        let mut counters = paint::Counters::default();
        for region in &damage.regions {
            let mut canvas = paint::Canvas::new(
                &mut self.pixels,
                self.width,
                self.height,
                self.width * 4,
                scale,
            )?;
            canvas.clock = self.clock;
            canvas.cache_layouts = true;
            let result = paint::render_region(
                scene,
                model,
                preedit,
                engine,
                &mut canvas,
                if damage.fallback { None } else { Some(*region) },
            );
            let c = match result {
                Ok(c) => c,
                Err(e) => {
                    self.previous = None;
                    return Err(e);
                }
            };
            counters.composition_ns += c.composition_ns;
            counters.work.add(&c.work);
            counters.nodes += c.nodes;
            counters.touched += c.touched;
            counters.shape_work += c.shape_work;
            counters.missing += c.missing;
            counters.realized += c.realized;
            counters.scratch_peak = counters.scratch_peak.max(c.scratch_peak);
            counters.glyph_cache_entries = c.glyph_cache_entries;
            counters.glyph_cache_bytes = c.glyph_cache_bytes;
        }
        if !damage.regions.is_empty() {
            self.paints += 1;
            let start = self.clock.map(|clock| clock());
            self.previous = Some(Previous {
                width: self.width,
                height: self.height,
                tree: scene.tree.clone(),
                tokens: scene.tokens.clone(),
                model: model.clone(),
                preedit: preedit.map(String::from),
                opacity: scene
                    .tree
                    .nodes
                    .iter()
                    .map(|n| scene.motion.opacity(&n.id, n.opacity))
                    .collect(),
                caret: scene.visual_caret,
                hidden: scene.caret_hidden,
                selected: scene.selected,
                scale,
            });
            if let Some(start) = start {
                counters.work.snapshot_ns += (self.clock.unwrap())().saturating_sub(start);
            }
        }
        Ok((damage, counters))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fragmentation_and_coverage_fall_back_and_round_outward() {
        let mut d = Damage::new(1920, 1080);
        d.add(Rect([1.1, 2.1, 1.1, 1.1]), 150);
        assert_eq!(d.regions, [[1, 3, 3, 2]]);
        for i in 0..MAX_REGIONS {
            d.add(Rect([i as f32 * 3.0, 20.0, 1.0, 1.0]), 100);
        }
        assert!(d.fallback);
        assert_eq!(d.pixels(), 1920 * 1080);
        let mut d = Damage::new(100, 100);
        d.add(Rect([0.0, 0.0, 90.0, 90.0]), 100);
        assert!(d.fallback);
    }
}
