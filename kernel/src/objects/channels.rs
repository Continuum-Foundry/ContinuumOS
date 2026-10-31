//! Bounded single-CPU nonblocking transport. Call with IF clear. No user
//! pointer, callback or registry borrow may survive dispatch. ABI is unfrozen.
use super::*;

pub const PAIRS: usize = 8;
pub const MESSAGES: usize = 8;
pub const INLINE: usize = 4096;
pub const ATTACHMENTS: usize = 8;
pub const QUEUE_BYTES: usize = 16384;
pub const QUEUE_HANDLES: usize = 16;
pub const ENDPOINT_RIGHTS: u64 = BASIC | TRANSFER | SEND | RECEIVE;

/// Copied from trusted launch binding, never from a send descriptor. Process
/// slot+generation and session reference are historical attribution, not proof
/// of current policy/session validity. Brokers must revalidate at use time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sender {
    pub process: u64,
    pub generation: u64,
    pub principal: u64,
    pub session: u64,
    pub service: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Move,
    Duplicate,
}
#[derive(Clone, Copy, Debug)]
pub struct Attachment {
    pub handle: u64,
    pub rights: u64,
    pub mode: Mode,
}
#[derive(Clone, Copy)]
struct Held {
    object: Id,
    rights: u64,
}
#[derive(Clone, Copy)]
struct Message {
    bytes: [u8; INLINE],
    length: usize,
    held: [Option<Held>; ATTACHMENTS],
    count: usize,
    sender: Sender,
}
impl Message {
    const EMPTY: Self = Self {
        bytes: [0; INLINE],
        length: 0,
        held: [None; ATTACHMENTS],
        count: 0,
        sender: Sender {
            process: 0,
            generation: 0,
            principal: 0,
            session: 0,
            service: false,
        },
    };
}
#[derive(Clone, Copy)]
struct Queue {
    messages: [Message; MESSAGES],
    head: usize,
    count: usize,
    bytes: usize,
    handles: usize,
}
impl Queue {
    const EMPTY: Self = Self {
        messages: [Message::EMPTY; MESSAGES],
        head: 0,
        count: 0,
        bytes: 0,
        handles: 0,
    };
}
#[derive(Clone, Copy)]
pub(super) struct Pair {
    endpoints: [Option<Id>; 2],
    queues: [Queue; 2],
}
impl Pair {
    pub(super) const EMPTY: Self = Self {
        endpoints: [None; 2],
        queues: [Queue::EMPTY; 2],
    };
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Usage {
    pub pairs: usize,
    pub messages: usize,
    pub bytes: usize,
    pub handles: usize,
}

impl Registry {
    /// Liveness only, no policy/session credential and no authority grant.
    pub fn sender_live(&self, sender: Sender) -> bool {
        self.senders.iter().any(|s| *s == Some(sender))
    }
    pub(super) fn channel_readiness(
        &self,
        space: SpaceId,
        endpoint: u64,
        send: Option<(usize, usize)>,
    ) -> Result<crate::wait::Readiness> {
        use crate::wait::Readiness;
        let (_, pair, side) =
            self.endpoint(space, endpoint, if send.is_some() { SEND } else { RECEIVE })?;
        if let Some((bytes, attachments)) = send {
            if !self.live(self.channels[pair].endpoints[1 - side]) {
                return Ok(Readiness::Closed);
            }
            let q = &self.channels[pair].queues[1 - side];
            Ok(
                if q.count < MESSAGES
                    && bytes <= QUEUE_BYTES - q.bytes
                    && attachments <= QUEUE_HANDLES - q.handles
                {
                    Readiness::Ready
                } else {
                    Readiness::Pending
                },
            )
        } else {
            let q = &self.channels[pair].queues[side];
            // Accepted messages remain drainable after peer closure.
            Ok(if q.count != 0 {
                Readiness::Ready
            } else if !self.live(self.channels[pair].endpoints[1 - side]) {
                Readiness::Closed
            } else {
                Readiness::Pending
            })
        }
    }
    /// Trusted admission only. Binding cannot subsequently be replaced by caller
    /// claims or endpoint delegation. P3-10 owns current session invalidation.
    pub fn admit_sender(&mut self, space: SpaceId, sender: Sender) -> Result<(u64, u64)> {
        if sender.generation == 0 || sender.principal == 0 || sender.session == 0 {
            return Err(Error::InvalidArgument);
        }
        if self
            .senders
            .iter()
            .flatten()
            .any(|s| s.process == sender.process && s.generation == sender.generation)
        {
            return Err(Error::InvalidArgument);
        }
        let result = self.admit(space)?;
        let t = self.table(space)?;
        self.senders[t] = Some(sender);
        Ok(result)
    }
    fn endpoint(&self, space: SpaceId, handle: u64, rights: u64) -> Result<(usize, usize, usize)> {
        let (t, _, _, body) = self.resolve(space, handle, Some(Kind::Channel), rights)?;
        let Body::Channel { pair, side } = body else {
            unreachable!()
        };
        Ok((t, pair, side))
    }
    fn live(&self, id: Option<Id>) -> bool {
        id.is_some_and(|id| {
            self.objects[id.slot].generation == id.generation
                && self.objects[id.slot].body.is_some()
        })
    }
    fn nonce_room(&self, count: usize) -> Result<()> {
        if count != 0
            && (self.next_nonce == 0 || count as u64 > u32::MAX as u64 - self.next_nonce as u64 + 1)
        {
            return Err(Error::QuotaExceeded);
        }
        Ok(())
    }
    /// Preview both output handles and complete the fallible copy BEFORE commit.
    /// A failed copy leaves output bytes unspecified and installs no authority.
    pub fn channel_pair(
        &mut self,
        space: SpaceId,
        process: u64,
        copy: impl FnOnce([u64; 2]) -> Result<()>,
    ) -> Result<()> {
        let t = self.target(space, process, CREATE)?;
        self.handle_room(t, 2)?;
        self.creation_room(t, 2)?;
        if self.tables[t].budget.is_some_and(|b| b.pairs == 0) {
            return Err(Error::QuotaExceeded);
        }
        self.nonce_room(2)?;
        let mut slots = self.tables[t]
            .handles
            .iter()
            .enumerate()
            .filter(|(_, h)| h.is_none())
            .map(|(i, _)| i);
        let hs = [
            slots.next().ok_or(Error::QuotaExceeded)?,
            slots.next().ok_or(Error::QuotaExceeded)?,
        ];
        let mut free = self
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| o.body.is_none() && o.generation != u64::MAX)
            .map(|(i, _)| i);
        let os = [
            free.next().ok_or(Error::QuotaExceeded)?,
            free.next().ok_or(Error::QuotaExceeded)?,
        ];
        let pair = self
            .channels
            .iter()
            .position(|p| p.endpoints == [None; 2])
            .ok_or(Error::QuotaExceeded)?;
        let values = [
            ((self.next_nonce as u64) << 32) | (hs[0] as u64 + 1),
            (((self.next_nonce as u64) + 1) << 32) | (hs[1] as u64 + 1),
        ];
        copy(values)?;
        self.created_objects(t, 2);
        if let Some(b) = &mut self.tables[t].budget {
            b.pairs -= 1;
        }
        let ids = [
            self.new_object(os[0], Body::Channel { pair, side: 0 }),
            self.new_object(os[1], Body::Channel { pair, side: 1 }),
        ];
        self.channels[pair].endpoints = [Some(ids[0]), Some(ids[1])];
        for i in 0..2 {
            assert_eq!(self.install(t, hs[i], ids[i], ENDPOINT_RIGHTS), values[i]);
        }
        Ok(())
    }
    /// Trusted initial endpoint grant, never a public cross-owner lookup.
    pub fn grant_endpoint(
        &mut self,
        source: SpaceId,
        handle: u64,
        target: SpaceId,
        rights: u64,
    ) -> Result<u64> {
        let (_, _, h, _) = self.resolve(source, handle, Some(Kind::Channel), TRANSFER)?;
        if rights & !h.rights != 0 {
            return Err(Error::AccessDenied);
        }
        let t = self.table(target)?;
        let slot = self.room(t)?;
        Ok(self.install(t, slot, h.object, rights))
    }
    /// Bytes have already been copied once from userspace. No source handle is
    /// changed until every descriptor and queue reservation has passed.
    pub fn channel_send(
        &mut self,
        space: SpaceId,
        endpoint: u64,
        bytes: &[u8],
        attachments: &[Attachment],
    ) -> Result<()> {
        self.channel_send_checked(space, endpoint, bytes, attachments, || Ok(()))
    }
    /// A final, non-mutating admission check after complete validation and quota
    /// reservation, immediately before the atomic enqueue decision. Failure
    /// retains all sender authority and queued state. No fallible work follows.
    pub fn channel_send_checked(
        &mut self,
        space: SpaceId,
        endpoint: u64,
        bytes: &[u8],
        attachments: &[Attachment],
        before_commit: impl FnOnce() -> Result<()>,
    ) -> Result<()> {
        if bytes.len() > INLINE || attachments.len() > ATTACHMENTS {
            return Err(Error::InvalidArgument);
        }
        let (t, pair, side) = self.endpoint(space, endpoint, SEND)?;
        let sender = self.senders[t].ok_or(Error::AccessDenied)?;
        if self.tables[t]
            .budget
            .is_some_and(|b| b.messages == 0 || bytes.len() as u64 > b.bytes)
        {
            return Err(Error::QuotaExceeded);
        }
        if !self.live(self.channels[pair].endpoints[1 - side]) {
            return Err(Error::PeerClosed);
        }
        let mut held = [None; ATTACHMENTS];
        let mut sources = [0; ATTACHMENTS];
        for (i, a) in attachments.iter().enumerate() {
            let needed = TRANSFER
                | if a.mode == Mode::Duplicate {
                    DUPLICATE
                } else {
                    0
                };
            let (_, slot, h, _) = self.resolve(space, a.handle, None, needed)?;
            if a.rights & !h.rights != 0 {
                return Err(Error::AccessDenied);
            }
            if sources[..i].contains(&slot) {
                return Err(Error::InvalidArgument);
            }
            sources[i] = slot;
            held[i] = Some(Held {
                object: h.object,
                rights: a.rights,
            });
        }
        let q = &mut self.channels[pair].queues[1 - side];
        if q.count == MESSAGES
            || bytes.len() > QUEUE_BYTES - q.bytes
            || attachments.len() > QUEUE_HANDLES - q.handles
        {
            return Err(Error::WouldBlock);
        }
        before_commit()?;
        if let Some(b) = &mut self.tables[t].budget {
            b.messages -= 1;
            b.bytes -= bytes.len() as u64;
        }
        // No fallible operations below: queue now owns one ref per attachment.
        let m = &mut q.messages[(q.head + q.count) % MESSAGES];
        m.bytes[..bytes.len()].copy_from_slice(bytes);
        m.length = bytes.len();
        m.count = attachments.len();
        m.sender = sender;
        m.held = held;
        q.count += 1;
        q.bytes += bytes.len();
        q.handles += attachments.len();
        for (i, a) in attachments.iter().enumerate() {
            let h = held[i].unwrap();
            self.objects[h.object.slot].handles += 1;
            if a.mode == Mode::Move {
                self.tables[t].handles[sources[i]] = None;
                self.objects[h.object.slot].handles -= 1;
            }
        }
        Ok(())
    }
    pub fn channel_size(&self, space: SpaceId, endpoint: u64) -> Result<(usize, usize)> {
        let (_, pair, side) = self.endpoint(space, endpoint, RECEIVE)?;
        let q = &self.channels[pair].queues[side];
        if q.count == 0 {
            return Err(if self.live(self.channels[pair].endpoints[1 - side]) {
                Error::WouldBlock
            } else {
                Error::PeerClosed
            });
        }
        let m = &q.messages[q.head];
        Ok((m.length, m.count))
    }
    /// Copy bounded bytes, predicted fresh handles and historical metadata before
    /// installing anything. Failed copy/sizing/table/nonce admission keeps head.
    /// After callback success the install+dequeue commit cannot fail. No separate
    /// acknowledgement exists, so caller death after commit drops installed refs.
    pub fn channel_receive(
        &mut self,
        space: SpaceId,
        endpoint: u64,
        byte_capacity: usize,
        handle_capacity: usize,
        copy: impl FnOnce(&[u8], &[u64], Sender) -> Result<()>,
    ) -> Result<()> {
        let (t, pair, side) = self.endpoint(space, endpoint, RECEIVE)?;
        let (bytes, count) = self.channel_size(space, endpoint)?;
        if byte_capacity < bytes || handle_capacity < count {
            return Err(Error::BufferTooSmall);
        }
        self.nonce_room(count)?;
        self.handle_room(t, count)?;
        let mut slots = [0; ATTACHMENTS];
        let mut values = [0; ATTACHMENTS];
        let mut free = self.tables[t]
            .handles
            .iter()
            .enumerate()
            .filter(|(_, h)| h.is_none())
            .map(|(i, _)| i);
        for i in 0..count {
            slots[i] = free.next().ok_or(Error::QuotaExceeded)?;
            values[i] = ((self.next_nonce as u64 + i as u64) << 32) | (slots[i] as u64 + 1);
        }
        let q = &self.channels[pair].queues[side];
        let m = &q.messages[q.head];
        copy(&m.bytes[..bytes], &values[..count], m.sender)?;
        let held = m.held;
        for i in 0..count {
            let h = held[i].unwrap();
            assert_eq!(self.install(t, slots[i], h.object, h.rights), values[i]);
            self.objects[h.object.slot].handles -= 1;
        }
        let q = &mut self.channels[pair].queues[side];
        q.messages[q.head] = Message::EMPTY;
        q.head = (q.head + 1) % MESSAGES;
        q.count -= 1;
        q.bytes -= bytes;
        q.handles -= count;
        Ok(())
    }
    pub fn channel_usage(&self) -> Usage {
        let mut u = Usage {
            pairs: 0,
            messages: 0,
            bytes: 0,
            handles: 0,
        };
        for p in &self.channels {
            u.pairs += usize::from(p.endpoints != [None; 2]);
            for q in &p.queues {
                u.messages += q.count;
                u.bytes += q.bytes;
                u.handles += q.handles;
            }
        }
        u
    }
    /// Trace from actual table handles/maps and trusted owner lifecycle roots.
    /// Queue refs alone must never keep abandoned endpoint cycles alive. A
    /// reachable endpoint roots only its incoming queue, not its peer endpoint.
    pub(super) fn collect_channels(&mut self) {
        let mut marked = [false; OBJECTS];
        for t in &self.tables {
            for h in t.handles.iter().flatten() {
                marked[h.object.slot] = true;
            }
            for m in t.mappings.iter().flatten() {
                marked[m.object.slot] = true;
            }
        }
        for (i, o) in self.objects.iter().enumerate() {
            if matches!(o.body, Some(Body::Process(s) | Body::Thread(s)) if self.table(s).is_ok()) {
                marked[i] = true;
            }
        }
        // At most OBJECTS new vertices, no recursion or heap allocation.
        for _ in 0..OBJECTS {
            let mut changed = false;
            for i in 0..OBJECTS {
                if !marked[i] {
                    continue;
                }
                if let Some(Body::Channel { pair, side }) = self.objects[i].body {
                    let q = &self.channels[pair].queues[side];
                    for j in 0..q.count {
                        let m = &q.messages[(q.head + j) % MESSAGES];
                        for h in m.held.iter().flatten() {
                            if !marked[h.object.slot] {
                                marked[h.object.slot] = true;
                                changed = true;
                            }
                        }
                    }
                }
            }
            if !changed {
                break;
            }
        }
        for pair in 0..PAIRS {
            for side in 0..2 {
                if self.channels[pair].endpoints[side].is_some_and(|id| !marked[id.slot]) {
                    let q = &mut self.channels[pair].queues[side];
                    for j in 0..q.count {
                        for h in q.messages[(q.head + j) % MESSAGES].held.iter().flatten() {
                            self.objects[h.object.slot].handles -= 1;
                        }
                    }
                    *q = Queue::EMPTY;
                    self.channels[pair].endpoints[side] = None;
                }
            }
        }
    }
}
