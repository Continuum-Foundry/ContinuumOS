use crate::*;
use alloc::{string::ToString, vec};
use editor_model::Model;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Row,
    Column,
    Overlay,
    Box,
    Scroll,
    List,
    Label,
    Button,
    Field,
    Editor,
    Toggle,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub id: String,
    pub kind: Kind,
    pub children: Vec<String>,
    pub rect: Rect,
    pub clip: bool,
    pub opacity: f32,
    pub text: String,
    pub action: String,
    pub resource: String,
    pub semantic_id: String,
    pub role: String,
    pub name: String,
    pub focusable: bool,
}
#[derive(Clone, Debug)]
pub struct Tree {
    pub nodes: Vec<Node>,
    pub root: String,
    pub revision: u64,
    pub generation: u64,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Dirty {
    pub layout: bool,
    pub paint: bool,
    pub semantics: bool,
}
impl Dirty {
    pub const ALL: Self = Self {
        layout: true,
        paint: true,
        semantics: true,
    };
}
#[derive(Clone, Debug)]
pub struct Tokens {
    /// surface/panel/text/muted/accent/focus/danger, straight RGBA8.
    pub colors: [[u8; 4]; 7],
    pub spacing: f32,
    pub radius: f32,
    pub control_radius: f32,
    pub body_size: u16,
    pub editor_size: u16,
    pub title_size: u16,
    pub tint: [u8; 4],
    pub fallback: [u8; 4],
    pub blur_radius: u32,
    pub passes: u32,
    pub reduced_effects: bool,
    pub reduced_motion: bool,
}
impl Tokens {
    pub fn validate(&self) -> Result<()> {
        if !self.spacing.is_finite()
            || !(0.0..=64.0).contains(&self.spacing)
            || !self.radius.is_finite()
            || !(0.0..=64.0).contains(&self.radius)
            || !self.control_radius.is_finite()
            || !(0.0..=64.0).contains(&self.control_radius)
            || [self.body_size, self.editor_size, self.title_size]
                .iter()
                .any(|n| !(8..=48).contains(n))
            || self.blur_radius != 8
            || self.passes != 3
            || self.fallback[3] != 255
        {
            return Err(Error::Bounds);
        }
        Ok(())
    }
}
impl Tree {
    pub fn node(&self, id: &str) -> Result<&Node> {
        self.nodes.iter().find(|n| n.id == id).ok_or(Error::Invalid)
    }
    pub fn validate(&self) -> Result<()> {
        if self.nodes.is_empty()
            || self.nodes.len() > 1024
            || self.revision == 0
            || self.generation == 0
        {
            return Err(Error::Bounds);
        }
        let mut ids = Vec::new();
        let mut semantics = Vec::new();
        for n in &self.nodes {
            n.rect.validate()?;
            if !valid_id(&n.id)
                || !valid_id(&n.semantic_id)
                || ids.contains(&n.id.as_str())
                || semantics.contains(&n.semantic_id.as_str())
            {
                return Err(Error::Duplicate);
            }
            ids.push(n.id.as_str());
            semantics.push(n.semantic_id.as_str());
            if !n.opacity.is_finite()
                || !(0.0..=1.0).contains(&n.opacity)
                || n.children.len() > 64
                || n.text.len() > 4096
                || n.name.len() > 4096
            {
                return Err(Error::Bounds);
            }
            if !["", "document.unsaved", "files.synthetic", "icon.missing"]
                .contains(&n.resource.as_str())
                || ![
                    "",
                    "edit.insert@1",
                    "search.set@1",
                    "files.select@1",
                    "launcher.open@1",
                    "settings.effects@1",
                    "settings.motion@1",
                ]
                .contains(&n.action.as_str())
            {
                return Err(Error::Resource);
            }
        }
        fn walk<'a>(t: &'a Tree, id: &'a str, depth: usize, seen: &mut Vec<&'a str>) -> Result<()> {
            if depth > 32 || seen.contains(&id) {
                return Err(Error::Invalid);
            }
            seen.push(id);
            for c in &t.node(id)?.children {
                walk(t, c, depth + 1, seen)?
            }
            Ok(())
        }
        let mut seen = Vec::new();
        walk(self, &self.root, 1, &mut seen)?;
        if seen.len() != self.nodes.len() {
            return Err(Error::Invalid);
        }
        Ok(())
    }
    pub fn preorder(&self) -> Vec<&Node> {
        fn walk<'a>(t: &'a Tree, id: &str, out: &mut Vec<&'a Node>) {
            if let Ok(n) = t.node(id) {
                out.push(n);
                for child in &n.children {
                    walk(t, child, out)
                }
            }
        }
        let mut out = Vec::new();
        walk(self, &self.root, &mut out);
        out
    }
    /// Panel surfaces occlude underlying targets even where no control is hit.
    pub fn hit(&self, x: f32, y: f32) -> Option<&Node> {
        fn walk<'a>(t: &'a Tree, id: &str, x: f32, y: f32, clip: Rect) -> Option<&'a Node> {
            let n = t.node(id).ok()?;
            let clip = if n.clip { clip.intersect(n.rect) } else { clip };
            if !clip.contains(x, y) {
                return None;
            }
            for c in n.children.iter().rev() {
                if let Some(hit) = walk(t, c, x, y, clip) {
                    return Some(hit);
                }
            }
            n.rect.contains(x, y).then_some(n)
        }
        walk(
            self,
            &self.root,
            x,
            y,
            Rect([-16384.0, -16384.0, 32768.0, 32768.0]),
        )
    }
    pub fn semantics(&self, m: &Model) -> Vec<Semantic> {
        self.preorder()
            .iter()
            .map(|n| {
                let value = match n.kind {
                    Kind::Editor => m.document().text().to_string(),
                    Kind::Field => m.document().query().to_string(),
                    _ => String::new(),
                };
                let realized = if n.kind == Kind::List {
                    realize(m.session().file_scroll, 10000, n.rect.0[3], 24.0).unwrap_or_default()
                } else {
                    vec![]
                };
                Semantic {
                    id: n.semantic_id.clone(),
                    widget: n.id.clone(),
                    role: n.role.clone(),
                    name: n.name.clone(),
                    value,
                    action: n.action.clone(),
                    bounds: n.rect,
                    focused: m.session().focus == n.id,
                    checked: match n.id.as_str() {
                        "reduced-effects" => m.preferences().reduced_effects,
                        "reduced-motion" => m.preferences().reduced_motion,
                        _ => false,
                    },
                    logical_count: if n.kind == Kind::List { 10000 } else { 0 },
                    realized,
                }
            })
            .collect()
    }
}
pub fn realize(first: u32, count: u32, height: f32, row_height: f32) -> Result<Vec<u32>> {
    if count == 0
        || count > 1_000_000_000
        || !height.is_finite()
        || !(0.0..=16384.0).contains(&height)
        || !row_height.is_finite()
        || row_height <= 0.0
    {
        return Err(Error::Bounds);
    }
    let rows = (height / row_height) as usize + 2;
    if rows > 64 {
        return Err(Error::Exhausted);
    }
    let start = first.min(count.saturating_sub(1));
    Ok((start..count.min(start.saturating_add(rows as u32))).collect())
}
pub struct Scene {
    pub tree: Tree,
    pub tokens: Tokens,
    pub dirty: Dirty,
    pub layout_passes: u64,
    pub paint_passes: u64,
    pub semantic_passes: u64,
}
impl Scene {
    pub fn new(tree: Tree, tokens: Tokens) -> Result<Self> {
        tree.validate()?;
        tokens.validate()?;
        Ok(Self {
            tree,
            tokens,
            dirty: Dirty::ALL,
            layout_passes: 0,
            paint_passes: 0,
            semantic_passes: 0,
        })
    }
    /// Full staging before publication; rejection leaves current data untouched.
    pub fn publish(&mut self, tree: Tree, tokens: Tokens) -> Result<Rect> {
        tree.validate()?;
        tokens.validate()?;
        if tree.revision != self.tree.revision.checked_add(1).ok_or(Error::Revision)?
            || tree.generation < self.tree.generation
        {
            return Err(Error::Revision);
        }
        let mut damage = self
            .tree
            .node(&self.tree.root)?
            .rect
            .union(tree.node(&tree.root)?.rect);
        for n in &self.tree.nodes {
            damage = damage.union(n.rect)
        }
        for n in &tree.nodes {
            damage = damage.union(n.rect)
        }
        self.tree = tree;
        self.tokens = tokens;
        self.dirty = Dirty::ALL;
        Ok(damage.expand(24.0))
    }
    pub fn layout(&mut self) -> bool {
        if !self.dirty.layout {
            return false;
        }
        self.layout_passes += 1;
        self.dirty.layout = false;
        true
    }
    pub fn semantics(&mut self, m: &Model) -> Vec<Semantic> {
        self.semantic_passes += 1;
        self.dirty.semantics = false;
        self.tree.semantics(m)
    }
}
