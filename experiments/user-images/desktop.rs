#![no_std]
#![no_main]
include!(env!("CONTINUUM_RUNTIME_IDS"));
#[path = "../../kernel/src/reclaiming_heap.rs"]
mod reclaiming_heap;
use core::{
    alloc::{GlobalAlloc, Layout},
    arch::global_asm,
    cell::UnsafeCell,
    panic::PanicInfo,
    ptr,
};
unsafe fn syscall(op: u64, a: u64, b: u64, c: u64, d: u64) -> (u64, u64) {
    let status: u64;
    let value: u64;
    unsafe {
        core::arch::asm!("syscall",inlateout("rax")op=>status,in("rdi")a,in("rsi")b,inlateout("rdx")c=>value,in("r10")d,in("r8")0u64,in("r9")0u64,lateout("rcx")_,lateout("r11")_);
    }
    (status, value)
}
const BASE: usize = 0x40000000;
struct Owner(UnsafeCell<reclaiming_heap::Heap<512>>);
unsafe impl Sync for Owner {} // one owning thread, no signal/callback reentry
static HEAP: Owner = Owner(UnsafeCell::new(reclaiming_heap::Heap::new(0)));
struct Allocator;
#[global_allocator]
static ALLOCATOR: Allocator = Allocator;
unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        let offset = unsafe { (&mut *HEAP.0.get()).allocate(l.size(), l.align()) };
        match offset {
            None => ptr::null_mut(),
            Some(o) => {
                let p = (BASE + o) as *mut u8;
                unsafe {
                    ptr::write_bytes(p, 0, l.size());
                }
                p
            }
        }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        let Some(offset) = (p as usize).checked_sub(BASE) else {
            exit(97)
        };
        if !unsafe { (&mut *HEAP.0.get()).release(offset, l.size(), l.align()) } {
            exit(97);
        }
    }
}
global_asm!(r#"
.section .text.fixture_entry
.global fixture_entry
.type fixture_entry,@function
fixture_entry:
    sub rsp, 8
    call {main}
    add rsp, 8
    ret
.size fixture_entry, .-fixture_entry
"#,main=sym main);
fn word(n: usize) -> u64 {
    unsafe { ((0x70000000 + n * 8) as *const u64).read_volatile() }
}
#[track_caller]
fn check(ok: bool) {
    if !ok {
        let mut text = *b"desktop-check-0000000000";
        let mut line = core::panic::Location::caller().line();
        for ch in text[14..].iter_mut().rev() {
            *ch = b'0' + (line % 10) as u8;
            line /= 10;
        }
        unsafe {
            syscall(
                DIAGNOSTIC_LOG,
                text.as_ptr() as u64,
                text.len() as u64,
                0,
                0,
            );
        }
        exit(96);
    }
}
fn exit(code: u64) -> ! {
    unsafe { syscall(PROCESS_EXIT, code, word(2), 0, 0) };
    loop {
        core::hint::spin_loop();
    }
}
fn pause() {
    let timer = word(4);
    let (e, now) = unsafe { syscall(CLOCK_NOW, 0, 0, 0, 0) };
    check(e == 0);
    let arm = [now + 10_000_000, 0, 0];
    check(unsafe { syscall(TIMER_ARM, timer, arm.as_ptr() as u64, 24, 0) }.0 == 0);
    let item = [timer, 0, 0, 0];
    let wait = [item.as_ptr() as u64, 1, now + 100_000_000, 0];
    let status = unsafe { syscall(WAIT_MANY, wait.as_ptr() as u64, 32, 0, 0) }.0;
    check(status == 0 || status == TIMEDOUT);
}
extern "C" fn main() {
    let role = word(0);
    let cap = word(1) as usize;
    let mode = word(5);
    unsafe {
        *HEAP.0.get() = reclaiming_heap::Heap::new(cap);
    }
    // A full-stride 1080p surface and its enlarged copy for roles that need one;
    // smaller owners exercise font/control sized allocations. All are fallible.
    let surface = if cap >= 32 * 1024 * 1024 {
        8_847_360
    } else if cap >= 16 * 1024 * 1024 {
        4 * 1024 * 1024
    } else {
        256 * 1024
    };
    unsafe {
        let l = Layout::from_size_align(surface, 4096).unwrap();
        let p = ALLOCATOR.alloc(l);
        check(!p.is_null() && p as usize % 4096 == 0);
        for n in (0..surface).step_by(4096) {
            check(p.add(n).read_volatile() == 0);
            p.add(n).write_volatile(0x5a);
        }
        // Failed realloc preserves the original allocation and all its bytes.
        let failed = ALLOCATOR.realloc(p, l, cap + 1);
        check(failed.is_null());
        for n in (0..surface).step_by(4096) {
            check(p.add(n).read_volatile() == 0x5a);
        }
        let grown = surface + 65536;
        let q = ALLOCATOR.realloc(p, l, grown);
        check(!q.is_null());
        for n in (0..surface).step_by(4096) {
            check(q.add(n).read_volatile() == 0x5a);
        }
        ALLOCATOR.dealloc(q, Layout::from_size_align(grown, 4096).unwrap());
        // Repeated replacement/fragmentation uses no kernel memory or launches.
        for _ in 0..100 {
            check(syscall(THREAD_YIELD, word(3), 0, 0, 0).0 == 0);
            let a = Layout::from_size_align(65536, 4096).unwrap();
            let small = Layout::from_size_align(257, 64).unwrap();
            let p = ALLOCATOR.alloc(a);
            let q = ALLOCATOR.alloc(small);
            let r = ALLOCATOR.alloc(a);
            check(!p.is_null() && !q.is_null() && !r.is_null());
            for n in (0..65536).step_by(4096) {
                check(p.add(n).read_volatile() == 0);
                p.add(n).write_volatile(0xa5);
            }
            ALLOCATOR.dealloc(p, a);
            ALLOCATOR.dealloc(r, a);
            let s = ALLOCATOR.alloc(a);
            check(!s.is_null());
            for n in (0..65536).step_by(4096) {
                check(s.add(n).read_volatile() == 0);
            }
            ALLOCATOR.dealloc(q, small);
            ALLOCATOR.dealloc(s, a);
        }
        // Descriptor-table pressure is independent of byte exhaustion.
        let l = Layout::from_size_align(1, 1).unwrap();
        let mut slots = [ptr::null_mut(); 512];
        for p in &mut slots {
            *p = ALLOCATOR.alloc(l);
            check(!p.is_null());
        }
        check(ALLOCATOR.alloc(l).is_null());
        for p in slots {
            ALLOCATOR.dealloc(p, l);
        }
        check(
            (&*HEAP.0.get()).live() == 0
                && (&*HEAP.0.get()).used() == 0
                && (&*HEAP.0.get()).peak() > surface,
        );
        // Entire capacity is reclaimable after fragmentation, including padding.
        let all = Layout::from_size_align(cap, 4096).unwrap();
        let p = ALLOCATOR.alloc(all);
        check(!p.is_null());
        check(ALLOCATOR.alloc(l).is_null());
        ALLOCATOR.dealloc(p, all);
        // An unprivileged role cannot allocate raw kernel backing or launch helpers.
        check(syscall(MEMORY_ALLOCATE, 4096, 4096, 0, 0).0 == UNSUPPORTED);
        check(syscall(MEMORY_FREE, 0, 0, 0, 0).0 == UNSUPPORTED);
        check(syscall(PROCESS_CREATE, word(2), 0, 0, 0).0 == ACCESSDENIED);
        check(syscall(MEMORY_CREATE, word(2), 4096, 32 | 64, 0).0 == ACCESSDENIED);
        check(syscall(u64::MAX, 0, 0, 0, 0).0 == UNSUPPORTED);
        let token = b"desktop-heap-100-settled";
        check(
            syscall(
                DIAGNOSTIC_LOG,
                token.as_ptr() as u64,
                token.len() as u64,
                0,
                0,
            )
            .0 == 0,
        );
    }
    if mode != 0 {
        if role == 4 {
            match mode {
                1 => exit(42),
                2 => panic!("synthetic desktop panic"),
                _ => loop {
                    core::hint::spin_loop();
                },
            }
        }
        if role == 5 {
            exit(91);
        }
        if role != 0 {
            exit(0);
        }
    }
    loop {
        pause();
    }
}
#[panic_handler]
fn panic(_: &PanicInfo<'_>) -> ! {
    exit(90)
}

#[unsafe(no_mangle)]
unsafe extern "C" fn memset(d: *mut u8, b: i32, n: usize) -> *mut u8 {
    unsafe {
        core::arch::asm!("rep stosb",inout("rdi")d=>_,inout("rcx")n=>_,in("al")b as u8,options(nostack));
    }
    d
}
#[unsafe(no_mangle)]
unsafe extern "C" fn memcpy(d: *mut u8, s: *const u8, n: usize) -> *mut u8 {
    unsafe {
        core::arch::asm!("rep movsb",inout("rdi")d=>_,inout("rsi")s=>_,inout("rcx")n=>_,options(nostack));
    }
    d
}
#[unsafe(no_mangle)]
unsafe extern "C" fn memmove(d: *mut u8, s: *const u8, n: usize) -> *mut u8 {
    if d as usize <= s as usize {
        unsafe {
            memcpy(d, s, n);
        }
    } else {
        for i in (0..n).rev() {
            unsafe {
                d.add(i).write_volatile(s.add(i).read_volatile());
            }
        }
    }
    d
}
