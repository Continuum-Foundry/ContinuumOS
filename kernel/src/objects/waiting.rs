//! Internal rights-checked readiness. Watches retain nonce-bearing local handles,
//! not pointers or ownership references: closing a watched handle makes it Closed
//! even if a duplicate keeps its object alive. A watch never retargets a new slot.
use super::*;
use crate::wait::{ITEMS, Instant, Readiness, Timer};

pub const EVENT_RIGHTS: u64 = BASIC | CONTROL | TRANSFER;
pub const TIMER_RIGHTS: u64 = BASIC | CONTROL | TRANSFER;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Interest {
    Signalled,
    Exited,
    Readable,
    Writable { bytes: usize, attachments: usize },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Watch {
    pub handle: u64,
    pub interest: Interest,
}
#[derive(Clone, Copy)]
pub struct WatchSet {
    owner: SpaceId,
    watches: [Option<Watch>; ITEMS],
    len: usize,
}
impl WatchSet {
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}
impl Registry {
    pub fn event_create(&mut self, space: SpaceId, process: u64, signalled: bool) -> Result<u64> {
        let t = self.target(space, process, CREATE)?;
        let slot = self.room(t)?;
        let object = self.object_room()?;
        let id = self.new_object(object, Body::Event(signalled));
        Ok(self.install(t, slot, id, EVENT_RIGHTS))
    }
    pub fn event_set(&mut self, space: SpaceId, handle: u64, signalled: bool) -> Result<()> {
        let (_, _, h, _) = self.resolve(space, handle, Some(Kind::Event), CONTROL)?;
        self.objects[h.object.slot].body = Some(Body::Event(signalled));
        Ok(())
    }
    pub fn timer_create(&mut self, space: SpaceId, process: u64) -> Result<u64> {
        let t = self.target(space, process, CREATE)?;
        let slot = self.room(t)?;
        let object = self.object_room()?;
        let id = self.new_object(object, Body::Timer(Timer::EMPTY));
        Ok(self.install(t, slot, id, TIMER_RIGHTS))
    }
    pub fn timer_arm(
        &mut self,
        space: SpaceId,
        handle: u64,
        deadline: Instant,
        period: u64,
    ) -> Result<()> {
        let (_, _, h, body) = self.resolve(space, handle, Some(Kind::Timer), CONTROL)?;
        let Body::Timer(mut timer) = body else {
            unreachable!()
        };
        timer.arm(deadline, period);
        self.objects[h.object.slot].body = Some(Body::Timer(timer));
        Ok(())
    }
    pub fn timer_cancel(&mut self, space: SpaceId, handle: u64) -> Result<()> {
        let (_, _, h, body) = self.resolve(space, handle, Some(Kind::Timer), CONTROL)?;
        let Body::Timer(mut timer) = body else {
            unreachable!()
        };
        timer.cancel();
        self.objects[h.object.slot].body = Some(Body::Timer(timer));
        Ok(())
    }
    fn validate_watch(&self, owner: SpaceId, watch: Watch) -> Result<()> {
        let (_, _, _, body) = self.resolve(owner, watch.handle, None, QUERY)?;
        match (body, watch.interest) {
            (Body::Event(_) | Body::Timer(_), Interest::Signalled) => Ok(()),
            (Body::Process(_) | Body::Thread(_), Interest::Exited) => Ok(()),
            (Body::Channel { .. }, Interest::Readable) => self
                .resolve(owner, watch.handle, Some(Kind::Channel), RECEIVE)
                .map(|_| ()),
            (Body::Channel { .. }, Interest::Writable { bytes, attachments }) => {
                if bytes > channels::INLINE || attachments > channels::ATTACHMENTS {
                    return Err(Error::InvalidArgument);
                }
                self.resolve(owner, watch.handle, Some(Kind::Channel), SEND)
                    .map(|_| ())
            }
            _ => Err(Error::WrongType),
        }
    }
    /// All validation precedes publication. The caller must observe/register
    /// without dropping its single-CPU exclusion boundary.
    pub fn watch_set(&self, owner: SpaceId, watches: &[Watch]) -> Result<WatchSet> {
        if watches.is_empty() || watches.len() > ITEMS {
            return Err(Error::InvalidArgument);
        }
        let mut set = WatchSet {
            owner,
            watches: [None; ITEMS],
            len: watches.len(),
        };
        for (i, watch) in watches.iter().copied().enumerate() {
            if watches[..i].iter().any(|w| w.handle == watch.handle) {
                return Err(Error::InvalidArgument);
            }
            self.validate_watch(owner, watch)?;
            set.watches[i] = Some(watch);
        }
        Ok(set)
    }
    pub fn watch_observe(&mut self, set: &WatchSet, now: Instant) -> Result<[Readiness; ITEMS]> {
        let mut result = [Readiness::Pending; ITEMS];
        for (i, watch) in set.watches[..set.len].iter().flatten().enumerate() {
            let resolved = self.resolve(set.owner, watch.handle, None, QUERY);
            let (_, _, h, body) = match resolved {
                Err(Error::BadHandle) => {
                    result[i] = Readiness::Closed;
                    continue;
                }
                value => value?,
            };
            let ready = match (body, watch.interest) {
                (Body::Event(value), Interest::Signalled) => value,
                (Body::Timer(mut timer), Interest::Signalled) => {
                    timer.observe(now).map_err(|_| Error::InvalidArgument)?;
                    self.objects[h.object.slot].body = Some(Body::Timer(timer));
                    timer.ready()
                }
                (Body::Process(target) | Body::Thread(target), Interest::Exited) => {
                    self.table(target).is_err()
                }
                (Body::Channel { .. }, Interest::Readable) => {
                    result[i] = self.channel_readiness(set.owner, watch.handle, None)?;
                    continue;
                }
                (Body::Channel { .. }, Interest::Writable { bytes, attachments }) => {
                    result[i] = self.channel_readiness(
                        set.owner,
                        watch.handle,
                        Some((bytes, attachments)),
                    )?;
                    continue;
                }
                _ => return Err(Error::WrongType),
            };
            result[i] = if ready {
                Readiness::Ready
            } else {
                Readiness::Pending
            };
        }
        Ok(result)
    }
}
