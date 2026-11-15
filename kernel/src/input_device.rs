//! Continuous VirtIO eventq adapter, sharing P4-03 discovery/reset and scoped backing.
//! Only fixed device descriptors are published. All parsing/policy runs in CPL3.
use super::*;
struct Ring {
    dev: Device,
    p: PhysAddr,
    dma: d::Dma,
    grant: d::Grant,
    used: u16,
    avail: u16,
    notify: u64,
    sequence: u64,
}
static INPUT: Local<[Option<Ring>; 2]> = Local(UnsafeCell::new([const { None }; 2]));
const OWNER: d::Owner = d::Owner {
    process: u64::MAX,
    generation: 1,
};
fn bit(dev: Device, kind: u8, code: u16) -> Result<bool> {
    w8(dev.config, 0x11);
    w8(dev.config + 1, kind);
    let size = r8(dev.config + 2);
    need(size <= 128, "input_bitmap_size")?;
    Ok(code / 8 < u16::from(size)
        && r8(dev.config + 8 + u64::from(code / 8)) & (1 << (code % 8)) != 0)
}
fn absolute(dev: Device, axis: u8) -> Result<()> {
    w8(dev.config, 0x12);
    w8(dev.config + 1, axis);
    need(
        r8(dev.config + 2) == 20 && r32(dev.config + 8) == 0 && r32(dev.config + 12) == 32767,
        "input_abs_range",
    )
}
pub(crate) fn initialize() -> Result<()> {
    for slot in 0..2 {
        let dev = discover((4 + slot as u16) * 8, slot)?;
        need(dev.id == 0x1052, "input_device_id")?;
        unsafe {
            (&mut *RESOURCES.0.get())[slot] = Some(dev.resource);
        }
        negotiate(dev)?;
        if slot == 0 {
            for code in [1, 28, 29, 30, 42, 54, 56, 57, 97, 100] {
                need(bit(dev, 1, code)?, "input_keys")?;
            }
        } else {
            for code in [272, 273, 274] {
                need(bit(dev, 1, code)?, "input_buttons")?;
            }
            for axis in 0..2 {
                need(bit(dev, 3, axis as u16)?, "input_axes")?;
                absolute(dev, axis)?;
            }
        }
        let grant =
            unsafe { (&mut *BROKER.0.get()).grant(OWNER, dev.resource, d::MAP | d::IRQ | d::DMA)? };
        let (p, mut dma) = allocate_dma()?;
        let notify = queue(dev, p, 0)?;
        // Statusq is enabled empty; this subset changes no LEDs.
        let _ = queue(dev, p, 1)?;
        for head in 0..8u16 {
            let address = p.0 + 12288 + u64::from(head) * 8;
            d::chain(
                &[d::Descriptor {
                    address,
                    len: 8,
                    flags: 2,
                    next: 0,
                }],
                0,
                dma.range(),
                d::Direction::FromDevice,
            )?;
            put(p, u64::from(head) * 16, &address.to_le_bytes());
            put(p, u64::from(head) * 16 + 8, &8u32.to_le_bytes());
            put(p, u64::from(head) * 16 + 12, &2u16.to_le_bytes());
            dma.submit(head)?;
            put(p, 4100 + u64::from(head) * 2, &head.to_le_bytes());
        }
        fence(Ordering::Release);
        put(p, 4098, &8u16.to_le_bytes());
        fence(Ordering::SeqCst);
        ISRS[slot].store(dev.isr, Ordering::Relaxed);
        timer::device_route(dev.irq, false)?;
        command(dev.bdf, 2 | 4);
        w8(dev.common + 20, 15);
        need(r8(dev.common + 20) == 15, "input_driver_ok")?;
        w16(notify, 0);
        unsafe {
            (&mut *INPUT.0.get())[slot] = Some(Ring {
                dev,
                p,
                dma,
                grant,
                used: 0,
                avail: 8,
                notify,
                sequence: 0,
            });
        }
        event(
            "m1.input_ready",
            format_args!(
                "slot={slot} queue=8 event_bytes=8 status_empty=true keymap=us-ascii min=0 max=32767"
            ),
        );
    }
    Ok(())
}
/// Peek a copied completion; successful user copy is required before recycle.
pub(crate) fn read(
    mut copy: impl FnMut(&mut [u8]) -> core::result::Result<(), u64>,
) -> Result<u64> {
    for slot in 0..2 {
        let Some(r) = (unsafe { &mut *INPUT.0.get() })[slot].as_mut() else {
            continue;
        };
        need(r8(r.dev.common + 20) == 15, "input_device_reset")?;
        let used = get16(r.p, 8194);
        fence(Ordering::Acquire);
        need(used.wrapping_sub(r.used) <= 8, "input_used_jump")?;
        if used == r.used {
            continue;
        }
        // Validate the complete visible completion batch before republishing a
        // head. Duplicate IDs within that batch cannot represent legitimate reuse.
        let mut seen = 0u16;
        for n in 0..used.wrapping_sub(r.used) {
            let at = 8196 + u64::from(r.used.wrapping_add(n) % 8) * 8;
            let head = get32(r.p, at);
            let len = get32(r.p, at + 4);
            need(
                head < 8 && len == 8 && seen & (1 << head) == 0,
                "input_batch",
            )?;
            r.dma.check_completion(head, len, 8)?;
            seen |= 1 << head;
        }
        let at = 8196 + u64::from(r.used % 8) * 8;
        let head = get32(r.p, at);
        let len = get32(r.p, at + 4);
        need(head < 8 && len == 8, "input_completion")?;
        let mut bytes = [0; 32];
        bytes[..8].copy_from_slice(
            &r.sequence
                .checked_add(1)
                .ok_or(error("input", "sequence"))?
                .to_le_bytes(),
        );
        bytes[8..16].copy_from_slice(&timer::elapsed()?.to_le_bytes());
        bytes[16..24].copy_from_slice(&(slot as u64).to_le_bytes());
        for n in 0..8 {
            bytes[24 + n] = paging::runtime_read(r.p, 12288 + u64::from(head) * 8 + n as u64);
        }
        // Verify DMA ownership before copying or releasing the entry, including duplicate heads.
        // read() is single-threaded with interrupts disabled; no mutation occurs across a copy.
        r.dma.check_completion(head, len, 8)?;
        if copy(&mut bytes).is_err() {
            return Err(error("input", "copy"));
        }
        r.dma.complete(head, len, 8)?;
        r.sequence += 1;
        r.used = r.used.wrapping_add(1);
        r.dma.recycle(head as u16)?;
        put(
            r.p,
            4100 + u64::from(r.avail % 8) * 2,
            &(head as u16).to_le_bytes(),
        );
        fence(Ordering::Release);
        r.avail = r.avail.wrapping_add(1);
        put(r.p, 4098, &r.avail.to_le_bytes());
        fence(Ordering::SeqCst);
        w16(r.notify, 0);
        #[cfg(feature = "input-test")]
        record(
            "m1.input_raw",
            "desktop",
            "ok",
            format_args!(
                "device={slot} sequence={} ns={} kind={} code={} value={} head={head} irq={}",
                r.sequence,
                u64::from_le_bytes(bytes[8..16].try_into().unwrap()),
                u16::from_le_bytes(bytes[24..26].try_into().unwrap()),
                u16::from_le_bytes(bytes[26..28].try_into().unwrap()),
                i32::from_le_bytes(bytes[28..32].try_into().unwrap()),
                IRQS[slot].load(Ordering::Relaxed)
            ),
        );
        return Ok(32);
    }
    Ok(0)
}
pub(crate) fn stop() -> Result<()> {
    for slot in 0..2 {
        if let Some(r) = (unsafe { &mut *INPUT.0.get() })[slot].as_mut() {
            r.dma.stop();
            need(reset(r.dev, Some(&mut r.dma), false)?, "input_quarantine")?;
            ISRS[slot].store(0, Ordering::Relaxed);
            timer::device_route(r.dev.irq, true)?;
        }
        if let Some(r) = (unsafe { &mut *INPUT.0.get() })[slot].take() {
            let range = unsafe { (&mut *RETAIN.0.get()).release(r.dma) }
                .map_err(|_| error("input", "release"))?;
            paging::runtime_release(r.p, (range.len / 4096) as usize);
            unsafe {
                (&mut *BROKER.0.get()).revoke(OWNER, r.grant)?;
                (&mut *RESOURCES.0.get())[slot] = None;
            }
            for page in 0..r.dev.pages {
                paging::driver_unmap(slot as u64 * 16 + page)?;
            }
        }
    }
    need(
        unsafe { (&*RETAIN.0.get()).inventory() } == (0, 0),
        "input_retention_settlement",
    )?;
    record(
        "m1.input_stopped",
        "desktop",
        "ok",
        format_args!("dma_bytes=0 grants=0 reset_before_release=true"),
    );
    Ok(())
}
