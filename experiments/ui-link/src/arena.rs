//! One-shot probe allocation, not a production heap. Deallocation deliberately
//! retains bytes until process teardown; repeated frames eventually exhaust it.
use core::{
    alloc::{GlobalAlloc, Layout},
    cell::UnsafeCell,
    ptr,
    sync::atomic::{AtomicUsize, Ordering},
};

#[repr(C, align(4096))]
pub struct Arena<const N: usize> {
    bytes: UnsafeCell<[u8; N]>,
    used: AtomicUsize,
}
// SAFETY: CAS hands out disjoint ranges, backing storage never moves while used.
// No method resets the cursor or exposes an alias to allocated storage.
unsafe impl<const N: usize> Sync for Arena<N> {}
impl<const N: usize> Arena<N> {
    pub const fn new() -> Self {
        Self {
            bytes: UnsafeCell::new([0; N]),
            used: AtomicUsize::new(0),
        }
    }
    pub fn used(&self) -> usize {
        self.used.load(Ordering::Relaxed)
    }
}
unsafe impl<const N: usize> GlobalAlloc for Arena<N> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let base = self.bytes.get().cast::<u8>();
        let mut old = self.used.load(Ordering::Relaxed);
        loop {
            // Align the absolute address, including requests above arena alignment.
            let Some(address) = (base as usize)
                .checked_add(old)
                .and_then(|a| a.checked_add(layout.align() - 1))
            else {
                return ptr::null_mut();
            };
            let start = (address & !(layout.align() - 1)) - base as usize;
            let Some(end) = start.checked_add(layout.size().max(1)) else {
                return ptr::null_mut();
            };
            if end > N {
                return ptr::null_mut();
            }
            match self
                .used
                .compare_exchange_weak(old, end, Ordering::Relaxed, Ordering::Relaxed)
            {
                Ok(_) => {
                    // SAFETY: checked range lies in the owned array, uniquely reserved above.
                    return unsafe { base.add(start) };
                }
                Err(next) => old = next,
            }
        }
    }
    unsafe fn dealloc(&self, _: *mut u8, _: Layout) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn alignment_writes_and_disjoint_ranges() {
        let arena = Arena::<65536>::new();
        let mut ranges = std::vec::Vec::new();
        for alignment in [1, 2, 8, 16, 64, 4096, 8192] {
            let l = Layout::from_size_align(37, alignment).unwrap();
            let p = unsafe { arena.alloc(l) };
            assert!(!p.is_null());
            assert_eq!(p as usize % alignment, 0);
            assert!(
                ranges
                    .iter()
                    .all(|&(a, b)| p as usize >= b || p as usize + 37 <= a)
            );
            ranges.push((p as usize, p as usize + 37));
            unsafe {
                p.write_bytes(alignment as u8, 37);
            }
            assert!(
                unsafe { core::slice::from_raw_parts(p, 37) }
                    .iter()
                    .all(|&b| b == alignment as u8)
            );
        }
    }
    #[test]
    fn exact_limit_failure_is_atomic_and_no_reuse() {
        let arena = Arena::<64>::new();
        let full = Layout::from_size_align(64, 1).unwrap();
        let p = unsafe { arena.alloc(full) };
        assert!(!p.is_null());
        unsafe {
            arena.dealloc(p, full);
        }
        assert!(unsafe { arena.alloc(Layout::new::<u8>()) }.is_null());
        assert_eq!(arena.used(), 64);
    }
    #[test]
    fn oversize_overflow_zeroing_and_reallocation() {
        let arena = Arena::<256>::new();
        assert!(
            unsafe { arena.alloc(Layout::from_size_align(isize::MAX as usize, 1).unwrap()) }
                .is_null()
        );
        assert_eq!(arena.used(), 0);
        let l = Layout::from_size_align(17, 16).unwrap();
        let p = unsafe { arena.alloc_zeroed(l) };
        assert!(!p.is_null());
        assert_eq!(unsafe { core::slice::from_raw_parts(p, 17) }, &[0; 17]);
        unsafe {
            p.write_bytes(42, 17);
        }
        let q = unsafe { arena.realloc(p, l, 32) };
        assert!(!q.is_null());
        assert_eq!(unsafe { core::slice::from_raw_parts(q, 17) }, &[42; 17]);
        let before = arena.used();
        assert!(
            unsafe { arena.realloc(q, Layout::from_size_align(32, 16).unwrap(), 4096) }.is_null()
        );
        assert_eq!(arena.used(), before);
        assert_eq!(unsafe { *q }, 42);
    }
}
