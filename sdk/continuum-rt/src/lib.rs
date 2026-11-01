//! Bounded x86-64 M3 userspace subset. Bytes, never Rust layout, cross a boundary.
#![no_std]
pub mod allocator;
pub mod generated;
#[cfg(target_os = "none")]
pub mod guest;
pub mod wire;
pub use generated::*;
pub use wire::{
    AuthoritySession, Handle, MonotonicNs, PolicyEpoch, ProcessGeneration, RouteEpoch, WallClockNs,
};

/// Raw register interface. Pointer-bearing calls require valid live buffers for
/// the complete call; output bytes and handles may only be used after success.
/// This is unsafe because a successful map/unmap/protect can invalidate Rust data.
#[cfg(target_os = "none")]
pub unsafe fn syscall(op: u64, a: u64, b: u64, c: u64, d: u64) -> (u64, u64) {
    let status;
    let value;
    unsafe {
        core::arch::asm!("syscall", inlateout("rax") op => status,
            in("rdi") a, in("rsi") b, inlateout("rdx") c => value,
            in("r10") d, in("r8") 0u64, in("r9") 0u64,
            lateout("rcx") _, lateout("r11") _);
    }
    (status, value)
}
#[cfg(target_os = "none")]
pub fn log(bytes: &[u8]) -> Result<(), u64> {
    let (e, _) = unsafe {
        syscall(
            DIAGNOSTIC_LOG,
            bytes.as_ptr() as u64,
            bytes.len() as u64,
            0,
            0,
        )
    };
    if e == 0 { Ok(()) } else { Err(e) }
}
#[cfg(target_os = "none")]
pub fn exit(reason: u64, process: Handle) -> ! {
    unsafe {
        syscall(PROCESS_EXIT, reason, process.0, 0, 0);
        core::arch::asm!("ud2", options(noreturn));
    }
}
