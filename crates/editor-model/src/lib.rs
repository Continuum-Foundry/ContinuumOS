//! Hosted logical state and action model. IDs model authority; they are not capabilities.
//! `alloc` requires an allocator from a future guest runtime. No guest linkage is claimed.
#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::{collections::VecDeque, string::String, vec::Vec};
use unicode_segmentation::UnicodeSegmentation;

mod snapshot;
pub use snapshot::{MAX_SNAPSHOT, STATE_VERSION};
pub const ACTION_VERSION: u16 = 1;
pub const MAX_TEXT: usize = 65536;
pub const MAX_QUERY: usize = 4096;
pub const HISTORY: usize = 32;
pub const RESULTS: usize = 32;
pub const AUTHORITIES: usize = 16;
pub const UNICODE_VERSION: (u64, u64, u64) = unicode_segmentation::UNICODE_VERSION;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Invalid,
    Bounds,
    Unsupported,
    Conflict,
    Denied,
    Expired,
    Cancelled,
    Deadline,
    DuplicateMismatch,
    RetryExpired,
    Exhausted,
}
pub type Result<T> = core::result::Result<T, Error>;

/// Direction is preserved: anchor and caret need not be in ascending order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Selection {
    pub anchor: u32,
    pub caret: u32,
}
impl Selection {
    pub fn new(anchor: u32, caret: u32) -> Self {
        Self { anchor, caret }
    }
    fn range(self) -> core::ops::Range<usize> {
        self.anchor.min(self.caret) as usize..self.anchor.max(self.caret) as usize
    }
}
fn boundary(text: &str, at: usize) -> bool {
    at == text.len() || text.grapheme_indices(true).any(|(i, _)| i == at)
}
fn selection_valid(text: &str, selection: Selection) -> bool {
    boundary(text, selection.anchor as usize) && boundary(text, selection.caret as usize)
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Frame {
    text: String,
    selection: Selection,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document {
    id: u64,
    revision: u64,
    frame: Frame,
    saved: String,
    undo: VecDeque<Frame>,
    redo: VecDeque<Frame>,
    scroll: u32,
    query: String,
}
impl Document {
    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn text(&self) -> &str {
        &self.frame.text
    }
    pub fn selection(&self) -> Selection {
        self.frame.selection
    }
    pub fn dirty(&self) -> bool {
        self.frame.text != self.saved
    }
    pub fn scroll(&self) -> u32 {
        self.scroll
    }
    pub fn query(&self) -> &str {
        &self.query
    }
    pub fn undo_len(&self) -> usize {
        self.undo.len()
    }
    pub fn redo_len(&self) -> usize {
        self.redo.len()
    }
    /// Exact, case-sensitive, non-overlapping UTF-8 matches; empty query has none.
    pub fn matches(&self) -> Vec<(u32, u32)> {
        if self.query.is_empty() {
            return Vec::new();
        }
        self.text()
            .match_indices(&self.query)
            .map(|(i, s)| (i as u32, (i + s.len()) as u32))
            .collect()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
    HighContrast,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Preferences {
    pub theme: Theme,
    pub text_size: u16,
    pub reduced_motion: bool,
    pub reduced_effects: bool,
}
/// Session state is deliberately outside the document. Transient gestures are view-owned.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Session {
    pub window: [i32; 4],
    pub focus: String,
    pub file_scroll: u32,
    pub selected_file: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Model {
    revision: u64,
    document: Document,
    preferences: Preferences,
    session: Session,
}
impl Model {
    pub fn new(
        id: u64,
        revision: u64,
        text: &str,
        saved: &str,
        selection: Selection,
    ) -> Result<Self> {
        if text.len() > MAX_TEXT || saved.len() > MAX_TEXT {
            return Err(Error::Bounds);
        }
        let model = Self {
            revision: 1,
            document: Document {
                id,
                revision,
                frame: Frame {
                    text: text.into(),
                    selection,
                },
                saved: saved.into(),
                undo: VecDeque::new(),
                redo: VecDeque::new(),
                scroll: 0,
                query: String::new(),
            },
            preferences: Preferences {
                theme: Theme::Dark,
                text_size: 14,
                reduced_motion: false,
                reduced_effects: false,
            },
            session: Session {
                window: [8, 8, 420, 328],
                focus: "editor".into(),
                file_scroll: 0,
                selected_file: 0,
            },
        };
        model.validate()?;
        Ok(model)
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn document(&self) -> &Document {
        &self.document
    }
    pub fn preferences(&self) -> &Preferences {
        &self.preferences
    }
    pub fn session(&self) -> &Session {
        &self.session
    }
    fn validate(&self) -> Result<()> {
        let d = &self.document;
        if self.revision == 0 || d.revision == 0 || d.id == 0 {
            return Err(Error::Invalid);
        }
        if d.saved.len() > MAX_TEXT
            || d.query.len() > MAX_QUERY
            || d.scroll > 1_000_000
            || d.undo.len() > HISTORY
            || d.redo.len() > HISTORY
        {
            return Err(Error::Bounds);
        }
        for f in core::iter::once(&d.frame)
            .chain(d.undo.iter())
            .chain(d.redo.iter())
        {
            if f.text.len() > MAX_TEXT {
                return Err(Error::Bounds);
            }
            if !selection_valid(&f.text, f.selection) {
                return Err(Error::Invalid);
            }
        }
        if !(8..=72).contains(&self.preferences.text_size)
            || self.session.file_scroll > 9999
            || self.session.selected_file > 9999
            || !valid_focus(&self.session.focus)
            || self
                .session
                .window
                .iter()
                .any(|n| !(-16384..=16384).contains(n))
            || self.session.window[2] <= 0
            || self.session.window[3] <= 0
        {
            return Err(Error::Invalid);
        }
        Ok(())
    }
    fn apply(&mut self, action: &Action) -> Result<bool> {
        let old = self.clone(); // Bounded transactional candidate; no partially applied edit escapes.
        let d = &mut self.document;
        match action {
            Action::Insert(_) | Action::Delete => {
                let text = if let Action::Insert(text) = action {
                    text.as_str()
                } else {
                    ""
                };
                if text.len() > MAX_QUERY {
                    return Err(Error::Bounds);
                }
                let range = d.frame.selection.range();
                let len = d.text().len() - range.len() + text.len();
                if len > MAX_TEXT {
                    return Err(Error::Bounds);
                }
                if range.is_empty() && text.is_empty() {
                    return Ok(false);
                }
                let mut next = d.frame.clone();
                next.text.replace_range(range.clone(), text);
                let end = range.start + text.len();
                // Insertion can join a following combining/ZWJ sequence. Snap right.
                let caret = next
                    .text
                    .grapheme_indices(true)
                    .map(|(i, _)| i)
                    .chain(core::iter::once(next.text.len()))
                    .find(|i| *i >= end)
                    .ok_or(Error::Invalid)? as u32;
                next.selection = Selection::new(caret, caret);
                push(&mut d.undo, d.frame.clone());
                d.redo.clear();
                d.frame = next;
                d.revision = d.revision.checked_add(1).ok_or(Error::Exhausted)?;
            }
            Action::Select(selection) => {
                if !selection_valid(d.text(), *selection) {
                    return Err(Error::Invalid);
                }
                d.frame.selection = *selection;
            }
            Action::Undo | Action::Redo => {
                let (from, to) = if *action == Action::Undo {
                    (&mut d.undo, &mut d.redo)
                } else {
                    (&mut d.redo, &mut d.undo)
                };
                if let Some(frame) = from.pop_back() {
                    push(to, core::mem::replace(&mut d.frame, frame));
                    d.revision = d.revision.checked_add(1).ok_or(Error::Exhausted)?;
                }
            }
            Action::Search(query) => {
                if query.len() > MAX_QUERY {
                    return Err(Error::Bounds);
                }
                d.query = query.clone();
            }
            Action::Scroll(value) => d.scroll = *value,
            Action::Theme(value) => self.preferences.theme = *value,
            Action::TextSize(value) => self.preferences.text_size = *value,
            Action::Motion(value) => self.preferences.reduced_motion = *value,
            Action::Effects(value) => self.preferences.reduced_effects = *value,
            Action::FileSelect(value) => self.session.selected_file = *value,
            Action::FileScroll(value) => self.session.file_scroll = *value,
            Action::LauncherOpen => self.session.focus = "editor".into(),
            Action::Focus(value) => self.session.focus = value.clone(),
            Action::Window(value) => self.session.window = *value,
            Action::Unknown(_) => return Err(Error::Unsupported),
        }
        self.validate()?;
        if *self == old {
            return Ok(false);
        }
        self.revision = self.revision.checked_add(1).ok_or(Error::Exhausted)?;
        Ok(true)
    }
}
fn valid_focus(s: &str) -> bool {
    matches!(
        s,
        "editor"
            | "search"
            | "files-window"
            | "launch-editor"
            | "reduced-effects"
            | "reduced-motion"
    )
}
fn push(history: &mut VecDeque<Frame>, frame: Frame) {
    if history.len() == HISTORY {
        history.pop_front();
    }
    history.push_back(frame);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Insert(String),
    Delete,
    Select(Selection),
    Undo,
    Redo,
    Search(String),
    Scroll(u32),
    Theme(Theme),
    TextSize(u16),
    Motion(bool),
    Effects(bool),
    FileSelect(u32),
    FileScroll(u32),
    LauncherOpen,
    Focus(String),
    Window([i32; 4]),
    Unknown(String),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    Document,
    Preference,
    Session,
}
impl Action {
    pub fn name(&self) -> &str {
        match self {
            Self::Insert(_) => "edit.insert",
            Self::Delete => "edit.delete",
            Self::Select(_) => "edit.select",
            Self::Undo => "edit.undo",
            Self::Redo => "edit.redo",
            Self::Search(_) => "search.set",
            Self::Scroll(_) => "document.scroll",
            Self::Theme(_) => "settings.theme",
            Self::TextSize(_) => "settings.text-size",
            Self::Motion(_) => "settings.motion",
            Self::Effects(_) => "settings.effects",
            Self::FileSelect(_) => "files.select",
            Self::FileScroll(_) => "files.scroll",
            Self::LauncherOpen => "launcher.open",
            Self::Focus(_) => "session.focus",
            Self::Window(_) => "session.window",
            Self::Unknown(s) => s,
        }
    }
    pub fn effect(&self) -> Effect {
        match self {
            Self::Theme(_) | Self::TextSize(_) | Self::Motion(_) | Self::Effects(_) => {
                Effect::Preference
            }
            Self::FileSelect(_)
            | Self::FileScroll(_)
            | Self::LauncherOpen
            | Self::Focus(_)
            | Self::Window(_) => Effect::Session,
            _ => Effect::Document,
        }
    }
    fn bounded(&self) -> bool {
        match self {
            Self::Insert(s) | Self::Search(s) => s.len() <= MAX_QUERY,
            Self::Unknown(s) | Self::Focus(s) => s.len() <= 64,
            _ => true,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rights {
    pub edit: bool,
    pub preferences: bool,
    pub session: bool,
}
impl Rights {
    pub const ALL: Self = Self {
        edit: true,
        preferences: true,
        session: true,
    };
    fn allows(self, effect: Effect) -> bool {
        match effect {
            Effect::Document => self.edit,
            Effect::Preference => self.preferences,
            Effect::Session => self.session,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub version: u16,
    pub caller: u64,
    pub id: u64,
    pub epoch: u64,
    pub document: u64,
    pub expected_document_revision: u64,
    pub expected_model_revision: u64,
    pub deadline: u64,
    pub context: Option<u64>,
    pub action: Action,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub status: Result<bool>,
    pub document_revision: u64,
    pub model_revision: u64,
}
/// Metadata is issued by the host owner, never accepted from the invocation payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Context {
    pub id: u64,
    pub caller: u64,
    pub provider_epoch: u64,
    pub document: u64,
    pub source_revision: u64,
    pub provenance: String,
    pub purpose: Effect,
    pub expires: u64,
    pub size: u32,
    pub selection: Selection,
    pub permitted_action: String,
}
#[derive(Clone)]
struct Authority {
    caller: u64,
    rights: Rights,
    high_water: u64,
}
#[derive(Clone)]
struct Record {
    request: Request,
    outcome: Outcome,
}
pub struct Provider {
    model: Model,
    epoch: u64,
    authorities: Vec<Authority>,
    contexts: Vec<Context>,
    results: VecDeque<Record>,
}
impl Provider {
    pub fn new(model: Model, epoch: u64) -> Result<Self> {
        model.validate()?;
        if epoch == 0 {
            return Err(Error::Invalid);
        }
        Ok(Self {
            model,
            epoch,
            authorities: Vec::new(),
            contexts: Vec::new(),
            results: VecDeque::new(),
        })
    }
    pub fn model(&self) -> &Model {
        &self.model
    }
    pub fn epoch(&self) -> u64 {
        self.epoch
    }
    /// Owner-only host control plane. UI adapters receive no mutable provider reference.
    pub fn grant(&mut self, caller: u64, rights: Rights) -> Result<()> {
        if caller == 0 {
            return Err(Error::Invalid);
        }
        if let Some(a) = self.authorities.iter_mut().find(|a| a.caller == caller) {
            a.rights = rights;
            return Ok(());
        }
        if self.authorities.len() == AUTHORITIES {
            return Err(Error::Bounds);
        }
        self.authorities.push(Authority {
            caller,
            rights,
            high_water: 0,
        });
        Ok(())
    }
    pub fn issue_context(&mut self, context: Context) -> Result<()> {
        if context.id == 0
            || context.provider_epoch != self.epoch
            || context.document != self.model.document.id
            || context.source_revision != self.model.document.revision
            || context.provenance.is_empty()
            || context.provenance.len() > 64
            || context.size > MAX_QUERY as u32
            || !selection_valid(self.model.document.text(), context.selection)
            || context.selection.range().len() != context.size as usize
            || context.permitted_action.is_empty()
            || context.permitted_action.len() > 64
            || !self
                .authorities
                .iter()
                .any(|a| a.caller == context.caller && a.rights.allows(context.purpose))
        {
            return Err(Error::Invalid);
        }
        if self.contexts.len() == AUTHORITIES || self.contexts.iter().any(|c| c.id == context.id) {
            return Err(Error::Bounds);
        }
        self.contexts.push(context);
        Ok(())
    }
    pub fn revoke_context(&mut self, id: u64) {
        self.contexts.retain(|c| c.id != id);
    }
    pub fn resolve_context(&self, caller: u64, id: u64, now: u64) -> Result<&str> {
        let c = self
            .contexts
            .iter()
            .find(|c| c.id == id && c.caller == caller)
            .ok_or(Error::Denied)?;
        if !self
            .authorities
            .iter()
            .any(|a| a.caller == caller && a.rights.allows(c.purpose))
        {
            return Err(Error::Denied);
        }
        if now >= c.expires {
            return Err(Error::Expired);
        }
        if c.source_revision != self.model.document.revision {
            return Err(Error::Conflict);
        }
        Ok(&self.model.document.text()[c.selection.range()])
    }
    pub fn outcome(&self, status: Result<bool>) -> Outcome {
        Outcome {
            status,
            document_revision: self.model.document.revision,
            model_revision: self.model.revision,
        }
    }
    /// Host supplies monotonic time and cancellation. Exact retries report original commits.
    pub fn execute(&mut self, request: Request, now: u64, cancelled: bool) -> Outcome {
        let reject = |error| self.outcome(Err(error));
        if !request.action.bounded() {
            return reject(Error::Bounds);
        }
        if request.version != ACTION_VERSION || matches!(request.action, Action::Unknown(_)) {
            return reject(Error::Unsupported);
        }
        let Some(index) = self
            .authorities
            .iter()
            .position(|a| a.caller == request.caller)
        else {
            return reject(Error::Denied);
        };
        if !self.authorities[index]
            .rights
            .allows(request.action.effect())
        {
            return reject(Error::Denied);
        }
        if request.epoch != self.epoch || request.document != self.model.document.id {
            return reject(Error::Conflict);
        }
        if request.id == 0 {
            return reject(Error::Invalid);
        }
        if let Some(record) = self
            .results
            .iter()
            .find(|r| r.request.caller == request.caller && r.request.id == request.id)
        {
            return if record.request == request {
                record.outcome
            } else {
                reject(Error::DuplicateMismatch)
            };
        }
        if request.id <= self.authorities[index].high_water {
            return reject(Error::RetryExpired);
        }
        let result = (|| {
            if cancelled {
                return Err(Error::Cancelled);
            }
            if now >= request.deadline {
                return Err(Error::Deadline);
            }
            if request.expected_document_revision != self.model.document.revision
                || request.expected_model_revision != self.model.revision
            {
                return Err(Error::Conflict);
            }
            if let Some(id) = request.context {
                let c = self
                    .contexts
                    .iter()
                    .find(|c| c.id == id)
                    .ok_or(Error::Denied)?;
                if c.caller != request.caller
                    || c.provider_epoch != self.epoch
                    || c.document != request.document
                    || c.purpose != request.action.effect()
                    || c.permitted_action != request.action.name()
                {
                    return Err(Error::Denied);
                }
                if now >= c.expires {
                    return Err(Error::Expired);
                }
                if c.source_revision != self.model.document.revision {
                    return Err(Error::Conflict);
                }
            }
            let mut candidate = self.model.clone();
            let changed = candidate.apply(&request.action)?;
            self.model = candidate;
            Ok(changed)
        })();
        let outcome = self.outcome(result);
        self.authorities[index].high_water = request.id;
        if self.results.len() == RESULTS {
            self.results.pop_front();
        }
        self.results.push_back(Record { request, outcome });
        outcome
    }
    /// Trusted owner restores logical data only; authority/deduplication stay session-owned.
    pub fn restore(&mut self, bytes: &[u8]) -> Result<()> {
        let candidate = Model::decode(bytes)?;
        if candidate.document.id != self.model.document.id
            || candidate.revision < self.model.revision
            || candidate.document.revision < self.model.document.revision
            || (candidate.revision == self.model.revision && candidate != self.model)
        {
            return Err(Error::Conflict);
        }
        self.model = candidate;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entry {
    Button,
    Shortcut,
    Semantic,
    Automation,
}
/// A view borrows state; dropping and rebuilding this adapter cannot own or reset a document.
pub struct Adapter<'a> {
    provider: &'a mut Provider,
    entry: Entry,
}
impl<'a> Adapter<'a> {
    pub fn new(provider: &'a mut Provider, entry: Entry) -> Self {
        Self { provider, entry }
    }
    pub fn model(&self) -> &Model {
        self.provider.model()
    }
    pub fn invoke(&mut self, request: Request, now: u64, cancelled: bool) -> Outcome {
        let _origin = self.entry; // Diagnostic origin never changes authorization or semantics.
        self.provider.execute(request, now, cancelled)
    }
}

#[cfg(test)]
mod tests;
