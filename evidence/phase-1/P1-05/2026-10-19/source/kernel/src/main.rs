#![no_std]
#![no_main]

mod boot;
mod exceptions;
mod paging;
use continuum_kernel::{BootInfoV1, Error, Text, identity};
use core::{
    arch::asm,
    cell::UnsafeCell,
    fmt::{self, Write},
    panic::PanicInfo,
    sync::atomic::{AtomicBool, Ordering},
};
use limine::{
    BaseRevision,
    request::{RequestsEndMarker, RequestsStartMarker},
};

#[used]
#[unsafe(link_section = ".limine_requests_start")]
static START: RequestsStartMarker = RequestsStartMarker::new();
#[used]
#[unsafe(link_section = ".limine_requests")]
static BASE: BaseRevision = BaseRevision::with_revision(3);
#[used]
#[unsafe(link_section = ".limine_requests_end")]
static END: RequestsEndMarker = RequestsEndMarker::new();

// These privileged operations are only valid on the declared x86-64 COM1/QEMU
// platform, after Limine enters ring 0. No interrupt handler calls this logger.
unsafe fn out8(port: u16, value: u8) {
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
    }
}
unsafe fn in8(port: u16) -> u8 {
    let value: u8;
    unsafe {
        asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack, preserves_flags));
    }
    value
}
fn serial_init() {
    unsafe {
        out8(0x3f9, 0); // UART interrupts off
        out8(0x3fb, 0x80); // divisor latch
        out8(0x3f8, 1);
        out8(0x3f9, 0); // 115200 baud
        out8(0x3fb, 3); // 8N1
        out8(0x3fa, 0xc7);
        out8(0x3fc, 3);
    }
}
fn serial(bytes: &[u8]) -> bool {
    for &byte in bytes {
        let mut ready = false;
        // Iteration bound, not a time measurement; no guest clock exists yet.
        for _ in 0..100_000 {
            if unsafe { in8(0x3fd) } & 0x20 != 0 {
                ready = true;
                break;
            }
            core::hint::spin_loop();
        }
        if !ready {
            return false;
        }
        unsafe {
            out8(0x3f8, byte);
        }
    }
    true
}
fn halt() -> ! {
    loop {
        unsafe {
            asm!("cli", "hlt", options(nomem, nostack));
        }
    }
}
#[cfg(feature = "emulator-test")]
fn test_exit(value: u32) -> ! {
    // Emulator-only ABI: configured isa-debug-exit, 32-bit write to port 0xf4.
    unsafe {
        asm!("out dx, eax", in("dx") 0xf4u16, in("eax") value, options(nomem, nostack, preserves_flags));
    }
    halt()
}
// Single CPU with IF clear. Copy state before formatting; no mutable reference
// into diagnostic state remains live while output is attempted.
#[derive(Clone, Copy)]
struct LogState {
    session: Text<65>,
    sequence: u64,
}
struct Local<T>(UnsafeCell<T>);
unsafe impl<T> Sync for Local<T> {}
static LOG: Local<LogState> = Local(UnsafeCell::new(LogState {
    session: Text::EMPTY,
    sequence: 0,
}));
static BOOT_INFO: Local<BootInfoV1> = Local(UnsafeCell::new(BootInfoV1::EMPTY));
static LOG_BUSY: AtomicBool = AtomicBool::new(false);
static FATAL_BUSY: AtomicBool = AtomicBool::new(false);
struct Record {
    bytes: [u8; 512],
    len: usize,
}
impl Write for Record {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let end = self.len.checked_add(text.len()).ok_or(fmt::Error)?;
        if end > self.bytes.len() {
            return Err(fmt::Error);
        }
        self.bytes[self.len..end].copy_from_slice(text.as_bytes());
        self.len = end;
        Ok(())
    }
}
fn failure_exit() -> ! {
    #[cfg(feature = "emulator-test")]
    test_exit(0x11);
    #[cfg(not(feature = "emulator-test"))]
    halt()
}
fn record(event: &str, stage: &str, outcome: &str, detail: fmt::Arguments<'_>) {
    if LOG_BUSY.swap(true, Ordering::SeqCst) {
        fatal(event, stage, detail);
    }
    emit(event, stage, outcome, false, detail);
    LOG_BUSY.store(false, Ordering::SeqCst);
}
fn fatal(event: &str, stage: &str, detail: fmt::Arguments<'_>) -> ! {
    // A second reporting fault never acquires a lock or recursively formats.
    if FATAL_BUSY.swap(true, Ordering::SeqCst) {
        let _ = serial(b"\nm1.emergency_failed\n");
        failure_exit();
    }
    let emergency = LOG_BUSY.swap(true, Ordering::SeqCst);
    if emergency {
        let _ = serial(b"\n");
    }
    emit(event, stage, "fatal", emergency, detail);
    #[cfg(feature = "emulator-test")]
    test_exit(0x12);
    #[cfg(not(feature = "emulator-test"))]
    halt()
}
fn emit(event: &str, stage: &str, outcome: &str, emergency: bool, detail: fmt::Arguments<'_>) {
    // Single CPU; session is immutable after initialization. No reference into
    // LOG survives formatting or port IO; fatal reentry never resumes old code.
    let state = unsafe { LOG.0.get().read() };
    let Some(next) = state.sequence.checked_add(1) else {
        failure_exit()
    };
    unsafe {
        (*LOG.0.get()).sequence = next;
    }
    let session = if state.session.bytes().is_empty() {
        "unavailable"
    } else {
        state.session.as_str()
    };
    let mut line = Record {
        bytes: [0; 512],
        len: 0,
    };
    if write!(&mut line, "{event} version=4 build={} session={session} session_origin=host seq={} stage={stage} outcome={outcome} emergency={emergency} {detail}\n",
        env!("CONTINUUM_BUILD_ID"), state.sequence).is_err() || !serial(&line.bytes[..line.len]) {
        failure_exit();
    }
}
fn reject(e: Error) -> ! {
    record(
        "m1.boot_rejected",
        "metadata",
        "rejected",
        format_args!("field={} reason={}", e.field, e.reason),
    );
    failure_exit()
}

