//! Host-only allocation and process counters shared by measurement harnesses.
use crate::Result;
use serde_json::{Value, json};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed},
};
pub(crate) static LIVE: AtomicU64 = AtomicU64::new(0);
pub(crate) static PEAK: AtomicU64 = AtomicU64::new(0);
// Rust allocations only: foreign/runtime/OS allocation remains in process counters.
struct Counted;
pub(crate) static COUNT: AtomicBool = AtomicBool::new(false);
pub(crate) static ALLOCS: AtomicU64 = AtomicU64::new(0);
pub(crate) static BYTES: AtomicU64 = AtomicU64::new(0);
#[global_allocator]
static ALLOCATOR: Counted = Counted;
unsafe impl GlobalAlloc for Counted {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(l) };
        if !p.is_null() {
            let live = LIVE.fetch_add(l.size() as u64, Relaxed) + l.size() as u64;
            PEAK.fetch_max(live, Relaxed);
        }
        if !p.is_null() && COUNT.load(Relaxed) {
            ALLOCS.fetch_add(1, Relaxed);
            BYTES.fetch_add(l.size() as u64, Relaxed);
        }
        p
    }
    unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
        let p = unsafe { System.alloc_zeroed(l) };
        if !p.is_null() {
            let live = LIVE.fetch_add(l.size() as u64, Relaxed) + l.size() as u64;
            PEAK.fetch_max(live, Relaxed);
        }
        if !p.is_null() && COUNT.load(Relaxed) {
            ALLOCS.fetch_add(1, Relaxed);
            BYTES.fetch_add(l.size() as u64, Relaxed);
        }
        p
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        let p = unsafe { System.realloc(p, l, n) };
        if !p.is_null() {
            let live = LIVE.fetch_add(n as u64, Relaxed) + n as u64;
            LIVE.fetch_sub(l.size() as u64, Relaxed);
            PEAK.fetch_max(live - l.size() as u64, Relaxed);
        }
        if !p.is_null() && COUNT.load(Relaxed) {
            ALLOCS.fetch_add(1, Relaxed);
            BYTES.fetch_add(n as u64, Relaxed);
        }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        LIVE.fetch_sub(l.size() as u64, Relaxed);
        unsafe { System.dealloc(p, l) }
    }
}
#[repr(C)]
#[derive(Default)]
struct Memory {
    cb: u32,
    faults: u32,
    peak_ws: usize,
    ws: usize,
    peak_paged: usize,
    paged: usize,
    peak_nonpaged: usize,
    nonpaged: usize,
    pagefile: usize,
    peak_pagefile: usize,
    private: usize,
}
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetCurrentProcess() -> *mut std::ffi::c_void;
    fn GetProcessTimes(
        p: *mut std::ffi::c_void,
        created: *mut u64,
        exited: *mut u64,
        kernel: *mut u64,
        user: *mut u64,
    ) -> i32;
}
#[link(name = "psapi")]
unsafe extern "system" {
    fn GetProcessMemoryInfo(p: *mut std::ffi::c_void, m: *mut Memory, cb: u32) -> i32;
}
pub(crate) fn process() -> Result<Value> {
    let mut m = Memory {
        cb: std::mem::size_of::<Memory>() as u32,
        ..Default::default()
    };
    let (mut c, mut e, mut k, mut u) = (0, 0, 0, 0);
    // Current-process pseudo-handle, initialized buffers with exact Windows ABI sizes.
    if unsafe { GetProcessMemoryInfo(GetCurrentProcess(), &mut m, m.cb) } == 0
        || unsafe { GetProcessTimes(GetCurrentProcess(), &mut c, &mut e, &mut k, &mut u) } == 0
    {
        return Err("Windows process accounting failed".into());
    }
    Ok(
        json!({"cpu_100ns":k+u,"working_set":m.ws,"private_commit":m.private,"peak_working_set":m.peak_ws,"peak_commit":m.peak_pagefile,"page_faults":m.faults}),
    )
}
