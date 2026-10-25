//! P3-02 internal single-CPU backend. No public handles, scheduler or user ABI.
//! Tables are precharged image storage; data runs come only from sealed M1 RAM.
use crate::{
    PhysAddr, Range, Result, VirtAddr, error,
    memory::{PAGE, Permission, Tables},
};

pub const USER_END: u64 = 0x8000_0000;
pub const STACK_BASE: u64 = 0x7ffe_0000;
pub const STACK_BYTES: u64 = 65536;
pub const PROCESS_LIMIT: u64 = 64 * 1024 * 1024;

/// Implementations supply zeroed, aligned exclusive runs and infallible release
/// of admitted runs. read/write operate only on those runs, never arbitrary RAM.
/// Published runs have RO/NX supervisor aliases. Writes are staging-only; no
/// pointer/reference escapes. Publication invalidates the old CR3 before reuse.
pub trait Store {
    fn allocate(&mut self, pages: usize, alignment: u64) -> Result<PhysAddr>;
    fn release(&mut self, base: PhysAddr, pages: usize);
    fn read(&self, base: PhysAddr, offset: u64) -> u8;
    fn write(&mut self, base: PhysAddr, offset: u64, byte: u8);
    fn publish(&mut self, old: PhysAddr, new: PhysAddr);
    fn retire(&mut self, root: PhysAddr);
    fn prepare_kernel_stack(&mut self, slot: usize) -> Result<Range>;
    fn release_kernel_stack(&mut self, slot: usize);
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpaceId {
    slot: usize,
    generation: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AllocationId {
    slot: usize,
    generation: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Access {
    Read,
    Write,
    Execute,
}
impl Access {
    fn permission(self) -> Permission {
        match self {
            Self::Read => Permission::ReadOnly,
            Self::Write => Permission::ReadWrite,
            Self::Execute => Permission::ReadExecute,
        }
    }
}
#[derive(Clone, Copy)]
struct Allocation {
    generation: u64,
    owner: Option<SpaceId>,
    base: PhysAddr,
    requested: u64,
    pages: usize,
    maps: usize,
    writers: usize,
    executors: usize,
}
impl Allocation {
    const EMPTY: Self = Self {
        generation: 0,
        owner: None,
        base: PhysAddr(0),
        requested: 0,
        pages: 0,
        maps: 0,
        writers: 0,
        executors: 0,
    };
}
#[derive(Clone, Copy)]
struct Mapping {
    allocation: AllocationId,
    address: u64,
    offset: u64,
    bytes: u64,
    access: Access,
}
struct Space<const M: usize, const T: usize> {
    generation: u64,
    live: bool,
    committed: u64,
    requested: u64,
    bank: usize,
    pins: usize,
    kernel_stack: Option<Range>,
    maps: [Option<Mapping>; M],
    tables: [Tables<T>; 2],
}
impl<const M: usize, const T: usize> Space<M, T> {
    const EMPTY: Self = Self {
        generation: 0,
        live: false,
        committed: 0,
        requested: 0,
        bank: 0,
        pins: 0,
        kernel_stack: None,
        maps: [None; M],
        tables: [const { Tables::EMPTY }; 2],
    };
}
/// Non-copyable, explicit internal pin; no user pointer is retained across waits.
pub struct CopyPin {
    slot: usize,
    generation: u64,
}
#[derive(Clone, Copy)]
struct Pin {
    generation: u64,
    space: Option<SpaceId>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Inventory {
    pub spaces: usize,
    pub allocations: usize,
    pub requested: u64,
    pub committed: u64,
    pub resident: u64,
    pub mappings: usize,
    pub mapped_bytes: u64,
    pub page_tables: usize,
    pub metadata_bytes: usize,
    pub table_storage_bytes: usize,
    pub user_guard_bytes: u64,
    pub active_pins: usize,
    pub kernel_stack_bytes: u64,
    pub kernel_guard_bytes: u64,
}
pub struct Runtime<const S: usize, const A: usize, const M: usize, const T: usize> {
    spaces: [Space<M, T>; S],
    allocations: [Allocation; A],
    pins: [Pin; 32],
    kernel: [u64; 256],
    initialized: bool,
}
impl<const S: usize, const A: usize, const M: usize, const T: usize> Runtime<S, A, M, T> {
    pub const EMPTY: Self = Self {
        spaces: [const { Space::EMPTY }; S],
        allocations: [Allocation::EMPTY; A],
        pins: [Pin {
            generation: 0,
            space: None,
        }; 32],
        kernel: [0; 256],
        initialized: false,
    };
    /// Called once, while IF is clear. address supplies actual physical backing
    /// for image-owned table banks; it is not a user-selectable physical address.
    pub fn initialize(
        &mut self,
        kernel: [u64; 256],
        mut address: impl FnMut(*const u8) -> Result<PhysAddr>,
    ) -> Result<()> {
        if self.initialized || kernel.iter().any(|e| e & 4 != 0) {
            return Err(error("runtime", "initialization"));
        }
        for s in &mut self.spaces {
            for t in &mut s.tables {
                t.initialize(address(t as *const _ as *const u8)?)?;
                t.inherit_kernel(&kernel)?;
            }
        }
        self.kernel = kernel;
        self.initialized = true;
        Ok(())
    }
    fn space(&self, id: SpaceId) -> Result<usize> {
        if id.slot >= S
            || !self.spaces[id.slot].live
            || self.spaces[id.slot].generation != id.generation
        {
            return Err(error("space", "stale"));
        }
        Ok(id.slot)
    }
    fn allocation(&self, id: AllocationId) -> Result<Allocation> {
        let a = self
            .allocations
            .get(id.slot)
            .ok_or(error("allocation", "stale"))?;
        if a.owner.is_none() || a.generation != id.generation {
            return Err(error("allocation", "stale"));
        }
        Ok(*a)
    }
    fn owned(&self, owner: SpaceId, id: AllocationId) -> Result<Allocation> {
        self.space(owner)?;
        let a = self.allocation(id)?;
        if a.owner != Some(owner) {
            return Err(error("allocation", "owner"));
        }
        Ok(a)
    }
    fn mutable(&self, id: SpaceId) -> Result<usize> {
        let i = self.space(id)?;
        if self.spaces[i].pins != 0 {
            return Err(error("space", "busy"));
        }
        Ok(i)
    }
    pub fn create(&mut self) -> Result<SpaceId> {
        if !self.initialized {
            return Err(error("runtime", "uninitialized"));
        }
        let i = self
            .spaces
            .iter()
            .position(|s| !s.live && s.generation != u64::MAX)
            .ok_or(error("space", "quota"))?;
        let s = &mut self.spaces[i];
        s.generation += 1;
        s.live = true;
        Ok(SpaceId {
            slot: i,
            generation: s.generation,
        })
    }
    pub fn create_with_stack(&mut self, store: &mut impl Store) -> Result<SpaceId> {
        self.create_with_stack_bytes(store, &[])
    }
    /// Trusted loader tail (synthetic return address), written before publication.
    pub fn create_with_stack_bytes(
        &mut self,
        store: &mut impl Store,
        tail: &[u8],
    ) -> Result<SpaceId> {
        if tail.len() as u64 > STACK_BYTES {
            return Err(error("stack", "tail"));
        }
        let space = self.create()?;
        let result = (|| {
            let stack = store.prepare_kernel_stack(space.slot)?;
            if stack.len != STACK_BYTES
                || stack.base % PAGE != 0
                || stack.base < 0xffff_8000_0000_1000
                || stack.base.checked_add(STACK_BYTES + PAGE).is_none()
            {
                return Err(error("stack", "range"));
            }
            self.spaces[space.slot].kernel_stack = Some(stack);
            let stack = self.allocate(store, space, STACK_BYTES, PAGE)?.unwrap();
            self.write_staging(store, space, stack, STACK_BYTES - tail.len() as u64, tail)?;
            self.map(
                store,
                space,
                stack,
                STACK_BASE,
                0,
                STACK_BYTES,
                Access::Write,
            )
        })();
        if let Err(e) = result {
            self.destroy(store, space)?;
            return Err(e);
        }
        Ok(space)
    }
    pub fn kernel_stack(&self, space: SpaceId) -> Result<Range> {
        self.spaces[self.space(space)?]
            .kernel_stack
            .ok_or(error("stack", "absent"))
    }
    pub fn allocate(
        &mut self,
        store: &mut impl Store,
        owner: SpaceId,
        bytes: u64,
        alignment: u64,
    ) -> Result<Option<AllocationId>> {
        let i = self.mutable(owner)?;
        if !alignment.is_power_of_two() || alignment > 2 * 1024 * 1024 {
            return Err(error("allocation", "alignment"));
        }
        if bytes == 0 {
            return Ok(None);
        }
        let rounded = bytes
            .checked_add(PAGE - 1)
            .ok_or(error("allocation", "overflow"))?
            & !(PAGE - 1);
        if rounded > PROCESS_LIMIT - self.spaces[i].committed {
            return Err(error("allocation", "quota"));
        }
        let slot = self
            .allocations
            .iter()
            .position(|a| a.owner.is_none() && a.generation != u64::MAX)
            .ok_or(error("allocation", "quota"))?;
        // All accounting/metadata checked before requesting physical storage.
        let base = store.allocate((rounded / PAGE) as usize, alignment.max(PAGE))?;
        let generation = self.allocations[slot].generation + 1;
        self.allocations[slot] = Allocation {
            generation,
            owner: Some(owner),
            base,
            requested: bytes,
            pages: (rounded / PAGE) as usize,
            maps: 0,
            writers: 0,
            executors: 0,
        };
        self.spaces[i].committed += rounded;
        self.spaces[i].requested += bytes;
        Ok(Some(AllocationId { slot, generation }))
    }
    pub fn free(&mut self, store: &mut impl Store, owner: SpaceId, id: AllocationId) -> Result<()> {
        let i = self.mutable(owner)?;
        let a = self.owned(owner, id)?;
        if a.maps != 0 {
            return Err(error("allocation", "busy"));
        }
        store.release(a.base, a.pages);
        self.spaces[i].committed -= a.pages as u64 * PAGE;
        self.spaces[i].requested -= a.requested;
        self.allocations[id.slot].owner = None;
        Ok(())
    }
    pub fn realloc(
        &mut self,
        store: &mut impl Store,
        owner: SpaceId,
        id: AllocationId,
        bytes: u64,
        alignment: u64,
    ) -> Result<Option<AllocationId>> {
        self.mutable(owner)?;
        let a = self.owned(owner, id)?;
        if a.maps != 0 {
            return Err(error("allocation", "busy"));
        }
        // Even realloc-to-zero validates alignment. Old run survives any failure.
        let new = self.allocate(store, owner, bytes, alignment)?;
        if let Some(new) = new {
            let b = self.allocation(new)?;
            for offset in 0..a.requested.min(bytes) {
                let byte = store.read(a.base, offset);
                store.write(b.base, offset, byte);
            }
        }
        self.free(store, owner, id)?;
        Ok(new)
    }
    pub fn write_staging(
        &self,
        store: &mut impl Store,
        owner: SpaceId,
        id: AllocationId,
        offset: u64,
        bytes: &[u8],
    ) -> Result<()> {
        let a = self.owned(owner, id)?;
        if a.maps != 0 || self.spaces[owner.slot].pins != 0 {
            return Err(error("allocation", "busy"));
        }
        if offset
            .checked_add(bytes.len() as u64)
            .is_none_or(|n| n > a.requested)
        {
            return Err(error("allocation", "range"));
        }
        for (n, &b) in bytes.iter().enumerate() {
            store.write(a.base, offset + n as u64, b);
        }
        Ok(())
    }
    /// Trusted internal sharing only. P3-06 adds public rights and backing refs.
    /// Whole-allocation W^X is conservative, including disjoint subranges.
    pub fn map(
        &mut self,
        store: &mut impl Store,
        space: SpaceId,
        allocation: AllocationId,
        address: u64,
        offset: u64,
        bytes: u64,
        access: Access,
    ) -> Result<()> {
        let i = self.mutable(space)?;
        let a = self.allocation(allocation)?;
        let range = user_range(address, bytes)?;
        if offset % PAGE != 0
            || offset
                .checked_add(bytes)
                .is_none_or(|n| n > a.pages as u64 * PAGE)
        {
            return Err(error("mapping", "backing_range"));
        }
        if self.spaces[i].maps.iter().flatten().any(|m| {
            range.overlaps(Range {
                base: m.address,
                len: m.bytes,
            })
        }) {
            return Err(error("mapping", "conflict"));
        }
        // Reserve the stack guard holes even before the stack is installed.
        if range.overlaps(Range {
            base: STACK_BASE - PAGE,
            len: PAGE,
        }) || range.overlaps(Range {
            base: STACK_BASE + STACK_BYTES,
            len: PAGE,
        }) {
            return Err(error("mapping", "guard"));
        }
        if access == Access::Write && a.executors != 0
            || access == Access::Execute && a.writers != 0
        {
            return Err(error("mapping", "alias_wx"));
        }
        if a.owner != Some(space) && bytes > PROCESS_LIMIT - self.spaces[i].committed {
            return Err(error("mapping", "quota"));
        }
        let slot = self.spaces[i]
            .maps
            .iter()
            .position(Option::is_none)
            .ok_or(error("mapping", "quota"))?;
        let mapping = Mapping {
            allocation,
            address,
            offset,
            bytes,
            access,
        };
        self.rebuild(store, i, slot, Some(mapping))?;
        if a.owner != Some(space) {
            self.spaces[i].committed += bytes;
        }
        self.ref_change(mapping, true);
        Ok(())
    }
    fn ref_change(&mut self, m: Mapping, add: bool) {
        let a = &mut self.allocations[m.allocation.slot];
        let update = |n: &mut usize| if add { *n += 1 } else { *n -= 1 };
        update(&mut a.maps);
        if m.access == Access::Write {
            update(&mut a.writers);
        }
        if m.access == Access::Execute {
            update(&mut a.executors);
        }
    }
    fn mapping_slot(&self, i: usize, base: u64) -> Result<usize> {
        self.spaces[i]
            .maps
            .iter()
            .position(|m| m.is_some_and(|m| m.address == base))
            .ok_or(error("mapping", "absent"))
    }
    pub fn unmap(&mut self, store: &mut impl Store, space: SpaceId, base: u64) -> Result<()> {
        let i = self.mutable(space)?;
        let slot = self.mapping_slot(i, base)?;
        let old = self.spaces[i].maps[slot].unwrap();
        self.rebuild(store, i, slot, None)?;
        if self.allocations[old.allocation.slot].owner != Some(space) {
            self.spaces[i].committed -= old.bytes;
        }
        self.ref_change(old, false);
        Ok(())
    }
    pub fn protect(
        &mut self,
        store: &mut impl Store,
        space: SpaceId,
        base: u64,
        access: Access,
    ) -> Result<()> {
        let i = self.mutable(space)?;
        let slot = self.mapping_slot(i, base)?;
        let old = self.spaces[i].maps[slot].unwrap();
        let a = self.allocation(old.allocation)?;
        if access == Access::Write && a.executors - usize::from(old.access == Access::Execute) != 0
            || access == Access::Execute
                && a.writers - usize::from(old.access == Access::Write) != 0
        {
            return Err(error("mapping", "alias_wx"));
        }
        let new = Mapping { access, ..old };
        self.rebuild(store, i, slot, Some(new))?;
        self.ref_change(old, false);
        self.ref_change(new, true);
        Ok(())
    }
    fn rebuild(
        &mut self,
        store: &mut impl Store,
        i: usize,
        slot: usize,
        replacement: Option<Mapping>,
    ) -> Result<()> {
        let bank = 1 - self.spaces[i].bank;
        let physical = self.spaces[i].tables[bank].root();
        self.spaces[i].tables[bank].initialize(physical)?;
        self.spaces[i].tables[bank].inherit_kernel(&self.kernel)?;
        for n in 0..M {
            let m = if n == slot {
                replacement
            } else {
                self.spaces[i].maps[n]
            };
            if let Some(m) = m {
                let a = self.allocation(m.allocation)?;
                for offset in (0..m.bytes).step_by(PAGE as usize) {
                    self.spaces[i].tables[bank].map_user(
                        VirtAddr(m.address + offset),
                        PhysAddr(a.base.0 + m.offset + offset),
                        m.access.permission(),
                    )?;
                }
            }
        }
        // No fallible operation follows publication. New root includes shared
        // supervisor mappings. Adapter reloads CR3 if this space is active.
        let old = self.root_index(i);
        store.publish(old, physical);
        self.spaces[i].bank = bank;
        self.spaces[i].maps[slot] = replacement;
        Ok(())
    }
    fn root_index(&self, i: usize) -> PhysAddr {
        self.spaces[i].tables[self.spaces[i].bank].root()
    }
    pub fn root(&self, space: SpaceId) -> Result<PhysAddr> {
        Ok(self.root_index(self.space(space)?))
    }
    pub fn tables(&self, space: SpaceId) -> Result<&Tables<T>> {
        let i = self.space(space)?;
        Ok(&self.spaces[i].tables[self.spaces[i].bank])
    }
    pub fn backing(&self, owner: SpaceId, id: AllocationId) -> Result<(PhysAddr, u64)> {
        let a = self.owned(owner, id)?;
        Ok((a.base, a.pages as u64 * PAGE))
    }
    pub fn pin(
        &mut self,
        space: SpaceId,
        address: u64,
        bytes: u64,
        write: bool,
    ) -> Result<CopyPin> {
        let i = self.space(space)?;
        if bytes != 0 {
            let end = address.checked_add(bytes).ok_or(error("copy", "range"))?;
            if address < 0x10000 || end > USER_END {
                return Err(error("copy", "range"));
            }
            let mut at = address;
            while at < end {
                let m = self.spaces[i]
                    .maps
                    .iter()
                    .flatten()
                    .find(|m| at >= m.address && at < m.address + m.bytes)
                    .ok_or(error("copy", "absent"))?;
                if write && m.access != Access::Write {
                    return Err(error("copy", "permission"));
                }
                at = (m.address + m.bytes).min(end);
            }
        }
        let slot = self
            .pins
            .iter()
            .position(|p| p.space.is_none() && p.generation != u64::MAX)
            .ok_or(error("copy", "quota"))?;
        let generation = self.pins[slot].generation + 1;
        self.pins[slot] = Pin {
            generation,
            space: Some(space),
        };
        self.spaces[i].pins += 1;
        Ok(CopyPin { slot, generation })
    }
    pub fn unpin(&mut self, pin: CopyPin) -> Result<()> {
        let p = self.pins.get_mut(pin.slot).ok_or(error("copy", "stale"))?;
        if p.generation != pin.generation || p.space.is_none() {
            return Err(error("copy", "stale"));
        }
        let space = p.space.take().unwrap();
        self.spaces[space.slot].pins -= 1;
        Ok(())
    }
    pub fn destroy(&mut self, store: &mut impl Store, space: SpaceId) -> Result<()> {
        let i = self.mutable(space)?;
        // Shared backing survives its owner. Owner teardown is Busy until every
        // foreign mapping is gone; no dangling owner ID or implicit revocation.
        for (n, a) in self
            .allocations
            .iter()
            .enumerate()
            .filter(|(_, a)| a.owner == Some(space))
        {
            let local = self.spaces[i]
                .maps
                .iter()
                .flatten()
                .filter(|m| m.allocation.slot == n)
                .count();
            if a.maps != local {
                return Err(error("space", "shared_busy"));
            }
        }
        // Build one empty root, rather than fallible per-mapping teardown.
        let bank = 1 - self.spaces[i].bank;
        let root = self.spaces[i].tables[bank].root();
        self.spaces[i].tables[bank].initialize(root)?;
        self.spaces[i].tables[bank].inherit_kernel(&self.kernel)?;
        store.publish(self.root_index(i), root);
        store.retire(root);
        self.spaces[i].bank = bank;
        for n in 0..M {
            if let Some(m) = self.spaces[i].maps[n].take() {
                if self.allocations[m.allocation.slot].owner != Some(space) {
                    self.spaces[i].committed -= m.bytes;
                }
                self.ref_change(m, false);
            }
        }
        for n in 0..A {
            if self.allocations[n].owner == Some(space) {
                let id = AllocationId {
                    slot: n,
                    generation: self.allocations[n].generation,
                };
                self.free(store, space, id)?;
            }
        }
        self.spaces[i].live = false;
        if self.spaces[i].kernel_stack.take().is_some() {
            store.release_kernel_stack(i);
        }
        Ok(())
    }
    pub fn inventory(&self) -> Inventory {
        let mut v = Inventory {
            spaces: 0,
            allocations: 0,
            requested: 0,
            committed: 0,
            resident: 0,
            mappings: 0,
            mapped_bytes: 0,
            page_tables: 0,
            metadata_bytes: core::mem::size_of::<Self>()
                - S * 2 * core::mem::size_of::<Tables<T>>(),
            table_storage_bytes: S * 2 * core::mem::size_of::<Tables<T>>(),
            user_guard_bytes: 0,
            active_pins: 0,
            kernel_stack_bytes: 0,
            kernel_guard_bytes: 0,
        };
        for s in self.spaces.iter().filter(|s| s.live) {
            v.spaces += 1;
            v.requested += s.requested;
            v.committed += s.committed;
            v.active_pins += s.pins;
            v.page_tables += s.tables[s.bank].used;
            v.user_guard_bytes += 2 * PAGE;
            if s.kernel_stack.is_some() {
                v.kernel_stack_bytes += STACK_BYTES;
                v.kernel_guard_bytes += 2 * PAGE;
            }
            for m in s.maps.iter().flatten() {
                v.mappings += 1;
                v.mapped_bytes += m.bytes;
            }
        }
        for a in self.allocations.iter().filter(|a| a.owner.is_some()) {
            v.allocations += 1;
            v.resident += a.pages as u64 * PAGE;
        }
        v
    }
}
pub fn user_range(address: u64, bytes: u64) -> Result<Range> {
    let range = Range::new(address, bytes, "mapping")?;
    if address % PAGE != 0 || bytes % PAGE != 0 || address < 0x10000 || range.end() > USER_END {
        return Err(error("mapping", "user_range"));
    }
    Ok(range)
}

#[cfg(test)]
mod tests;
