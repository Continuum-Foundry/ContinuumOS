//! Hosted semantic consumer contract. Independent of raster nodes and paint order.
use crate::{
    scene::{Kind, Tree, realize},
    *,
};
use alloc::{format, string::ToString};
use editor_model::{Action, Entry};

#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub id: String,
    pub parent: Option<String>,
    pub role: String,
    pub name: String,
    pub value: String,
    pub actions: Vec<String>,
    pub bounds: Rect,
    pub focused: bool,
    pub checked: bool,
    pub selected: bool,
    pub position: Option<u32>,
    pub count: u32,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub scene_revision: u64,
    pub model_revision: u64,
    pub items: Vec<Item>,
}
#[derive(Clone, Debug, PartialEq)]
pub enum Notice {
    Added(String),
    Removed(String),
    Focus(String),
    Value(String),
    State(String),
    Bounds(String),
}
impl Snapshot {
    pub fn build(tree: &Tree, model: &editor_model::Model) -> Result<Self> {
        tree.validate()?;
        let mut items = Vec::new();
        let logical = tree.semantics(model);
        for n in tree.preorder() {
            let sem = logical
                .iter()
                .find(|s| s.widget == n.id)
                .ok_or(Error::Invalid)?;
            let mut actions = Vec::new();
            if n.focusable {
                actions.push("focus@1".into());
            }
            if !n.action.is_empty() {
                actions.push(n.action.clone());
            }
            if n.kind == Kind::List {
                actions.push("files.navigate@1".into());
            }
            let parent = tree
                .nodes
                .iter()
                .find(|p| p.children.contains(&n.id))
                .map(|p| p.semantic_id.clone());
            items.push(Item {
                id: n.semantic_id.clone(),
                parent,
                role: n.role.clone(),
                name: n.name.clone(),
                value: sem.value.clone(),
                actions,
                bounds: n.rect,
                focused: sem.focused,
                checked: sem.checked,
                selected: false,
                position: None,
                count: sem.logical_count,
            });
            if n.kind == Kind::List {
                for row in realize(model.session().file_scroll, 10000, n.rect.0[3], 24.0)? {
                    if items.len() == 1088 {
                        return Err(Error::Exhausted);
                    }
                    let state = crate::paint::lcg_at(20261020, row + 1);
                    items.push(Item {
                        id: format!("virtual:{}:{row}", n.semantic_id),
                        parent: Some(n.semantic_id.clone()),
                        role: "listitem".into(),
                        name: format!("File {row:05} — {state:08x}.txt"),
                        value: row.to_string(),
                        actions: alloc::vec!["files.select@1".into()],
                        bounds: Rect([
                            n.rect.0[0],
                            n.rect.0[1] + (row - model.session().file_scroll) as f32 * 24.0,
                            n.rect.0[2],
                            24.0,
                        ])
                        .intersect(n.rect),
                        focused: false,
                        checked: false,
                        selected: row == model.session().selected_file,
                        position: Some(row + 1),
                        count: 10000,
                    });
                }
            }
        }
        if items.len() > 1088 {
            return Err(Error::Exhausted);
        }
        Ok(Self {
            scene_revision: tree.revision,
            model_revision: model.revision(),
            items,
        })
    }
    pub fn diff(&self, old: &Self) -> Vec<Notice> {
        let mut out = Vec::new();
        for n in &self.items {
            match old.items.iter().find(|o| o.id == n.id) {
                None => out.push(Notice::Added(n.id.clone())),
                Some(o) => {
                    if n.focused != o.focused {
                        out.push(Notice::Focus(n.id.clone()));
                    }
                    if n.value != o.value {
                        out.push(Notice::Value(n.id.clone()));
                    }
                    if n.checked != o.checked
                        || n.selected != o.selected
                        || n.actions != o.actions
                        || n.name != o.name
                        || n.parent != o.parent
                    {
                        out.push(Notice::State(n.id.clone()));
                    }
                    if n.bounds != o.bounds {
                        out.push(Notice::Bounds(n.id.clone()));
                    }
                }
            }
        }
        for o in &old.items {
            if !self.items.iter().any(|n| n.id == o.id) {
                out.push(Notice::Removed(o.id.clone()));
            }
        }
        out
    }
}
#[derive(Clone, Debug)]
pub enum Command {
    Focus,
    Activate,
    SetText(String),
    SelectRow(u32),
    Navigate(i32),
}
impl crate::input::Controller {
    pub fn semantic(
        &mut self,
        tree: &Tree,
        revision: u64,
        id: &str,
        command: Command,
    ) -> Result<()> {
        if revision != tree.revision {
            return Err(Error::Revision);
        }
        if let Some(rest) = id.strip_prefix("virtual:") {
            let (parent, index) = rest.rsplit_once(':').ok_or(Error::Input)?;
            let row = index.parse::<u32>().map_err(|_| Error::Input)?;
            if index != row.to_string() {
                return Err(Error::Input);
            }
            let n = tree
                .nodes
                .iter()
                .find(|n| n.semantic_id == parent && n.kind == Kind::List)
                .ok_or(Error::Input)?;
            if !self.modal.is_empty() && !tree.descendant(&n.id, &self.modal) {
                return Err(Error::Input);
            }
            if !realize(
                self.provider.model().session().file_scroll,
                10000,
                n.rect.0[3],
                24.0,
            )?
            .contains(&row)
            {
                return Err(Error::Input);
            }
            if !matches!(command, Command::Activate) {
                return Err(Error::Input);
            }
            return self.select_file(tree, &n.id, row, Entry::Semantic);
        }
        let n = tree
            .nodes
            .iter()
            .find(|n| n.semantic_id == id)
            .ok_or(Error::Input)?;
        if !self.modal.is_empty() && !tree.descendant(&n.id, &self.modal) {
            return Err(Error::Input);
        }
        match command {
            Command::Focus if n.focusable => {
                self.invoke(Action::Focus(n.id.clone()), Entry::Semantic)
            }
            Command::Activate => self.activate(tree, &n.id, Entry::Semantic),
            Command::SetText(text) => match n.kind {
                Kind::Editor if n.action == "edit.insert@1" => {
                    self.invoke(Action::Insert(text), Entry::Semantic)
                }
                Kind::Field if n.action == "search.set@1" => {
                    self.invoke(Action::Search(text), Entry::Semantic)
                }
                _ => Err(Error::Input),
            },
            Command::SelectRow(row) if n.kind == Kind::List && row < 10000 => {
                self.select_file(tree, &n.id, row, Entry::Semantic)
            }
            Command::Navigate(delta) if n.kind == Kind::List => {
                let row = (self.provider.model().session().selected_file as i64 + delta as i64)
                    .clamp(0, 9999) as u32;
                self.select_file(tree, &n.id, row, Entry::Semantic)
            }
            _ => Err(Error::Input),
        }
    }
    pub fn select_file(&mut self, tree: &Tree, id: &str, row: u32, entry: Entry) -> Result<()> {
        if row >= 10000 {
            return Err(Error::Bounds);
        }
        let n = tree.node(id)?;
        if n.kind != Kind::List {
            return Err(Error::Input);
        }
        let visible = (n.rect.0[3] / 24.0) as u32;
        if visible == 0 {
            return Err(Error::Bounds);
        }
        self.invoke(Action::FileSelect(row), entry)?;
        let first = self.provider.model().session().file_scroll;
        if row < first {
            self.invoke(Action::FileScroll(row), entry)?;
        } else if row >= first + visible {
            self.invoke(Action::FileScroll(row + 1 - visible), entry)?;
        }
        Ok(())
    }
}
