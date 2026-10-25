#![no_std]
#![no_main]
use core::{arch::asm, panic::PanicInfo, ptr::{addr_of,addr_of_mut}};
#[cfg(user_a)] const SEED:u64=0xa101;
#[cfg(user_b)] const SEED:u64=0xb202;
#[cfg(service)] const SEED:u64=0xc303;
#[used] static LABEL:[u8;17]=*b"CNTM3 fixture v0!";
#[used] static mut DATA:u64=SEED;
#[used] static mut BSS:[u8;8193]=[0;8193];
// Independently linked SysV function entry. The loader constructs the call-shaped
// stack; P3-04 will supply IRETQ and a process_exit trampoline. Never run in P3-03.
#[unsafe(no_mangle)]
#[unsafe(link_section=".text.fixture_entry")]
pub extern "C" fn fixture_entry(startup:*const u8)->u64 {
    unsafe {
        let n=startup.read_volatile() as u64 ^ addr_of!(DATA).read_volatile() ^ addr_of!(LABEL).cast::<u8>().read_volatile() as u64;
        addr_of_mut!(BSS).cast::<u8>().write_volatile(n as u8);
        asm!("", in("rax") n, options(nostack,nomem));
        n
    }
}
#[panic_handler] fn panic(_: &PanicInfo<'_>)->! {loop{core::hint::spin_loop();}}
