//! Ephemeral synthetic resource authority, shared with the CPL3 experiment.
//! Callers/instances must be authenticated and live at every entry by the adapter.
use crate::policy::{self, Broker, Identity, Request as PolicyRequest, Scope};
pub const SERVICE: u64 = 1;
pub const PROTOCOL: u64 = 0x525445;
pub const INCOMPATIBLE: u64 = 14;
pub const UNAVAILABLE: u64 = 9;
pub const PENDING: u64 = 6;
pub const CANCELLED: u64 = 8;
pub const RETENTION: u64 = 1_000_000_000;
pub const SCOPE: Scope = Scope {
    kind: policy::FILE,
    resource: 1,
    owner: 101,
};
type Result<T> = policy::Result<T>;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request {
    pub caller: Identity,
    pub id: u64,
    pub service: u64,
    pub protocol: u64,
    pub major: u64,
    pub features: u64,
    pub epoch: u64,
    pub policy_epoch: u64,
    pub grant: u64,
    pub deadline: u64,
    pub delta: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reply {
    pub request: Request,
    pub status: u64,
    pub revision: u64,
    pub value: u64,
}
#[derive(Clone, Copy)]
struct Job {
    request: Request,
    forwarded: Option<Identity>,
}
#[derive(Clone, Copy)]
struct Record {
    reply: Reply,
    until: u64,
}
pub struct Authority {
    admin: Identity,
    pub policy: Broker,
    pub epoch: u64,
    pub writer: Option<Identity>,
    pub published: Option<(Identity, u64)>,
    pub revision: u64,
    pub value: u64,
    queue: [Option<Job>; 4],
    results: [Option<Record>; 8],
    // Never evict high-water marks within an incarnation: forgetting a result
    // cannot turn a timed-out/reordered retry into another mutation.
    seen: [Option<(Identity, u64)>; 4],
}
impl Authority {
    pub const fn new(admin: Identity, epoch: u64) -> Self {
        Self {
            admin,
            policy: Broker::new(admin, epoch),
            epoch,
            writer: None,
            published: None,
            revision: 0,
            value: 0,
            queue: [None; 4],
            results: [None; 8],
            seen: [None; 4],
        }
    }
    fn admin(&self, caller: Identity) -> Result<()> {
        if caller == self.admin {
            Ok(())
        } else {
            Err(policy::DENIED)
        }
    }
    fn check(&self, caller: Identity, grant: u64, req: Request, now: u64) -> Result<()> {
        self.policy
            .preview(
                caller,
                grant,
                PolicyRequest {
                    scope: SCOPE,
                    operation: policy::WRITE,
                    revision: self.policy.revision(),
                    epoch: req.policy_epoch,
                    deadline: req.deadline,
                },
                now,
            )
            .map(|_| ())
    }
    pub fn fence(&mut self, caller: Identity) -> Result<u64> {
        self.admin(caller)?;
        let next = self.epoch.checked_add(1).ok_or(policy::QUOTA)?;
        self.writer = None;
        self.epoch = next;
        Ok(next)
    }
    pub fn grant_writer(
        &mut self,
        caller: Identity,
        instance: Identity,
        grant: u64,
        now: u64,
    ) -> Result<()> {
        self.admin(caller)?;
        if !instance.service || self.writer.is_some() {
            return Err(policy::DENIED);
        }
        let req = Request {
            caller: instance,
            id: 1,
            service: SERVICE,
            protocol: PROTOCOL,
            major: 1,
            features: 0,
            epoch: self.epoch,
            policy_epoch: self.policy.epoch(),
            grant,
            deadline: u64::MAX,
            delta: 1,
        };
        self.check(instance, grant, req, now)?;
        self.writer = Some(instance);
        Ok(())
    }
    pub fn publish(&mut self, caller: Identity, instance: Identity, epoch: u64) -> Result<()> {
        self.admin(caller)?;
        if epoch != self.epoch || self.writer != Some(instance) {
            return Err(policy::STALE);
        }
        self.published = Some((instance, epoch));
        Ok(())
    }
    fn envelope(&self, actual: Identity, req: Request) -> Result<()> {
        if req.service != SERVICE || req.protocol != PROTOCOL || req.major != 1 || req.features != 0
        {
            return Err(INCOMPATIBLE);
        }
        if req.caller != actual || req.id == 0 || req.delta == 0 {
            return Err(policy::DENIED);
        }
        Ok(())
    }
    fn expire(&mut self, now: u64) {
        for record in &mut self.results {
            if record.is_some_and(|r| now >= r.until) {
                *record = None;
            }
        }
    }
    pub fn admit(&mut self, actual: Identity, req: Request, now: u64) -> Result<Option<Reply>> {
        self.envelope(actual, req)?;
        self.check(actual, req.grant, req, now)?;
        self.expire(now);
        if let Some(r) = self
            .results
            .iter()
            .flatten()
            .find(|r| r.reply.request.caller == actual && r.reply.request.id == req.id)
        {
            return if r.reply.request == req {
                Ok(Some(r.reply))
            } else {
                Err(policy::DENIED)
            };
        }
        if let Some(j) = self
            .queue
            .iter()
            .flatten()
            .find(|j| j.request.caller == actual && j.request.id == req.id)
        {
            return Err(if j.request == req {
                PENDING
            } else {
                policy::DENIED
            });
        }
        if req.epoch != self.epoch {
            return Err(policy::STALE);
        }
        if self.published != self.writer.map(|w| (w, self.epoch)) || self.writer.is_none() {
            return Err(UNAVAILABLE);
        }
        let prior = self
            .seen
            .iter()
            .position(|s| s.is_some_and(|(c, _)| c == actual));
        if prior.is_some_and(|i| req.id <= self.seen[i].unwrap().1) {
            return Err(policy::STALE);
        }
        let seen = prior
            .or_else(|| self.seen.iter().position(Option::is_none))
            .ok_or(policy::QUOTA)?;
        let slot = self
            .queue
            .iter()
            .position(Option::is_none)
            .ok_or(policy::QUOTA)?;
        // Reserve an eventual immutable result before admitting more work.
        if self.results.iter().flatten().count() + self.queue.iter().flatten().count() == 8 {
            return Err(policy::QUOTA);
        }
        self.queue[slot] = Some(Job {
            request: req,
            forwarded: None,
        });
        self.seen[seen] = Some((actual, req.id));
        Ok(None)
    }
    fn index(&self, req: Request) -> Result<usize> {
        self.queue
            .iter()
            .position(|j| j.is_some_and(|j| j.request == req))
            .ok_or(policy::STALE)
    }
    pub fn forward(&mut self, req: Request, now: u64) -> Result<Identity> {
        let i = self.index(req)?;
        self.check(req.caller, req.grant, req, now)?;
        if req.epoch != self.epoch {
            return Err(policy::STALE);
        }
        let writer = self.writer.ok_or(UNAVAILABLE)?;
        if self.published != Some((writer, self.epoch)) {
            return Err(UNAVAILABLE);
        }
        self.queue[i].as_mut().unwrap().forwarded = Some(writer);
        Ok(writer)
    }
    fn finish(&mut self, i: usize, status: u64, now: u64) -> Result<Reply> {
        let slot = self
            .results
            .iter()
            .position(Option::is_none)
            .ok_or(policy::QUOTA)?;
        let reply = Reply {
            request: self.queue[i].unwrap().request,
            status,
            revision: self.revision,
            value: self.value,
        };
        self.results[slot] = Some(Record {
            reply,
            until: now.saturating_add(RETENTION),
        });
        self.queue[i] = None;
        Ok(reply)
    }
    /// This is the only mutation point. Route lookup and endpoint possession are
    /// never sufficient. Failed commits become immutable explicit results.
    pub fn commit(
        &mut self,
        actual: Identity,
        service_grant: u64,
        req: Request,
        now: u64,
    ) -> Result<Reply> {
        let i = self.index(req)?;
        let outcome = (|| {
            if req.epoch != self.epoch {
                return Err(policy::STALE);
            }
            if self.writer != Some(actual) || self.queue[i].unwrap().forwarded != Some(actual) {
                return Err(policy::DENIED);
            }
            self.check(req.caller, req.grant, req, now)?;
            self.check(actual, service_grant, req, now)?;
            let revision = self.revision.checked_add(1).ok_or(policy::QUOTA)?;
            let value = self.value.checked_add(req.delta).ok_or(policy::QUOTA)?;
            self.revision = revision;
            self.value = value;
            Ok(())
        })();
        self.finish(i, outcome.err().unwrap_or(0), now)
    }
    pub fn cancel(&mut self, actual: Identity, req: Request, now: u64) -> Result<Reply> {
        self.envelope(actual, req)?;
        self.check(actual, req.grant, req, now)?;
        if let Some(r) = self
            .results
            .iter()
            .flatten()
            .find(|r| r.reply.request == req && now < r.until)
        {
            return Ok(r.reply);
        }
        let i = self.index(req)?;
        self.finish(i, CANCELLED, now)
    }
    pub fn reject(&mut self, req: Request, status: u64, now: u64) -> Result<Reply> {
        let i = self.index(req)?;
        self.finish(i, status, now)
    }
    pub fn reply(&self, actual: Identity, req: Request, reply: Reply, now: u64) -> Result<()> {
        self.envelope(actual, req)?;
        if reply.request != req || req.epoch != self.epoch {
            return Err(policy::STALE);
        }
        if !self
            .results
            .iter()
            .flatten()
            .any(|r| r.reply == reply && now < r.until)
        {
            return Err(policy::DENIED);
        }
        Ok(())
    }
    pub fn queued(&self) -> usize {
        self.queue.iter().flatten().count()
    }
}
#[cfg(test)]
mod tests;
