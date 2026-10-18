#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.kernel_probe_entry")]
pub extern "C" fn kernel_probe_entry() -> ! {
    // Compile/link only: no boot protocol, stack setup, serial device or allocator.
    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
