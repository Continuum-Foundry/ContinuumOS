extern crate std;
use super::*;
use crate::{PhysAddr, Range, VirtAddr};
use std::{boxed::Box, collections::BTreeMap, vec, vec::Vec};
fn w16(b: &mut [u8], o: usize, n: u16) {
    b[o..o + 2].copy_from_slice(&n.to_le_bytes());
}
fn w32(b: &mut [u8], o: usize, n: u32) {
    b[o..o + 4].copy_from_slice(&n.to_le_bytes());
}
fn w64(b: &mut [u8], o: usize, n: u64) {
    b[o..o + 8].copy_from_slice(&n.to_le_bytes());
}
fn image() -> Vec<u8> {
    let mut b = vec![0; 0x4000];
    b[..16].copy_from_slice(b"\x7fELF\x02\x01\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00");
    w16(&mut b, 16, 2);
    w16(&mut b, 18, 62);
    w32(&mut b, 20, 1);
    w64(&mut b, 24, 0x400000);
    w64(&mut b, 32, 64);
    w64(&mut b, 40, 0x3000);
    w16(&mut b, 52, 64);
    w16(&mut b, 54, 56);
    w16(&mut b, 56, 3);
    w16(&mut b, 58, 64);
    w16(&mut b, 60, 1);
    for (i, flags) in [5, 4, 6].into_iter().enumerate() {
        let h = 64 + i * 56;
        w32(&mut b, h, 1);
        w32(&mut b, h + 4, flags);
        w64(&mut b, h + 8, (i as u64 + 1) * 0x1000);
        w64(&mut b, h + 16, 0x400000 + i as u64 * 0x1000);
        w64(&mut b, h + 24, 0x400000 + i as u64 * 0x1000);
        w64(&mut b, h + 32, 8);
        w64(&mut b, h + 40, if i == 2 { 8193 } else { 8 });
        w64(&mut b, h + 48, 4096);
        b[(i + 1) * 4096..(i + 1) * 4096 + 8].fill(i as u8 + 1);
    }
    // Keep the section table separate from the data bytes.
    w64(&mut b, 40, 0x3800);
    b
}
#[test]
fn sha_known_vectors() {
    assert_eq!(
        sha256(b""),
        [
            0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14, 0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f,
            0xb9, 0x24, 0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c, 0xa4, 0x95, 0x99, 0x1b,
            0x78, 0x52, 0xb8, 0x55
        ]
    );
    assert_eq!(
        sha256(b"abc"),
        [
            0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae,
            0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61,
            0xf2, 0x00, 0x15, 0xad
        ]
    );
    assert_eq!(&sha256(&[b'a'; 1000000])[..4], &[0xcd, 0xc7, 0x6e, 0x5c]);
}
#[test]
fn parser_mutation_matrix() {
    let good = image();
    assert_eq!(parse(&good).unwrap().count, 3);
    let mut gnu = good.clone();
    gnu[7] = 3;
    assert!(parse(&gnu).is_ok());
    let mut cases: Vec<(Vec<u8>, &str)> = Vec::new();
    for o in [0, 4, 5, 6, 7, 8, 15] {
        let mut b = good.clone();
        b[o] ^= 1;
        cases.push((b, "identity"));
    }
    for (o, v) in [
        (16, 3),
        (18, 3),
        (52, 63),
        (54, 55),
        (58, 63),
        (56, 0),
        (56, 17),
        (60, 0),
        (60, 129),
        (62, 1),
    ] {
        let mut b = good.clone();
        w16(&mut b, o, v);
        cases.push((
            b,
            if o < 32 {
                "target"
            } else if [52, 54, 58].contains(&o) {
                "header_size"
            } else {
                "header_count"
            },
        ));
    }
    for (o, v, reason) in [
        (20, 2, "target"),
        (48, 1, "target"),
        (64, 2, "program_kind"),
        (64, 3, "program_kind"),
        (64, 7, "program_kind"),
        (64, 6, "program_kind"),
        (64 + 4, 7, "permission"),
        (64 + 4, 0, "permission"),
        (64 + 4, 1, "permission"),
        (64 + 4, 0x105, "permission"),
    ] {
        let mut b = good.clone();
        w32(&mut b, o, v);
        cases.push((b, reason));
    }
    for (o, v, reason) in [
        (32, u64::MAX, "overflow"),
        (40, u64::MAX, "overflow"),
        (72, u64::MAX, "overflow"),
        (96, 9, "size"),
        (104, 0, "size"),
        (112, 3, "alignment"),
        (112, 0, "alignment"),
        (112, 0x400000, "alignment"),
        (88, 0, "alignment"),
        (80, 0x400001, "alignment"),
        (80, u64::MAX, "alignment"),
        (104, u64::MAX, "overflow"),
        (104, PROCESS_LIMIT, "commitment"),
        (24, 0, "entry"),
        (24, 0x401000, "entry"),
        (24, 0x400008, "entry"),
    ] {
        let mut b = good.clone();
        w64(&mut b, o, v);
        cases.push((b, reason));
    }
    for (addr, reason) in [
        (0, "user_range"),
        (0xffff800000000000, "user_range"),
        (STARTUP, "user_range"),
        (STACK_BASE - PAGE, "user_range"),
        (0x400000, "overlap"),
    ] {
        let mut b = good.clone();
        w64(&mut b, 64 + 56 + 16, addr);
        w64(&mut b, 64 + 56 + 24, addr);
        cases.push((b, reason));
    }
    for kind in [4, 6, 9, 11, 19] {
        let mut b = good.clone();
        w32(&mut b, 0x3804, kind);
        cases.push((b, "section_obligation"));
    }
    let mut tls = good.clone();
    w64(&mut tls, 0x3808, 0x400);
    cases.push((tls, "section_obligation"));
    let mut sym = good.clone();
    w32(&mut sym, 0x3804, 2);
    w64(&mut sym, 0x3818, 0x3900);
    w64(&mut sym, 0x3820, 24);
    w64(&mut sym, 0x3838, 24);
    w32(&mut sym, 0x3900, 1);
    sym[0x3904] = 0x10;
    cases.push((sym, "undefined_symbol"));
    let mut b = good.clone();
    w64(&mut b, 64 + 56 + 16, 0x400800);
    w64(&mut b, 64 + 56 + 24, 0x400800);
    w64(&mut b, 64 + 56 + 8, 0x1800);
    cases.push((b, "overlap")); // page alias without byte overlap
    let mut r = runtime();
    let mut ram = Ram::new();
    let mut l = Launcher::<1>::EMPTY;
    let empty = r.inventory();
    for (b, reason) in cases {
        assert_eq!(parse(&b).err(), Some(bad(reason)), "expected {reason}");
        assert_eq!(
            l.bootstrap(&mut r, &mut ram, trusted(&b), Binding::UserA)
                .err(),
            Some(bad(reason))
        );
        assert_eq!(r.inventory(), empty);
        assert_eq!(l.live(), 0);
        assert_eq!(ram.calls, 0);
    }
    for n in [0, 15, 63, 120, 231, 0x1007, 0x2007, 0x3007, 0x383f] {
        assert!(parse(&good[..n]).is_err(), "truncation {n}");
    }
    assert_eq!(
        parse(&vec![0; MAX_IMAGE + 1]).err(),
        Some(bad("image_limit"))
    );
}
struct Ram {
    pages: BTreeMap<u64, Vec<u8>>,
    next: u64,
    fail_at: usize,
    calls: usize,
}
impl Ram {
    fn new() -> Self {
        Self {
            pages: BTreeMap::new(),
            next: 0x200000,
            fail_at: usize::MAX,
            calls: 0,
        }
    }
}
impl Store for Ram {
    fn allocate(&mut self, n: usize, a: u64) -> Result<PhysAddr> {
        let call = self.calls;
        self.calls += 1;
        if call == self.fail_at {
            return Err(error("frames", "exhausted"));
        }
        let p = self.next.div_ceil(a) * a;
        self.next = p + n as u64 * PAGE;
        self.pages.insert(p, vec![0; n * 4096]);
        Ok(PhysAddr(p))
    }
    fn release(&mut self, p: PhysAddr, n: usize) {
        assert_eq!(self.pages.remove(&p.0).unwrap().len(), n * 4096);
    }
    fn read(&self, p: PhysAddr, o: u64) -> u8 {
        self.pages[&p.0][o as usize]
    }
    fn write(&mut self, p: PhysAddr, o: u64, b: u8) {
        self.pages.get_mut(&p.0).unwrap()[o as usize] = b;
    }
    fn publish(&mut self, _: PhysAddr, _: PhysAddr) {}
    fn retire(&mut self, _: PhysAddr) {}
    fn prepare_kernel_stack(&mut self, slot: usize) -> Result<Range> {
        Ok(Range {
            base: 0xffffc10000001000 + slot as u64 * 73728,
            len: STACK_BYTES,
        })
    }
    fn release_kernel_stack(&mut self, _: usize) {}
}
type R = Runtime<3, 32, 32, 16>;
fn runtime() -> Box<R> {
    let mut r = Box::new(R::EMPTY);
    r.initialize([0; 256], |p| Ok(PhysAddr(p as u64))).unwrap();
    r
}
fn trusted(b: &[u8]) -> TrustedImage<'_> {
    TrustedImage {
        bytes: b,
        digest: sha256(b),
        size: b.len(),
    }
}
#[test]
fn transactional_allocation_and_table_failures() {
    let b = image();
    for fail in 0..6 {
        let mut r = runtime();
        let mut ram = Ram::new();
        let mut l = Launcher::<3>::EMPTY;
        let empty = r.inventory();
        ram.fail_at = fail;
        assert!(
            l.bootstrap(&mut r, &mut ram, trusted(&b), Binding::UserA)
                .is_err()
        );
        assert_eq!(l.live(), 0);
        assert_eq!(r.inventory(), empty);
        assert!(ram.pages.is_empty());
    }
    // Table exhaustion after stack construction (new branch requires 3 more tables).
    for limit in [1, 2, 3, 4, 5] {
        macro_rules! check {
            ($n:expr) => {{
                let mut r = Box::new(Runtime::<1, 32, 32, $n>::EMPTY);
                r.initialize([0; 256], |p| Ok(PhysAddr(p as u64))).unwrap();
                let mut ram = Ram::new();
                let mut l = Launcher::<1>::EMPTY;
                let empty = r.inventory();
                assert!(
                    l.bootstrap(&mut r, &mut ram, trusted(&b), Binding::UserA)
                        .is_err()
                );
                assert_eq!(r.inventory(), empty);
                assert_eq!(l.live(), 0);
                assert!(ram.pages.is_empty());
            }};
        }
        match limit {
            1 => check!(1),
            2 => check!(2),
            3 => check!(3),
            4 => check!(4),
            _ => check!(5),
        }
    }
}
#[test]
fn loaded_bytes_permissions_startup_and_authority_reuse() {
    let b = image();
    let mut r = runtime();
    let mut ram = Ram::new();
    let mut l = Launcher::<3>::EMPTY;
    let empty = r.inventory();
    let mut ids = Vec::new();
    for binding in [Binding::UserA, Binding::UserB, Binding::Service] {
        let id = l.bootstrap(&mut r, &mut ram, trusted(&b), binding).unwrap();
        ids.push(id);
        let p = l.get(id).unwrap();
        assert_eq!(p.context, binding.context());
        assert_eq!(p.stack % 16, 8);
        for (address, flags) in [
            (0x400000, 5),
            (0x401000, 4),
            (0x402000, 6),
            (STARTUP, 4),
            (RETURN, 5),
            (STACK_BASE, 6),
        ] {
            let leaf = r
                .tables(p.space)
                .unwrap()
                .inspect(VirtAddr(address))
                .unwrap();
            assert_eq!(leaf & 7, if flags == 6 { 7 } else { 5 });
            assert_eq!(leaf >> 63, if flags == 5 { 0 } else { 1 });
        }
        let data = r
            .tables(p.space)
            .unwrap()
            .inspect(VirtAddr(0x402000))
            .unwrap()
            & 0x000fffffffff000;
        assert_eq!(&ram.pages[&data][..8], &b[0x3000..0x3008]);
        assert!(ram.pages[&data][8..].iter().all(|x| *x == 0));
        let start = r
            .tables(p.space)
            .unwrap()
            .inspect(VirtAddr(STARTUP))
            .unwrap()
            & 0x000fffffffff000;
        assert_eq!(&ram.pages[&start][..64], &STARTUP_BYTES);
        let stack = r
            .tables(p.space)
            .unwrap()
            .inspect(VirtAddr(STACK_BASE))
            .unwrap()
            & 0x000fffffffff000;
        assert_eq!(
            &ram.pages[&stack][STACK_BYTES as usize - 8..],
            &RETURN.to_le_bytes()
        );
        let c = p.context;
        for (principal, session, service, grants, commitment) in [
            (c.principal + 1, c.session, c.service, 0, PAGE),
            (c.principal, c.session + 1, c.service, 0, PAGE),
            (c.principal, c.session, !c.service, 0, PAGE),
            (c.principal, c.session, c.service, 4, PAGE),
            (c.principal, c.session, c.service, 0, PROCESS_LIMIT + 1),
        ] {
            assert_eq!(
                l.child_context(id, principal, session, service, grants, commitment)
                    .err(),
                Some(error("launch", "authority"))
            );
        }
        // Even fabricated startup handle/features bytes are never a grant source.
        ram.pages.get_mut(&start).unwrap()[16..32].fill(0xff);
        assert_eq!(l.get(id).unwrap().context, c);
        assert_eq!(
            l.child_context(id, c.principal, c.session, c.service, 0, PAGE)
                .unwrap()
                .grant_ceiling,
            0
        );
    }
    assert!(
        l.bootstrap(&mut r, &mut ram, trusted(&b), Binding::UserA)
            .is_err()
    );
    for id in &ids {
        l.destroy(&mut r, &mut ram, *id).unwrap();
        assert!(l.get(*id).is_err());
    }
    assert_eq!(r.inventory(), empty);
    assert!(ram.pages.is_empty());
    let new = l
        .bootstrap(&mut r, &mut ram, trusted(&b), Binding::UserB)
        .unwrap();
    assert_eq!(new.slot, ids[0].slot);
    assert!(new.generation > ids[0].generation);
    assert!(l.child_context(ids[0], 101, 1001, false, 0, PAGE).is_err());
    assert!(l.destroy(&mut r, &mut ram, ids[0]).is_err());
    l.destroy(&mut r, &mut ram, new).unwrap();
    assert_eq!(r.inventory(), empty);
}
#[test]
fn digest_and_parse_fail_without_context_or_frames() {
    let mut r = runtime();
    let mut ram = Ram::new();
    let mut l = Launcher::<1>::EMPTY;
    let empty = r.inventory();
    let b = image();
    let mut wrong = trusted(&b);
    wrong.digest[0] ^= 1;
    assert_eq!(
        l.bootstrap(&mut r, &mut ram, wrong, Binding::UserA).err(),
        Some(error("image", "digest"))
    );
    let mut size = trusted(&b);
    size.size += 1;
    assert!(l.bootstrap(&mut r, &mut ram, size, Binding::UserA).is_err());
    let mut malformed = b.clone();
    malformed[4] = 1;
    assert_eq!(
        l.bootstrap(&mut r, &mut ram, trusted(&malformed), Binding::UserA)
            .err(),
        Some(bad("identity"))
    );
    assert_eq!(ram.calls, 0);
    assert_eq!(r.inventory(), empty);
    assert_eq!(l.live(), 0);
}
#[test]
fn generation_wrap_retires_process_slot() {
    let mut r = runtime();
    let mut ram = Ram::new();
    let mut l = Launcher::<1>::EMPTY;
    l.slots[0].generation = u64::MAX;
    assert_eq!(
        l.bootstrap(&mut r, &mut ram, trusted(&image()), Binding::UserA)
            .err(),
        Some(error("process", "quota"))
    );
    assert_eq!(ram.calls, 0);
}
