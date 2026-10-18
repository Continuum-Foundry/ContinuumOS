#![no_std]
#![no_main]

use core::panic::PanicInfo;

// Experimental SysV C entry: caller aligns RSP to 16 bytes before CALL
// (entry RSP % 16 == 8, valid return address), clears DF and supplies a u64
// in RDI; result returns in RAX. No red zone and no syscalls yet.
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.user_probe_entry")]
pub extern "C" fn user_probe_entry(input: u64) -> u64 {
    input.rotate_left(7) ^ 0x434f_4e54_494e_5555
}

#[panic_handler]
fn panic(_: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
