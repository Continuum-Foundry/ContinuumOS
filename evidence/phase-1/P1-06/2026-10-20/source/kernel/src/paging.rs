//! x86-64 single-core memory adapter. See memory-ownership-runbook.md for unsafe review.
use crate::{Local, record};
use continuum_kernel::{
    PhysAddr, Result, VirtAddr, error,
    memory::{Frames, LIMIT, NX, PAGE, Permission, Tables, WRITE},
};
use core::{
    arch::{asm, global_asm},
    ptr::addr_of,
};
// Fixed image-owned storage, not stack temporaries or a general heap.
const TABLE_CAPACITY: usize = 2304;
static FRAMES: Local<Frames> = Local(core::cell::UnsafeCell::new(Frames::EMPTY));
static TABLES: Local<Tables<TABLE_CAPACITY>> = Local(core::cell::UnsafeCell::new(Tables::EMPTY));
const SCRATCH: u64 = 0xffff_c000_0000_0000;
unsafe extern "C" {
    static __image_start: u8;
    static __text_end: u8;
    static __rodata_end: u8;
    static __image_end: u8;
    static diagnostic_nx_page: u8;
    #[cfg(feature = "memory-test")]
    fn diagnostic_guard(address: u64) -> !;
    #[cfg(feature = "memory-test")]
    fn diagnostic_write(address: u64) -> !;
    #[cfg(feature = "memory-test")]
    fn diagnostic_execute(address: u64) -> !;
    #[cfg(feature = "memory-test")]
    fn diagnostic_double(stack_bottom: u64) -> !;
    #[cfg(feature = "memory-test")]
    static diagnostic_guard_site: u8;
    #[cfg(feature = "memory-test")]
    static diagnostic_write_site: u8;
    #[cfg(feature = "memory-test")]
    static diagnostic_double_site: u8;
}
global_asm!(include_str!("memory-entry.S"));
pub fn mode(fixture: &str) -> Option<u64> {
    match fixture {
        "memory-ownership" => Some(10),
        "memory-exhaustion" => Some(11),
        "guard-fault" => Some(12),
        "write-protect-fault" => Some(13),
        "execute-disable-fault" => Some(14),
        "emergency-stack" => Some(15),
        _ => None,
    }
}
fn need(condition: bool) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(error("memory", "self_check"))
    }
}
fn read_cr3() -> u64 {
    let v;
    unsafe {
        asm!("mov {}, cr3", out(reg) v, options(nomem, nostack));
    }
    v
}
fn invalidate(v: u64) {
    unsafe {
        asm!("invlpg [{}]", in(reg) v, options(nostack, preserves_flags));
    }
}
/// Bootstrap only, IF clear. Copy firmware ACPI bytes through a short-lived RO/NX
/// WB window, never through a loader pointer. No reference escapes the mapping.
pub fn copy_acpi(base: u64, output: &mut [u8]) -> Result<()> {
    let info = unsafe { &*crate::BOOT_INFO.0.get() };
    continuum_kernel::platform::acpi_range(info.regions(), base, output.len() as u64)?;
    let tables = unsafe { &mut *TABLES.0.get() };
    let mut at = 0;
    while at < output.len() {
        let p = base + at as u64;
        let offset = (p % PAGE) as usize;
        let count = (4096 - offset).min(output.len() - at);
        tables.map(
            VirtAddr(SCRATCH),
            PhysAddr(p & !(PAGE - 1)),
            Permission::ReadOnly,
        )?;
        invalidate(SCRATCH);
        for i in 0..count {
            output[at + i] =
                unsafe { ((SCRATCH + offset as u64 + i as u64) as *const u8).read_volatile() };
        }
        tables.unmap(VirtAddr(SCRATCH))?;
        invalidate(SCRATCH);
        at += count;
    }
    Ok(())
}
pub fn device(base: u64, slot: u64) -> Result<u64> {
    let info = unsafe { &*crate::BOOT_INFO.0.get() };
    continuum_kernel::platform::device_range(info.regions(), base)?;
    need(slot < 3)?;
    let v = SCRATCH + PAGE * (slot + 1);
    let tables = unsafe { &mut *TABLES.0.get() };
    tables.map(VirtAddr(v), PhysAddr(base), Permission::Device)?;
    invalidate(v);
    need(tables.inspect(VirtAddr(v))? & !(0x20 | 0x40) == base | NX | 0x1b)?;
    Ok(v)
}
fn zero(p: PhysAddr, hhdm: u64) -> Result<()> {
    let address = hhdm.checked_add(p.0).ok_or(error("zero", "overflow"))?;
    continuum_kernel::virtual_range(address, PAGE, "zero")?;
    // Only eligible usable RAM, already mapped RW/NX in our active tables.
    unsafe {
        core::ptr::write_bytes(address as *mut u8, 0, PAGE as usize);
    }
    Ok(())
}
fn check_zero(address: u64) -> Result<()> {
    // Volatile observations keep the guest experiment independent of compiler
    // reasoning about the earlier zero/dirty stores.
    for offset in 0..PAGE {
        need(unsafe { ((address + offset) as *const u8).read_volatile() } == 0)?;
    }
    Ok(())
}
pub fn establish(mode: u64) -> Result<()> {
    // Single CPU, IF clear; references are never published to handlers. All
    // handlers terminate, so fatal interruption cannot resume these mutations.
    let info = unsafe { &*crate::BOOT_INFO.0.get() };
    let frames = unsafe { &mut *FRAMES.0.get() };
    let tables = unsafe { &mut *TABLES.0.get() };
    let start = addr_of!(__image_start) as u64;
    let text_end = addr_of!(__text_end) as u64;
    let ro_end = addr_of!(__rodata_end) as u64;
    let end = addr_of!(__image_end) as u64;
    let physical = |v: u64| -> Result<PhysAddr> {
        let offset = v.checked_sub(start).ok_or(error("image", "address"))?;
        if v >= end {
            return Err(error("image", "address"));
        }
        Ok(PhysAddr(
            info.kernel
                .base
                .checked_add(offset)
                .ok_or(error("image", "overflow"))?,
        ))
    };
    need(start < text_end && text_end < ro_end && ro_end < end)?;
    frames.initialize(info.regions())?;
    let mut reserved = Ok(());
    info.reservations(|r, _| {
        if reserved.is_ok() {
            reserved = frames.reserve(r);
        }
    });
    reserved?;
    frames.seal();
    tables.initialize(physical(TABLES.0.get() as u64)?)?;
    let stacks = crate::exceptions::stacks();
    for v in (start..end).step_by(PAGE as usize) {
        if stacks.iter().any(|&s| v == s || v == s + 69632) {
            continue;
        }
        tables.map(
            VirtAddr(v),
            physical(v)?,
            if v < text_end {
                Permission::ReadExecute
            } else if v < ro_end {
                Permission::ReadOnly
            } else {
                Permission::ReadWrite
            },
        )?;
    }
    // The only direct mapping is eligible ordinary RAM. Kernel image, loader,
    // modules, framebuffer, firmware and ACPI have no direct-map aliases.
    frames.each_eligible(|p| {
        tables.map(
            VirtAddr(
                info.hhdm
                    .0
                    .checked_add(p.0)
                    .ok_or(error("mapping", "overflow"))?,
            ),
            p,
            Permission::ReadWrite,
        )
    })?;
    need(
        tables.inspect(VirtAddr(addr_of!(diagnostic_nx_page) as u64))? & (NX | WRITE) == NX | WRITE,
    )?;
    let before = read_cr3();
    record(
        "m1.memory_switch",
        "memory",
        "ok",
        format_args!(
            "before={before:#x} after={:#x} table_pages={} table_capacity={TABLE_CAPACITY} physical_limit={LIMIT:#x} eligible_frames={} mapped_direct_frames={} kernel_pages={} reclaimed_bytes=0",
            tables.root().0,
            tables.used,
            frames.total,
            frames.total,
            (end - start) / PAGE - 6
        ),
    );
    // NX is required for this declared profile: fail instead of quietly weakening
    // W^X. No PCID/LA57 transition is supported. Clear PGE to flush loader globals.
    let extended = core::arch::x86_64::__cpuid(0x80000000);
    need(
        extended.eax >= 0x80000001 && core::arch::x86_64::__cpuid(0x80000001).edx & (1 << 20) != 0,
    )?;
    let cr4: u64;
    unsafe {
        asm!("mov {}, cr4",out(reg) cr4,options(nomem,nostack));
    }
    need(cr4 & ((1 << 17) | (1 << 12)) == 0)?;
    let (lo, hi): (u32, u32);
    unsafe {
        asm!("rdmsr",in("ecx") 0xc0000080u32,out("eax") lo,out("edx") hi,options(nostack));
        asm!("wrmsr",in("ecx") 0xc0000080u32,in("eax") lo | (1<<11),in("edx") hi,options(nostack));
        asm!("mov cr4, {}",in(reg) cr4 & !(1u64<<7),options(nostack));
        asm!("mov cr3, {}",in(reg) tables.root().0,options(nostack));
        let cr0: u64;
        asm!("mov {}, cr0",out(reg) cr0,options(nostack));
        asm!("mov cr0, {}",in(reg) cr0 | (1u64<<16),options(nostack));
    }
    let (actual_lo, actual_hi): (u32, u32);
    let actual_cr0: u64;
    unsafe {
        asm!("rdmsr",in("ecx") 0xc0000080u32,out("eax") actual_lo,out("edx") actual_hi,options(nostack));
        asm!("mov {}, cr0",out(reg) actual_cr0,options(nostack));
    }
    let _ = actual_hi;
    need(
        read_cr3() == tables.root().0
            && before != read_cr3()
            && actual_lo & (1 << 11) != 0
            && actual_cr0 & (1 << 16) != 0,
    )?;
    // Inspect every image leaf, every direct-map leaf and both ends of all stacks.
    for v in (start..end).step_by(PAGE as usize) {
        let guard = stacks.iter().any(|&s| v == s || v == s + 69632);
        let leaf = tables.inspect(VirtAddr(v))?;
        if guard {
            need(leaf == 0)?;
        } else {
            let permission = if v < text_end {
                1
            } else if v < ro_end {
                1 | NX
            } else {
                3 | NX
            };
            need(leaf & !(0x20 | 0x40) == physical(v)?.0 | permission)?;
        }
        need(tables.inspect(VirtAddr(info.hhdm.0 + physical(v)?.0))? == 0)?;
    }
    frames.each_eligible(|p| {
        need(tables.inspect(VirtAddr(info.hhdm.0 + p.0))? & !(0x20 | 0x40) == p.0 | 3 | NX)
    })?;
    // Allocate, dirty, release and reuse: newly supplied frames must be zeroed.
    let mut batch = [PhysAddr(0); 8];
    if mode == 11 {
        frames.set_test_quota(8)?;
    }
    for i in 0..8 {
        let p = frames.allocate(|p| zero(p, info.hhdm.0))?;
        need(frames.eligible(p) && !batch[..i].contains(&p))?;
        check_zero(info.hhdm.0 + p.0)?;
        for offset in 0..PAGE {
            unsafe {
                ((info.hhdm.0 + p.0 + offset) as *mut u8).write_volatile(0xa5);
            }
        }
        record(
            "m1.frame_test",
            "memory",
            "ok",
            format_args!(
                "index={i} physical={:#x} zeroed=true unique=true reserved=false",
                p.0
            ),
        );
        batch[i] = p;
    }
    if mode == 11 {
        need(frames.allocate(|p| zero(p, info.hhdm.0)) == Err(error("frames", "exhausted")))?;
    }
    for p in batch {
        frames.release(p)?;
    }
    need(frames.release(batch[0]) == Err(error("frames", "double_free")))?;
    need(
        frames.release(PhysAddr(LIMIT)).is_err()
            && frames.release(PhysAddr(batch[0].0 + 1)).is_err()
            && frames.release(PhysAddr(info.kernel.base)).is_err(),
    )?;
    let reused = frames.allocate(|p| zero(p, info.hhdm.0))?;
    need(reused == batch[0])?;
    check_zero(info.hhdm.0 + reused.0)?;
    // Exercise a real live remap + INVLPG, then remove the temporary alias.
    tables.map(VirtAddr(SCRATCH), reused, Permission::ReadWrite)?;
    invalidate(SCRATCH);
    unsafe {
        (SCRATCH as *mut u64).write_volatile(0x11223344);
    }
    need(unsafe { ((info.hhdm.0 + reused.0) as *const u64).read_volatile() } == 0x11223344)?;
    need(
        tables.map(VirtAddr(SCRATCH), batch[1], Permission::ReadWrite)
            == Err(error("mapping", "conflict")),
    )?;
    need(tables.unmap(VirtAddr(SCRATCH))? == reused)?;
    invalidate(SCRATCH);
    let second = frames.allocate(|p| zero(p, info.hhdm.0))?;
    tables.map(VirtAddr(SCRATCH), second, Permission::ReadWrite)?;
    invalidate(SCRATCH);
    need(unsafe { (SCRATCH as *const u64).read_volatile() } == 0)?;
    tables.unmap(VirtAddr(SCRATCH))?;
    invalidate(SCRATCH);
    frames.release(second)?;
    frames.release(reused)?;
    record(
        "m1.memory_ready",
        "memory",
        "ok",
        format_args!(
            "cr3={:#x} wp=true nx=true supervisor=true image_audited=true direct_audited=true guards=6 aliases=0 batch=8 reuse_zero=true invalid_free=rejected double_free=rejected conflict=rejected invalidation=passed exhaustion={} allocated={} test_quota={}",
            read_cr3(),
            if mode == 11 {
                "passed"
            } else {
                "not-requested"
            },
            frames.allocated,
            if mode == 11 { 8 } else { 0 }
        ),
    );
    #[cfg(feature = "memory-test")]
    unsafe {
        let (target, site) = match mode {
            12 => (stacks[0], addr_of!(diagnostic_guard_site) as u64),
            13 => (start, addr_of!(diagnostic_write_site) as u64),
            14 => (
                addr_of!(diagnostic_nx_page) as u64,
                addr_of!(diagnostic_nx_page) as u64,
            ),
            15 => (stacks[0] + PAGE, addr_of!(diagnostic_double_site) as u64),
            _ => return Ok(()),
        };
        record(
            "m1.fault_armed",
            "memory",
            "ok",
            format_args!("mode={mode} target={target:#x} site={site:#x}"),
        );
        match mode {
            12 => diagnostic_guard(target),
            13 => diagnostic_write(target),
            14 => diagnostic_execute(target),
            15 => diagnostic_double(target),
            _ => {}
        }
    }
    Ok(())
}
