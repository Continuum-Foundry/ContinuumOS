//! Bounded two-pass layout. Rules are internal values, not a resource codec.
use crate::{
    scene::{Kind, Tree},
    *,
};
use alloc::vec;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Size {
    Fixed(f32),
    Fill,
    Content,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Placement {
    /// Parent-local origin. Stretch axes retain their authored trailing margin.
    Anchored {
        x: f32,
        y: f32,
        stretch: [bool; 2],
    },
    Flow,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    pub id: String,
    pub size: [Size; 2],
    pub placement: Placement,
    pub padding: f32,
    pub gap: f32,
    pub scroll: [f32; 2],
}
#[derive(Clone, Debug)]
pub struct Plan {
    pub rules: Vec<Rule>,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Work {
    pub measured: usize,
    pub arranged: usize,
    pub depth: usize,
}

impl Plan {
    /// Retain the locked absolute scene at its original viewport; nested origins
    /// become local. Explicit stretch axes may then be enabled by the host shell.
    pub fn anchored(tree: &Tree) -> Result<Self> {
        tree.validate()?;
        let rules = tree
            .nodes
            .iter()
            .map(|n| {
                let parent = tree.nodes.iter().find(|p| p.children.contains(&n.id));
                let origin = parent
                    .map(|p| [p.rect.0[0], p.rect.0[1]])
                    .unwrap_or([0.0; 2]);
                Rule {
                    id: n.id.clone(),
                    size: [Size::Fixed(n.rect.0[2]), Size::Fixed(n.rect.0[3])],
                    placement: Placement::Anchored {
                        x: n.rect.0[0] - origin[0],
                        y: n.rect.0[1] - origin[1],
                        stretch: [false; 2],
                    },
                    padding: 0.0,
                    gap: 0.0,
                    scroll: [0.0; 2],
                }
            })
            .collect();
        Ok(Self { rules })
    }
    pub fn rule_mut(&mut self, id: &str) -> Result<&mut Rule> {
        self.rules
            .iter_mut()
            .find(|r| r.id == id)
            .ok_or(Error::Invalid)
    }
    fn rule(&self, id: &str) -> Result<&Rule> {
        self.rules.iter().find(|r| r.id == id).ok_or(Error::Invalid)
    }
    pub fn arrange(&self, source: &Tree, viewport: Rect) -> Result<(Tree, Work)> {
        source.validate()?;
        viewport.validate()?;
        if viewport.0[2] <= 0.0 || viewport.0[3] <= 0.0 || self.rules.len() != source.nodes.len() {
            return Err(Error::Bounds);
        }
        let mut ids = Vec::new();
        for r in &self.rules {
            source.node(&r.id)?;
            if ids.contains(&r.id) {
                return Err(Error::Duplicate);
            }
            ids.push(r.id.clone());
            for size in r.size {
                if let Size::Fixed(v) = size {
                    if !v.is_finite() || !(0.0..=16384.0).contains(&v) {
                        return Err(Error::Bounds);
                    }
                }
            }
            if !r.padding.is_finite()
                || !(0.0..=256.0).contains(&r.padding)
                || !r.gap.is_finite()
                || !(0.0..=256.0).contains(&r.gap)
                || r.scroll
                    .iter()
                    .any(|v| !v.is_finite() || !(0.0..=16384.0).contains(v))
            {
                return Err(Error::Bounds);
            }
            if let Placement::Anchored { x, y, .. } = r.placement {
                if [x, y].iter().any(|v| !v.is_finite() || v.abs() > 16384.0) {
                    return Err(Error::Bounds);
                }
            }
        }
        let mut measured = vec![[0.0; 2]; source.nodes.len()];
        let mut work = Work::default();
        fn measure(
            p: &Plan,
            t: &Tree,
            id: &str,
            depth: usize,
            sizes: &mut [[f32; 2]],
            work: &mut Work,
        ) -> Result<[f32; 2]> {
            let n = t.node(id)?;
            let r = p.rule(id)?;
            work.measured += 1;
            work.depth = work.depth.max(depth);
            let mut natural = [0.0f32; 2];
            let mut flows = 0;
            for c in &n.children {
                let s = measure(p, t, c, depth + 1, sizes, work)?;
                let cr = p.rule(c)?;
                match cr.placement {
                    Placement::Flow => {
                        flows += 1;
                        match n.kind {
                            Kind::Row => {
                                natural[0] += s[0];
                                natural[1] = natural[1].max(s[1]);
                            }
                            Kind::Column => {
                                natural[1] += s[1];
                                natural[0] = natural[0].max(s[0]);
                            }
                            _ => {
                                for a in 0..2 {
                                    natural[a] = natural[a].max(s[a]);
                                }
                            }
                        }
                    }
                    Placement::Anchored { x, y, .. } => {
                        natural[0] = natural[0].max(x + s[0]);
                        natural[1] = natural[1].max(y + s[1]);
                    }
                }
            }
            if flows > 1 {
                if n.kind == Kind::Row {
                    natural[0] += (flows - 1) as f32 * r.gap;
                }
                if n.kind == Kind::Column {
                    natural[1] += (flows - 1) as f32 * r.gap;
                }
            }
            for a in 0..2 {
                natural[a] = match r.size[a] {
                    Size::Fixed(v) => v,
                    Size::Fill => 0.0,
                    Size::Content => {
                        if n.children.is_empty() {
                            n.rect.0[a + 2]
                        } else {
                            natural[a] + 2.0 * r.padding
                        }
                    }
                };
                if !natural[a].is_finite() || natural[a] > 16384.0 {
                    return Err(Error::Bounds);
                }
            }
            sizes[t
                .nodes
                .iter()
                .position(|n| n.id == id)
                .ok_or(Error::Invalid)?] = natural;
            Ok(natural)
        }
        measure(self, source, &source.root, 1, &mut measured, &mut work)?;
        let mut out = source.clone();
        fn arrange(
            p: &Plan,
            source: &Tree,
            out: &mut Tree,
            id: &str,
            frame: Rect,
            sizes: &[[f32; 2]],
            work: &mut Work,
        ) -> Result<()> {
            frame.validate()?;
            let at = source
                .nodes
                .iter()
                .position(|n| n.id == id)
                .ok_or(Error::Invalid)?;
            let n = &source.nodes[at];
            let r = p.rule(id)?;
            out.nodes[at].rect = frame;
            if n.kind == Kind::Scroll {
                out.nodes[at].clip = true;
            }
            if n.kind == Kind::List {
                crate::scene::realize(0, 10000, frame.0[3], 24.0)?;
            }
            work.arranged += 1;
            let inner = [
                (frame.0[2] - 2.0 * r.padding).max(0.0),
                (frame.0[3] - 2.0 * r.padding).max(0.0),
            ];
            let axis = if n.kind == Kind::Row {
                Some(0)
            } else if n.kind == Kind::Column {
                Some(1)
            } else {
                None
            };
            let mut fixed = 0.0;
            let mut fills = 0;
            let mut flows = 0;
            for c in &n.children {
                let cr = p.rule(c)?;
                if cr.placement == Placement::Flow {
                    flows += 1;
                    if let Some(a) = axis {
                        if cr.size[a] == Size::Fill {
                            fills += 1;
                        } else {
                            fixed += sizes[source
                                .nodes
                                .iter()
                                .position(|n| n.id == *c)
                                .ok_or(Error::Invalid)?][a];
                        }
                    }
                }
            }
            let mut cursor = 0.0;
            let share = axis
                .map(|a| {
                    ((inner[a] - fixed - (flows as f32 - 1.0).max(0.0) * r.gap).max(0.0))
                        / (fills.max(1) as f32)
                })
                .unwrap_or(0.0);
            for c in &n.children {
                let ci = source
                    .nodes
                    .iter()
                    .position(|n| n.id == *c)
                    .ok_or(Error::Invalid)?;
                let cr = p.rule(c)?;
                let mut size = sizes[ci];
                let mut origin = [r.padding; 2];
                match cr.placement {
                    Placement::Flow => {
                        for a in 0..2 {
                            if cr.size[a] == Size::Fill {
                                size[a] = if axis == Some(a) { share } else { inner[a] };
                            }
                        }
                        if let Some(a) = axis {
                            origin[a] += cursor;
                            cursor += size[a] + r.gap;
                        }
                    }
                    Placement::Anchored { x, y, stretch } => {
                        origin = [x, y];
                        for a in 0..2 {
                            if cr.size[a] == Size::Fill {
                                size[a] = (inner[a] - origin[a]).max(0.0);
                            }
                            if stretch[a] {
                                size[a] = (size[a] + frame.0[a + 2] - n.rect.0[a + 2]).max(0.0);
                            }
                        }
                    }
                }
                for a in 0..2 {
                    origin[a] += frame.0[a];
                    if n.kind == Kind::Scroll {
                        origin[a] -= r.scroll[a];
                    }
                }
                arrange(
                    p,
                    source,
                    out,
                    c,
                    Rect([origin[0], origin[1], size[0], size[1]]),
                    sizes,
                    work,
                )?;
            }
            Ok(())
        }
        arrange(
            self,
            source,
            &mut out,
            &source.root,
            viewport,
            &measured,
            &mut work,
        )?;
        out.validate()?;
        Ok((out, work))
    }
}

pub fn logical_pointer(x: f32, y: f32, scale: u16) -> Result<[f32; 2]> {
    if !x.is_finite() || !y.is_finite() || ![100, 150, 200].contains(&scale) {
        return Err(Error::Bounds);
    }
    Ok([x * 100.0 / scale as f32, y * 100.0 / scale as f32])
}
