//! Bounded, allocation-free ownership model and four-level 4 KiB mapping builder.
//! No hardware access here. The guest adapter owns zeroing and TLB invalidation.
use crate::{PhysAddr, Range, Region, Result, VirtAddr, error, virtual_range};
use core::cell::UnsafeCell;
pub const PAGE: u64 = 4096;
pub const LIMIT: u64 = 1 << 32;
const WORDS: usize = (LIMIT / PAGE / 64) as usize;
pub fn page(address: u64) -> Result<()> {
    if address % PAGE != 0 {
        return Err(error("page", "alignment"));
    }
    Ok(())
}
pub fn inward(r: Range) -> Result<Range> {
    r.physical("range")?;
    let base = r
        .base
        .checked_add(PAGE - 1)
        .ok_or(error("range", "overflow"))?
        & !(PAGE - 1);
    let end = r.end() & !(PAGE - 1);
    Ok(Range {
        base,
        len: end.saturating_sub(base),
    })
}
pub struct Frames {
    eligible: [u64; WORDS],
    live: [u64; WORDS],
    pub total: usize,
    pub allocated: usize,
    ready: bool,
    quota: usize,
}
impl Frames {
    pub const EMPTY: Self = Self {
        eligible: [0; WORDS],
        live: [0; WORDS],
        total: 0,
        allocated: 0,
        ready: false,
        quota: usize::MAX,
    };
    /// A failed setup is unusable; no partially validated map is published.
    pub fn initialize(&mut self, regions: &[Region]) -> Result<()> {
        self.ready = false;
        self.total = 0;
        self.allocated = 0;
        self.quota = usize::MAX;
        self.eligible.fill(0);
        self.live.fill(0);
        if regions.is_empty() || regions.len() > crate::MAX_REGIONS {
            return Err(error("frames", "metadata_capacity"));
        }
        for (i, r) in regions.iter().enumerate() {
            r.range.physical("frames")?;
            if r.kind == 0 && r.range.end() > LIMIT {
                return Err(error("frames", "address_capacity"));
            }
            for other in &regions[..i] {
                if r.range.overlaps(other.range) && (r.kind == 0 || other.kind == 0) {
                    return Err(error("frames", "overlap"));
                }
            }
        }
        for r in regions.iter().filter(|r| r.kind == 0) {
            let r = inward(r.range)?;
            for i in r.base / PAGE..r.end() / PAGE {
                self.eligible[i as usize / 64] |= 1 << (i % 64);
                self.total += 1;
            }
        }
        // Physical page zero never becomes an ordinary allocation.
        self.reserve_inner(Range { base: 0, len: PAGE });
        Ok(())
    }
    fn reserve_inner(&mut self, r: Range) {
        let start = (r.base / PAGE).min(LIMIT / PAGE);
        let end = r.end().div_ceil(PAGE).min(LIMIT / PAGE);
        for i in start..end {
            let word = &mut self.eligible[i as usize / 64];
            let bit = 1 << (i % 64);
            if *word & bit != 0 {
                *word &= !bit;
                self.total -= 1;
            }
        }
    }
    pub fn reserve(&mut self, r: Range) -> Result<()> {
        if self.ready {
            return Err(error("frames", "sealed"));
        }
        r.physical("reservation")?;
        self.reserve_inner(r);
        Ok(())
    }
    pub fn seal(&mut self) {
        self.ready = true;
    }
    pub fn eligible(&self, p: PhysAddr) -> bool {
        p.0 < LIMIT
            && p.0 % PAGE == 0
            && self.eligible[(p.0 / PAGE / 64) as usize] & (1 << (p.0 / PAGE % 64)) != 0
    }
    /// Test quota is explicit and may only be set before allocations.
    pub fn set_test_quota(&mut self, quota: usize) -> Result<()> {
        if self.allocated != 0 {
            return Err(error("frames", "busy"));
        }
        self.quota = quota;
        Ok(())
    }
    pub fn allocate(&mut self, zero: impl FnOnce(PhysAddr) -> Result<()>) -> Result<PhysAddr> {
        if !self.ready {
            return Err(error("frames", "unsealed"));
        }
        if self.allocated >= self.quota {
            return Err(error("frames", "exhausted"));
        }
        for i in 0..WORDS {
            let available = self.eligible[i] & !self.live[i];
            if available != 0 {
                let bit = available.trailing_zeros();
                let p = PhysAddr((i as u64 * 64 + u64::from(bit)) * PAGE);
                zero(p)?; // A zeroing failure must not publish an allocation.
                self.live[i] |= 1u64 << bit;
                self.allocated += 1;
                return Ok(p);
            }
        }
        Err(error("frames", "exhausted"))
    }
    pub fn release(&mut self, p: PhysAddr) -> Result<()> {
        page(p.0)?;
        if !self.ready || !self.eligible(p) {
            return Err(error("frames", "not_owned"));
        }
        let index = (p.0 / PAGE / 64) as usize;
        let bit = 1 << (p.0 / PAGE % 64);
        if self.live[index] & bit == 0 {
            return Err(error("frames", "double_free"));
        }
        self.live[index] &= !bit;
        self.allocated -= 1;
        Ok(())
    }
    pub fn each_eligible(&self, mut visit: impl FnMut(PhysAddr) -> Result<()>) -> Result<()> {
        if !self.ready {
            return Err(error("frames", "unsealed"));
        }
        for (i, &word) in self.eligible.iter().enumerate() {
            let mut bits = word;
            while bits != 0 {
                let bit = bits.trailing_zeros();
                visit(PhysAddr((i as u64 * 64 + u64::from(bit)) * PAGE))?;
                bits &= bits - 1;
            }
        }
        Ok(())
    }
}
pub const PRESENT: u64 = 1;
pub const WRITE: u64 = 2;
pub const NX: u64 = 1 << 63;
const ADDRESS: u64 = 0x000f_ffff_ffff_f000;
#[derive(Clone, Copy)]
pub enum Permission {
    ReadExecute,
    ReadOnly,
    ReadWrite,
}
impl Permission {
    pub fn bits(self) -> u64 {
        PRESENT
            | match self {
                Self::ReadExecute => 0,
                Self::ReadOnly => NX,
                Self::ReadWrite => WRITE | NX,
            }
    }
}
#[repr(C, align(4096))]
pub struct Tables<const N: usize> {
    // Hardware may update accessed/dirty bits while these tables are active.
    // Never create Rust references into paging entries; use aligned volatile IO.
    entries: [UnsafeCell<[u64; 512]>; N],
    pub used: usize,
    physical: u64,
}
impl<const N: usize> Tables<N> {
    pub const EMPTY: Self = Self {
        entries: [const { UnsafeCell::new([0; 512]) }; N],
        used: 0,
        physical: 0,
    };
    pub fn initialize(&mut self, physical: PhysAddr) -> Result<()> {
        page(physical.0)?;
        Range::new(physical.0, N as u64 * PAGE, "tables")?.physical("tables")?;
        if N == 0 {
            return Err(error("tables", "capacity"));
        }
        self.physical = physical.0;
        self.clear(0);
        self.used = 1;
        Ok(())
    }
    pub fn root(&self) -> PhysAddr {
        PhysAddr(self.physical)
    }
    fn read(&self, table: usize, index: usize) -> u64 {
        // Indices originate only from checked table offsets and 9-bit indices.
        unsafe {
            self.entries[table]
                .get()
                .cast::<u64>()
                .add(index)
                .read_volatile()
        }
    }
    fn write(&mut self, table: usize, index: usize, value: u64) {
        unsafe {
            self.entries[table]
                .get()
                .cast::<u64>()
                .add(index)
                .write_volatile(value);
        }
    }
    fn clear(&mut self, table: usize) {
        // Only a not-yet-published table is cleared. No live mapping is erased.
        for index in 0..512 {
            self.write(table, index, 0);
        }
    }
    fn child(&self, entry: u64) -> Result<usize> {
        if entry & (PRESENT | WRITE | 4 | 128 | NX) != PRESENT | WRITE {
            return Err(error("mapping", "parent_flags"));
        }
        let offset = (entry & ADDRESS)
            .checked_sub(self.physical)
            .ok_or(error("mapping", "foreign_table"))?;
        let i = (offset / PAGE) as usize;
        if i >= self.used {
            return Err(error("mapping", "foreign_table"));
        }
        Ok(i)
    }
    fn indices(v: VirtAddr) -> Result<[usize; 4]> {
        page(v.0)?;
        virtual_range(v.0, PAGE, "mapping")?;
        Ok([39, 30, 21, 12].map(|s| ((v.0 >> s) & 511) as usize))
    }
    pub fn map(&mut self, v: VirtAddr, p: PhysAddr, permission: Permission) -> Result<()> {
        if self.used == 0 {
            return Err(error("tables", "uninitialized"));
        }
        page(p.0)?;
        Range::new(p.0, PAGE, "mapping")?.physical("mapping")?;
        let indices = Self::indices(v)?;
        let mut t = 0;
        for &index in &indices[..3] {
            let entry = self.read(t, index);
            t = if entry == 0 {
                if self.used == N {
                    return Err(error("tables", "capacity"));
                }
                let next = self.used;
                self.clear(next);
                self.used += 1;
                self.write(
                    t,
                    index,
                    self.physical + next as u64 * PAGE | PRESENT | WRITE,
                );
                next
            } else {
                self.child(entry)?
            };
        }
        if self.read(t, indices[3]) != 0 {
            return Err(error("mapping", "conflict"));
        }
        self.write(t, indices[3], p.0 | permission.bits());
        Ok(())
    }
    pub fn inspect(&self, v: VirtAddr) -> Result<u64> {
        if self.used == 0 {
            return Err(error("tables", "uninitialized"));
        }
        let indices = Self::indices(v)?;
        let mut t = 0;
        for &i in &indices[..3] {
            let entry = self.read(t, i);
            if entry == 0 {
                return Ok(0);
            }
            t = self.child(entry)?;
        }
        Ok(self.read(t, indices[3]))
    }
    /// Caller must invalidate this virtual page before reusing its old backing.
    pub fn unmap(&mut self, v: VirtAddr) -> Result<PhysAddr> {
        let old = self.inspect(v)?;
        if old & PRESENT == 0 {
            return Err(error("mapping", "absent"));
        }
        let indices = Self::indices(v)?;
        let mut t = 0;
        for &i in &indices[..3] {
            t = self.child(self.read(t, i))?;
        }
        self.write(t, indices[3], 0);
        Ok(PhysAddr(old & ADDRESS))
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    fn r(base: u64, len: u64, kind: u64) -> Region {
        Region {
            range: Range { base, len },
            kind,
        }
    }
    #[test]
    fn reservations_edges_holes_and_exhaustion() {
        let mut f = Frames::EMPTY;
        // Independently constructed unsorted map, partial edges and a hole.
        f.initialize(&[
            r(0x10000, 0x1000, 0),
            r(0x1001, 0x4ffe, 0),
            r(0x8000, 0x2000, 5),
        ])
        .unwrap();
        f.reserve(Range {
            base: 0x3001,
            len: 1,
        })
        .unwrap();
        f.reserve(Range {
            base: 0x3000,
            len: 0x1000,
        })
        .unwrap(); // union, no double accounting
        f.seal();
        let mut pages = std::vec::Vec::new();
        while let Ok(p) = f.allocate(|p| {
            assert_eq!(p.0 % PAGE, 0);
            Ok(())
        }) {
            pages.push(p.0);
        }
        assert_eq!(pages, [0x2000, 0x4000, 0x10000]);
        assert_eq!(f.total, 3);
        assert!(f.release(PhysAddr(0x8000)).is_err());
        assert!(f.release(PhysAddr(LIMIT)).is_err());
        assert!(f.release(PhysAddr(0x2001)).is_err());
        f.release(PhysAddr(0x2000)).unwrap();
        assert_eq!(
            f.release(PhysAddr(0x2000)),
            Err(error("frames", "double_free"))
        );
        assert!(f.allocate(|_| Err(error("zero", "failed"))).is_err());
        assert_eq!(f.allocated, 2);
        assert_eq!(f.allocate(|_| Ok(())).unwrap(), PhysAddr(0x2000));
    }
    #[test]
    fn malformed_capacity_and_all_reserved() {
        let mut f = Frames::EMPTY;
        for map in [
            std::vec![r(u64::MAX - 2, 8, 0)],
            std::vec![r(LIMIT, PAGE, 0)],
            std::vec![r(0x1000, PAGE, 0), r(0x1001, 1, 5)],
            std::vec![r(0x1000,PAGE,0);513],
        ] {
            assert!(f.initialize(&map).is_err());
            assert!(f.allocate(|_| Ok(())).is_err());
        }
        f.initialize(&[r(0, PAGE, 0), r(0x1000, PAGE, 5)]).unwrap();
        f.seal();
        assert_eq!(f.total, 0);
        assert_eq!(f.allocate(|_| Ok(())), Err(error("frames", "exhausted")));
        assert!(
            inward(Range {
                base: u64::MAX,
                len: 1
            })
            .is_err()
        );
    }
    #[test]
    fn mapping_conflicts_capacity_and_permissions() {
        let mut t = Tables::<8>::EMPTY;
        t.initialize(PhysAddr(0x100000)).unwrap();
        let v = VirtAddr(0xffff_8000_0000_0000);
        t.map(v, PhysAddr(0x1000), Permission::ReadExecute).unwrap();
        assert_eq!(t.inspect(v).unwrap(), 0x1001);
        assert!(t.map(v, PhysAddr(0x2000), Permission::ReadWrite).is_err());
        assert_eq!(t.unmap(v).unwrap(), PhysAddr(0x1000));
        assert!(t.unmap(v).is_err());
        t.map(v, PhysAddr(0x2000), Permission::ReadWrite).unwrap();
        assert_eq!(t.inspect(v).unwrap(), 0x2003 | NX);
        assert!(
            t.map(VirtAddr(0x800000000000), PhysAddr(0), Permission::ReadOnly)
                .is_err()
        );
        assert!(
            t.map(VirtAddr(1), PhysAddr(0), Permission::ReadOnly)
                .is_err()
        );
        assert!(
            t.map(VirtAddr(0), PhysAddr(1), Permission::ReadOnly)
                .is_err()
        );
        let mut small = Tables::<3>::EMPTY;
        small.initialize(PhysAddr(0x1000)).unwrap();
        assert_eq!(
            small.map(VirtAddr(0), PhysAddr(0), Permission::ReadOnly),
            Err(error("tables", "capacity"))
        );
        assert_eq!(small.inspect(VirtAddr(0)).unwrap(), 0);
    }
}
