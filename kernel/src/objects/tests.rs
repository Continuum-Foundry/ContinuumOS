extern crate std;
mod channels;
mod waiting;
use super::*;
use crate::{
    PhysAddr, Range, Result as MemoryResult, error,
    memory::PAGE,
    owned_memory::{STACK_BASE, STACK_BYTES},
};
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
    fn allocate(&mut self, pages: usize, alignment: u64) -> MemoryResult<PhysAddr> {
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
    fn prepare_kernel_stack(&mut self, slot: usize) -> MemoryResult<Range> {
        Ok(Range {
            base: 0xffff_c100_0000_1000 + slot as u64 * 73728,
            len: STACK_BYTES,
        })
    }
    fn release_kernel_stack(&mut self, _: usize) {}
}

type R = Runtime<3, 64, 48, 16>;
fn setup() -> (Box<R>, Ram, Registry) {
    let mut r = Box::new(R::EMPTY);
    r.initialize([0; 256], |p| Ok(PhysAddr(p as u64))).unwrap();
    (r, Ram::new(), Registry::EMPTY)
}
#[test]
fn rights_types_stale_cross_process_and_nonce_exhaustion() {
    let (mut r, mut b, mut c) = setup();
    let s = r.create().unwrap();
    let other = r.create().unwrap();
    let (p, t) = c.admit(s).unwrap();
    let (q, _) = c.admit(other).unwrap();
    assert_eq!(c.info(other, p), Err(Error::BadHandle));
    assert_eq!(c.info(s, q), Err(Error::BadHandle));
    for h in [0, 1, u64::MAX, p ^ (1 << 32)] {
        assert_eq!(c.info(s, h), Err(Error::BadHandle));
    }
    assert_eq!(
        c.create(&mut r, &mut b, s, t, 4096, READ),
        Err(Error::WrongType)
    );
    let narrow = c.duplicate(s, p, QUERY | CLOSE).unwrap();
    assert_eq!(c.duplicate(s, narrow, QUERY), Err(Error::AccessDenied));
    assert_eq!(
        c.create(&mut r, &mut b, s, narrow, 4096, READ),
        Err(Error::AccessDenied)
    );
    assert_eq!(c.duplicate(s, p, u64::MAX), Err(Error::AccessDenied));
    let h = c.duplicate(s, p, 0).unwrap();
    assert_eq!(c.info(s, h), Err(Error::AccessDenied));
    assert_eq!(c.close(&mut r, &mut b, s, h), Err(Error::AccessDenied));
    c.close(&mut r, &mut b, s, narrow).unwrap();
    assert_eq!(c.close(&mut r, &mut b, s, narrow), Err(Error::BadHandle));
    let new = c.duplicate(s, p, QUERY | CLOSE).unwrap();
    assert_ne!(new, narrow);
    assert_eq!(c.info(s, narrow), Err(Error::BadHandle));
    c.next_nonce = u32::MAX;
    let last = c.duplicate(s, p, BASIC).unwrap();
    assert_eq!(last >> 32, u32::MAX as u64);
    c.close(&mut r, &mut b, s, last).unwrap();
    assert_eq!(c.duplicate(s, p, BASIC), Err(Error::QuotaExceeded));
    c.remove(&mut r, &mut b, s).unwrap();
    c.remove(&mut r, &mut b, other).unwrap();
    r.destroy(&mut b, s).unwrap();
    r.destroy(&mut b, other).unwrap();
    assert_eq!(c.inventory().objects, 0);
    assert!(b.pages.is_empty());
    let s = r.create().unwrap();
    assert_eq!(c.admit(s), Err(Error::QuotaExceeded));
}
#[test]
fn immutable_mapping_outlives_creator_and_all_handles_with_pin_barriers() {
    let (mut r, mut b, mut c) = setup();
    let s = r.create_with_stack(&mut b).unwrap();
    let q = r.create_with_stack(&mut b).unwrap();
    let (p, _) = c.admit(s).unwrap();
    let (process, _) = c.admit(q).unwrap();
    let h = c.create(&mut r, &mut b, s, p, 8192, READ).unwrap();
    c.seed(&r, &mut b, s, h, b"retained").unwrap();
    let grant = c.grant_read(s, h, q).unwrap();
    assert_eq!(
        c.map(
            &mut r,
            &mut b,
            q,
            process,
            grant,
            0x400000,
            0,
            8192,
            Access::Write
        ),
        Err(Error::AccessDenied)
    );
    c.map(
        &mut r,
        &mut b,
        q,
        process,
        grant,
        0x400000,
        0,
        8192,
        Access::Read,
    )
    .unwrap();
    c.close(&mut r, &mut b, s, h).unwrap();
    c.remove(&mut r, &mut b, s).unwrap();
    r.destroy(&mut b, s).unwrap();
    assert_eq!(
        c.info(q, grant).unwrap(),
        (Kind::Memory as u64) << 32 | BASIC | MAP | READ
    );
    assert_eq!(r.inventory().resident, STACK_BYTES + 8192);
    assert_eq!(r.inventory().committed, STACK_BYTES + 8192 + 8192); // orphan backing plus mapping
    c.close(&mut r, &mut b, q, grant).unwrap();
    assert_eq!(c.inventory().backing_bytes, 8192); // mapping has an independent ref
    for a in [Access::Write, Access::Execute] {
        assert_eq!(
            c.protect(&mut r, &mut b, q, process, 0x400000, a),
            Err(Error::AccessDenied)
        );
    }
    let pin = r.pin(q, 0x400000, 8, false).unwrap();
    let before = (r.inventory(), c.inventory());
    assert_eq!(
        c.unmap(&mut r, &mut b, q, process, 0x400000),
        Err(Error::WouldBlock)
    );
    assert_eq!(c.remove(&mut r, &mut b, q), Err(Error::WouldBlock));
    assert_eq!((r.inventory(), c.inventory()), before);
    r.unpin(pin).unwrap();
    c.unmap(&mut r, &mut b, q, process, 0x400000).unwrap();
    assert_eq!(c.inventory().backing_bytes, 0);
    c.remove(&mut r, &mut b, q).unwrap();
    r.destroy(&mut b, q).unwrap();
    assert!(b.pages.is_empty());
    assert_eq!(c.inventory(), Registry::EMPTY.inventory());
}
#[test]
fn quotas_unwind_aliases_and_mapping_authority() {
    let (mut r, mut b, mut c) = setup();
    let s = r.create().unwrap();
    let (p, t) = c.admit(s).unwrap();
    let initial = (r.inventory(), c.inventory());
    b.fail = true;
    assert_eq!(
        c.create(&mut r, &mut b, s, p, 4096, READ),
        Err(Error::OutOfMemory)
    );
    assert_eq!((r.inventory(), c.inventory()), initial);
    b.fail = false;
    for (n, flags) in [
        (0, READ),
        (4097, READ),
        (4096, READ | WRITE | EXECUTE),
        (u64::MAX, READ),
        (4096, u64::MAX),
    ] {
        assert_eq!(
            c.create(&mut r, &mut b, s, p, n, flags),
            Err(Error::InvalidArgument)
        );
        assert_eq!((r.inventory(), c.inventory()), initial);
    }
    let h = c.create(&mut r, &mut b, s, p, 8192, READ | WRITE).unwrap();
    assert_eq!(c.grant_read(s, h, s), Err(Error::AccessDenied));
    assert_eq!(
        c.map(&mut r, &mut b, s, t, h, 0x400000, 0, 4096, Access::Read),
        Err(Error::WrongType)
    );
    let n = c.duplicate(s, h, BASIC | MAP | READ).unwrap();
    assert_eq!(
        c.map(&mut r, &mut b, s, p, n, 0x400000, 0, 4096, Access::Write),
        Err(Error::AccessDenied)
    );
    c.map(&mut r, &mut b, s, p, n, 0x400000, 0, 4096, Access::Read)
        .unwrap();
    c.close(&mut r, &mut b, s, n).unwrap();
    assert_eq!(
        c.protect(&mut r, &mut b, s, p, 0x400000, Access::Write),
        Err(Error::AccessDenied)
    );
    for (base, off, len) in [
        (0x400000, 0, 4096),
        (0xffff800000000000, 0, 4096),
        (STACK_BASE - 4096, 0, 4096),
        (0x600000, u64::MAX - 4095, 8192),
        (0x600000, 8192, 4096),
        (0x600001, 0, 4096),
    ] {
        let before = (r.inventory(), c.inventory());
        assert!(
            c.map(&mut r, &mut b, s, p, h, base, off, len, Access::Read)
                .is_err()
        );
        assert_eq!((r.inventory(), c.inventory()), before);
    }
    let mut values = std::vec::Vec::new();
    while let Ok(v) = c.duplicate(s, h, BASIC) {
        values.push(v);
    }
    let full = (r.inventory(), c.inventory());
    assert_eq!(
        c.create(&mut r, &mut b, s, p, 4096, READ),
        Err(Error::QuotaExceeded)
    );
    assert_eq!((r.inventory(), c.inventory()), full);
    for v in values {
        c.close(&mut r, &mut b, s, v).unwrap();
    }
    c.unmap(&mut r, &mut b, s, p, 0x400000).unwrap();
    for n in 0..MAPPINGS {
        c.map(
            &mut r,
            &mut b,
            s,
            p,
            h,
            0x1000000 + n as u64 * 4096,
            0,
            4096,
            Access::Read,
        )
        .unwrap();
    }
    let full = (r.inventory(), c.inventory());
    assert_eq!(
        c.map(&mut r, &mut b, s, p, h, 0x2000000, 0, 4096, Access::Read),
        Err(Error::QuotaExceeded)
    );
    assert_eq!((r.inventory(), c.inventory()), full);
    c.remove(&mut r, &mut b, s).unwrap();
    r.destroy(&mut b, s).unwrap();
    assert!(b.pages.is_empty());
}

