use crate::{
    scene::{Dirty, Kind, Tree},
    *,
};
use alloc::{string::ToString, vec};
use editor_model::{Action, Adapter, Entry, Model, Provider, Request, Rights, Selection};
use text_core::Composition;

#[derive(Clone, Debug)]
pub enum EventKind {
    Key {
        usage: u16,
        down: bool,
    },
    Text(String),
    Composition {
        phase: String,
        text: String,
    },
    Pointer {
        x: f32,
        y: f32,
        button: u8,
        down: bool,
    },
    Move {
        x: f32,
        y: f32,
    },
    Scroll(f32),
    Focus,
    Capture,
    Reset,
}
#[derive(Clone, Debug)]
pub struct Event {
    pub sequence: u64,
    pub time_us: u64,
    pub revision: u64,
    pub target: String,
    pub kind: EventKind,
}
pub struct Controller {
    pub provider: Provider,
    pub capture: String,
    pub keys: Vec<u16>,
    pub buttons: Vec<u8>,
    pub composition: Option<Composition>,
    pub cancelled: bool,
    pub dropped: u64,
    pub resets: u64,
    pub last_action: String,
    pub actions: Vec<String>,
    pub dirty: Dirty,
    sequence: u64,
    time_us: u64,
    request: u64,
    pub pending: Vec<Event>,
    pub reset_reason: String,
    capture_owner: Option<(String, Rect, String, String)>,
    pub modal: String,
    return_focus: String,
    pub visual_caret: Option<text_core::VisualCaret>,
    shaped: Option<text_core::Layout>,
    resync: bool,
}
impl Controller {
    pub fn new(model: Model) -> Result<Self> {
        if model.document().text().len() > text_core::MAX_TEXT {
            return Err(Error::Bounds);
        }
        let mut provider = Provider::new(model, 7).map_err(|_| Error::Model)?;
        provider.grant(1, Rights::ALL).map_err(|_| Error::Model)?;
        Ok(Self {
            provider,
            capture: String::new(),
            keys: vec![],
            buttons: vec![],
            composition: None,
            cancelled: false,
            dropped: 0,
            resets: 0,
            last_action: String::new(),
            actions: vec![],
            dirty: Dirty::ALL,
            sequence: 0,
            time_us: 0,
            request: 0,
            pending: Vec::with_capacity(256),
            reset_reason: String::new(),
            capture_owner: None,
            modal: String::new(),
            return_focus: String::new(),
            visual_caret: None,
            shaped: None,
            resync: false,
        })
    }
    pub fn request(&mut self, action: Action) -> Result<Request> {
        self.request = self.request.checked_add(1).ok_or(Error::Exhausted)?;
        let m = self.provider.model();
        Ok(Request {
            version: 1,
            caller: 1,
            id: self.request,
            epoch: self.provider.epoch(),
            document: m.document().id(),
            expected_document_revision: m.document().revision(),
            expected_model_revision: m.revision(),
            deadline: u64::MAX,
            context: None,
            action,
        })
    }
    pub fn invoke(&mut self, action: Action, entry: Entry) -> Result<()> {
        if let Action::Insert(ref text) = action {
            self.admit_insert(text)?;
        }
        // Accepted text stays renderable on undo/redo as well as direct insertion.
        if let Action::Search(ref text) = action {
            text_core::validate_text(text).map_err(|_| Error::Text)?;
        }
        let changes_focus =
            matches!(&action,Action::Focus(id) if *id!=self.provider.model().session().focus);
        let paint_only = matches!(
            action,
            Action::Focus(_)
                | Action::Select(_)
                | Action::FileSelect(_)
                | Action::FileScroll(_)
                | Action::Effects(_)
                | Action::Motion(_)
                | Action::Theme(_)
                | Action::Search(_)
        );
        let name = action.name().to_string();
        let request = self.request(action)?;
        Adapter::new(&mut self.provider, entry)
            .invoke(request, self.time_us, false)
            .status
            .map_err(|_| Error::Model)?;
        self.last_action = alloc::format!("{name}@1");
        if self.actions.len() == 256 {
            self.actions.remove(0);
        }
        self.actions.push(self.last_action.clone());
        if changes_focus {
            self.composition = None;
            self.visual_caret = None;
        }
        self.dirty.paint = true;
        self.dirty.semantics = true;
        if !paint_only {
            self.dirty.layout = true
        }
        Ok(())
    }
    pub fn reset(&mut self, reason: &str, dropped: u64) {
        self.capture.clear();
        self.capture_owner = None;
        self.keys.clear();
        self.buttons.clear();
        self.composition = None;
        self.cancelled = true;
        self.resets += 1;
        self.dropped += dropped;
        self.reset_reason = reason.into();
        self.visual_caret = None;
        self.dirty.paint = true;
        self.dirty.semantics = true;
    }
    fn admit_insert(&self, text: &str) -> Result<()> {
        let d = self.provider.model().document();
        let s = d.selection();
        let removed = s.anchor.abs_diff(s.caret) as usize;
        if d.text().len() - removed + text.len() > text_core::MAX_TEXT {
            return Err(Error::Bounds);
        }
        let mut projected = d.text().to_string();
        projected.replace_range(
            s.anchor.min(s.caret) as usize..s.anchor.max(s.caret) as usize,
            text,
        );
        text_core::validate_text(&projected).map_err(|_| Error::Text)?;
        Ok(())
    }
    pub fn enqueue(&mut self, event: Event) -> Result<()> {
        if self.pending.len() == 256 {
            let count = self.pending.len() as u64 + 1;
            self.pending.clear();
            self.reset("queue-overflow", count);
            self.resync = true;
            return Err(Error::Exhausted);
        }
        self.pending.push(event);
        Ok(())
    }
    pub fn reconcile(&mut self, tree: &Tree) -> Result<()> {
        if !self.capture.is_empty() && tree.node(&self.capture).is_err() {
            self.reset("capture-owner-removed", 0)
        }
        if self
            .capture_owner
            .as_ref()
            .is_some_and(|(id, rect, semantic, action)| {
                tree.node(id).is_ok_and(|n| {
                    n.rect != *rect
                        || n.semantic_id != *semantic
                        || n.action != *action
                        || !n.focusable
                })
            })
        {
            self.reset("capture-owner-moved-or-replaced", 0);
        }
        if !self.modal.is_empty() && tree.node(&self.modal).is_err() {
            self.dismiss_modal(tree)?;
        }
        let focused = &self.provider.model().session().focus;
        if !tree.node(focused).is_ok_and(|n| n.focusable) {
            if self.composition.is_some() {
                self.reset("focus-owner-removed", 0)
            }
            let fallback = if tree.node("editor").is_ok_and(|n| n.focusable) {
                "editor"
            } else {
                tree.preorder()
                    .into_iter()
                    .find(|n| n.focusable)
                    .map(|n| n.id.as_str())
                    .ok_or(Error::Input)?
            };
            self.invoke(Action::Focus(fallback.into()), Entry::Automation)?;
        }
        Ok(())
    }
    pub fn drain(&mut self, tree: &Tree) -> Result<()> {
        let pending = core::mem::take(&mut self.pending);
        let count = pending.len();
        for (at, e) in pending.into_iter().enumerate() {
            let resets = self.resets;
            if let Err(error) = self.event(tree, e) {
                self.reset("queue-rejected-event", (count - at) as u64);
                self.resync = true;
                return Err(error);
            }
            if self.resets != resets {
                self.dropped += (count - at - 1) as u64;
                self.resync = true;
                break;
            }
        }
        Ok(())
    }
    pub fn open_modal(&mut self, tree: &Tree, id: &str) -> Result<()> {
        if !self.modal.is_empty() {
            return Err(Error::Input);
        }
        tree.node(id)?;
        let first = tree
            .preorder()
            .into_iter()
            .find(|n| n.focusable && tree.descendant(&n.id, id))
            .ok_or(Error::Input)?
            .id
            .clone();
        let previous = self.provider.model().session().focus.clone();
        self.reset("modal-open", 0);
        self.invoke(Action::Focus(first), Entry::Shortcut)?;
        self.return_focus = previous;
        self.modal = id.into();
        Ok(())
    }
    pub fn dismiss_modal(&mut self, tree: &Tree) -> Result<()> {
        self.reset("modal-dismiss", 0);
        self.modal.clear();
        let focus = if tree.node(&self.return_focus).is_ok_and(|n| n.focusable) {
            self.return_focus.clone()
        } else {
            tree.preorder()
                .into_iter()
                .find(|n| n.focusable)
                .map(|n| n.id.clone())
                .ok_or(Error::Input)?
        };
        self.return_focus.clear();
        self.invoke(Action::Focus(focus), Entry::Shortcut)
    }
    /// Shape before dispatch. Visual arrow affinity stays view-owned, outside checkpoints.
    pub fn event_shaped(
        &mut self,
        tree: &Tree,
        event: Event,
        engine: &mut text_core::Engine<'_>,
        tokens: &crate::scene::Tokens,
        scale: u16,
    ) -> Result<()> {
        let visual_key = matches!(
            event.kind,
            EventKind::Key {
                usage: 79 | 80 | 81 | 82 | 74 | 77,
                down: true
            }
        ) && self.provider.model().session().focus == "editor";
        if let Ok(n) = tree.node("editor")
            && visual_key
        {
            let width = ((n.rect.0[2] - 2.0 * tokens.spacing).max(1.0) * scale as f32 / 100.0 + 0.5)
                .min(4096.0) as u16;
            self.shaped = Some(
                engine
                    .layout(
                        self.provider.model().document().text(),
                        text_core::Config {
                            mono: true,
                            size: tokens.editor_size,
                            scale,
                            width,
                            ligatures: true,
                        },
                    )
                    .map_err(|_| Error::Text)?,
            );
        }
        let result = self.event(tree, event);
        self.shaped = None;
        if result.is_err() {
            self.reset("selected-input-rejected", 0);
            self.resync = true;
        }
        result
    }
    pub fn activate(&mut self, tree: &Tree, target: &str, entry: Entry) -> Result<()> {
        let n = tree.node(target)?;
        let action = match n.action.as_str() {
            "launcher.open@1" => Action::LauncherOpen,
            "settings.effects@1" => {
                Action::Effects(!self.provider.model().preferences().reduced_effects)
            }
            "settings.motion@1" => {
                Action::Motion(!self.provider.model().preferences().reduced_motion)
            }
            _ => return Err(Error::Input),
        };
        self.invoke(action, entry)
    }
    pub fn event(&mut self, tree: &Tree, event: Event) -> Result<()> {
        // Releases and reset have no geometry dependency. Retire held state even
        // after removal/revision change; a late release must never activate anything.
        let release = matches!(
            event.kind,
            EventKind::Key { down: false, .. }
                | EventKind::Pointer { down: false, .. }
                | EventKind::Reset
        );
        // Reject stale pointer geometry before consuming the event; caller must re-hit-test.
        if event.revision != tree.revision && !release {
            return Err(Error::Revision);
        }
        if event.sequence > 4096 {
            return Err(Error::Exhausted);
        }
        if ((!self.resync
            && event.sequence != self.sequence.checked_add(1).ok_or(Error::Exhausted)?)
            || event.sequence <= self.sequence)
            || event.time_us < self.time_us
            || event.time_us > 60_000_000
        {
            return Err(Error::Input);
        }
        if !event.target.is_empty() && !release {
            tree.node(&event.target)?;
        }
        self.sequence = event.sequence;
        self.time_us = event.time_us;
        self.resync = false;
        if let EventKind::Key { usage, down: false } = event.kind {
            self.keys.retain(|k| *k != usage);
            return Ok(());
        }
        if release && event.revision != tree.revision {
            self.reset("release-across-revision", 0);
            return Ok(());
        }
        if let EventKind::Pointer {
            button,
            down: false,
            ..
        } = event.kind
        {
            if self
                .capture_owner
                .as_ref()
                .is_some_and(|(id, rect, semantic, action)| {
                    tree.node(id).map_or(true, |n| {
                        n.rect != *rect || n.semantic_id != *semantic || n.action != *action
                    })
                })
            {
                self.reset("release-owner-moved-or-replaced", 0);
                return Ok(());
            }
            if self.cancelled && self.buttons.is_empty() {
                return Ok(());
            }
            if tree.node(&event.target).is_err() {
                self.reset("release-owner-removed", 0);
                return Ok(());
            }
            if !(1..=8).contains(&button) {
                return Err(Error::Bounds);
            }
        }
        let target = event.target.as_str();
        if !self.modal.is_empty()
            && !target.is_empty()
            && !tree.descendant(target, &self.modal)
            && !release
        {
            return Err(Error::Input);
        }
        match event.kind {
            EventKind::Focus => {
                if !tree.node(target)?.focusable {
                    return Err(Error::Input);
                }
                self.invoke(Action::Focus(target.into()), Entry::Shortcut)?;
            }
            EventKind::Capture => {
                if !target.is_empty() && !tree.node(target)?.focusable {
                    return Err(Error::Input);
                }
                self.capture = target.into();
                self.capture_owner = if target.is_empty() {
                    None
                } else {
                    let n = tree.node(target)?;
                    Some((
                        n.id.clone(),
                        n.rect,
                        n.semantic_id.clone(),
                        n.action.clone(),
                    ))
                };
            }
            EventKind::Reset => self.reset("explicit-reset", 0),
            EventKind::Key { usage, down } => {
                if down {
                    if self.keys.len() == 256 {
                        return Err(Error::Exhausted);
                    }
                    if self.keys.contains(&usage) {
                        return Err(Error::Input);
                    }
                    self.keys.push(usage);
                    if usage == 41 {
                        if !self.modal.is_empty() {
                            self.dismiss_modal(tree)?;
                        } else {
                            self.reset("escape", 0);
                        }
                        return Ok(());
                    }
                    if self.composition.is_some()
                        && ![224, 225, 226, 227, 228, 229, 230, 231].contains(&usage)
                    {
                        return Ok(());
                    }
                    if self.provider.model().session().focus == "editor"
                        && (self.keys.contains(&224) || self.keys.contains(&228))
                        && [4, 28, 29].contains(&usage)
                    {
                        let action = match usage {
                            4 => Action::Select(Selection::new(
                                0,
                                self.provider.model().document().text().len() as u32,
                            )),
                            28 => Action::Redo,
                            _ => Action::Undo,
                        };
                        self.invoke(action, Entry::Shortcut)?;
                    } else if self.provider.model().session().focus == "editor"
                        && [42, 76, 79, 80, 81, 82, 74, 77].contains(&usage)
                    {
                        let d = self.provider.model().document();
                        let sel = d.selection();
                        let caret = sel.caret as usize;
                        let boundaries = text_core::boundaries(d.text());
                        let previous = boundaries
                            .iter()
                            .copied()
                            .filter(|b| *b < caret)
                            .next_back()
                            .unwrap_or(0);
                        let next = boundaries
                            .iter()
                            .copied()
                            .find(|b| *b > caret)
                            .unwrap_or(d.text().len());
                        if usage == 42 || usage == 76 {
                            if sel.anchor == sel.caret {
                                self.invoke(
                                    Action::Select(Selection::new(
                                        if usage == 42 {
                                            previous as u32
                                        } else {
                                            caret as u32
                                        },
                                        if usage == 42 {
                                            caret as u32
                                        } else {
                                            next as u32
                                        },
                                    )),
                                    Entry::Shortcut,
                                )?;
                            }
                            self.invoke(Action::Delete, Entry::Shortcut)?;
                        } else {
                            let visual = if let Some(layout) = &self.shaped {
                                Some(
                                    layout
                                        .visual_move(caret, self.visual_caret, usage)
                                        .map_err(|_| Error::Text)?,
                                )
                            } else {
                                None
                            };
                            let at = visual.map(|v| v.byte).unwrap_or(if usage == 80 {
                                previous
                            } else {
                                next
                            }) as u32;
                            self.invoke(
                                Action::Select(Selection::new(
                                    if self.keys.contains(&225) || self.keys.contains(&229) {
                                        sel.anchor
                                    } else {
                                        at
                                    },
                                    at,
                                )),
                                Entry::Shortcut,
                            )?;
                            self.visual_caret = visual;
                        }
                    } else if self.provider.model().session().focus == "search" && usage == 42 {
                        let query = self.provider.model().document().query();
                        let at = text_core::boundaries(query)
                            .into_iter()
                            .rev()
                            .nth(1)
                            .unwrap_or(0);
                        self.invoke(Action::Search(query[..at].to_string()), Entry::Shortcut)?;
                    } else if usage == 43 {
                        let order: Vec<_> = tree
                            .preorder()
                            .into_iter()
                            .filter(|n| n.focusable)
                            .filter(|n| {
                                self.modal.is_empty() || tree.descendant(&n.id, &self.modal)
                            })
                            .collect();
                        let at = order
                            .iter()
                            .position(|n| n.id == self.provider.model().session().focus)
                            .unwrap_or(0);
                        if order.is_empty() {
                            return Err(Error::Input);
                        }
                        let backwards = self.keys.contains(&225) || self.keys.contains(&229);
                        let next = &order
                            [(at + if backwards { order.len() - 1 } else { 1 }) % order.len()]
                        .id;
                        self.invoke(Action::Focus(next.clone()), Entry::Shortcut)?;
                    } else if usage == 44 || usage == 40 {
                        let focus = self.provider.model().session().focus.clone();
                        if matches!(tree.node(&focus)?.kind, Kind::Button | Kind::Toggle) {
                            self.activate(tree, &focus, Entry::Shortcut)?;
                        }
                    } else if usage == 81 || usage == 82 {
                        if target == "files-window"
                            && self.provider.model().session().focus == target
                        {
                            let old = self.provider.model().session().selected_file;
                            self.select_file(
                                tree,
                                target,
                                if usage == 81 {
                                    (old + 1).min(9999)
                                } else {
                                    old.saturating_sub(1)
                                },
                                Entry::Shortcut,
                            )?;
                        }
                    }
                } else {
                    let at = self
                        .keys
                        .iter()
                        .position(|n| *n == usage)
                        .ok_or(Error::Input)?;
                    self.keys.remove(at);
                }
            }
            EventKind::Text(text) => {
                if self.composition.is_some()
                    || target != self.provider.model().session().focus
                    || self
                        .keys
                        .iter()
                        .any(|k| [224, 226, 227, 228, 230, 231].contains(k))
                {
                    return Err(Error::Input);
                }
                let action = match tree.node(target)?.kind {
                    Kind::Editor => Action::Insert(if cfg!(feature = "behavior-v2") {
                        text.to_uppercase()
                    } else {
                        text
                    }),
                    Kind::Field => Action::Search(alloc::format!(
                        "{}{}",
                        self.provider.model().document().query(),
                        text
                    )),
                    _ => return Err(Error::Input),
                };
                self.invoke(action, Entry::Shortcut)?;
            }
            EventKind::Scroll(delta) => {
                if !delta.is_finite() || tree.node(target)?.kind != Kind::List {
                    return Err(Error::Bounds);
                }
                let row = (self.provider.model().session().file_scroll as i64
                    + (delta / 24.0) as i64)
                    .clamp(0, 9986) as u32;
                self.invoke(Action::FileScroll(row), Entry::Button)?;
            }
            EventKind::Move { x, y } => {
                if !x.is_finite()
                    || !y.is_finite()
                    || self.capture != target
                    || self.buttons.is_empty()
                {
                    return Err(Error::Input);
                }
            }
            EventKind::Pointer { x, y, button, down } => {
                if !x.is_finite() || !y.is_finite() || button == 0 || button > 8 {
                    return Err(Error::Bounds);
                }
                let hit = if self.capture.is_empty() {
                    tree.hit(x, y).ok_or(Error::Input)?
                } else {
                    tree.node(&self.capture)?
                };
                if hit.id != target {
                    return Err(Error::Input);
                }
                if down {
                    if self.buttons.contains(&button) {
                        return Err(Error::Input);
                    }
                    self.buttons.push(button);
                    self.cancelled = false;
                    if hit.focusable {
                        self.capture = hit.id.clone();
                        self.capture_owner = Some((
                            hit.id.clone(),
                            hit.rect,
                            hit.semantic_id.clone(),
                            hit.action.clone(),
                        ));
                        self.invoke(Action::Focus(hit.id.clone()), Entry::Button)?;
                    }
                    if hit.kind == Kind::List {
                        let row = ((y - hit.rect.0[1]) / 24.0) as u32
                            + self.provider.model().session().file_scroll;
                        self.invoke(Action::FileSelect(row.min(9999)), Entry::Button)?;
                    }
                } else {
                    let at = self
                        .buttons
                        .iter()
                        .position(|b| *b == button)
                        .ok_or(Error::Input)?;
                    self.buttons.remove(at);
                    if matches!(hit.kind, Kind::Button | Kind::Toggle) && hit.rect.contains(x, y) {
                        self.activate(tree, target, Entry::Button)?;
                    }
                    if self.buttons.is_empty() {
                        self.capture.clear();
                        self.capture_owner = None;
                    }
                }
            }
            EventKind::Composition { phase, text } => {
                if target != "editor" || self.provider.model().session().focus != target {
                    return Err(Error::Input);
                }
                match phase.as_str() {
                    "start" => {
                        if self.composition.is_some() {
                            return Err(Error::Input);
                        }
                        let mut c = Composition::begin(&self.provider);
                        c.update(&text, text.len()).map_err(|_| Error::Text)?;
                        self.composition = Some(c)
                    }
                    "update" => self
                        .composition
                        .as_mut()
                        .ok_or(Error::Input)?
                        .update(&text, text.len())
                        .map_err(|_| Error::Text)?,
                    "cancel" => {
                        self.composition.take().ok_or(Error::Input)?.cancel();
                    }
                    "commit" => {
                        self.admit_insert(&text)?;
                        self.composition
                            .as_mut()
                            .ok_or(Error::Input)?
                            .update(&text, text.len())
                            .map_err(|_| Error::Text)?;
                        let request = self.request(Action::Insert(text))?;
                        self.composition
                            .as_ref()
                            .unwrap()
                            .commit(&mut self.provider, request, self.time_us, false)
                            .map_err(|_| Error::Text)?
                            .status
                            .map_err(|_| Error::Model)?;
                        self.composition = None;
                        self.last_action = "edit.insert@1".into();
                        if self.actions.len() == 256 {
                            self.actions.remove(0);
                        }
                        self.actions.push(self.last_action.clone());
                    }
                    _ => return Err(Error::Input),
                }
                self.dirty.paint = true;
                self.dirty.semantics = true;
                self.dirty.layout = true;
            }
        }
        Ok(())
    }
    /// Hit a grapheme stop with the same shaped geometry used for painting.
    pub fn select_at(&mut self, layout: &text_core::Layout, line: usize, x: f32) -> Result<()> {
        let caret = layout.visual_hit(line, x).map_err(|_| Error::Text)?;
        let byte = caret.byte as u32;
        let anchor = if self.keys.contains(&225) || self.keys.contains(&229) {
            self.provider.model().document().selection().anchor
        } else {
            byte
        };
        self.invoke(Action::Select(Selection::new(anchor, byte)), Entry::Button)?;
        self.visual_caret = Some(caret);
        Ok(())
    }
    pub fn drag_select_at(
        &mut self,
        layout: &text_core::Layout,
        line: usize,
        x: f32,
    ) -> Result<()> {
        if self.capture != "editor" || self.buttons.is_empty() {
            return Err(Error::Input);
        }
        let caret = layout.visual_hit(line, x).map_err(|_| Error::Text)?;
        let anchor = self.provider.model().document().selection().anchor;
        self.invoke(
            Action::Select(Selection::new(anchor, caret.byte as u32)),
            Entry::Button,
        )?;
        self.visual_caret = Some(caret);
        Ok(())
    }
}
