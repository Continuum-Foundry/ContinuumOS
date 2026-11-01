//! Nonrendering selected-model/portable-text/libm workload under actual hooks.
use super::*;
extern crate alloc;
use core::{
    alloc::{GlobalAlloc, Layout},
    hint::black_box,
};
use unicode_segmentation::UnicodeSegmentation;
#[global_allocator]
static ALLOCATOR: sdk::allocator::Allocator = sdk::allocator::Allocator;
fn verify(v: bool, stage: &[u8]) {
    if !v {
        log(stage);
        trace(format_args!(
            "sdk-allocator-status-{}",
            sdk::allocator::last_error()
        ));
        sdk::exit(99, sdk::Handle(word(3)));
    }
}
pub fn run(mode: u64) {
    if mode == 0 {
        probe::run();
    }
    if mode == 2 {
        panic!("bounded-panic");
    }
    if mode == 3 {
        let mut consumer = alloc::vec::Vec::<u8>::new();
        verify(
            consumer.try_reserve_exact(65537).is_err(),
            b"sdk-failed-oom",
        );
        log(b"sdk-oom-contained");
        sdk::exit(2, sdk::Handle(word(3)));
    }
    if mode == 4 {
        log(b"sdk-startup-return");
        return;
    }
    let baseline = sdk::allocator::live();
    // Prove the bounded heap reuses the complete backing and metadata slots.
    let full = Layout::from_size_align(65536, 4096).unwrap();
    let p = unsafe { ALLOCATOR.alloc(full) };
    verify(
        !p.is_null() && (p as usize) % 4096 == 0,
        b"sdk-failed-full-heap",
    );
    let small = Layout::from_size_align(64, 64).unwrap();
    verify(
        unsafe { ALLOCATOR.alloc(small) }.is_null(),
        b"sdk-failed-capacity",
    );
    verify(
        sdk::allocator::shutdown() == Err(WOULDBLOCK),
        b"sdk-failed-live-shutdown",
    );
    unsafe {
        ALLOCATOR.dealloc(p, full);
    }
    let mut slots = [core::ptr::null_mut(); 32];
    for p in &mut slots {
        *p = unsafe { ALLOCATOR.alloc(small) };
        verify(!p.is_null(), b"sdk-failed-slot-admission");
    }
    verify(
        unsafe { ALLOCATOR.alloc(small) }.is_null(),
        b"sdk-failed-slot-bound",
    );
    for p in slots {
        unsafe {
            ALLOCATOR.dealloc(p, small);
        }
    }
    verify(
        sdk::allocator::live() == baseline,
        b"sdk-failed-slot-settlement",
    );
    log(b"sdk-heap-capacity-reused");
    let start = now();
    let mut cycles = 0;
    let marker = if mode == 1 { 1.25f64 } else { 2.5f64 };
    while now() - start < 80_000_000 || cycles < 20 {
        {
            let l = Layout::from_size_align(8192, 64).unwrap();
            let p = unsafe { ALLOCATOR.alloc(l) };
            verify(!p.is_null(), b"sdk-failed-alloc");
            for i in 0..8192 {
                verify(unsafe { p.add(i).read_volatile() } == 0, b"sdk-failed-zero");
                unsafe {
                    p.add(i).write_volatile(0xa5);
                }
            }
            let q = unsafe { ALLOCATOR.realloc(p, l, 12288) };
            verify(!q.is_null(), b"sdk-failed-realloc");
            for i in 0..12288 {
                verify(
                    unsafe { q.add(i).read_volatile() } == if i < 8192 { 0xa5 } else { 0 },
                    b"sdk-failed-copy",
                );
            }
            let failed =
                unsafe { ALLOCATOR.realloc(q, Layout::from_size_align(12288, 64).unwrap(), 65537) };
            verify(failed.is_null(), b"sdk-failed-failure-atomicity");
            verify(unsafe { q.read_volatile() } == 0xa5, b"sdk-failed-retained");
            unsafe {
                ALLOCATOR.dealloc(q, Layout::from_size_align(12288, 64).unwrap());
            }
            let text = "Cafe\u{301} 👩\u{200d}💻 אבג";
            verify(text.graphemes(true).count() == 10, b"sdk-failed-graphemes");
            let model =
                editor_model::Model::new(1, 1, text, text, editor_model::Selection::new(0, 0))
                    .unwrap();
            verify(
                model.document().text() == text && !model.document().dirty(),
                b"sdk-failed-model",
            );
            let x = black_box(marker);
            let result = libm::sqrt(x * x) + libm::sin(x);
            let until = now() + 7_000_000;
            // Actual timer preemption while FP values are live; no syscall/yield
            // in the loop. The paired process has a different marker.
            let mut v = black_box(result);
            for _ in 0..1_000 {
                v = black_box(v + 0.0);
            }
            let mut fp_marker = marker.to_bits();
            if cycles == 0 {
                unsafe {
                    asm!("movq xmm15,{value}","xorpd xmm14,xmm14","mov ecx,2000000",
                    "2:","addsd xmm15,xmm14","dec ecx","jnz 2b","movq {value},xmm15",
                    value=inout(reg)fp_marker,out("ecx")_,out("xmm14")_,out("xmm15")_);
                }
            }
            verify(fp_marker == marker.to_bits(), b"sdk-failed-fp-register");
            verify(
                (v - (marker + libm::sin(marker))).abs() < 1e-12,
                b"sdk-failed-libm",
            );
            if now() < until {
                yield_now();
            }
        }
        verify(sdk::allocator::live() == baseline, b"sdk-failed-reclaim");
        cycles += 1;
    }
    trace(format_args!("sdk-runtime-{}-{}-reclaimed", mode, cycles));
}