#[test]
fn object_slot_retirement_and_page_table_failure_unwind() {
    let (mut r, mut b, mut c) = setup();
    let s = r.create().unwrap();
    let (p, _) = c.admit(s).unwrap();
    // Every vacant object slot is at its terminal generation: no resurrection.
    for o in c.objects.iter_mut().filter(|o| o.body.is_none()) {
        o.generation = u64::MAX;
    }
    let before = (r.inventory(), c.inventory());
    assert_eq!(
        c.create(&mut r, &mut b, s, p, 4096, READ),
        Err(Error::QuotaExceeded)
    );
    assert_eq!((r.inventory(), c.inventory()), before);
    c.remove(&mut r, &mut b, s).unwrap();
    r.destroy(&mut b, s).unwrap();

    let mut r = Box::new(Runtime::<1, 8, 8, 2>::EMPTY);
    r.initialize([0; 256], |p| Ok(PhysAddr(p as u64))).unwrap();
    let s = r.create().unwrap();
    let mut c = Registry::EMPTY;
    let (p, _) = c.admit(s).unwrap();
    let h = c.create(&mut r, &mut b, s, p, 4096, READ).unwrap();
    let before = (r.inventory(), c.inventory());
    assert_eq!(
        c.map(&mut r, &mut b, s, p, h, 0x400000, 0, 4096, Access::Read),
        Err(Error::QuotaExceeded)
    );
    assert_eq!((r.inventory(), c.inventory()), before);
    c.close(&mut r, &mut b, s, h).unwrap();
    c.remove(&mut r, &mut b, s).unwrap();
    r.destroy(&mut b, s).unwrap();
    assert!(b.pages.is_empty());
}

#[test]
fn closing_self_handles_preserves_admitted_owner_lifecycle() {
    let (mut r, mut b, mut c) = setup();
    let s = r.create().unwrap();
    let (p, t) = c.admit(s).unwrap();
    c.close(&mut r, &mut b, s, p).unwrap();
    c.close(&mut r, &mut b, s, t).unwrap();
    assert_eq!(c.inventory().handles, 0);
    assert_eq!(c.inventory().objects, 2);
    assert_eq!(r.inventory().spaces, 1);
    c.remove(&mut r, &mut b, s).unwrap();
    r.destroy(&mut b, s).unwrap();
    assert_eq!(c.inventory(), Registry::EMPTY.inventory());
    assert!(b.pages.is_empty());
}
