use super::*;

fn fixture() -> BootInfoV1 {
    let mut b = BootInfoV1::EMPTY;
    b.prerequisites = REQUIRED;
    b.hhdm = VirtAddr(0xffff_8000_0000_0000);
    for (i, (base, len, kind)) in [
        (0x1000, 0xff000, 0),
        (0x100000, 0x100000, 5),
        (0x200000, 0x200000, 6),
        (0x800000, 0x400000, 7),
    ]
    .into_iter()
    .enumerate()
    {
        b.regions[i] = Region {
            range: Range { base, len },
            kind,
        };
    }
    b.region_count = 4;
    b.kernel = Range {
        base: 0x200000,
        len: 0x10000,
    };
    b.executable_file = Range {
        base: 0x300000,
        len: 0x4000,
    };
    b.framebuffers[0] = Framebuffer {
        address: VirtAddr(b.hhdm.0 + 0x800000),
        range: Range {
            base: 0x800000,
            len: 640 * 480 * 4,
        },
        width: 640,
        height: 480,
        pitch: 2560,
        bpp: 32,
        model: 1,
        sizes: [8, 8, 8],
        shifts: [16, 8, 0],
    };
    b.framebuffer_count = 1;
    b.acpi_root = Some(PhysAddr(0xf0000)); // locator only; deliberately outside ACPI map
    b
}
#[test]
fn required_revision_and_capacities_fail_closed() {
    let mut b = fixture();
    assert_eq!(b.validate(), Ok(()));
    for bit in 0..5 {
        b.prerequisites = REQUIRED & !(1 << bit);
        assert_eq!(
            b.validate(),
            Err(error(
                [
                    "base",
                    "memory",
                    "hhdm",
                    "executable_address",
                    "executable_file"
                ][bit],
                "missing"
            ))
        );
    }
    b.prerequisites = REQUIRED;
    b.version = 2;
    assert_eq!(b.validate(), Err(error("boot_info", "revision")));
    assert_eq!(
        revision(1, 0, "response"),
        Err(error("response", "revision"))
    );
    assert_eq!(count(512, MAX_REGIONS, "memory"), Ok(512));
    assert!(count(513, MAX_REGIONS, "memory").is_err());
    assert!(count(u64::MAX, MAX_REGIONS, "memory").is_err());
    b.version = 1;
    b.region_count = MAX_REGIONS + 1;
    assert_eq!(b.validate(), Err(error("memory", "capacity")));
    b.region_count = 4;
    b.module_count = MAX_MODULES + 1;
    assert_eq!(b.validate(), Err(error("modules", "capacity")));
    b.module_count = 0;
    b.framebuffer_count = MAX_FRAMEBUFFERS + 1;
    assert_eq!(b.validate(), Err(error("framebuffers", "capacity")));
    b.framebuffer_count = 0;
    b.acpi_root = None;
    assert_eq!(b.validate(), Ok(()));
}
#[test]
fn ranges_and_address_translations() {
    assert_eq!(
        Range::new(0, 0, "range"),
        Err(error("range", "zero_length"))
    );
    assert_eq!(
        Range::new(u64::MAX, 2, "range"),
        Err(error("range", "overflow"))
    );
    assert!(
        Range {
            base: 1 << 52,
            len: 1
        }
        .physical("p")
        .is_err()
    );
    assert!(virtual_range(0x7fff_ffff_ffff, 2, "v").is_err());
    assert!(virtual_range(0x8000_0000_0000, 1, "v").is_err());
    assert!(virtual_range(u64::MAX, 2, "v").is_err());
    assert!(from_hhdm(0x1000, 10, 0xffff_8000_0000_0000, "v").is_err());
    assert_eq!(
        from_hhdm(0xffff_8000_0010_0000, 4096, 0xffff_8000_0000_0000, "v"),
        Ok(Range {
            base: 0x100000,
            len: 4096
        })
    );
}
#[test]
fn memory_map_alignment_order_and_protocol_overlap_rules() {
    let mut b = fixture();
    b.regions[0].range.len = 0;
    assert_eq!(b.validate_memory(), Err(error("memory", "zero_length")));
    b = fixture();
    b.regions[0].range.len += 1;
    assert_eq!(b.validate_memory(), Err(error("memory", "alignment")));
    b = fixture();
    b.regions[1].range.base = 0xff000;
    assert_eq!(b.validate_memory(), Err(error("memory", "overlap")));
    b = fixture();
    b.regions.swap(0, 1);
    assert_eq!(b.validate_memory(), Err(error("memory", "order")));
    b = fixture();
    b.regions[3] = Region {
        range: Range {
            base: 0x200001,
            len: 200,
        },
        kind: 99,
    };
    assert_eq!(b.validate_memory(), Ok(())); // non-usable overlaps are legal
    let mut unknown_reserved = false;
    b.framebuffer_count = 0;
    b.reservations(|r, why| {
        if r.base == 0x200001 {
            unknown_reserved = why == ReservationReason::Firmware;
        }
    });
    assert!(unknown_reserved);
    for i in 0..MAX_REGIONS {
        b.regions[i] = Region {
            range: Range {
                base: (i as u64) * 4096,
                len: 4096,
            },
            kind: 1,
        };
    }
    b.region_count = MAX_REGIONS;
    assert_eq!(b.validate_memory(), Ok(()));
}
#[test]
fn owned_text_checks_termination_encoding_and_boundaries() {
    let mut text = Text::<4>::EMPTY;
    let mut source = *b"abc\0";
    assert_eq!(text.copy_cstr(&source, "string"), Ok(()));
    source[0] = b'z';
    assert_eq!(source[0], b'z');
    assert_eq!(text.as_str(), "abc");
    assert_eq!(
        text.copy_cstr(b"abcd\0", "string"),
        Err(error("string", "unterminated"))
    );
    assert!(text.copy_cstr(&[0xff, 0], "string").is_err());
    assert!(text.copy_cstr(b"a\nb\0", "string").is_err());
    let mut b = fixture();
    let source = b"boot():/module\0";
    b.modules[0].path.copy_cstr(source, "path").unwrap();
    let copy = b.modules[0];
    b.modules[0].path.copy_cstr(b"changed\0", "path").unwrap();
    assert_eq!(copy.path.as_str(), "boot():/module");
}
#[test]
fn framebuffer_arithmetic_pitch_formats_and_backing() {
    let mut b = fixture();
    assert_eq!(b.validate(), Ok(()));
    b.framebuffers[0].pitch += 16;
    b.framebuffers[0].range.len = b.framebuffers[0].pitch * 480;
    assert_eq!(b.validate(), Ok(())); // padded pitch is valid
    b = fixture();
    b.framebuffers[0].pitch = 1;
    assert_eq!(b.validate(), Err(error("framebuffer", "pitch")));
    b = fixture();
    b.framebuffers[0].width = u64::MAX;
    assert_eq!(b.validate(), Err(error("framebuffer", "overflow")));
    b = fixture();
    b.framebuffers[0].height = u64::MAX;
    assert_eq!(b.validate(), Err(error("framebuffer", "overflow")));
    b = fixture();
    b.framebuffers[0].width = 0;
    assert_eq!(b.validate(), Err(error("framebuffer", "dimensions")));
    for (sizes, shifts) in [
        ([0, 8, 8], [16, 8, 0]),
        ([8, 8, 8], [8, 8, 0]),
        ([9, 8, 8], [24, 8, 0]),
    ] {
        b = fixture();
        b.framebuffers[0].sizes = sizes;
        b.framebuffers[0].shifts = shifts;
        assert_eq!(b.validate(), Err(error("framebuffer", "format")));
    }
    b = fixture();
    b.framebuffers[0].model = 0;
    assert_eq!(b.validate(), Err(error("framebuffer", "format")));
    b = fixture();
    b.framebuffers[0].range.base += 4096;
    assert_eq!(b.validate(), Err(error("framebuffer", "range")));
    b = fixture();
    b.regions[3].kind = 1;
    assert_eq!(b.validate(), Err(error("framebuffer", "backing")));
    b = fixture();
    b.framebuffers[0].bpp = 24;
    b.framebuffers[0].pitch = 1920;
    b.framebuffers[0].range.len = 1920 * 480;
    assert_eq!(b.validate(), Ok(()));
}
#[test]
fn reservations_cover_live_kernel_loader_module_and_acpi_storage() {
    let mut b = fixture();
    b.modules[0].range = Range {
        base: 0x380000,
        len: 4096,
    };
    b.module_count = 1;
    assert_eq!(b.validate(), Ok(()));
    b.executable_file = Range {
        base: 0x180000,
        len: 4096,
    };
    assert_eq!(b.validate(), Ok(())); // raw ELF may be loader-reclaimable
    b.executable_file = Range {
        base: 0x2000,
        len: 4096,
    };
    assert_eq!(b.validate(), Err(error("executable_file", "backing")));
    b.executable_file = Range {
        base: 0x300000,
        len: 0x4000,
    };
    for (range, reason) in [
        (b.kernel, ReservationReason::Kernel),
        (b.executable_file, ReservationReason::ExecutableFile),
        (b.regions[1].range, ReservationReason::LoaderLive),
        (b.modules[0].range, ReservationReason::Module),
        (b.framebuffers[0].range, ReservationReason::Framebuffer),
        (
            Range {
                base: 0xf0000,
                len: 36,
            },
            ReservationReason::Acpi,
        ),
    ] {
        let mut found = false;
        b.reservations(|r, why| found |= r == range && why == reason);
        assert!(found);
    }
    b.modules[0].range = b.kernel;
    assert_eq!(b.validate(), Err(error("module", "overlap")));
    b.module_count = 0;
    b.kernel.base = 0x500000;
    assert_eq!(b.validate(), Err(error("kernel", "backing")));
    b = fixture();
    b.acpi_root = Some(PhysAddr(0xf0014));
    assert_eq!(b.validate(), Ok(())); // byte locator, not an aligned struct reference
    b.acpi_root = Some(PhysAddr(0));
    assert_eq!(b.validate(), Err(error("acpi", "physical_address")));
}
#[test]
fn exact_host_session_configuration() {
    let mut b = Text::<1024>::EMPTY;
    b.copy_cstr(b"session=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef fixture=normal\0", "args").unwrap();
    let (id, f) = identity(&b).unwrap();
    assert_eq!(id.len(), 64);
    assert_eq!(f, "normal");
    let long_fixture = *b"session=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef fixture=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\0";
    b.copy_cstr(&long_fixture, "args").unwrap();
    assert!(identity(&b).is_err());
    for invalid in [&b"session=short fixture=normal\0"[..], &b"session=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef fixture=normal extra=yes\0"[..]] {
        b.copy_cstr(invalid, "args").unwrap(); assert!(identity(&b).is_err());
    }
}
