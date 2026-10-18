#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};
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
fn fail(message: &[u8]) -> ! {
    let _ = serial(b"\n");
    let _ = serial(message);
    #[cfg(feature = "emulator-test")]
    test_exit(0x11);
    #[cfg(not(feature = "emulator-test"))]
    halt()
}

// Limine base revision 3: ring 0, SysV ABI without FP/SIMD, IF/DF clear,
// >=64 KiB loader-owned stack with a zero return address. This function never
// returns. No loader memory (stack, GDT, paging, requests) is reclaimed in P1-02.
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.kernel_entry")]
pub extern "C" fn kernel_entry() -> ! {
    unsafe {
        asm!("cli", options(nomem, nostack));
    }
    serial_init();
    if !BASE.is_supported() {
        fail(b"m1.protocol_rejected version=1\n");
    }
    if cfg!(feature = "fault-before-entry") {
        halt();
    }
    let record = concat!(
        "\nm1.kernel_entry version=1 build=",
        env!("CONTINUUM_BUILD_ID"),
        "\n"
    );
    if !serial(record.as_bytes()) {
        fail(b"m1.serial_failed version=1\n");
    }
    #[cfg(feature = "emulator-test")]
    test_exit(0x10);
    #[cfg(not(feature = "emulator-test"))]
    halt()
}

#[panic_handler]
fn panic(_: &PanicInfo<'_>) -> ! {
    // Fixed literal: formatting arbitrary panic arguments could panic again.
    fail(b"m1.panic version=1\n")
}
