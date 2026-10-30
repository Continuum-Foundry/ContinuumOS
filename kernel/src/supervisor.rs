//! Bounded supervision policy. No pointers, allocator, or privileged entry state.
//! A subtree receives prepaid lifetime credits. Credits are never refunded within
//! a bootstrap epoch, including after join: retained IPC references stay charged.
use crate::user_image::Context;

pub const SLOTS: usize = 16;
pub const EPOCH_UNITS: u64 = 8; // control owner + at most seven child admissions
pub const MEMORY: u64 = 1024 * 1024;
pub const HANDLES: usize = 12;
pub const MAPPINGS: usize = 16;
pub const OBJECT_CREATES: u64 = 10; // plus two bootstrap objects
pub const TIMERS: u64 = 4;
pub const MESSAGES: u64 = 4;
pub const MESSAGE_BYTES: u64 = 8192;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Stale,
    Denied,
    Quota,
    State,
    WouldBlock,
    Backoff,
    Quarantined,
}
pub type Result<T> = core::result::Result<T, Error>;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Prepared,
    Running,
    Stopping,
    Reaped,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Exit {
    pub code: u64,
    pub vector: u64,
}
#[derive(Clone, Copy)]
struct Node {
    token: u64,
    parent: u64,
    context: Context,
    state: State,
    exit: Option<Exit>,
    remaining: u64,
    log_epoch: u64,
    logs: u64,
}
pub struct Tree {
    nodes: [Option<Node>; SLOTS],
    next: u64,
    root: u64,
    closed: bool,
    restart_count: u64,
    restart_after: u64,
}
impl Tree {
    pub const EMPTY: Self = Self {
        nodes: [None; SLOTS],
        next: 1,
        root: 0,
        closed: true,
        restart_count: 0,
        restart_after: 0,
    };
    fn index(&self, token: u64) -> Result<usize> {
        self.nodes
            .iter()
            .position(|n| n.is_some_and(|n| n.token == token))
            .ok_or(Error::Stale)
    }
    fn node(&self, token: u64) -> Result<Node> {
        Ok(self.nodes[self.index(token)?].unwrap())
    }
    pub fn context(&self, token: u64) -> Result<Context> {
        Ok(self.node(token)?.context)
    }
    pub fn parent(&self, token: u64) -> Result<u64> {
        Ok(self.node(token)?.parent)
    }
    pub fn state(&self, token: u64) -> Result<State> {
        Ok(self.node(token)?.state)
    }
    pub fn root(&self) -> u64 {
        self.root
    }
    pub fn closed(&self) -> bool {
        self.closed
    }
    pub fn live(&self) -> usize {
        self.nodes.iter().flatten().count()
    }
    pub fn tokens(&self) -> [u64; SLOTS] {
        core::array::from_fn(|i| self.nodes[i].map_or(0, |n| n.token))
    }
    pub fn first_child(&self, caller: u64, parent: u64) -> Result<u64> {
        self.node(caller)?;
        self.node(parent)?;
        if caller != parent && caller != self.root {
            return Err(Error::Denied);
        }
        Ok(self
            .nodes
            .iter()
            .flatten()
            .filter(|n| n.parent == parent)
            .map(|n| n.token)
            .min()
            .unwrap_or(0))
    }
    fn insert(&mut self, parent: u64, context: Context, units: u64) -> Result<u64> {
        let slot = self
            .nodes
            .iter()
            .position(Option::is_none)
            .ok_or(Error::Quota)?;
        let token = self.next;
        self.next = self.next.checked_add(1).ok_or(Error::Quota)?;
        self.nodes[slot] = Some(Node {
            token,
            parent,
            context,
            state: State::Prepared,
            exit: None,
            remaining: units - 1,
            log_epoch: 0,
            logs: 0,
        });
        Ok(token)
    }
    /// Call only after all image, context, object and scheduler preparation succeeds.
    pub fn bootstrap(&mut self, context: Context) -> Result<u64> {
        if self.live() != 0 {
            return Err(Error::State);
        }
        let token = self.insert(0, context, EPOCH_UNITS)?;
        self.root = token;
        self.closed = false;
        self.restart_count = 0;
        self.restart_after = 0;
        Ok(token)
    }
    pub fn check_child(
        &self,
        parent: u64,
        claimed: Context,
        units: u64,
        threads: u64,
        priority: u64,
        inherited: u64,
    ) -> Result<()> {
        let p = self.node(parent)?;
        if self.closed || p.state != State::Running {
            return Err(Error::Denied);
        }
        if claimed.principal != p.context.principal
            || claimed.session != p.context.session
            || claimed.service
            || claimed.grant_ceiling & !p.context.grant_ceiling != 0
            || claimed.commitment_ceiling != MEMORY
            || priority != 0
            || inherited != 0
        {
            return Err(Error::Denied);
        }
        if threads != 1 || units == 0 || units > p.remaining {
            return Err(Error::Quota);
        }
        if self.next == u64::MAX || self.live() == SLOTS {
            return Err(Error::Quota);
        }
        Ok(())
    }
    pub fn child(&mut self, parent: u64, context: Context, units: u64) -> Result<u64> {
        self.check_child(parent, context, units, 1, 0, 0)?;
        let token = self.insert(parent, context, units)?;
        let i = self.index(parent)?;
        self.nodes[i].as_mut().unwrap().remaining -= units;
        Ok(token)
    }
    fn controls(&self, caller: u64, target: u64) -> Result<()> {
        let c = self.node(caller)?;
        let t = self.node(target)?;
        if c.state != State::Running || (t.parent != caller && caller != self.root) {
            return Err(Error::Denied);
        }
        Ok(())
    }
    pub fn start(&mut self, caller: u64, target: u64) -> Result<()> {
        self.controls(caller, target)?;
        if self.closed {
            return Err(Error::Denied);
        }
        let i = self.index(target)?;
        let n = self.nodes[i].as_mut().unwrap();
        if n.state != State::Prepared {
            return Err(Error::State);
        }
        n.state = State::Running;
        Ok(())
    }
    pub fn start_root(&mut self) -> Result<()> {
        let i = self.index(self.root)?;
        if self.closed || self.nodes[i].unwrap().state != State::Prepared {
            return Err(Error::State);
        }
        self.nodes[i].as_mut().unwrap().state = State::Running;
        Ok(())
    }
    pub fn descendant(&self, token: u64, ancestor: u64) -> bool {
        let mut at = token;
        for _ in 0..SLOTS {
            if at == ancestor {
                return true;
            }
            let Ok(n) = self.node(at) else {
                return false;
            };
            if n.parent == 0 {
                return false;
            }
            at = n.parent;
        }
        false
    }
    pub fn terminate(&mut self, caller: u64, target: u64) -> Result<()> {
        self.controls(caller, target)?;
        self.stop(target)
    }
    /// Orphans are killed, never silently adopted by a new generation/session.
    pub fn stop(&mut self, target: u64) -> Result<()> {
        self.node(target)?;
        if target == self.root {
            self.closed = true;
        }
        let tokens = self.tokens();
        for t in tokens.into_iter().filter(|t| *t != 0) {
            if self.descendant(t, target) {
                let i = self.index(t)?;
                let n = self.nodes[i].as_mut().unwrap();
                if n.state != State::Reaped {
                    n.state = State::Stopping;
                }
            }
        }
        Ok(())
    }
    /// Reparent only tombstones after execution/resources have been destroyed.
    /// Live orphan lineage is preserved until the adapter has drained every task.
    pub fn reaped(&mut self, token: u64, exit: Exit) -> Result<()> {
        let i = self.index(token)?;
        let n = self.nodes[i].as_mut().unwrap();
        if n.state == State::Reaped {
            return Err(Error::State);
        }
        n.state = State::Reaped;
        n.exit = Some(exit);
        Ok(())
    }
    pub fn join(&mut self, caller: u64, target: u64) -> Result<Exit> {
        self.controls(caller, target)?;
        let i = self.index(target)?;
        let exit = self.nodes[i].unwrap().exit.ok_or(Error::WouldBlock)?;
        // A dead parent's identity cannot be recycled while descendants exist.
        if self.nodes.iter().flatten().any(|n| n.parent == target) {
            return Err(Error::WouldBlock);
        }
        self.nodes[i] = None;
        Ok(exit)
    }
    pub fn clear_epoch(&mut self) -> Result<()> {
        if !self.closed
            || self
                .nodes
                .iter()
                .flatten()
                .any(|n| n.state != State::Reaped)
        {
            return Err(Error::State);
        }
        self.nodes = [None; SLOTS];
        self.root = 0;
        Ok(()) // next token is NEVER reset
    }
    /// Trusted supervisor restart authority: two retries, 10/20 ms backoff.
    /// New create still requires fresh context and consumes a lifetime unit.
    pub fn restart(&mut self, caller: u64, now: u64) -> Result<()> {
        if caller != self.root || self.closed || self.state(caller)? != State::Running {
            return Err(Error::Denied);
        }
        if self.restart_count == 2 {
            return Err(Error::Quarantined);
        }
        if now < self.restart_after {
            return Err(Error::Backoff);
        }
        self.restart_count += 1;
        self.restart_after = now.saturating_add(10_000_000 * self.restart_count);
        Ok(())
    }
    pub fn log(&mut self, caller: u64, now: u64) -> Result<()> {
        let i = self.index(caller)?;
        let n = self.nodes[i].as_mut().unwrap();
        let epoch = now / 1_000_000_000;
        if n.log_epoch != epoch {
            n.log_epoch = epoch;
            n.logs = 0;
        }
        if n.logs == 4 {
            return Err(Error::Quota);
        }
        n.logs += 1;
        Ok(()) // <=32 logs/sec/epoch, including the reserved control owner
    }
}

#[cfg(test)]
mod tests;
