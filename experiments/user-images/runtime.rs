#![allow(dead_code)]
//! Minimal integer-only no-std runtime. Page-run allocator reclaims on free;
//! no probe arena, host OS, unwinding, TLS, FP/SIMD or allocating failure path.
use core::{
    alloc::{GlobalAlloc, Layout},
    arch::asm,
    ptr,
};
include!(env!("CONTINUUM_RUNTIME_IDS"));
pub fn call_version(op: u64, a: u64, b: u64, version: u64, features: u64) -> (u64, u64) {
    let status: u64;
    let value: u64;
    unsafe {
        asm!("syscall",inlateout("rax") op=>status,in("rdi") a,in("rsi") b,
        inlateout("rdx") 0u64=>value,in("r10") 0u64,in("r8") features,in("r9") version,
        lateout("rcx") _,lateout("r11") _);
    }
    (status, value)
}
pub fn call(op: u64, a: u64, b: u64) -> (u64, u64) {
    call_version(op, a, b, 0, 0)
}
pub fn log(bytes: &[u8]) {
    let _ = call(DIAGNOSTIC_LOG, bytes.as_ptr() as u64, bytes.len() as u64);
}
pub fn exit(code: u64) -> ! {
    let _ = call(PROCESS_EXIT, code, 0);
    unsafe {
        asm!("ud2", options(noreturn));
    }
}
pub fn oom() -> ! {
    log(b"oom");
    exit(2)
}
pub fn check(v: bool) {
    if !v {
        log(b"fixture-failed");
        exit(99)
    }
}
pub struct Allocator;
#[global_allocator]
static ALLOCATOR: Allocator = Allocator;
unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let (e, p) = call(MEMORY_ALLOCATE, layout.size() as u64, layout.align() as u64);
        if e == 0 {
            p as *mut u8
        } else {
            ptr::null_mut()
        }
    }
    unsafe fn dealloc(&self, p: *mut u8, _: Layout) {
        check(call(MEMORY_FREE, p as u64, 0).0 == 0);
    }
    unsafe fn realloc(&self, p: *mut u8, old: Layout, new: usize) -> *mut u8 {
        if new == 0 {
            unsafe { self.dealloc(p, old) };
            return ptr::null_mut();
        }
        let Ok(layout) = Layout::from_size_align(new, old.align()) else {
            return ptr::null_mut();
        };
        let q = unsafe { self.alloc(layout) };
        if !q.is_null() {
            for i in 0..old.size().min(new) {
                unsafe {
                    q.add(i).write_volatile(p.add(i).read_volatile());
                }
            }
            unsafe { self.dealloc(p, old) };
        }
        q
    }
}
pub fn allocate(bytes: usize) -> *mut u8 {
    let layout = Layout::from_size_align(bytes, 16).unwrap_or_else(|_| oom());
    let p = unsafe { ALLOCATOR.alloc(layout) };
    if p.is_null() {
        oom();
    }
    p
}
pub unsafe fn release(p: *mut u8, bytes: usize) {
    unsafe {
        ALLOCATOR.dealloc(p, Layout::from_size_align(bytes, 16).unwrap());
    }
}
pub fn allocator_check() {
    let old = Layout::from_size_align(8192, 16).unwrap();
    let p = allocate(8192);
    for i in 0..8192 {
        unsafe {
            check(p.add(i).read_volatile() == 0);
            p.add(i).write_volatile((i & 255) as u8);
        }
    }
    let failed = unsafe { ALLOCATOR.realloc(p, old, 33 * 1024 * 1024) };
    check(failed.is_null());
    for i in 0..8192 {
        unsafe {
            check(p.add(i).read_volatile() == (i & 255) as u8);
        }
    }
    let q = unsafe { ALLOCATOR.realloc(p, old, 12288) };
    check(!q.is_null());
    for i in 0..12288 {
        unsafe {
            check(q.add(i).read_volatile() == if i < 8192 { (i & 255) as u8 } else { 0 });
        }
    }
    unsafe { release(q, 12288) };
    let p = allocate(8192);
    for i in 0..8192 {
        unsafe {
            check(p.add(i).read_volatile() == 0);
        }
    }
    unsafe { release(p, 8192) };
}
// Freestanding compiler intrinsics; volatile scalar accesses prevent recursion.
#[unsafe(no_mangle)]
unsafe extern "C" fn memcpy(
    d: *mut core::ffi::c_void,
    s: *const core::ffi::c_void,
    n: usize,
) -> *mut core::ffi::c_void {
    for i in 0..n {
        unsafe {
            d.cast::<u8>()
                .add(i)
                .write_volatile(s.cast::<u8>().add(i).read_volatile());
        }
    }
    d
}
#[unsafe(no_mangle)]
unsafe extern "C" fn memset(d: *mut core::ffi::c_void, b: i32, n: usize) -> *mut core::ffi::c_void {
    for i in 0..n {
        unsafe {
            d.cast::<u8>().add(i).write_volatile(b as u8);
        }
    }
    d
}
