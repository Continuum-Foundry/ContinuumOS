//! P3-06 single-CPU capability registry. All entry points run with IF clear;
//! no user reference, pin, or registry borrow survives a dispatch. Object IDs
//! never cross the syscall boundary. Numeric handles are not secrets.
use crate::{
    Error as MemoryError,
    owned_memory::{Access, AllocationId, Runtime, SpaceId, Store},
};

pub const QUERY: u64 = 1;
pub const DUPLICATE: u64 = 2;
pub const CLOSE: u64 = 4;
pub const CREATE: u64 = 8;
pub const MAP: u64 = 16;
pub const READ: u64 = 32;
pub const WRITE: u64 = 64;
pub const EXECUTE: u64 = 128;
pub const CONTROL: u64 = 256;
pub const TRANSFER: u64 = 512;
pub const SEND: u64 = 1024;
pub const RECEIVE: u64 = 2048;
pub const BASIC: u64 = QUERY | DUPLICATE | CLOSE;
pub const MEMORY_RIGHTS: u64 = BASIC | MAP | READ | WRITE | EXECUTE;
pub const HANDLES: usize = 32;
pub const MAPPINGS: usize = 32;
pub const OBJECTS: usize = 128;
pub const OWNERS: usize = 16;
pub const BYTE_LIMIT: u64 = 32 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    BadHandle,
    WrongType,
    AccessDenied,
    InvalidArgument,
    QuotaExceeded,
    OutOfMemory,
    WouldBlock,
    PeerClosed,
    BufferTooSmall,
}
pub type Result<T> = core::result::Result<T, Error>;
fn memory_error(e: MemoryError) -> Error {
    match e.reason {
        "quota" | "capacity" => Error::QuotaExceeded,
        "exhausted" => Error::OutOfMemory,
        "busy" => Error::WouldBlock,
        "alias_wx" => Error::AccessDenied,
        _ => Error::InvalidArgument,
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Process = 1,
    Thread = 2,
    Memory = 3,
    Channel = 4,
    Event = 5,
    Timer = 6,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Id {
    slot: usize,
    generation: u64,
}
#[derive(Clone, Copy)]
enum Body {
    Process(SpaceId),
    Thread(SpaceId),
    Event(bool),
    Timer(crate::wait::Timer),
    Channel {
        pair: usize,
        side: usize,
    },
    Memory {
        allocation: AllocationId,
        bytes: u64,
        ceiling: u64,
        creator: SpaceId,
    },
}
impl Body {
    fn kind(self) -> Kind {
        match self {
            Self::Process(_) => Kind::Process,
            Self::Thread(_) => Kind::Thread,
            Self::Memory { .. } => Kind::Memory,
            Self::Channel { .. } => Kind::Channel,
            Self::Event(_) => Kind::Event,
            Self::Timer(_) => Kind::Timer,
        }
    }
}
#[derive(Clone, Copy)]
struct Object {
    generation: u64,
    body: Option<Body>,
    handles: usize,
    maps: usize,
}
impl Object {
    const EMPTY: Self = Self {
        generation: 0,
        body: None,
        handles: 0,
        maps: 0,
    };
}
#[derive(Clone, Copy)]
struct Handle {
    nonce: u32,
    object: Id,
    rights: u64,
}
#[derive(Clone, Copy)]
struct Mapping {
    object: Id,
    base: u64,
    bytes: u64,
    ceiling: u64,
}
#[derive(Clone, Copy)]
struct Table {
    space: Option<SpaceId>,
    handles: [Option<Handle>; HANDLES],
    mappings: [Option<Mapping>; MAPPINGS],
    created: u64,
    budget: Option<Budget>,
}
#[derive(Clone, Copy)]
struct Budget {
    objects: u64,
    pairs: u64,
    timers: u64,
    messages: u64,
    bytes: u64,
}
impl Table {
    const EMPTY: Self = Self {
        space: None,
        handles: [None; HANDLES],
        mappings: [None; MAPPINGS],
        created: 0,
        budget: None,
    };
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Inventory {
    pub owners: usize,
    pub objects: usize,
    pub handles: usize,
    pub mappings: usize,
    pub backing_bytes: u64,
    pub mapped_bytes: u64,
}
pub struct Registry {
    tables: [Table; OWNERS],
    objects: [Object; OBJECTS],
    next_nonce: u32,
    channels: [channels::Pair; channels::PAIRS],
    senders: [Option<channels::Sender>; OWNERS],
}
impl Registry {
    pub const EMPTY: Self = Self {
        tables: [Table::EMPTY; OWNERS],
        objects: [Object::EMPTY; OBJECTS],
        next_nonce: 1,
        channels: [channels::Pair::EMPTY; channels::PAIRS],
        senders: [None; OWNERS],
    };
    fn table(&self, space: SpaceId) -> Result<usize> {
        self.tables
            .iter()
            .position(|t| t.space == Some(space))
            .ok_or(Error::BadHandle)
    }
    fn room(&self, table: usize) -> Result<usize> {
        self.handle_room(table, 1)?;
        if self.next_nonce == 0 {
            return Err(Error::QuotaExceeded);
        }
        self.tables[table]
            .handles
            .iter()
            .position(Option::is_none)
            .ok_or(Error::QuotaExceeded)
    }
    fn handle_room(&self, t: usize, count: usize) -> Result<()> {
        let limit = if self.tables[t].budget.is_some() {
            crate::supervisor::HANDLES
        } else {
            HANDLES
        };
        if self.tables[t].handles.iter().flatten().count() + count > limit {
            return Err(Error::QuotaExceeded);
        }
        Ok(())
    }
    fn creation_room(&self, t: usize, count: u64) -> Result<()> {
        if self.tables[t].budget.is_some_and(|b| count > b.objects) {
            return Err(Error::QuotaExceeded);
        }
        Ok(())
    }
    fn created_objects(&mut self, t: usize, count: u64) {
        if let Some(b) = &mut self.tables[t].budget {
            b.objects -= count;
        }
    }
    /// Trusted pre-entry bound. Lifetime credits deliberately do not refund.
    pub fn constrain(&mut self, space: SpaceId) -> Result<()> {
        let t = self.table(space)?;
        if self.tables[t].budget.is_some() || self.tables[t].handles.iter().flatten().count() != 2 {
            return Err(Error::InvalidArgument);
        }
        self.tables[t].budget = Some(Budget {
            objects: crate::supervisor::OBJECT_CREATES,
            pairs: 1,
            timers: crate::supervisor::TIMERS,
            messages: crate::supervisor::MESSAGES,
            bytes: crate::supervisor::MESSAGE_BYTES,
        });
        Ok(())
    }
    fn object_room(&self) -> Result<usize> {
        self.objects
            .iter()
            .position(|o| o.body.is_none() && o.generation != u64::MAX)
            .ok_or(Error::QuotaExceeded)
    }
    fn new_object(&mut self, slot: usize, body: Body) -> Id {
        let generation = self.objects[slot].generation + 1;
        self.objects[slot] = Object {
            generation,
            body: Some(body),
            handles: 0,
            maps: 0,
        };
        Id { slot, generation }
    }
    fn install(&mut self, t: usize, slot: usize, object: Id, rights: u64) -> u64 {
        let nonce = self.next_nonce;
        assert!(nonce != 0 && self.tables[t].handles[slot].is_none());
        self.next_nonce = nonce.checked_add(1).unwrap_or(0);
        self.tables[t].handles[slot] = Some(Handle {
            nonce,
            object,
            rights,
        });
        self.objects[object.slot].handles += 1;
        (u64::from(nonce) << 32) | (slot as u64 + 1)
    }
    fn resolve(
        &self,
        space: SpaceId,
        value: u64,
        kind: Option<Kind>,
        rights: u64,
    ) -> Result<(usize, usize, Handle, Body)> {
        let t = self.table(space)?;
        let slot = (value as u32).checked_sub(1).ok_or(Error::BadHandle)? as usize;
        let h = self.tables[t]
            .handles
            .get(slot)
            .copied()
            .flatten()
            .filter(|h| h.nonce == (value >> 32) as u32)
            .ok_or(Error::BadHandle)?;
        let object = &self.objects[h.object.slot];
        let body = object
            .body
            .filter(|_| object.generation == h.object.generation)
            .ok_or(Error::BadHandle)?;
        if kind.is_some_and(|k| body.kind() != k) {
            return Err(Error::WrongType);
        }
        if h.rights & rights != rights {
            return Err(Error::AccessDenied);
        }
        Ok((t, slot, h, body))
    }
    /// Trusted admission only. Atomic initial self-process and owner-thread grants.
    pub fn admit(&mut self, space: SpaceId) -> Result<(u64, u64)> {
        if self.table(space).is_ok() {
            return Err(Error::InvalidArgument);
        }
        let t = self
            .tables
            .iter()
            .position(|t| t.space.is_none())
            .ok_or(Error::QuotaExceeded)?;
        if self.next_nonce == 0 || self.next_nonce == u32::MAX {
            return Err(Error::QuotaExceeded);
        }
        let mut free = self
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| o.body.is_none() && o.generation != u64::MAX)
            .map(|(i, _)| i);
        let a = free.next().ok_or(Error::QuotaExceeded)?;
        let b = free.next().ok_or(Error::QuotaExceeded)?;
        self.tables[t] = Table {
            space: Some(space),
            ..Table::EMPTY
        };
        let process = self.new_object(a, Body::Process(space));
        let thread = self.new_object(b, Body::Thread(space));
        let ph = self.install(t, 0, process, BASIC | CREATE | MAP | CONTROL);
        let th = self.install(t, 1, thread, BASIC | CONTROL);
        Ok((ph, th))
    }
    pub fn info(&self, space: SpaceId, value: u64) -> Result<u64> {
        let (_, _, h, b) = self.resolve(space, value, None, QUERY)?;
        // Explicit packed scalar: kind in high 32 bits, rights low 32. No IDs/pointers.
        Ok((b.kind() as u64) << 32 | h.rights)
    }
    pub fn duplicate(&mut self, space: SpaceId, value: u64, rights: u64) -> Result<u64> {
        let (t, _, h, _) = self.resolve(space, value, None, DUPLICATE)?;
        if rights & !h.rights != 0 {
            return Err(Error::AccessDenied);
        }
        let slot = self.room(t)?;
        Ok(self.install(t, slot, h.object, rights))
    }
    /// No public transfer until P3-07. Launch may explicitly grant only immutable
    /// R/NX memory, bounded by both the source handle and backing ceiling.
    pub fn grant_read(&mut self, from: SpaceId, value: u64, to: SpaceId) -> Result<u64> {
        let (_, _, h, body) =
            self.resolve(from, value, Some(Kind::Memory), DUPLICATE | MAP | READ)?;
        let Body::Memory { ceiling, .. } = body else {
            unreachable!()
        };
        if ceiling != READ {
            return Err(Error::AccessDenied);
        }
        let t = self.table(to)?;
        let slot = self.room(t)?;
        Ok(self.install(t, slot, h.object, h.rights & (BASIC | MAP | READ)))
    }
    pub fn control(&self, space: SpaceId, value: u64, kind: Kind) -> Result<()> {
        let (_, _, _, body) = self.resolve(space, value, Some(kind), CONTROL)?;
        let target = match body {
            Body::Process(s) | Body::Thread(s) => s,
            _ => return Err(Error::WrongType),
        };
        if target != space {
            return Err(Error::AccessDenied);
        }
        Ok(())
    }
    fn target(&self, space: SpaceId, process: u64, rights: u64) -> Result<usize> {
        let (t, _, _, body) = self.resolve(space, process, Some(Kind::Process), rights)?;
        if !matches!(body, Body::Process(s) if s == space) {
            return Err(Error::AccessDenied);
        }
        Ok(t)
    }
    pub fn inventory(&self) -> Inventory {
        let mut i = Inventory {
            owners: 0,
            objects: 0,
            handles: 0,
            mappings: 0,
            backing_bytes: 0,
            mapped_bytes: 0,
        };
        for t in &self.tables {
            i.owners += usize::from(t.space.is_some());
            i.handles += t.handles.iter().flatten().count();
            for m in t.mappings.iter().flatten() {
                i.mappings += 1;
                i.mapped_bytes += m.bytes;
            }
        }
        for o in &self.objects {
            if let Some(b) = o.body {
                i.objects += 1;
                if let Body::Memory { bytes, .. } = b {
                    i.backing_bytes += bytes;
                }
            }
        }
        i
    }
}

impl Registry {
    /// P3-07 profile creation: transfer is a delegable right, never recovered
    /// from payload IDs. Historical profiles retain their P3-06 grant set.
    pub fn create_transferable<const S: usize, const A: usize, const M: usize, const T: usize>(
        &mut self,
        r: &mut Runtime<S, A, M, T>,
        store: &mut impl Store,
        space: SpaceId,
        process: u64,
        bytes: u64,
        ceiling: u64,
    ) -> Result<u64> {
        let h = self.create(r, store, space, process, bytes, ceiling)?;
        let (t, slot, _, _) = self.resolve(space, h, None, 0)?;
        self.tables[t].handles[slot].as_mut().unwrap().rights |= TRANSFER;
        Ok(h)
    }
    pub fn sweep<const S: usize, const A: usize, const M: usize, const T: usize>(
        &mut self,
        r: &mut Runtime<S, A, M, T>,
        store: &mut impl Store,
    ) -> Result<()> {
        self.quiescent(r)?;
        self.collect(r, store)
    }
    fn quiescent<const S: usize, const A: usize, const M: usize, const T: usize>(
        &self,
        r: &Runtime<S, A, M, T>,
    ) -> Result<()> {
        for s in self.tables.iter().filter_map(|t| t.space) {
            r.ensure_unpinned(s).map_err(memory_error)?;
        }
        Ok(())
    }
    fn collect<const S: usize, const A: usize, const M: usize, const T: usize>(
        &mut self,
        r: &mut Runtime<S, A, M, T>,
        store: &mut impl Store,
    ) -> Result<()> {
        self.collect_channels();
        for slot in 0..OBJECTS {
            let o = self.objects[slot];
            if o.handles != 0 || o.maps != 0 {
                continue;
            }
            if matches!(o.body, Some(Body::Process(s) | Body::Thread(s)) if self.table(s).is_ok()) {
                continue; // trusted admitted-owner lifecycle reference
            }
            if let Some(Body::Memory {
                allocation,
                bytes,
                creator,
                ..
            }) = o.body
            {
                r.release_object(store, allocation).map_err(memory_error)?;
                if let Ok(t) = self.table(creator) {
                    self.tables[t].created -= bytes;
                }
            }
            self.objects[slot].body = None;
        }
        Ok(())
    }
    pub fn close<const S: usize, const A: usize, const M: usize, const T: usize>(
        &mut self,
        r: &mut Runtime<S, A, M, T>,
        store: &mut impl Store,
        space: SpaceId,
        value: u64,
    ) -> Result<()> {
        let (t, slot, h, _) = self.resolve(space, value, None, CLOSE)?;
        self.quiescent(r)?;
        self.tables[t].handles[slot] = None;
        self.objects[h.object.slot].handles -= 1;
        self.collect(r, store)
    }
    pub fn create<const S: usize, const A: usize, const M: usize, const T: usize>(
        &mut self,
        r: &mut Runtime<S, A, M, T>,
        store: &mut impl Store,
        space: SpaceId,
        process: u64,
        bytes: u64,
        ceiling: u64,
    ) -> Result<u64> {
        let t = self.target(space, process, CREATE)?;
        if bytes == 0
            || bytes % 4096 != 0
            || bytes > 65536
            || ![READ, READ | WRITE, READ | EXECUTE].contains(&ceiling)
        {
            return Err(Error::InvalidArgument);
        }
        let slot = self.room(t)?;
        let object_slot = self.object_room()?;
        self.creation_room(t, 1)?;
        if bytes > BYTE_LIMIT - self.tables[t].created {
            return Err(Error::QuotaExceeded);
        }
        let a = r
            .allocate(store, space, bytes, 4096)
            .map_err(memory_error)?
            .ok_or(Error::OutOfMemory)?;
        if let Err(e) = r.hold_object(space, a) {
            r.free(store, space, a).expect("failed hold unwind");
            return Err(memory_error(e));
        }
        // All fallible resource checks precede backing publication/handle install.
        let id = self.new_object(
            object_slot,
            Body::Memory {
                allocation: a,
                bytes,
                ceiling,
                creator: space,
            },
        );
        self.tables[t].created += bytes;
        self.created_objects(t, 1);
        Ok(self.install(t, slot, id, BASIC | MAP | ceiling))
    }
    pub fn seed<const S: usize, const A: usize, const M: usize, const T: usize>(
        &self,
        r: &Runtime<S, A, M, T>,
        store: &mut impl Store,
        space: SpaceId,
        handle: u64,
        bytes: &[u8],
    ) -> Result<()> {
        let (_, _, _, b) = self.resolve(space, handle, Some(Kind::Memory), READ)?;
        let Body::Memory { allocation, .. } = b else {
            unreachable!()
        };
        r.write_staging(store, space, allocation, 0, bytes)
            .map_err(memory_error)
    }
    pub fn map<const S: usize, const A: usize, const M: usize, const T: usize>(
        &mut self,
        r: &mut Runtime<S, A, M, T>,
        store: &mut impl Store,
        space: SpaceId,
        process: u64,
        memory: u64,
        base: u64,
        offset: u64,
        bytes: u64,
        access: Access,
    ) -> Result<()> {
        let t = self.target(space, process, MAP)?;
        let right = match access {
            Access::Read => READ,
            Access::Write => READ | WRITE,
            Access::Execute => READ | EXECUTE,
        };
        let (_, _, h, b) = self.resolve(space, memory, Some(Kind::Memory), MAP | right)?;
        let Body::Memory {
            allocation,
            bytes: length,
            ceiling,
            ..
        } = b
        else {
            unreachable!()
        };
        if ceiling & right != right {
            return Err(Error::AccessDenied);
        }
        if bytes == 0
            || offset % 4096 != 0
            || bytes % 4096 != 0
            || offset.checked_add(bytes).is_none_or(|v| v > length)
        {
            return Err(Error::InvalidArgument);
        }
        let slot = self.tables[t]
            .mappings
            .iter()
            .position(Option::is_none)
            .ok_or(Error::QuotaExceeded)?;
        let used: u64 = self.tables[t]
            .mappings
            .iter()
            .flatten()
            .map(|m| m.bytes)
            .sum();
        if bytes > BYTE_LIMIT - used {
            return Err(Error::QuotaExceeded);
        }
        r.map(store, space, allocation, base, offset, bytes, access)
            .map_err(memory_error)?;
        self.tables[t].mappings[slot] = Some(Mapping {
            object: h.object,
            base,
            bytes,
            ceiling: ceiling & h.rights,
        });
        self.objects[h.object.slot].maps += 1;
        Ok(())
    }
    fn mapping(&self, t: usize, base: u64) -> Result<usize> {
        self.tables[t]
            .mappings
            .iter()
            .position(|m| m.is_some_and(|m| m.base == base))
            .ok_or(Error::InvalidArgument)
    }
    pub fn protect<const S: usize, const A: usize, const M: usize, const T: usize>(
        &mut self,
        r: &mut Runtime<S, A, M, T>,
        store: &mut impl Store,
        space: SpaceId,
        process: u64,
        base: u64,
        access: Access,
    ) -> Result<()> {
        let t = self.target(space, process, MAP)?;
        let slot = self.mapping(t, base)?;
        let m = self.tables[t].mappings[slot].unwrap();
        let right = match access {
            Access::Read => READ,
            Access::Write => READ | WRITE,
            Access::Execute => READ | EXECUTE,
        };
        if m.ceiling & right != right {
            return Err(Error::AccessDenied);
        }
        r.protect(store, space, base, access).map_err(memory_error)
    }
    pub fn unmap<const S: usize, const A: usize, const M: usize, const T: usize>(
        &mut self,
        r: &mut Runtime<S, A, M, T>,
        store: &mut impl Store,
        space: SpaceId,
        process: u64,
        base: u64,
    ) -> Result<()> {
        let t = self.target(space, process, MAP)?;
        let slot = self.mapping(t, base)?;
        self.quiescent(r)?;
        let m = self.tables[t].mappings[slot].unwrap();
        r.unmap(store, space, base).map_err(memory_error)?;
        self.tables[t].mappings[slot] = None;
        self.objects[m.object.slot].maps -= 1;
        self.collect(r, store)
    }
    /// Supervisor calls on its surviving root/stack, before Runtime::destroy.
    /// Copies must finish/unpin first. Remaining foreign handles/maps keep memory.
    pub fn remove<const S: usize, const A: usize, const M: usize, const T: usize>(
        &mut self,
        r: &mut Runtime<S, A, M, T>,
        store: &mut impl Store,
        space: SpaceId,
    ) -> Result<()> {
        let t = self.table(space)?;
        self.quiescent(r)?;
        // Unmapping a valid admitted mapping rebuilds a subset of its tables;
        // it requires no frames and fits the already precharged table capacity.
        for slot in 0..MAPPINGS {
            if let Some(m) = self.tables[t].mappings[slot] {
                r.unmap(store, space, m.base)
                    .expect("admitted teardown subset");
                self.objects[m.object.slot].maps -= 1;
                self.tables[t].mappings[slot] = None;
            }
        }
        for h in self.tables[t].handles.iter().flatten() {
            self.objects[h.object.slot].handles -= 1;
        }
        self.tables[t] = Table::EMPTY;
        self.senders[t] = None;
        self.collect(r, store)?;
        Ok(())
    }
}
pub mod channels;
#[cfg(test)]
mod tests;
pub mod waiting;
