//! Single-core xAPIC/HPET adapter. No allocation or logging in IRQ context.
use crate::{Local, record};
use continuum_kernel::{Result, error, platform as p};
use core::{
    arch::{asm, global_asm},
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
};
static BUFFER: Local<[u8; p::MAX_TABLE]> = Local(core::cell::UnsafeCell::new([0; p::MAX_TABLE]));
static APIC: AtomicU64 = AtomicU64::new(0);
static TICKS: AtomicU64 = AtomicU64::new(0);
static OVERFLOW: AtomicBool = AtomicBool::new(false);
static RESOLUTION: AtomicU64 = AtomicU64::new(0);
static SPURIOUS: AtomicU64 = AtomicU64::new(0);
#[cfg(feature = "scheduler-test")]
static OBSERVER: Local<(u64, u64)> = Local(core::cell::UnsafeCell::new((0, 0)));
/// Diagnostic HPET elapsed observation; not the P3-08 public deadline clock.
#[cfg(feature = "scheduler-test")]
pub(crate) fn sample_ns() -> u64 {
    let (base, fs) = unsafe { OBSERVER.0.get().read() };
    assert!(base != 0);
    ((reference(base) as u128 * fs as u128) / 1_000_000) as u64
}
#[cfg(feature = "user-boundary-test")]
static USER_INTERRUPTS: AtomicU64 = AtomicU64::new(0);
#[cfg(feature = "user-boundary-test")]
pub(crate) fn user_interrupts() -> u64 {
    USER_INTERRUPTS.load(Ordering::Relaxed)
}
global_asm!(include_str!("timer-entry.S"), interrupt = sym interrupt);
unsafe extern "C" {
    pub fn timer_vector();
    pub fn spurious_vector();
    fn timer_sentinels() -> u64;
}
pub fn mode(s: &str) -> Option<u64> {
    match s {
        "timer-progress" => Some(20),
        "timer-masked" => Some(21),
        "clock-read" => Some(22),
        "idle-wakeup" => Some(23),
        "timer-exception" => Some(24),
        "acpi-corrupt" => Some(25),
        "timer-unsupported" => Some(26),
        "timer-calibration" => Some(27),
        _ => None,
    }
}
fn need(ok: bool, reason: &'static str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(error("timer", reason))
    }
}
unsafe fn read32(base: u64, offset: u64) -> u32 {
    unsafe { ((base + offset) as *const u32).read_volatile() }
}
unsafe fn write32(base: u64, offset: u64, v: u32) {
    unsafe { ((base + offset) as *mut u32).write_volatile(v) }
}
unsafe fn read64(base: u64, offset: u64) -> u64 {
    unsafe { ((base + offset) as *const u64).read_volatile() }
}
unsafe fn write64(base: u64, offset: u64, v: u64) {
    unsafe { ((base + offset) as *mut u64).write_volatile(v) }
}
fn msr(id: u32) -> u64 {
    let (lo, hi): (u32, u32);
    unsafe {
        asm!("rdmsr",in("ecx") id,out("eax")lo,out("edx")hi,options(nostack));
    }
    (hi as u64) << 32 | lo as u64
}
extern "C" fn interrupt(vector: u64, _cs: u64, _frame: *mut u8) {
    if vector == 255 {
        SPURIOUS.store(
            SPURIOUS.load(Ordering::Relaxed).saturating_add(1),
            Ordering::Relaxed,
        );
        return;
    }
    // Only this non-nested handler writes the counter. Readers use one atomic
    // load; no retry loop, lock or allocation occurs in interrupt context.
    #[cfg(feature = "user-boundary-test")]
    if _cs == continuum_kernel::user_boundary::USER_CS {
        USER_INTERRUPTS.store(
            USER_INTERRUPTS.load(Ordering::Relaxed).saturating_add(1),
            Ordering::Relaxed,
        );
    }
    if let Some(next) = TICKS.load(Ordering::Relaxed).checked_add(1) {
        TICKS.store(next, Ordering::Relaxed);
    } else {
        OVERFLOW.store(true, Ordering::Relaxed);
    }
    // LAPIC EOI is the only device action. HPET generates no interrupts.
    unsafe {
        write32(APIC.load(Ordering::Relaxed), 0xb0, 0);
    }
    #[cfg(feature = "scheduler-test")]
    crate::user_entry::scheduling::tick(_cs, _frame);
}
fn table(base: u64, b: &mut [u8; p::MAX_TABLE]) -> Result<&mut [u8]> {
    crate::paging::copy_acpi(base, &mut b[..36])?;
    let n = p::table_len(&b[..36])?;
    crate::paging::copy_acpi(base, &mut b[..n])?;
    Ok(&mut b[..n])
}
fn discover(mode: u64) -> Result<(u64, u64, u64, u32)> {
    let info = unsafe { &*crate::BOOT_INFO.0.get() };
    let root = info.acpi_root.ok_or(error("rsdp", "missing"))?.0;
    let mut r = [0; 36];
    crate::paging::copy_acpi(root, &mut r)?;
    #[cfg(feature = "timer-test")]
    if mode == 25 {
        r[16] ^= 1;
    }
    let x = p::rsdp(&r)?;
    let mut entries = [0; 128];
    // Single bootstrap caller, IF clear; only this scope borrows BUFFER.
    let buffer = unsafe { &mut *BUFFER.0.get() };
    let n = p::xsdt(table(x, buffer)?, &mut entries)?;
    let bsp = (core::arch::x86_64::__cpuid(1).ebx >> 24) as u8;
    let (mut apic, mut hpet) = (None, None);
    for address in &entries[..n] {
        let b = table(*address, buffer)?;
        let sig: [u8; 4] = b[..4].try_into().unwrap();
        p::table(b, &sig)?;
        match &sig {
            b"APIC" => {
                need(apic.is_none(), "duplicate_madt")?;
                apic = Some(p::madt(b, bsp)?);
            }
            b"HPET" => {
                need(hpet.is_none(), "duplicate_hpet")?;
                hpet = Some(p::hpet(b)?);
            }
            _ => {}
        }
    }
    #[cfg(feature = "timer-test")]
    if mode == 26 {
        hpet = None;
    }
    let (a, io) = apic.ok_or(error("madt", "missing"))?;
    let (h, id) = hpet.ok_or(error("hpet", "missing"))?;
    need(a != h && io != h, "device_overlap")?;
    record(
        "m1.acpi_ready",
        "timer",
        "ok",
        format_args!(
            "rsdp={root:#x} xsdt={x:#x} tables={n} apic={a:#x} ioapic={io:#x} hpet={h:#x} checksums=true ranges=true"
        ),
    );
    let _ = mode;
    Ok((a, io, h, id))
}
fn reference(base: u64) -> u64 {
    unsafe { read64(base, 0xf0) }
}
fn wait(base: u64, start: u64, ticks: u64) -> Result<u64> {
    for _ in 0..20_000_000 {
        let d = p::delta(reference(base), start, 64)?;
        if d >= ticks {
            return Ok(d);
        }
        core::hint::spin_loop();
    }
    Err(error("timer", "reference_timeout"))
}
pub fn uptime() -> Result<(u64, u64)> {
    let resolution = RESOLUTION.load(Ordering::Relaxed);
    need(resolution != 0, "clock_uninitialized")?;
    need(!OVERFLOW.load(Ordering::Relaxed), "counter_overflow")?;
    let count = TICKS.load(Ordering::Relaxed);
    Ok((
        count,
        count
            .checked_mul(resolution)
            .ok_or(error("clock", "overflow"))?,
    ))
}
pub fn establish(mode: u64) -> Result<()> {
    need(
        core::arch::x86_64::__cpuid(1).edx & (1 << 16) != 0,
        "no_pat",
    )?;
    need(
        core::arch::x86_64::__cpuid(1).edx & (1 << 9) != 0,
        "no_apic",
    )?;
    let pat = msr(0x277);
    need(pat & 255 == 6 && (pat >> 24) & 255 == 0, "cache_policy")?;
    let (ap, io, hp, id) = discover(mode)?;
    let apic_msr = msr(0x1b);
    need(
        apic_msr & 0x000f_ffff_ffff_f000 == ap
            && apic_msr & (1 << 11) != 0
            && apic_msr & (1 << 10) == 0
            && apic_msr & (1 << 8) != 0,
        "apic_mode",
    )?;
    let a = crate::paging::device(ap, 0)?;
    let h = crate::paging::device(hp, 1)?;
    let io = crate::paging::device(io, 2)?;
    APIC.store(a, Ordering::Relaxed);
    unsafe {
        need(
            read32(a, 0x20) >> 24 == core::arch::x86_64::__cpuid(1).ebx >> 24,
            "apic_id",
        )?;
        need((read32(a, 0x30) >> 16) & 255 == 5, "apic_version")?;
        // Mask every redirection entry in the discovered single IOAPIC.
        write32(io, 0, 1);
        let io_version = read32(io, 0x10);
        let max = (io_version >> 16) & 255;
        need(io_version & 255 == 0x20 && max < 120, "ioapic_version")?;
        for i in 0..=max {
            write32(io, 0, 0x10 + i * 2);
            let low = read32(io, 0x10);
            write32(io, 0x10, low | (1 << 16));
            need(read32(io, 0x10) & (1 << 16) != 0, "ioapic_mask")?;
        }
        // All local sources masked, PICs remain masked. LINT0/1 NMI/ExtINT
        // firmware policy is deliberately disabled for this single-core profile.
        for offset in [0x320, 0x330, 0x340, 0x350, 0x360, 0x370] {
            write32(a, offset, (1 << 16) | 65);
        }
        write32(a, 0x80, 0);
        write32(a, 0xf0, 0x100 | 255);
        write32(a, 0x3e0, 3); // divide by 16
        let cap = read64(h, 0);
        let fs = p::period(cap)?;
        #[cfg(feature = "scheduler-test")]
        OBSERVER.0.get().write((h, fs));
        need(cap as u32 == id, "hpet_id")?;
        write64(h, 0x10, 0);
        for i in 0..=((cap >> 8) & 31) {
            let offset = 0x100 + i * 0x20;
            write64(h, offset, read64(h, offset) & !((1 << 2) | (1 << 14)));
        }
        write64(h, 0xf0, 0);
        write64(h, 0x10, 1); // main counter only, legacy replacement disabled
        let target = 20_000_000_000_000u64.div_ceil(fs); // 20 ms in femtoseconds
        let before = reference(h);
        write32(a, 0x380, u32::MAX);
        let reference_ticks = wait(h, before, target)?;
        let elapsed = u32::MAX as u64 - read32(a, 0x390) as u64;
        write32(a, 0x380, 0);
        #[cfg(feature = "timer-test")]
        let elapsed = if mode == 27 { 0 } else { elapsed };
        let (initial, resolution) = p::calibration(elapsed, reference_ticks, fs)?;
        // P3-05's locked nominal quantum is 5 ms. Choose it before the one-time
        // delivered-tick epoch; M1's 10-ms calibration/regressions stay unchanged.
        let (initial, resolution) = if (36..=39).contains(&mode) {
            let half = initial / 2;
            need(half != 0, "scheduler_calibration")?;
            (half, resolution * u64::from(half) / u64::from(initial))
        } else {
            (initial, resolution)
        };
        need(resolution > 0, "resolution")?;
        record(
            "m1.timer_calibrated",
            "timer",
            "ok",
            format_args!(
                "apic_elapsed={elapsed} hpet_ticks={reference_ticks} period_fs={fs} divide=16 initial={initial} resolution_ns={resolution} cache=uc"
            ),
        );
        // This is the one and only uptime initialization for this boot.
        need(TICKS.load(Ordering::Relaxed) == 0, "epoch_reinitialized")?;
        need(
            RESOLUTION.load(Ordering::Relaxed) == 0,
            "epoch_reinitialized",
        )?;
        RESOLUTION.store(resolution, Ordering::Relaxed);
        record(
            "m1.timer_ready",
            "timer",
            "ok",
            format_args!(
                "vector=64 spurious=255 ticks=0 uptime_ns=0 resolution_ns={resolution} epoch=kernel pic=masked lvt=timer-only"
            ),
        );
        write32(a, 0x320, 64 | (1 << 17));
        write32(a, 0x380, initial);
        let mut last = 0;
        let mut changes = 0;
        for sample in 0..100 {
            let before = TICKS.load(Ordering::Relaxed);
            // Real interrupts return through IRETQ to the same continuation.
            // CLI on return lets snapshot read an exact count/time pair.
            need(timer_sentinels() == 1, "register_preservation")?;
            let (count, ns) = uptime()?;
            need(count > before && ns >= last, "progress")?;
            if ns > last {
                changes += 1;
            }
            last = ns;
            record(
                "m1.clock_sample",
                "timer",
                "ok",
                format_args!(
                    "index={sample} ticks={count} uptime_ns={ns} continuation=true registers=true"
                ),
            );
        }
        if mode == 21 {
            write32(a, 0x320, 64 | (1 << 17) | (1 << 16));
            // Drain an already pending interrupt before starting the interval.
            asm!("sti", "nop", "nop", "cli", options(nostack));
            let (before, _) = uptime()?;
            let start = reference(h);
            asm!("sti", options(nostack));
            let polled = wait(h, start, target * 3);
            asm!("cli", options(nostack));
            let polled = polled?;
            let (after, _) = uptime()?;
            need(before == after, "mask_delivery")?;
            write32(a, 0x320, 64 | (1 << 17));
            write32(a, 0x380, initial);
            need(timer_sentinels() == 1, "register_preservation")?;
            let (resumed, _) = uptime()?;
            need(resumed > after, "unmask_delivery")?;
            record(
                "m1.timer_masked",
                "timer",
                "ok",
                format_args!(
                    "before={before} after={after} resumed={resumed} hpet_ticks={polled} responsive=true"
                ),
            );
        }
        let (count, ns) = uptime()?;
        record(
            "m1.timer_progress",
            "timer",
            "ok",
            format_args!(
                "ticks={count} uptime_ns={ns} reads=100 changes={changes} halts=100 registers=true spurious={}",
                SPURIOUS.load(Ordering::Relaxed)
            ),
        );
        #[cfg(feature = "timer-test")]
        if mode == 24 {
            asm!("sti", options(nostack));
            unsafe extern "C" {
                fn diagnostic_invalid_opcode() -> !;
            }
            diagnostic_invalid_opcode();
        }
        // Keep delivery enabled after establishment. Handlers never touch logs.
        asm!("sti", options(nostack));
    }
    Ok(())
}
#[cfg(not(feature = "emulator-test"))]
pub fn idle() -> ! {
    loop {
        unsafe {
            asm!("sti", "hlt", options(nostack));
        }
    }
}
