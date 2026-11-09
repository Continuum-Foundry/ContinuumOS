//! P4-03 Q35 modern PCI VirtIO probe. Temporary trusted kernel driver; no user
//! MMIO/DMA pointers escape. Not an input parser or development transport.
use crate::{Local, paging, record, timer};
use continuum_kernel::{PhysAddr, Result, device as d, error};
use core::{
    arch::asm,
    cell::UnsafeCell,
    sync::atomic::{AtomicU64, Ordering, fence},
};
static ISRS: [AtomicU64; 3] = [const { AtomicU64::new(0) }; 3];
static IRQS: [AtomicU64; 3] = [const { AtomicU64::new(0) }; 3];
static RETAIN: Local<d::Retention> = Local(UnsafeCell::new(d::Retention::EMPTY));
static BROKER: Local<d::Broker> = Local(UnsafeCell::new(d::Broker::EMPTY));
static RESOURCES: Local<[Option<d::Resource>; 3]> = Local(UnsafeCell::new([None; 3]));
static GRANTS: Local<[Option<(d::Owner, d::Grant)>; 3]> = Local(UnsafeCell::new([None; 3]));
static DEVICES: Local<[Option<Device>; 3]> = Local(UnsafeCell::new([None; 3]));
struct Active {
    device: Device,
    backing: PhysAddr,
    dma: d::Dma,
}
static ACTIVE: Local<Option<Active>> = Local(UnsafeCell::new(None));
fn need(ok: bool, reason: &'static str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(error("device", reason))
    }
}
fn event(name: &str, args: core::fmt::Arguments<'_>) {
    record(name, "device", "ok", args);
}
fn pci_address(bdf: u16, offset: u8) -> u32 {
    0x80000000 | (u32::from(bdf) << 8) | u32::from(offset & !3)
}
fn out32(port: u16, value: u32) {
    unsafe {
        asm!("out dx, eax", in("dx")port, in("eax")value, options(nostack, nomem, preserves_flags));
    }
}
fn pci(bdf: u16, offset: u8) -> u32 {
    out32(0xcf8, pci_address(bdf, offset));
    let v;
    unsafe {
        asm!("in eax, dx", in("dx")0xcfcu16, out("eax")v, options(nostack, nomem, preserves_flags));
    }
    v
}
fn pci_write(bdf: u16, offset: u8, value: u32) {
    out32(0xcf8, pci_address(bdf, offset));
    out32(0xcfc, value);
}
fn command(bdf: u16, value: u16) {
    out32(0xcf8, pci_address(bdf, 4));
    // Write only Command, never echo W1C Status bits.
    unsafe {
        asm!("out dx, ax", in("dx")0xcfcu16, in("ax")value, options(nostack, nomem, preserves_flags));
    }
}
fn byte(bdf: u16, at: u8) -> u8 {
    (pci(bdf, at) >> ((at & 3) * 8)) as u8
}
// Addresses below are constructed exclusively from validated, UC/NX supervisor
// mappings and capability-local offsets. No references to device memory exist.
fn r8(a: u64) -> u8 {
    unsafe { (a as *const u8).read_volatile() }
}
fn r16(a: u64) -> u16 {
    unsafe { (a as *const u16).read_volatile() }
}
fn r32(a: u64) -> u32 {
    unsafe { (a as *const u32).read_volatile() }
}
fn w8(a: u64, v: u8) {
    unsafe { (a as *mut u8).write_volatile(v) }
}
fn w16(a: u64, v: u16) {
    unsafe { (a as *mut u16).write_volatile(v) }
}
fn w32(a: u64, v: u32) {
    unsafe { (a as *mut u32).write_volatile(v) }
}
fn w64(a: u64, v: u64) {
    w32(a, v as u32);
    w32(a + 4, (v >> 32) as u32);
}
#[derive(Clone, Copy)]
struct Device {
    bdf: u16,
    id: u16,
    irq: u8,
    common: u64,
    notify: u64,
    notify_len: u64,
    multiplier: u32,
    isr: u64,
    config: u64,
    slot: usize,
    pages: u64,
    resource: d::Resource,
}
fn discover(bdf: u16, slot: usize) -> Result<Device> {
    let id = (pci(bdf, 0) >> 16) as u16;
    need([0x1052, 0x1043].contains(&id), "unsupported_id")?;
    need(
        byte(bdf, 14) & 0x7f == 0 && pci(bdf, 4) & (1 << 20) != 0,
        "pci_header",
    )?;
    // No decode/DMA/INTx while probing a newly admitted function's BARs.
    command(bdf, 1 << 10);
    let mut caps = [(0u8, 0u32, 0u32); 5];
    let mut multiplier = 0;
    let mut seen = [false; 256];
    let mut at = byte(bdf, 0x34);
    let mut steps = 0;
    while at != 0 {
        need(
            at >= 0x40 && at % 4 == 0 && !seen[at as usize] && steps < 48,
            "cap_chain",
        )?;
        seen[at as usize] = true;
        steps += 1;
        if byte(bdf, at) == 9 {
            let len = byte(bdf, at + 2);
            need(
                len >= 16 && u16::from(at) + u16::from(len) <= 256,
                "cap_length",
            )?;
            let kind = byte(bdf, at + 3) as usize;
            if (1..=4).contains(&kind) {
                need(
                    caps[kind].2 == 0 && (kind != 2 || len >= 20),
                    "cap_duplicate",
                )?;
                caps[kind] = (byte(bdf, at + 4), pci(bdf, at + 8), pci(bdf, at + 12));
                if kind == 2 {
                    multiplier = pci(bdf, at + 16);
                }
            }
        }
        at = byte(bdf, at + 1);
    }
    let bar = caps[1].0;
    need(
        bar < 5
            && caps[1].2 >= 56
            && caps[2].2 >= 2
            && caps[3].2 >= 1
            && caps[4].2 >= if id == 0x1052 { 136 } else { 8 },
        "cap_required",
    )?;
    need(
        caps[1..].iter().all(|c| c.0 == bar),
        "multi_bar_unsupported",
    )?;
    let offset = 0x10 + bar * 4;
    let low = pci(bdf, offset);
    let high = pci(bdf, offset + 4);
    event(
        "m1.device_bar",
        format_args!("slot={slot} bar={bar} low={low} high={high}"),
    );
    need(low & 7 == 4, "bar_type")?; // 64-bit MMIO; cache policy remains UC
    pci_write(bdf, offset, u32::MAX);
    pci_write(bdf, offset + 4, u32::MAX);
    let mask = (u64::from(pci(bdf, offset + 4)) << 32) | u64::from(pci(bdf, offset) & !15);
    pci_write(bdf, offset, low);
    pci_write(bdf, offset + 4, high);
    let bytes = (!mask).checked_add(1).ok_or(error("device", "bar_size"))?;
    need(
        bytes.is_power_of_two() && (4096..=65536).contains(&bytes),
        "bar_size",
    )?;
    let resource = d::Resource::new(
        (u64::from(high) << 32) | u64::from(low & !15),
        bytes,
        d::Kind::Mmio,
    )?;
    let physical_bits = core::arch::x86_64::__cpuid(0x80000008).eax & 255;
    need(
        (32..=52).contains(&physical_bits) && resource.range().end() <= (1u64 << physical_bits),
        "cpu_physical_width",
    )?;
    need(
        !unsafe { &*RESOURCES.0.get() }
            .iter()
            .flatten()
            .any(|r| r.range().overlaps(resource.range())),
        "bar_overlap",
    )?;
    let base = resource.range().base;
    for (i, &(_, start, len)) in caps.iter().enumerate().skip(1) {
        need(
            u64::from(start) + u64::from(len) <= bytes && start % if i == 3 { 1 } else { 4 } == 0,
            "cap_bounds",
        )?;
        for &(_, s, l) in &caps[1..i] {
            need(
                u64::from(start) >= u64::from(s) + u64::from(l)
                    || u64::from(s) >= u64::from(start) + u64::from(len),
                "cap_overlap",
            )?;
        }
    }
    let mut virtual_base = 0;
    for page in 0..bytes / 4096 {
        let v = paging::driver_page(base + page * 4096, slot as u64 * 16 + page)?;
        if page == 0 {
            virtual_base = v;
        }
    }
    command(bdf, 2 | (1 << 10));
    // Q35 root slots below 25 have fixed PIRQ[E-H] swizzling in the selected
    // emulator. The PCI Interrupt Line byte describes the legacy PIC route.
    need(
        pci(0, 0) == 0x29c08086
            && pci(31 * 8, 0) == 0x29188086
            && (4 * 8..=6 * 8).contains(&bdf)
            && bdf % 8 == 0
            && byte(bdf, 0x3d) == 1,
        "irq_route",
    )?;
    let irq = 20 + ((bdf / 8) % 4) as u8;
    let d = Device {
        bdf,
        id,
        irq,
        common: virtual_base + u64::from(caps[1].1),
        notify: virtual_base + u64::from(caps[2].1),
        notify_len: u64::from(caps[2].2),
        multiplier,
        isr: virtual_base + u64::from(caps[3].1),
        config: virtual_base + u64::from(caps[4].1),
        slot,
        pages: bytes / 4096,
        resource,
    };
    event(
        "m1.device_discovered",
        format_args!(
            "slot={slot} bdf={bdf} id={id} base={base} bytes={bytes} irq={irq} uc=true user=false iommu=false"
        ),
    );
    Ok(d)
}
pub(crate) fn interrupt() {
    for i in 0..3 {
        let isr = ISRS[i].load(Ordering::Relaxed);
        if isr != 0 && r8(isr) & 1 != 0 {
            IRQS[i].fetch_add(1, Ordering::Relaxed);
        }
    }
}
fn reset(dev: Device, dma: Option<&mut d::Dma>, hide_completion: bool) -> Result<bool> {
    command(dev.bdf, 2 | 4 | (1 << 10));
    w8(dev.common + 20, 0);
    let begin = timer::sample_ns();
    while r8(dev.common + 20) != 0 || hide_completion {
        if timer::sample_ns() - begin >= 100_000_000 {
            if let Some(dma) = dma {
                dma.timeout();
            }
            event(
                "m1.device_reset",
                format_args!(
                    "slot={} complete=false retained=true timeout_ns=100000000",
                    dev.slot
                ),
            );
            return Ok(false);
        }
        core::hint::spin_loop();
    }
    fence(Ordering::SeqCst);
    let _ = r8(dev.isr);
    // Device reports reset complete; remove bus mastering as defense in depth.
    command(dev.bdf, 2 | (1 << 10));
    if let Some(dma) = dma {
        dma.reset_observed()?;
    }
    event(
        "m1.device_reset",
        format_args!(
            "slot={} complete=true retained=false status=0 masked=true",
            dev.slot
        ),
    );
    Ok(true)
}
fn negotiate(dev: Device) -> Result<()> {
    need(reset(dev, None, false)?, "initial_reset")?;
    w8(dev.common + 20, 1);
    w8(dev.common + 20, 3);
    w32(dev.common, 0);
    let lo = r32(dev.common + 4);
    w32(dev.common, 1);
    let hi = r32(dev.common + 4);
    let offered = u64::from(lo) | (u64::from(hi) << 32);
    let selected = d::features(offered, 1 << 32)?;
    need(d::features(offered, 1 << 63).is_err(), "required_unknown")?;
    w32(dev.common + 8, 0);
    w32(dev.common + 12, selected as u32);
    w32(dev.common + 8, 1);
    w32(dev.common + 12, (selected >> 32) as u32);
    w8(dev.common + 20, 11);
    need(
        r8(dev.common + 20) == 11 && r16(dev.common + 18) >= 2,
        "features_ok",
    )?;
    event(
        "m1.device_features",
        format_args!(
            "slot={} offered={offered} selected={selected} required_unknown_denied=true",
            dev.slot
        ),
    );
    Ok(())
}
// Physical RAM is an exclusive zeroed run from M1's frame allocator. Access uses
// staging-only supervisor aliases, never a CPU pointer supplied by a descriptor.
fn put(p: PhysAddr, offset: u64, bytes: &[u8]) {
    paging::runtime_write_bytes(p, offset, bytes);
}
fn get16(p: PhysAddr, offset: u64) -> u16 {
    u16::from_le_bytes([
        paging::runtime_read(p, offset),
        paging::runtime_read(p, offset + 1),
    ])
}
fn get32(p: PhysAddr, offset: u64) -> u32 {
    u32::from_le_bytes(core::array::from_fn(|i| {
        paging::runtime_read(p, offset + i as u64)
    }))
}
fn allocate_dma() -> Result<(PhysAddr, d::Dma)> {
    let p = paging::runtime_allocate(8, 4096)?;
    match unsafe { (&mut *RETAIN.0.get()).admit(p.0, 32768, 4096) } {
        Ok(dma) => Ok((p, dma)),
        Err(e) => {
            paging::runtime_release(p, 8);
            Err(e)
        }
    }
}
fn queue(dev: Device, p: PhysAddr, q: u16) -> Result<u64> {
    let offset = u64::from(q) * 16384;
    w16(dev.common + 22, q);
    need(r16(dev.common + 28) == 0, "queue_live")?;
    let size = d::queue_size(r16(dev.common + 24))?;
    w16(dev.common + 24, size);
    w16(dev.common + 26, 0xffff);
    let notify = u64::from(r16(dev.common + 30))
        .checked_mul(u64::from(dev.multiplier))
        .ok_or(error("device", "notify_overflow"))?;
    need(
        notify % 2 == 0 && notify + 2 <= dev.notify_len,
        "notify_bounds",
    )?;
    w64(dev.common + 32, p.0 + offset);
    w64(dev.common + 40, p.0 + offset + 4096);
    w64(dev.common + 48, p.0 + offset + 8192);
    w16(dev.common + 28, 1);
    need(r16(dev.common + 28) == 1, "queue_enable")?;
    event(
        "m1.device_queue",
        format_args!(
            "slot={} queue={q} size={size} base={} bytes=32768 desc={} avail={} used={} payload={} notify_offset={notify}",
            dev.slot,
            p.0,
            p.0 + offset,
            p.0 + offset + 4096,
            p.0 + offset + 8192,
            p.0 + offset + 12288
        ),
    );
    Ok(dev.notify + notify)
}
fn publish(p: PhysAddr, q: u16, dma: &mut d::Dma, input: bool, feedback: [u8; 8]) -> Result<()> {
    let base = u64::from(q) * 16384;
    let direction = if input {
        d::Direction::FromDevice
    } else {
        d::Direction::ToDevice
    };
    for head in 0..4u16 {
        let desc = d::Descriptor {
            address: p.0 + base + 12288 + u64::from(head) * 8,
            len: 8,
            flags: if input { 2 } else { 0 },
            next: 0,
        };
        d::chain(&[desc], 0, dma.range(), direction)?;
        put(p, base + u64::from(head) * 16, &desc.address.to_le_bytes());
        put(p, base + u64::from(head) * 16 + 8, &8u32.to_le_bytes());
        put(
            p,
            base + u64::from(head) * 16 + 12,
            &desc.flags.to_le_bytes(),
        );
        if !input {
            put(p, base + 12288 + u64::from(head) * 8, &feedback);
        }
        dma.submit(q * 4 + head)?;
        put(
            p,
            base + 4096 + 4 + u64::from(head) * 2,
            &head.to_le_bytes(),
        );
    }
    fence(Ordering::Release);
    put(p, base + 4096 + 2, &4u16.to_le_bytes());
    fence(Ordering::SeqCst);
    Ok(())
}
fn feedback(dev: Device) -> Result<[u8; 8]> {
    w8(dev.config, 0x11);
    w8(dev.config + 1, 17); // EV_BITS / EV_LED
    let size = r8(dev.config + 2);
    need(size <= 128, "input_config")?;
    let leds = size > 0 && r8(dev.config + 8) & 1 != 0;
    need(
        leds || d::feedback(false, true).is_err(),
        "unsupported_feedback",
    )?;
    let payload = d::feedback(leds, leds)?;
    event(
        "m1.device_feedback",
        format_args!(
            "slot={} led_advertised={leds} kind={} unsupported_led_denied={} semantic_claim=false",
            dev.slot,
            u16::from_le_bytes([payload[0], payload[1]]),
            !leds
        ),
    );
    Ok(payload)
}
fn used(
    dev: Device,
    p: PhysAddr,
    dma: &mut d::Dma,
    irq_expected: bool,
    grant: d::Grant,
) -> Result<()> {
    let base = 16384;
    let begin = timer::sample_ns();
    while get16(p, base + 8192 + 2) != 4
        || (irq_expected && IRQS[dev.slot].load(Ordering::Relaxed) == 0)
    {
        need(timer::sample_ns() - begin < 100_000_000, "used_deadline")?;
        unsafe {
            asm!("sti", "nop", "cli", options(nostack));
        }
    }
    fence(Ordering::Acquire);
    let owner = d::Owner {
        process: 1,
        generation: 1,
    };
    let broker = unsafe { &mut *BROKER.0.get() };
    broker.mask(owner, grant, !irq_expected)?;
    broker.notify(grant)?;
    broker.notify(grant)?;
    if !irq_expected {
        need(broker.wait(owner, grant)?.is_none(), "masked_latch")?;
        broker.mask(owner, grant, false)?;
    }
    need(broker.wait(owner, grant)? == Some(1), "delivered_latch")?;
    need(
        broker
            .ack(
                d::Owner {
                    process: 2,
                    ..owner
                },
                grant,
                1,
            )
            .is_err(),
        "cross_irq_ack",
    )?;
    broker.ack(owner, grant, 1)?;
    need(broker.ack(owner, grant, 1).is_err(), "duplicate_irq_ack")?;
    for n in 0..4 {
        let head = get32(p, base + 8192 + 4 + n * 8);
        let len = get32(p, base + 8192 + 8 + n * 8);
        need(head < 4, "used_head")?;
        dma.complete(4 + head, len, 8)?;
        event(
            "m1.device_used_entry",
            format_args!(
                "slot={} queue=1 head={head} bytes={len} capacity=8",
                dev.slot
            ),
        );
    }
    event(
        "m1.device_used",
        format_args!(
            "slot={} entries=4 irq_expected={irq_expected} irqs={} lost_irq_poll={} malformed_denied=true chain_negatives=5 irq_latch=true",
            dev.slot,
            IRQS[dev.slot].load(Ordering::Relaxed),
            !irq_expected
        ),
    );
    Ok(())
}
fn exercise(dev: Device) -> Result<()> {
    negotiate(dev)?;
    if dev.id == 0x1043 {
        // transport queues/control protocol belong to P4-12
        need(reset(dev, None, false)?, "console_reset")?;
        return Ok(());
    }
    let feedback = feedback(dev)?;
    for masked in [false, true] {
        if masked {
            negotiate(dev)?;
        }
        let (p, mut dma) = allocate_dma()?;
        let event_notify = queue(dev, p, 0)?;
        let status_notify = queue(dev, p, 1)?;
        let invalid = d::Descriptor {
            address: p.0 + 32768,
            len: 8,
            flags: 2,
            next: 0,
        };
        need(
            d::chain(&[invalid], 0, dma.range(), d::Direction::FromDevice).is_err(),
            "malformed",
        )?;
        for bad in [
            d::Descriptor {
                address: p.0,
                len: 0,
                ..invalid
            },
            d::Descriptor {
                address: p.0,
                len: 32769,
                ..invalid
            },
            d::Descriptor {
                address: p.0,
                flags: 4,
                ..invalid
            },
            d::Descriptor {
                address: p.0,
                flags: 3,
                ..invalid
            },
        ] {
            need(
                d::chain(&[bad], 0, dma.range(), d::Direction::FromDevice).is_err(),
                "bad_chain",
            )?;
        }
        let owner = d::Owner {
            process: 1,
            generation: 1,
        };
        let grant =
            unsafe { (&mut *BROKER.0.get()).grant(owner, dev.resource, d::MAP | d::IRQ | d::DMA)? };
        publish(p, 0, &mut dma, true, [0; 8])?;
        publish(p, 1, &mut dma, false, feedback)?;
        need(
            dma.submit(8).is_err() && dma.submit(0).is_err(),
            "queue_capacity",
        )?;
        ISRS[dev.slot].store(dev.isr, Ordering::Relaxed);
        IRQS[dev.slot].store(0, Ordering::Relaxed);
        timer::device_route(dev.irq, false)?;
        command(dev.bdf, 2 | 4 | if masked { 1 << 10 } else { 0 });
        w8(dev.common + 20, 15);
        need(r8(dev.common + 20) == 15, "driver_ok")?;
        w16(event_notify, 0);
        w16(status_notify, 1);
        used(dev, p, &mut dma, !masked, grant)?;
        if masked {
            need(IRQS[dev.slot].load(Ordering::Relaxed) == 0, "lost_irq")?;
        }
        dma.stop(); // cancellation / driver death with eventq still device-owned
        need(dma.submit(0).is_err(), "stopped_submit")?;
        if masked {
            need(!reset(dev, Some(&mut dma), true)?, "quarantine_hook")?;
            dma = match unsafe { (&mut *RETAIN.0.get()).release(dma) } {
                Err(dma) => dma,
                Ok(_) => return Err(error("device", "premature_free")),
            };
            let (devices, bytes) = unsafe { (&*RETAIN.0.get()).inventory() };
            event(
                "m1.device_quarantine",
                format_args!(
                    "slot={} devices={devices} bytes={bytes} cap_bytes={} cap_devices=3 released=false",
                    dev.slot,
                    d::QUARANTINE_BYTES
                ),
            );
        }
        need(reset(dev, Some(&mut dma), false)?, "reset_complete")?;
        ISRS[dev.slot].store(0, Ordering::Relaxed);
        let range = unsafe { (&mut *RETAIN.0.get()).release(dma) }
            .map_err(|_| error("device", "release"))?;
        need(range.base == p.0 && range.len == 32768, "release_range")?;
        paging::runtime_release(p, 8);
        unsafe {
            (&mut *BROKER.0.get()).revoke(owner, grant)?;
        }
        event(
            "m1.device_settled",
            format_args!(
                "slot={} masked={masked} retained_bytes=0 frames={} reset_before_release=true",
                dev.slot,
                paging::runtime_frames()
            ),
        );
    }
    Ok(())
}
pub(crate) fn run() -> Result<()> {
    let frames = paging::runtime_frames();
    // Exhaust real backing admission before enabling any device. A denied
    // fourth run must return its freshly allocated CPU-owned frames immediately.
    let mut held = [const { None }; 3];
    for slot in &mut held {
        *slot = Some(allocate_dma()?);
    }
    let full_frames = paging::runtime_frames();
    need(
        allocate_dma().is_err() && paging::runtime_frames() == full_frames,
        "admission_unwind",
    )?;
    for slot in &mut held {
        let (p, dma) = slot.take().unwrap();
        let r = unsafe { (&mut *RETAIN.0.get()).release(dma) }
            .map_err(|_| error("device", "admission_release"))?;
        need(r.base == p.0 && r.len == 32768, "admission_range")?;
        paging::runtime_release(p, 8);
    }
    need(
        paging::runtime_frames() == frames && unsafe { (&*RETAIN.0.get()).inventory() } == (0, 0),
        "admission_settlement",
    )?;
    event(
        "m1.device_admission",
        format_args!(
            "devices=3 bytes=98304 fourth_denied=true failed_backing_released=true before_frames={frames} after_frames={} active_dma=false",
            paging::runtime_frames()
        ),
    );
    let mut devices = [None; 3];
    let mut count = 0;
    for bdf in 0..256u16 {
        // one root bus, no bridge/hotplug discovery claim
        let ident = pci(bdf, 0);
        if ident as u16 != 0x1af4 {
            continue;
        }
        need(count < 3, "discovery_capacity")?;
        let dev = discover(bdf, count)?;
        // Bootstrap-only grant validates range conflicts before any DMA/IRQ.
        let owner = d::Owner {
            process: 1,
            generation: 1,
        };
        let grant =
            unsafe { (&mut *BROKER.0.get()).grant(owner, dev.resource, d::MAP | d::IRQ | d::DMA)? };
        let b = unsafe { &mut *BROKER.0.get() };
        need(
            b.map(owner, grant, 0, 4)? == dev.resource.range().base,
            "grant_map",
        )?;
        need(
            b.map(
                d::Owner {
                    process: 2,
                    ..owner
                },
                grant,
                0,
                4,
            )
            .is_err(),
            "forged_owner",
        )?;
        need(
            b.map(
                d::Owner {
                    generation: 2,
                    ..owner
                },
                grant,
                0,
                4,
            )
            .is_err(),
            "stale_owner",
        )?;
        need(
            b.map(owner, grant, dev.resource.range().len, 4).is_err(),
            "over_range",
        )?;
        need(b.grant(owner, dev.resource, d::MAP).is_err(), "overlap")?;
        b.notify(grant)?;
        b.notify(grant)?;
        b.mask(owner, grant, false)?;
        need(b.wait(owner, grant)? == Some(1), "irq_latch")?;
        b.ack(owner, grant, 1)?;
        need(b.ack(owner, grant, 1).is_err(), "duplicate_ack")?;
        b.revoke(owner, grant)?;
        need(b.map(owner, grant, 0, 4).is_err(), "stale_grant")?;
        let grant = b.grant(owner, dev.resource, d::IRQ)?;
        need(b.map(owner, grant, 0, 4).is_err(), "over_rights")?;
        b.revoke(owner, grant)?;
        event(
            "m1.device_authority",
            format_args!(
                "slot={count} forged=true stale=true overlap=true over_rights=true duplicate_irq=true cancelled=true all_denied=true"
            ),
        );
        unsafe {
            (&mut *RESOURCES.0.get())[count] = Some(dev.resource);
        }
        devices[count] = Some(dev);
        count += 1;
    }
    need(
        count == 3 && devices.iter().flatten().filter(|d| d.id == 0x1052).count() == 2,
        "device_matrix",
    )?;
    unsafe {
        *DEVICES.0.get() = devices;
    }
    for dev in devices.into_iter().flatten() {
        exercise(dev)?;
        timer::device_route(dev.irq, true)?;
        for page in 0..dev.pages {
            paging::driver_unmap(dev.slot as u64 * 16 + page)?;
        }
    }
    need(
        unsafe { (&*RETAIN.0.get()).inventory() } == (0, 0) && paging::runtime_frames() == frames,
        "settlement",
    )?;
    event(
        "m1.device_ready",
        format_args!(
            "devices=3 queue_setups=8 used_entries=16 quarantine_probes=2 retained_bytes=0 frames={frames} iommu=false"
        ),
    );
    Ok(())
}

