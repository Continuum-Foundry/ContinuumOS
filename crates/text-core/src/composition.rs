use super::*;
use editor_model::{Action, Outcome, Provider, Request, Selection};

/// Transient preedit; the same live object must survive a resource-only view reload.
/// Native IME/provider replacement is outside this experiment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Composition {
    document: u64,
    revision: u64,
    selection: Selection,
    preedit: String,
    caret: usize,
}
impl Composition {
    pub fn begin(provider: &Provider) -> Self {
        let doc = provider.model().document();
        Self {
            document: doc.id(),
            revision: doc.revision(),
            selection: doc.selection(),
            preedit: String::new(),
            caret: 0,
        }
    }
    pub fn update(&mut self, text: &str, caret: usize) -> Result<()> {
        if text.len() > 1024 {
            return Err(Error::Bounds);
        }
        if !is_boundary(text, caret) {
            return Err(Error::InvalidSelection);
        }
        let mut next = String::new();
        next.try_reserve(text.len())
            .map_err(|_| Error::Allocation)?;
        next.push_str(text);
        self.preedit = next;
        self.caret = caret;
        Ok(())
    }
    pub fn preedit(&self) -> &str {
        &self.preedit
    }
    pub fn caret(&self) -> usize {
        self.caret
    }
    /// Cancellation consumes only preedit and leaves the document/history untouched.
    pub fn cancel(self) {}
    /// Failed commit retains preedit. Request authority still goes through P2-03's executor.
    pub fn commit(
        &self,
        provider: &mut Provider,
        mut request: Request,
        now: u64,
        cancelled: bool,
    ) -> Result<Outcome> {
        let doc = provider.model().document();
        if doc.id() != self.document
            || doc.revision() != self.revision
            || doc.selection() != self.selection
        {
            return Err(Error::Conflict);
        }
        request.action = Action::Insert(self.preedit.clone());
        Ok(provider.execute(request, now, cancelled))
    }
}
