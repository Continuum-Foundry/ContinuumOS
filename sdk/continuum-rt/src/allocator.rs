//! Optional one-thread reclaiming heap: one 64 KiB RW object, 64-byte cells,
//! at most 32 live allocations, alignment <=4096. Shutdown releases the backing.
//! Individual reuse consumes no additional prepaid kernel allocation credits.
#[cfg(target_os = "none")]
mod guest {
    use crate::*;
    use core::{
        alloc::{GlobalAlloc, Layout},
        cell::UnsafeCell,
        ptr,
    };
    const BASE: u64 = 0x52000000;
    const BYTES: usize = 65536;
    const CELL: usize = 64;
    const CELLS: usize = BYTES / CELL;
    #[derive(Clone, Copy)]
    struct Block {
        start: usize,
        cells: usize,
        bytes: usize,
        align: usize,
    }
    struct Heap {
        process: u64,
        active: bool,
        used: [u64; 16],
        blocks: [Option<Block>; 32],
        last_error: u64,
    }
    impl Heap {
        const EMPTY: Self = Self {
            process: 0,
            active: false,
            used: [0; 16],
            blocks: [None; 32],
            last_error: 0,
        };
    }
    struct Owner(UnsafeCell<Heap>);
    // Exactly one user execution thread, no callbacks/IRQ/signal reentry. Other
    // processes have private image/BSS pages and cannot share this RW backing.
    unsafe impl Sync for Owner {}
    static HEAP: Owner = Owner(UnsafeCell::new(Heap::EMPTY));
    pub struct Allocator;
    /// Caller retains its self-process handle and reserves [BASE,BASE+65536)
    /// until shutdown. Must run before allocating and outside allocator reentry.
    pub unsafe fn initialize(process: Handle) -> Result<(), u64> {
        let h = unsafe { &mut *HEAP.0.get() };
        if h.active || process.0 == 0 {
            return Err(INVALIDARGUMENT);
        }
        let (e, m) = unsafe { syscall(MEMORY_CREATE, process.0, BYTES as u64, 32 | 64, 0) };
        if e != 0 {
            h.last_error = (1 << 32) | e;
            return Err(e);
        }
        let request = wire::map(BASE, 0, BYTES as u64, 1).unwrap();
        let (e, _) = unsafe { syscall(MEMORY_MAP, process.0, m, request.as_ptr() as u64, 32) };
        if unsafe { syscall(HANDLE_CLOSE, m, 0, 0, 0) }.0 != 0 {
            exit(97, process);
        }
        if e != 0 {
            h.last_error = (2 << 32) | e;
            return Err(e);
        }
        *h = Heap {
            process: process.0,
            active: true,
            ..Heap::EMPTY
        };
        Ok(())
    }
    pub fn live() -> u64 {
        unsafe { (&*HEAP.0.get()).blocks.iter().flatten().count() as u64 }
    }
    pub fn last_error() -> u64 {
        unsafe { (&*HEAP.0.get()).last_error }
    }
    pub fn shutdown() -> Result<(), u64> {
        let h = unsafe { &mut *HEAP.0.get() };
        if !h.active {
            return Err(INVALIDARGUMENT);
        }
        if h.blocks.iter().any(Option::is_some) {
            return Err(WOULDBLOCK);
        }
        assert!(h.used.iter().all(|v| *v == 0));
        let (e, _) = unsafe { syscall(MEMORY_UNMAP, h.process, BASE, 0, 0) };
        if e != 0 {
            return Err(e);
        }
        *h = Heap::EMPTY;
        Ok(())
    }
    unsafe impl GlobalAlloc for Allocator {
        unsafe fn alloc(&self, l: Layout) -> *mut u8 {
            let h = unsafe { &mut *HEAP.0.get() };
            if !h.active || l.size() > BYTES || l.align() > 4096 {
                h.last_error = INVALIDARGUMENT;
                return ptr::null_mut();
            }
            let Some(slot) = h.blocks.iter().position(Option::is_none) else {
                h.last_error = QUOTAEXCEEDED;
                return ptr::null_mut();
            };
            let cells = (l.size().max(1) + CELL - 1) / CELL;
            for start in 0..=CELLS - cells {
                let address = BASE + (start * CELL) as u64;
                if address % (l.align() as u64) != 0
                    || (start..start + cells).any(|i| h.used[i / 64] & (1u64 << (i % 64)) != 0)
                {
                    continue;
                }
                for i in start..start + cells {
                    h.used[i / 64] |= 1u64 << (i % 64);
                }
                h.blocks[slot] = Some(Block {
                    start,
                    cells,
                    bytes: l.size(),
                    align: l.align(),
                });
                h.last_error = 0;
                let p = address as *mut u8;
                for i in 0..l.size() {
                    unsafe {
                        p.add(i).write_volatile(0);
                    }
                }
                return p;
            }
            h.last_error = QUOTAEXCEEDED;
            ptr::null_mut()
        }
        unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
            let h = unsafe { &mut *HEAP.0.get() };
            let found = h
                .blocks
                .iter()
                .position(|b| b.is_some_and(|b| BASE + (b.start * CELL) as u64 == p as u64));
            let Some(slot) = found else {
                exit(97, Handle(h.process));
            };
            let block = h.blocks[slot].unwrap();
            if block.bytes != l.size() || block.align != l.align() {
                exit(97, Handle(h.process));
            }
            for i in block.start..block.start + block.cells {
                h.used[i / 64] &= !(1u64 << (i % 64));
            }
            h.blocks[slot] = None;
        }
    }
}
#[cfg(target_os = "none")]
pub use guest::*;