fn start_lease(slot: usize) -> Result<()> {
    need(unsafe { (&*ACTIVE.0.get()).is_none() }, "lease_capacity")?;
    let dev = unsafe { (&*DEVICES.0.get())[slot] }.ok_or(error("device", "lease_device"))?;
    for page in 0..dev.pages {
        paging::driver_page(
            dev.resource.range().base + page * 4096,
            slot as u64 * 16 + page,
        )?;
    }
    negotiate(dev)?;
    let (p, mut dma) = allocate_dma()?;
    let a = queue(dev, p, 0)?;
    let b = queue(dev, p, 1)?;
    let feedback = feedback(dev)?;
    publish(p, 0, &mut dma, true, [0; 8])?;
    publish(p, 1, &mut dma, false, feedback)?;
    ISRS[slot].store(dev.isr, Ordering::Relaxed);
    timer::device_route(dev.irq, false)?;
    command(dev.bdf, 2 | 4);
    w8(dev.common + 20, 15);
    w16(a, 0);
    w16(b, 1);
    unsafe {
        *ACTIVE.0.get() = Some(Active {
            device: dev,
            backing: p,
            dma,
        });
    }
    event(
        "m1.device_lease",
        format_args!("slot={slot} bytes=32768 outstanding=8 device_owned=true"),
    );
    Ok(())
}
pub(crate) fn owner_dead(process: u64, generation: u64) -> Result<()> {
    let owner = d::Owner {
        process,
        generation,
    };
    for slot in 0..3 {
        let grant = unsafe { (&*GRANTS.0.get())[slot] }.filter(|g| g.0 == owner);
        if let Some((_, grant)) = grant {
            if slot == 0 {
                let mut active = unsafe { (&mut *ACTIVE.0.get()).take() }
                    .ok_or(error("device", "missing_lease"))?;
                let eventq_used = get16(active.backing, 8194);
                need(eventq_used < 4, "death_pending_eventq")?;
                active.dma.stop();
                // Ownership is retained even if completion is not observable.
                if !reset(active.device, Some(&mut active.dma), false)? {
                    unsafe {
                        *ACTIVE.0.get() = Some(active);
                    }
                    return Err(error("device", "death_quarantined"));
                }
                ISRS[slot].store(0, Ordering::Relaxed);
                timer::device_route(active.device.irq, true)?;
                let range = unsafe { (&mut *RETAIN.0.get()).release(active.dma) }
                    .map_err(|_| error("device", "death_release"))?;
                paging::runtime_release(active.backing, (range.len / 4096) as usize);
                for page in 0..active.device.pages {
                    paging::driver_unmap(slot as u64 * 16 + page)?;
                }
                event(
                    "m1.device_owner_dead",
                    format_args!(
                        "slot={slot} process={process} generation={generation} reset_complete=true released_bytes=32768 pending_eventq=true eventq_used={eventq_used} eventq_submitted=4"
                    ),
                );
            }
            unsafe {
                (&mut *BROKER.0.get()).revoke(owner, grant)?;
                (&mut *GRANTS.0.get())[slot] = None;
            }
        }
    }
    Ok(())
}
pub(crate) fn finish() -> Result<()> {
    need(
        unsafe {
            (&*ACTIVE.0.get()).is_none()
                && (&*GRANTS.0.get()).iter().all(Option::is_none)
                && (&*RETAIN.0.get()).inventory() == (0, 0)
        },
        "final_inventory",
    )?;
    event(
        "m1.device_final",
        format_args!(
            "grants=0 dma_bytes=0 leases=0 frames={}",
            paging::runtime_frames()
        ),
    );
    Ok(())
}

