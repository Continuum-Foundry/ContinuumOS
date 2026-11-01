//! Packaged synthetic reference provider; not an account or durable policy service.
//! Identity arguments must come from IPC provenance plus a current liveness check.
pub const READ: u64 = 1;
pub const WRITE: u64 = 2;
pub const ADMIN: u64 = 4;
pub const DELEGATE: u64 = 8;
pub const FILE: u64 = 1;
pub const DISK: u64 = 2;
pub const ACCOUNT: u64 = 3;
pub const PROCESS: u64 = 4;
pub const DIAGNOSTIC: u64 = 5;
pub const SERVICE: u64 = 6;
pub const DENIED: u64 = 3;
pub const INVALID: u64 = 4;
pub const EXPIRED: u64 = 7;
pub const QUOTA: u64 = 11;
pub const STALE: u64 = 13;
pub const CONFLICT: u64 = 16;
pub type Result<T> = core::result::Result<T, u64>;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Identity {
    pub process: u64,
    pub generation: u64,
    pub principal: u64,
    pub session: u64,
    pub service: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scope {
    pub kind: u64,
    pub resource: u64,
    pub owner: u64,
}
#[derive(Clone, Copy)]
pub struct Grant {
    pub subject: Identity,
    pub scope: Scope,
    pub rights: u64,
    pub until: u64,
    pub epoch: u64,
}
#[derive(Clone, Copy)]
struct Session {
    principal: u64,
    id: u64,
    until: u64,
}
#[derive(Clone, Copy)]
struct Ticket {
    origin: Identity,
    service: Identity,
    grant: u64,
    request: Request,
    epoch: u64,
    used: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request {
    pub scope: Scope,
    pub operation: u64,
    pub revision: u64,
    pub epoch: u64,
    pub deadline: u64,
}
pub struct Broker {
    admin: Identity,
    epoch: u64,
    restored: bool,
    grants: [Option<Grant>; 32],
    tickets: [Option<Ticket>; 16],
    // Session invalidation is monotonic for this incarnation, even after restore.
    dead_sessions: [u64; 16],
    sessions: [Option<Session>; 16],
    revision: u64,
}
impl Broker {
    pub const fn new(admin: Identity, epoch: u64) -> Self {
        Self {
            admin,
            epoch,
            restored: false,
            grants: [None; 32],
            tickets: [None; 16],
            dead_sessions: [0; 16],
            sessions: [None; 16],
            revision: 1,
        }
    }
    pub fn epoch(&self) -> u64 {
        self.epoch
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    fn administrator(&self, caller: Identity) -> Result<()> {
        if caller == self.admin {
            Ok(())
        } else {
            Err(DENIED)
        }
    }
    pub fn restore(&mut self, caller: Identity) -> Result<()> {
        self.administrator(caller)?;
        self.restored = true;
        Ok(())
    }
    pub fn invalidate(&mut self, caller: Identity) -> Result<()> {
        self.administrator(caller)?;
        self.epoch = self.epoch.checked_add(1).ok_or(QUOTA)?;
        self.grants = [None; 32];
        self.tickets = [None; 16];
        Ok(())
    }
    pub fn expire_session(&mut self, caller: Identity, session: u64) -> Result<()> {
        self.administrator(caller)?;
        if session == 0 {
            return Err(INVALID);
        }
        if self.dead_sessions.contains(&session) {
            return Ok(());
        }
        *self
            .dead_sessions
            .iter_mut()
            .find(|s| **s == 0)
            .ok_or(QUOTA)? = session;
        Ok(())
    }
    pub fn lease_session(
        &mut self,
        caller: Identity,
        subject: Identity,
        until: u64,
        now: u64,
    ) -> Result<()> {
        self.administrator(caller)?;
        if !self.restored
            || until <= now
            || until - now > 60_000_000_000
            || subject.session == 0
            || subject.principal == 0
            || self.dead_sessions.contains(&subject.session)
        {
            return Err(EXPIRED);
        }
        if let Some(s) = self
            .sessions
            .iter_mut()
            .flatten()
            .find(|s| s.id == subject.session)
        {
            if s.principal != subject.principal || until > s.until {
                return Err(DENIED);
            }
            s.until = until;
            return Ok(());
        }
        *self
            .sessions
            .iter_mut()
            .find(|s| s.is_none())
            .ok_or(QUOTA)? = Some(Session {
            principal: subject.principal,
            id: subject.session,
            until,
        });
        Ok(())
    }
    fn session(&self, who: Identity, now: u64) -> Result<()> {
        if !self.restored {
            return Err(DENIED);
        }
        if who.generation == 0
            || who.principal == 0
            || who.session == 0
            || self.dead_sessions.contains(&who.session)
        {
            return Err(EXPIRED);
        }
        if !self
            .sessions
            .iter()
            .flatten()
            .any(|s| s.id == who.session && s.principal == who.principal && now < s.until)
        {
            return Err(EXPIRED);
        }
        Ok(())
    }
    /// Role translation: only a named live process, exact resource and operations.
    /// Broker ceiling admits synthetic owner A/B resource 1/2 and typed controls 1.
    pub fn assign(
        &mut self,
        caller: Identity,
        subject: Identity,
        scope: Scope,
        rights: u64,
        until: u64,
        now: u64,
    ) -> Result<u64> {
        self.administrator(caller)?;
        self.session(subject, now)?;
        let ceiling = match (scope.kind, scope.resource, scope.owner) {
            (FILE, 1, 101) | (FILE, 2, 202) => READ | WRITE | DELEGATE,
            (DISK | ACCOUNT | PROCESS | DIAGNOSTIC | SERVICE, 1, 101) => ADMIN,
            _ => 0,
        };
        if rights == 0 || rights & !ceiling != 0 || until <= now {
            return Err(DENIED);
        }
        let slot = self.grants.iter().position(Option::is_none).ok_or(QUOTA)?;
        self.grants[slot] = Some(Grant {
            subject,
            scope,
            rights,
            until,
            epoch: self.epoch,
        });
        Ok(slot as u64 + 1)
    }
    fn grant(&self, who: Identity, id: u64, now: u64) -> Result<Grant> {
        self.session(who, now)?;
        let g = self
            .grants
            .get(id.checked_sub(1).ok_or(DENIED)? as usize)
            .copied()
            .flatten()
            .ok_or(DENIED)?;
        if g.subject != who {
            return Err(DENIED);
        }
        if g.epoch != self.epoch {
            return Err(STALE);
        }
        if now >= g.until {
            return Err(EXPIRED);
        }
        Ok(g)
    }
    pub fn delegate(
        &mut self,
        caller: Identity,
        parent: u64,
        child: Identity,
        scope: Scope,
        rights: u64,
        until: u64,
        now: u64,
    ) -> Result<u64> {
        let g = self.grant(caller, parent, now)?;
        self.session(child, now)?;
        if g.rights & DELEGATE == 0
            || scope != g.scope
            || rights == 0
            || rights & !g.rights != 0
            || until > g.until
            || until <= now
            || child.principal != caller.principal
            || child.session != caller.session
        {
            return Err(DENIED);
        }
        let slot = self.grants.iter().position(Option::is_none).ok_or(QUOTA)?;
        self.grants[slot] = Some(Grant {
            subject: child,
            scope,
            rights,
            until,
            epoch: self.epoch,
        });
        Ok(slot as u64 + 1)
    }
    fn check(&self, caller: Identity, id: u64, req: Request, now: u64) -> Result<()> {
        let g = self.grant(caller, id, now)?;
        if req.epoch != self.epoch {
            return Err(STALE);
        }
        if now >= req.deadline {
            return Err(EXPIRED);
        }
        if req.scope != g.scope
            || ![READ, WRITE, ADMIN].contains(&req.operation)
            || req.operation & !g.rights != 0
        {
            return Err(DENIED);
        }
        if req.scope.kind == FILE && req.revision != self.revision {
            return Err(CONFLICT);
        }
        Ok(())
    }
    /// Preview confers no lease; execution must repeat this check at commit.
    pub fn preview(&self, caller: Identity, id: u64, req: Request, now: u64) -> Result<u64> {
        self.check(caller, id, req, now)?;
        Ok(self.revision)
    }
    pub fn execute(&mut self, caller: Identity, id: u64, req: Request, now: u64) -> Result<u64> {
        self.check(caller, id, req, now)?;
        if req.scope.kind == FILE && req.operation == WRITE {
            self.revision = self.revision.checked_add(1).ok_or(QUOTA)?;
        }
        Ok(self.revision)
    }
    /// Exact single-use on-behalf ticket. The service's own grant never supplies
    /// missing caller rights. Ticket is bound to a trusted forwarding process.
    pub fn ticket(
        &mut self,
        caller: Identity,
        grant: u64,
        service: Identity,
        req: Request,
        now: u64,
    ) -> Result<u64> {
        self.check(caller, grant, req, now)?;
        self.session(service, now)?;
        if !service.service {
            return Err(DENIED);
        }
        let slot = self.tickets.iter().position(Option::is_none).ok_or(QUOTA)?;
        self.tickets[slot] = Some(Ticket {
            origin: caller,
            service,
            grant,
            request: req,
            epoch: self.epoch,
            used: false,
        });
        Ok(slot as u64 + 1)
    }
    pub fn ticket_origin(&self, id: u64) -> Option<Identity> {
        self.tickets
            .get(id.checked_sub(1)? as usize)?
            .map(|t| t.origin)
    }
    pub fn forwarded(
        &mut self,
        service: Identity,
        own_grant: u64,
        ticket: u64,
        req: Request,
        now: u64,
    ) -> Result<u64> {
        let index = ticket.checked_sub(1).ok_or(DENIED)? as usize;
        let t = self.tickets.get(index).copied().flatten().ok_or(DENIED)?;
        if t.used || t.service != service || t.request != req || t.epoch != self.epoch {
            return Err(DENIED);
        }
        self.check(service, own_grant, req, now)?;
        self.check(t.origin, t.grant, req, now)?;
        let result = self.execute(t.origin, t.grant, req, now)?;
        self.tickets[index].as_mut().unwrap().used = true;
        Ok(result)
    }
}
