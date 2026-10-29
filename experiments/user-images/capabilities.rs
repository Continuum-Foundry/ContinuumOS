#![no_std]
#![no_main]
mod runtime;
use core::{
    arch::{asm, global_asm},
    panic::PanicInfo,
};
use runtime::*;
const QUERY: u64 = 1;
const DUP: u64 = 2;
const CLOSE: u64 = 4;
const MAP: u64 = 16;
const READ: u64 = 32;
const WRITE: u64 = 64;
const BASIC: u64 = QUERY | DUP | CLOSE;
global_asm!(r#"
.section .text.fixture_entry
.global fixture_entry
.type fixture_entry,@function
fixture_entry:
    sub rsp,8
    call {main}
    ud2
.size fixture_entry,.-fixture_entry
"#,main=sym main);
fn word(n: usize) -> u64 {
    unsafe { (0x70000000 as *const u64).add(n).read_volatile() }
}
fn syscall(op: u64, a: u64, b: u64, c: u64, d: u64) -> (u64, u64) {
    let status;
    let value;
    unsafe {
        asm!("syscall",inlateout("rax") op=>status,in("rdi") a,in("rsi") b,inlateout("rdx") c=>value,in("r10") d,in("r8") 0u64,in("r9") 0u64,lateout("rcx") _,lateout("r11") _);
    }
    (status, value)
}
fn done(code: u64) -> ! {
    let _ = syscall(PROCESS_EXIT, code, word(3), 0, 0);
    unsafe {
        asm!("ud2", options(noreturn));
    }
}
fn check(v: bool) {
    if !v {
        log(b"capability-failed");
        done(99);
    }
}
fn ok(op: u64, a: u64, b: u64, c: u64) -> u64 {
    let (e, v) = syscall(op, a, b, c, 0);
    check(e == 0);
    v
}
fn denied(op: u64, a: u64, b: u64, c: u64, expected: u64) {
    check(syscall(op, a, b, c, 0) == (expected, 0));
}
fn map(process: u64, memory: u64, base: u64, offset: u64, bytes: u64, access: u64) -> (u64, u64) {
    let mut request = [0u8; 32];
    for (index, value) in [base, offset, bytes, access].into_iter().enumerate() {
        request[index * 8..index * 8 + 8].copy_from_slice(&value.to_le_bytes());
    }
    syscall(MEMORY_MAP, process, memory, request.as_ptr() as u64, 32)
}
fn read_shared() {
    for (n, b) in b"shared-read".iter().enumerate() {
        check(unsafe { (0x50000000 as *const u8).add(n).read_volatile() } == *b);
    }
}
fn main() -> ! {
    let mode = word(0);
    let role = word(2);
    let p = word(3);
    let thread = word(4);
    let grant = word(5);
    let foreign = word(6);
    check(ok(OBJECT_INFO, p, 0, 0) >> 32 == 1);
    check(ok(OBJECT_INFO, thread, 0, 0) >> 32 == 2);
    denied(MEMORY_ALLOCATE, 4096, 4096, 0, UNSUPPORTED);
    denied(MEMORY_FREE, 0, 0, 0, UNSUPPORTED);
    denied(PROCESS_EXIT, 55, thread, 0, WRONGTYPE);
    denied(THREAD_YIELD, p, 0, 0, WRONGTYPE);
    if role == 1 {
        if mode >= 103 {
            let h = ok(MEMORY_CREATE, p, 4096, READ | WRITE);
            check(map(p, h, 0x60000000, 0, 4096, 1) == (0, 0));
            unsafe {
                (0x60000000 as *mut u64).write_volatile(0xabcdef);
            }
            // Keep private backing live while the peer performs its denied
            // hardware access; the reviewer checks fault-before-owner-reap.
            for _ in 0..32 {
                ok(THREAD_YIELD, thread, 0, 0);
            }
        }
        ok(HANDLE_CLOSE, grant, 0, 0);
        denied(OBJECT_INFO, grant, 0, 0, BADHANDLE);
        log(b"creator-close");
        done(0);
    }
    if role == 3 {
        denied(OBJECT_INFO, foreign, 0, 0, BADHANDLE);
        check(map(p, foreign, 0x50000000, 0, 4096, 0) == (BADHANDLE, 0));
        denied(MEMORY_CREATE, foreign, 4096, READ, BADHANDLE);
        log(b"same-user-denied");
        done(0);
    }
    // Distinct-owner process: only the explicit read grant authorizes access.
    denied(OBJECT_INFO, foreign, 0, 0, BADHANDLE);
    check(ok(OBJECT_INFO, grant, 0, 0) == (3u64 << 32) | (BASIC | MAP | READ));
    denied(HANDLE_DUPLICATE, grant, u64::MAX, 0, ACCESSDENIED);
    check(map(p, grant, 0x50000000, 0, 4096, 1) == (ACCESSDENIED, 0));
    check(map(p, grant, 0x50000000, 0, 4096, 2) == (ACCESSDENIED, 0));
    let d = ok(HANDLE_DUPLICATE, grant, BASIC | MAP | READ, 0);
    ok(HANDLE_CLOSE, grant, 0, 0);
    denied(HANDLE_CLOSE, grant, 0, 0, BADHANDLE);
    check(map(p, d, 0x50000000, 0, 4096, 0) == (0, 0));
    ok(HANDLE_CLOSE, d, 0, 0);
    read_shared();
    denied(MEMORY_PROTECT, p, 0x50000000, 1, ACCESSDENIED);
    denied(MEMORY_PROTECT, p, 0x50000000, 2, ACCESSDENIED);
    log(b"shared-after-close");
    if mode == 101 {
        unsafe {
            (0x50000000 as *mut u8).write_volatile(1);
        }
    }
    if mode == 102 {
        unsafe {
            asm!("call rax",in("rax") 0x50000000u64);
        }
    }
    if mode == 103 {
        unsafe {
            core::hint::black_box((0xffffffff80000000 as *const u64).read_volatile());
        }
    }
    if mode == 104 {
        unsafe {
            core::hint::black_box((0x60000000 as *const u64).read_volatile());
        }
    }
    if mode == 105 {
        unsafe {
            (0x60000000 as *mut u64).write_volatile(1);
        }
    }
    check(mode == 100);
    ok(MEMORY_UNMAP, p, 0x50000000, 0);
    denied(MEMORY_UNMAP, p, 0x50000000, 0, INVALIDARGUMENT);
    denied(OBJECT_INFO, 0, 0, 0, BADHANDLE);
    denied(OBJECT_INFO, u64::MAX, 0, 0, BADHANDLE);
    denied(MEMORY_CREATE, thread, 4096, READ, WRONGTYPE);
    let q = ok(HANDLE_DUPLICATE, p, QUERY | CLOSE, 0);
    denied(MEMORY_CREATE, q, 4096, READ, ACCESSDENIED);
    denied(HANDLE_DUPLICATE, q, QUERY, 0, ACCESSDENIED);
    ok(HANDLE_CLOSE, q, 0, 0);
    let q2 = ok(HANDLE_DUPLICATE, p, QUERY | CLOSE, 0);
    check(q2 != q);
    denied(OBJECT_INFO, q, 0, 0, BADHANDLE);
    ok(HANDLE_CLOSE, q2, 0, 0);
    let zero = ok(HANDLE_DUPLICATE, p, 0, 0);
    denied(OBJECT_INFO, zero, 0, 0, ACCESSDENIED);
    denied(HANDLE_CLOSE, zero, 0, 0, ACCESSDENIED);
    for (bytes, flags) in [
        (0, READ),
        (4097, READ),
        (65537, READ),
        (4096, READ | WRITE | 128),
        (4096, u64::MAX),
    ] {
        denied(MEMORY_CREATE, p, bytes, flags, INVALIDARGUMENT);
    }
    let h = ok(MEMORY_CREATE, p, 8192, READ | WRITE);
    check(ok(OBJECT_INFO, h, 0, 0) == (3u64 << 32) | (BASIC | MAP | READ | WRITE));
    for ptr in [0, 0xffff_ffff_8000_0000, 0x70000ff0] {
        check(syscall(MEMORY_MAP, p, h, ptr, 32) == (INVALIDARGUMENT, 0));
    }
    check(syscall(MEMORY_MAP, p, h, 0, 31) == (INVALIDARGUMENT, 0));
    check(map(p, h, 0x40000000, 0, 4096, 3) == (INVALIDARGUMENT, 0));
    check(map(p, p, 0x40000000, 0, 4096, 0) == (WRONGTYPE, 0));
    let read = ok(HANDLE_DUPLICATE, h, BASIC | MAP | READ, 0);
    check(map(p, read, 0x40000000, 0, 4096, 1) == (ACCESSDENIED, 0));
    check(map(p, read, 0x40000000, 0, 4096, 0) == (0, 0));
    ok(HANDLE_CLOSE, read, 0, 0);
    denied(MEMORY_PROTECT, p, 0x40000000, 1, ACCESSDENIED);
    ok(MEMORY_UNMAP, p, 0x40000000, 0);
    check(map(p, h, 0x40000000, 0, 8192, 1) == (0, 0));
    for n in 0..8192 {
        check(unsafe { (0x40000000 as *const u8).add(n).read_volatile() } == 0);
    }
    unsafe {
        (0x40000000 as *mut u8).write_volatile(0xa5);
    }
    denied(MEMORY_PROTECT, p, 0x40000000, 2, ACCESSDENIED);
    ok(MEMORY_PROTECT, p, 0x40000000, 0);
    ok(MEMORY_PROTECT, p, 0x40000000, 1);
    for (base, offset, n) in [
        (0x40000000, 0, 4096),
        (0xffffffff80000000, 0, 4096),
        (0x40000001, 0, 4096),
        (0x42000000, u64::MAX - 4095, 8192),
        (0x42000000, 8192, 4096),
        (0x42000000, 0, 0),
    ] {
        check(map(p, h, base, offset, n, 0) == (INVALIDARGUMENT, 0));
    }
    ok(MEMORY_UNMAP, p, 0x40000000, 0);
    ok(HANDLE_CLOSE, h, 0, 0);
    let h = ok(MEMORY_CREATE, p, 8192, READ | WRITE);
    check(map(p, h, 0x40000000, 0, 8192, 1) == (0, 0));
    for n in 0..8192 {
        check(unsafe { (0x40000000 as *const u8).add(n).read_volatile() } == 0);
    }
    ok(MEMORY_UNMAP, p, 0x40000000, 0);
    log(b"zero-reuse");
    let mut handles = [0u64; 32];
    let mut count = 0;
    loop {
        let (e, v) = syscall(HANDLE_DUPLICATE, h, BASIC, 0, 0);
        if e != 0 {
            check(e == QUOTAEXCEEDED && v == 0);
            break;
        }
        check(count < 32);
        handles[count] = v;
        count += 1;
    }
    check(count == 28);
    denied(MEMORY_CREATE, p, 4096, READ, QUOTAEXCEEDED);
    for v in &handles[..count] {
        ok(HANDLE_CLOSE, *v, 0, 0);
    }
    for n in 0..32 {
        check(map(p, h, 0x10000000 + n * 4096, 0, 4096, 0) == (0, 0));
    }
    check(map(p, h, 0x20000000, 0, 4096, 0) == (QUOTAEXCEEDED, 0));
    for n in 0..32 {
        ok(MEMORY_UNMAP, p, 0x10000000 + n * 4096, 0);
    }
    ok(HANDLE_CLOSE, h, 0, 0);
    log(b"rights-quotas-pass");
    done(0)
}
#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    done(99)
}
