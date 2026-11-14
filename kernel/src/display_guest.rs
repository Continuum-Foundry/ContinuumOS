//! Temporary trusted software scanout mediator. Display bytes never reach apps.
use crate::{Local, record};
use continuum_kernel::{
    Result,
    device::Owner,
    display::{Lease, Output},
    error,
};
use core::{arch::asm, cell::UnsafeCell};
static OUTPUT: Local<Option<(Output, u64)>> = Local(UnsafeCell::new(None));
static LEASE: Local<Lease> = Local(UnsafeCell::new(Lease::EMPTY));
static OWNER: Local<Option<(Owner, u64)>> = Local(UnsafeCell::new(None));
pub(crate) fn initialize() -> Result<()> {
    let info = unsafe { &*crate::BOOT_INFO.0.get() };
    let o = match Output::checked(info) {
        Ok(o) => o,
        Err(e) => {
            record(
                "m1.display_unavailable",
                "display",
                "ok",
                format_args!(
                    "field={} reason={} serial=true desktop_ready=false",
                    e.field, e.reason
                ),
            );
            return Ok(());
        }
    };
    #[cfg(feature = "display-test")]
    {
        if crate::paging::scanout(o, Some(2)) != Err(error("display", "injected_map")) {
            return Err(error("display", "map_unwind"));
        }
        record(
            "m1.display_map_unwind",
            "display",
            "ok",
            format_args!("pages=2 retained=0"),
        );
    }
    let base = crate::paging::scanout(o, None)?;
    // Full admitted extent, including every padding byte. No firmware calls.
    for n in 0..o.fb.range.len {
        unsafe {
            ((base + n) as *mut u8).write_volatile(0);
        }
    }
    unsafe {
        asm!("mfence", options(nostack, preserves_flags));
        OUTPUT.0.get().write(Some((o, base)));
    }
    #[cfg(feature = "display-test")]
    for n in 0..o.fb.range.len {
        if unsafe { ((base + n) as *const u8).read_volatile() } != 0 {
            return Err(error("display", "initial_clear"));
        }
    }
    record(
        "m1.display_output",
        "display",
        "ok",
        format_args!(
            "width={} height={} pitch={} bpp={} red={} green={} blue={} physical={:#x} bytes={} mapped_bytes={} cache=uc user=false writable=true nx=true initial_clear=complete padding=zero fence=cpu-copy vsync=false",
            o.fb.width,
            o.fb.height,
            o.fb.pitch,
            o.fb.bpp,
            o.fb.shifts[0],
            o.fb.shifts[1],
            o.fb.shifts[2],
            o.fb.range.base,
            o.fb.range.len,
            o.pages.len
        ),
    );
    Ok(())
}
pub(crate) fn owner_dead(process: u64, generation: u64) -> Result<()> {
    let owner = Owner {
        process,
        generation,
    };
    if unsafe { *OWNER.0.get() }.is_some_and(|(a, _)| a == owner) {
        unsafe {
            (&mut *LEASE.0.get()).release(owner)?;
            OWNER.0.get().write(None);
        }
        record(
            "m1.display_release",
            "display",
            "ok",
            format_args!("process={process} generation={generation} completed=true"),
        );
    }
    Ok(())
}
/// RGBX row copy. The caller's whole row is copied and unpinned before any MMIO.
/// At most 7680 private bytes; no reader retains a user pointer after return.
pub(crate) fn call(
    op: u64,
    role: u64,
    owner: Owner,
    a: u64,
    b: u64,
    c: u64,
    g: u64,
    mut copy: impl FnMut(u64, &mut [u8]) -> core::result::Result<(), u64>,
) -> Option<Result<u64>> {
    if !(0x4100..=0x4104).contains(&op) {
        return None;
    }
    let result = (|| {
        let (o, base) = unsafe { *OUTPUT.0.get() }.ok_or(error("display", "unavailable"))?;
        if role != 1 {
            return Err(error("display", "authority"));
        }
        if op == 0x4100 {
            let mut bound = unsafe { *OWNER.0.get() };
            if bound.is_none() {
                let generation = unsafe { (&mut *LEASE.0.get()).bind(owner, o.pages)? };
                bound = Some((owner, generation));
                unsafe {
                    OWNER.0.get().write(bound);
                }
            }
            let (active, generation) = bound.unwrap();
            if active != owner {
                return Err(error("display", "authority"));
            }
            return Ok(generation);
        }
        unsafe {
            (&mut *LEASE.0.get()).check(owner, g)?;
        }
        if op == 0x4101 {
            return Ok(o.fb.width | (o.fb.height << 32));
        }
        if op == 0x4103 || op == 0x4104 {
            return Err(error("display", "capture_or_mapping_denied"));
        }
        // b packs signed 32-bit x/y; c is RGBX pixel count, never a byte extent.
        let row = o.row(
            b as u32 as i32 as i64,
            (b >> 32) as u32 as i32 as i64,
            c as usize,
        )?;
        let mut source = [0u8; 1920 * 4];
        copy(a, &mut source[..c as usize * 4]).map_err(|_| error("display", "source"))?;
        let begin = crate::timer::sample_ns();
        if let Some(row) = row {
            let bytes = (o.fb.bpp / 8) as usize;
            for n in 0..row.count {
                let at = (row.skip + n) * 4;
                let pixel = o
                    .pixel([source[at], source[at + 1], source[at + 2]])
                    .to_le_bytes();
                for k in 0..bytes {
                    unsafe {
                        ((base + row.offset + (n * bytes + k) as u64) as *mut u8)
                            .write_volatile(pixel[k]);
                    }
                }
            }
        }
        unsafe {
            asm!("mfence", options(nostack, preserves_flags));
        }
        // Source release is synchronous even when clipping produced no writes.
        crate::user_entry::display_copy(
            owner.process,
            owner.generation,
            g,
            o.fb.width,
            o.fb.height,
            b,
            crate::timer::sample_ns() - begin,
        );
        Ok(c)
    })();
    #[cfg(feature = "display-test")]
    if op != 0x4102 || result.is_err() {
        record(
            "m1.display_call",
            "display",
            "ok",
            format_args!(
                "op={op} role={role} process={} generation={} supplied_generation={g} accepted={}",
                owner.process,
                owner.generation,
                result.is_ok()
            ),
        );
    }
    Some(result)
}