// Limine base revision 3: ring 0, SysV ABI without FP/SIMD, IF/DF clear,
// >=64 KiB loader-owned stack with a zero return address. This function never
// returns. No loader memory is reclaimed. Static storage is filled in place.
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.kernel_entry")]
pub extern "C" fn kernel_entry() -> ! {
    unsafe {
        asm!("cli", options(nomem, nostack));
    }
    serial_init();
    let _ = serial(b"\n");
    if !BASE.is_supported() {
        reject(continuum_kernel::error("base", "revision"));
    }
    if cfg!(feature = "fault-before-entry") {
        halt();
    }
    // Exclusive capture before publication; no allocator/interrupt/other CPU.
    let info = unsafe { &mut *BOOT_INFO.0.get() };
    if let Err(e) = unsafe { boot::capture(info) } {
        reject(e);
    }
    let arguments = info.arguments;
    let (session, fixture) = identity(&arguments).unwrap_or_else(|e| reject(e));
    let mut session_copy = [0u8; 65];
    session_copy[..64].copy_from_slice(session.as_bytes());
    unsafe {
        (*LOG.0.get())
            .session
            .copy_cstr(&session_copy, "session")
            .unwrap_or_else(|e| reject(e));
    }
    record(
        "m1.kernel_entry",
        "entry",
        "ok",
        format_args!("fixture={fixture}"),
    );
    #[cfg(feature = "boot-inject")]
    if let Err(e) = boot::inject(info, fixture) {
        reject(e);
    }
    #[cfg(not(feature = "boot-inject"))]
    if fixture != "normal"
        && !(cfg!(feature = "exception-test") && exceptions::mode(fixture).is_some())
        && !(cfg!(feature = "memory-test") && paging::mode(fixture).is_some())
    {
        reject(continuum_kernel::error("fixture", "disabled"));
    }
    if let Err(e) = info.validate() {
        reject(e);
    }
    let info: &'static BootInfoV1 = unsafe { &*BOOT_INFO.0.get() };
    record(
        "m1.boot_info_validated",
        "metadata",
        "ok",
        format_args!(
            "boot_info=1 regions={} modules={} framebuffers={} mode={} acpi={} hhdm={:#x}",
            info.region_count,
            info.module_count,
            info.framebuffer_count,
            if info.framebuffer_count == 0 {
                "serial-only"
            } else {
                "framebuffer"
            },
            if info.acpi_root.is_some() {
                "locator-only"
            } else {
                "absent"
            },
            info.hhdm.0
        ),
    );
    for (i, r) in info.regions().iter().enumerate() {
        record(
            "m1.memory_region",
            "summary",
            "ok",
            format_args!(
                "index={i} base={:#x} bytes={:#x} kind={}",
                r.range.base, r.range.len, r.kind
            ),
        );
    }
    for (i, f) in info.framebuffers[..info.framebuffer_count]
        .iter()
        .enumerate()
    {
        record(
            "m1.framebuffer",
            "summary",
            "ok",
            format_args!(
                "index={i} base={:#x} bytes={:#x} width={} height={} pitch={} bpp={} red={}:{} green={}:{} blue={}:{} cache=loader-mapping",
                f.range.base,
                f.range.len,
                f.width,
                f.height,
                f.pitch,
                f.bpp,
                f.sizes[0],
                f.shifts[0],
                f.sizes[1],
                f.shifts[1],
                f.sizes[2],
                f.shifts[2]
            ),
        );
    }
    for (i, m) in info.modules[..info.module_count].iter().enumerate() {
        record(
            "m1.module",
            "summary",
            "ok",
            format_args!(
                "index={i} base={:#x} bytes={:#x} path_bytes={} argument_bytes={}",
                m.range.base,
                m.range.len,
                m.path.bytes().len(),
                m.arguments.bytes().len()
            ),
        );
    }
    if let Some(p) = info.acpi_root {
        record(
            "m1.acpi_locator",
            "summary",
            "ok",
            format_args!("physical={:#x} table_validated=false", p.0),
        );
    }
    info.reservations(|r, reason| {
        record(
            "m1.reservation",
            "summary",
            "ok",
            format_args!(
                "base={:#x} bytes={:#x} reason={reason:?} retained=true",
                r.base, r.len
            ),
        )
    });
    exceptions::start(
        exceptions::mode(fixture)
            .or_else(|| paging::mode(fixture))
            .unwrap_or(0),
    )
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    exceptions::panic(info)
}
