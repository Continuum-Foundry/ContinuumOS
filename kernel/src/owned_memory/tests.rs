extern crate std;
use super::*;
use std::{boxed::Box, collections::BTreeMap, vec, vec::Vec};

struct Ram {
    pages: BTreeMap<u64, Vec<u8>>,
    next: u64,
    fail: bool,
    publications: usize,
}
impl Ram {
    fn new() -> Self {
        Self {
            pages: BTreeMap::new(),
            next: 0x200000,
            fail: false,
            publications: 0,
        }
    }
}
impl Store for Ram {
    fn allocate(&mut self, pages: usize, alignment: u64) -> Result<PhysAddr> {
        if self.fail {
            return Err(error("frames", "exhausted"));
        }
        let base = self.next.div_ceil(alignment) * alignment;
        self.next = base + pages as u64 * PAGE;
        assert!(self.pages.insert(base, vec![0; pages * 4096]).is_none());
        Ok(PhysAddr(base))
    }
    fn release(&mut self, base: PhysAddr, pages: usize) {
        assert_eq!(self.pages.remove(&base.0).unwrap().len(), pages * 4096);
    }
    fn read(&self, base: PhysAddr, offset: u64) -> u8 {
        self.pages[&base.0][offset as usize]
    }
    fn write(&mut self, base: PhysAddr, offset: u64, b: u8) {
        self.pages.get_mut(&base.0).unwrap()[offset as usize] = b;
    }
    fn publish(&mut self, _: PhysAddr, _: PhysAddr) {
        self.publications += 1;
    }
    fn retire(&mut self, _: PhysAddr) {}
    fn prepare_kernel_stack(&mut self, slot: usize) -> Result<Range> {
        Ok(Range {
            base: 0xffff_c100_0000_1000 + slot as u64 * 73728,
            len: STACK_BYTES,
        })
    }
    fn release_kernel_stack(&mut self, _: usize) {}
}
type R = Runtime<2, 16, 16, 16>;
fn runtime() -> Box<R> {
    let mut r = Box::new(R::EMPTY);
    let mut kernel = [0; 256];
    kernel[511 - 256] = 0x100003;
    r.initialize(kernel, |p| Ok(PhysAddr(p as u64))).unwrap();
    r
}
fn alloc(r: &mut R, b: &mut Ram, s: SpaceId, n: u64) -> AllocationId {
    r.allocate(b, s, n, 1).unwrap().unwrap()
}
#[test]
fn alignment_realloc_zero_and_transactional_oom() {
    let mut r = runtime();
    let mut b = Ram::new();
    let s = r.create().unwrap();
    for align in [1, 2, 8, 4096, 8192, 2 * 1024 * 1024] {
        let a = r.allocate(&mut b, s, 4097, align).unwrap().unwrap();
        let (p, n) = r.backing(s, a).unwrap();
        assert_eq!(p.0 % align, 0);
        assert_eq!(n, 8192);
        assert!(b.pages[&p.0].iter().all(|b| *b == 0));
        r.write_staging(&mut b, s, a, 0, b"hello").unwrap();
        let before = r.inventory();
        b.fail = true;
        assert!(r.realloc(&mut b, s, a, 12289, align).is_err());
        assert_eq!(before, r.inventory());
        b.fail = false;
        let new = r.realloc(&mut b, s, a, 12289, align).unwrap().unwrap();
        let (p, _) = r.backing(s, new).unwrap();
        assert_eq!(&b.pages[&p.0][..5], b"hello");
        assert!(b.pages[&p.0][4097..].iter().all(|b| *b == 0));
        assert!(r.free(&mut b, s, a).is_err());
        let shrunk = r.realloc(&mut b, s, new, 3, align).unwrap().unwrap();
        let (p, _) = r.backing(s, shrunk).unwrap();
        assert_eq!(&b.pages[&p.0][..3], b"hel");
        assert!(b.pages[&p.0][3..].iter().all(|b| *b == 0));
        assert_eq!(r.realloc(&mut b, s, shrunk, 0, align).unwrap(), None);
        assert!(r.free(&mut b, s, shrunk).is_err());
    }
    let baseline = r.inventory();
    for (size, align) in [
        (1, 0),
        (1, 3),
        (1, 4 * 1024 * 1024),
        (u64::MAX, 1),
        (PROCESS_LIMIT + 1, 1),
    ] {
        assert!(r.allocate(&mut b, s, size, align).is_err());
        assert_eq!(baseline, r.inventory());
    }
    assert_eq!(r.allocate(&mut b, s, 0, 1).unwrap(), None);
    r.destroy(&mut b, s).unwrap();
    assert!(b.pages.is_empty());
}
#[test]
fn private_walk_wx_shared_backing_and_copy_interleavings() {
    let mut r = runtime();
    let mut b = Ram::new();
    let s = r.create_with_stack(&mut b).unwrap();
    let other = r.create_with_stack(&mut b).unwrap();
    let a = alloc(&mut r, &mut b, s, 8192);
    r.map(&mut b, s, a, 0x400000, 0, 8192, Access::Write)
        .unwrap();
    let pin = r.pin(s, 0x400fff, 2, true).unwrap();
    let before = r.inventory();
    assert!(r.unmap(&mut b, s, 0x400000).is_err());
    assert!(r.protect(&mut b, s, 0x400000, Access::Read).is_err());
    assert!(r.destroy(&mut b, s).is_err());
    assert!(r.free(&mut b, s, a).is_err());
    assert_eq!(before, r.inventory());
    r.unpin(pin).unwrap();
    assert!(
        r.map(&mut b, other, a, 0x400000, 0, 8192, Access::Execute)
            .is_err()
    );
    r.protect(&mut b, s, 0x400000, Access::Execute).unwrap();
    assert!(r.pin(s, 0x400000, 1, true).is_err());
    assert!(r.pin(s, 0x401fff, 2, false).is_err());
    let pin = r.pin(s, u64::MAX, 0, true).unwrap();
    r.unpin(pin).unwrap();
    let t = r.tables(s).unwrap();
    // Walk actual four-level bytes independently of Tables::inspect.
    let mut table = 0;
    for shift in [39, 30, 21] {
        let e = t
            .entry(table, ((0x400000u64 >> shift) & 511) as usize)
            .unwrap();
        assert_eq!(e & 7, 7);
        assert_eq!(e & (1 << 63), 0);
        table = (((e & 0x000f_ffff_ffff_f000) - t.root().0) / 4096) as usize;
    }
    let leaf = t.entry(table, 0).unwrap();
    assert_eq!(leaf & 7, 5);
    assert_eq!(leaf & (1 << 63), 0);
    assert_eq!(t.entry(0, 511).unwrap(), 0x100003);
    assert_eq!(
        r.tables(other)
            .unwrap()
            .inspect(VirtAddr(0x400000))
            .unwrap(),
        0
    );
    assert_eq!(t.inspect(VirtAddr(STACK_BASE - PAGE)).unwrap(), 0);
    assert_eq!(t.inspect(VirtAddr(STACK_BASE + STACK_BYTES)).unwrap(), 0);
    r.map(&mut b, other, a, 0x600000, 0, 8192, Access::Read)
        .unwrap();
    assert!(r.free(&mut b, other, a).is_err());
    assert!(r.destroy(&mut b, s).is_err());
    let pin = r.pin(other, 0x600000, 8192, false).unwrap();
    // Owner can remove its mapping, but shared pin still prevents reuse via refs.
    r.unmap(&mut b, s, 0x400000).unwrap();
    assert!(r.free(&mut b, s, a).is_err());
    r.unpin(pin).unwrap();
    r.unmap(&mut b, other, 0x600000).unwrap();
    r.destroy(&mut b, s).unwrap();
    r.destroy(&mut b, other).unwrap();
    assert!(b.pages.is_empty());
}
#[test]
fn ranges_metadata_quotas_and_poisoned_owner_tokens() {
    let mut r = runtime();
    let mut b = Ram::new();
    let s = r.create().unwrap();
    let a = alloc(&mut r, &mut b, s, 4096);
    let before = r.inventory();
    for (address, offset, bytes) in [
        (0, 0, 4096),
        (0x10001, 0, 4096),
        (0x10000, 1, 4096),
        (0x10000, 0, 0),
        (0x10000, 0, 4097),
        (0x10000, 4096, 4096),
        (USER_END - 4096, 0, 8192),
        (0x800000000000, 0, 4096),
        (u64::MAX - 4095, 0, 8192),
        (STACK_BASE - PAGE, 0, 4096),
        (STACK_BASE + STACK_BYTES, 0, 4096),
    ] {
        assert!(
            r.map(&mut b, s, a, address, offset, bytes, Access::Read)
                .is_err()
        );
        assert_eq!(before, r.inventory());
    }
    assert!(r.write_staging(&mut b, s, a, u64::MAX, b"x").is_err());
    let other = r.create().unwrap();
    assert!(r.free(&mut b, other, a).is_err());
    assert!(r.create().is_err());
    r.destroy(&mut b, s).unwrap();
    let replacement = r.create().unwrap();
    assert_ne!(s, replacement);
    assert!(r.allocate(&mut b, s, 4096, 1).is_err());
    assert!(r.free(&mut b, replacement, a).is_err());
    for _ in 0..16 {
        alloc(&mut r, &mut b, replacement, 1);
    }
    let before = r.inventory();
    assert!(r.allocate(&mut b, replacement, 1, 1).is_err());
    assert_eq!(before, r.inventory());
    r.destroy(&mut b, replacement).unwrap();
    r.destroy(&mut b, other).unwrap();
    assert!(b.pages.is_empty());
}
#[test]
fn table_failure_keeps_old_root_mapping_and_inventory() {
    let mut r = Runtime::<1, 4, 4, 4>::EMPTY;
    let mut b = Ram::new();
    r.initialize([0; 256], |p| Ok(PhysAddr(p as u64))).unwrap();
    let s = r.create().unwrap();
    let a = r.allocate(&mut b, s, 4096, 1).unwrap().unwrap();
    r.map(&mut b, s, a, 0x400000, 0, 4096, Access::Read)
        .unwrap();
    let root = r.root(s).unwrap();
    let before = r.inventory();
    let publishes = b.publications;
    assert!(
        r.map(&mut b, s, a, 0x20000000, 0, 4096, Access::Read)
            .is_err()
    );
    assert_eq!(root, r.root(s).unwrap());
    assert_eq!(before, r.inventory());
    assert_eq!(publishes, b.publications);
    assert_ne!(r.tables(s).unwrap().inspect(VirtAddr(0x400000)).unwrap(), 0);
    r.destroy(&mut b, s).unwrap();
    assert!(b.pages.is_empty());
    // Failure at stack mapping and physical allocation stages rolls back create.
    let mut small = Runtime::<1, 4, 4, 3>::EMPTY;
    small
        .initialize([0; 256], |p| Ok(PhysAddr(p as u64)))
        .unwrap();
    let empty = small.inventory();
    assert!(small.create_with_stack(&mut b).is_err());
    assert_eq!(empty, small.inventory());
    assert!(b.pages.is_empty());
    b.fail = true;
    assert!(small.create_with_stack(&mut b).is_err());
    assert_eq!(empty, small.inventory());
}
#[test]
fn every_parent_table_stage_and_mapping_quota_rolls_back() {
    fn case<const T: usize>() {
        let mut r = Runtime::<1, 4, 4, T>::EMPTY;
        let mut b = Ram::new();
        r.initialize([0; 256], |p| Ok(PhysAddr(p as u64))).unwrap();
        let s = r.create().unwrap();
        let a = r.allocate(&mut b, s, 4096, 1).unwrap().unwrap();
        let before = r.inventory();
        let root = r.root(s).unwrap();
        if T < 4 {
            assert!(
                r.map(&mut b, s, a, 0x400000, 0, 4096, Access::Read)
                    .is_err()
            );
            assert_eq!(before, r.inventory());
            assert_eq!(root, r.root(s).unwrap());
        } else {
            r.map(&mut b, s, a, 0x400000, 0, 4096, Access::Read)
                .unwrap();
        }
        r.destroy(&mut b, s).unwrap();
        assert!(b.pages.is_empty());
    }
    case::<1>();
    case::<2>();
    case::<3>();
    case::<4>();
    let mut r = Runtime::<2, 4, 1, 16>::EMPTY;
    let mut b = Ram::new();
    r.initialize([0; 256], |p| Ok(PhysAddr(p as u64))).unwrap();
    let s = r.create().unwrap();
    let a = r.allocate(&mut b, s, 4096, 1).unwrap().unwrap();
    r.map(&mut b, s, a, 0x400000, 0, 4096, Access::Read)
        .unwrap();
    let before = r.inventory();
    assert!(
        r.map(&mut b, s, a, 0x500000, 0, 4096, Access::Read)
            .is_err()
    );
    assert_eq!(before, r.inventory());
    r.destroy(&mut b, s).unwrap();
    assert!(b.pages.is_empty());
}
#[test]
fn generation_wrap_retires_slots_and_precharge_is_global() {
    let mut r = runtime();
    let mut b = Ram::new();
    r.spaces[0].generation = u64::MAX;
    let s = r.create().unwrap();
    assert_eq!(s.slot, 1);
    assert!(r.create().is_err());
    r.allocations[0].generation = u64::MAX;
    let a = alloc(&mut r, &mut b, s, 4096);
    assert_eq!(a.slot, 1);
    r.destroy(&mut b, s).unwrap();
    assert!(b.pages.is_empty());
}
#[test]
fn fixed_storage_repeated_settlement_and_pin_capacity() {
    let mut r = runtime();
    let mut b = Ram::new();
    let empty = r.inventory();
    for _ in 0..1000 {
        let s = r.create_with_stack(&mut b).unwrap();
        let a = alloc(&mut r, &mut b, s, 1);
        r.map(&mut b, s, a, 0x400000, 0, 4096, Access::Write)
            .unwrap();
        r.destroy(&mut b, s).unwrap();
        assert_eq!(empty, r.inventory());
        assert!(b.pages.is_empty());
    }
    let s = r.create().unwrap();
    let pins: Vec<_> = (0..32)
        .map(|_| r.pin(s, u64::MAX, 0, false).unwrap())
        .collect();
    assert!(r.pin(s, 0, 0, false).is_err());
    for pin in pins {
        r.unpin(pin).unwrap();
    }
    r.destroy(&mut b, s).unwrap();
    assert_eq!(empty, r.inventory());
}