/// Experimental test-flavor syscall adapter. Trusted launch role, not a request
/// body, selects entitlement. It exposes mediated range checks, never raw MMIO.
pub(crate) fn call(
    op: u64,
    role: u64,
    process: u64,
    generation: u64,
    token: u64,
    offset: u64,
    width: u64,
) -> Option<core::result::Result<u64, u64>> {
    if !(0x4000..=0x4003).contains(&op) {
        return None;
    }
    let slot = token as usize;
    let owner = d::Owner {
        process,
        generation,
    };
    let result: Result<u64> = (|| {
        let expected_role = if slot < 2 { 2 } else { 5 };
        need(slot < 3 && role == expected_role, "role_denied")?;
        let resource = unsafe { (&*RESOURCES.0.get())[slot] }.ok_or(error("device", "resource"))?;
        let b = unsafe { &mut *BROKER.0.get() };
        let grants = unsafe { &mut *GRANTS.0.get() };
        if op == 0x4000 {
            need(grants[slot].is_none(), "already_granted")?;
            let grant = b.grant(owner, resource, d::MAP | d::IRQ)?;
            grants[slot] = Some((owner, grant));
            if slot == 0 {
                start_lease(slot)?;
            }
            return Ok(resource.range().len);
        }
        let (bound, grant) = grants[slot].ok_or(error("device", "no_grant"))?;
        need(bound == owner, "owner_denied")?;
        match op {
            0x4001 => {
                b.map(owner, grant, offset, width)?;
                Ok(width)
            }
            0x4002 => {
                b.ack(owner, grant, offset)?;
                Ok(0)
            }
            _ => {
                need(slot != 0, "live_lease_retained_until_death")?;
                b.revoke(owner, grant)?;
                grants[slot] = None;
                Ok(0)
            }
        }
    })();
    event(
        "m1.device_call",
        format_args!(
            "op={op} role={role} process={process} generation={generation} slot={token} offset={offset} width={width} denied={}",
            result.is_err()
        ),
    );
    Some(result.map_err(|_| 3)) // replaced by caller with frozen AccessDenied
}
