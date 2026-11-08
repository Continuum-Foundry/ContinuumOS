//! Bounded one-thread extent allocator. Metadata owns offsets, never user pointers.
//! Sorted live extents imply immediate coalescing of every freed gap. Allocation
//! scans at most N entries; alignment padding is reusable. No backing growth.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Block {
    offset: usize,
    bytes: usize,
    align: usize,
}
pub struct Heap<const N: usize> {
    bytes: usize,
    blocks: [Block; N],
    live: usize,
    used: usize,
    peak: usize,
}
impl<const N: usize> Heap<N> {
    pub const fn new(bytes: usize) -> Self {
        Self {
            bytes,
            blocks: [Block {
                offset: 0,
                bytes: 0,
                align: 1,
            }; N],
            live: 0,
            used: 0,
            peak: 0,
        }
    }
    pub fn live(&self) -> usize {
        self.live
    }
    pub fn used(&self) -> usize {
        self.used
    }
    pub fn peak(&self) -> usize {
        self.peak
    }
    pub fn allocate(&mut self, bytes: usize, align: usize) -> Option<usize> {
        if bytes == 0
            || bytes > self.bytes
            || !align.is_power_of_two()
            || align > 4096
            || self.live == N
        {
            return None;
        }
        let mut end = 0usize;
        for slot in 0..=self.live {
            let offset = end.checked_add(align - 1)? & !(align - 1);
            let next = if slot == self.live {
                self.bytes
            } else {
                self.blocks[slot].offset
            };
            if offset.checked_add(bytes)? <= next {
                self.blocks.copy_within(slot..self.live, slot + 1);
                self.blocks[slot] = Block {
                    offset,
                    bytes,
                    align,
                };
                self.live += 1;
                self.used += bytes;
                self.peak = self.peak.max(self.used);
                return Some(offset);
            }
            if slot < self.live {
                end = self.blocks[slot].offset + self.blocks[slot].bytes;
            }
        }
        None
    }
    pub fn release(&mut self, offset: usize, bytes: usize, align: usize) -> bool {
        let Some(slot) = self.blocks[..self.live].iter().position(|b| {
            *b == Block {
                offset,
                bytes,
                align,
            }
        }) else {
            return false;
        };
        self.blocks.copy_within(slot + 1..self.live, slot);
        self.live -= 1;
        self.used -= bytes;
        true
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fragmentation_coalesces_and_alignment_padding_is_reusable() {
        let mut h = Heap::<8>::new(16 * 1024 * 1024);
        let a = h.allocate(1, 1).unwrap();
        let b = h.allocate(8 * 1024 * 1024, 4096).unwrap();
        assert_eq!(b, 4096);
        let c = h.allocate(4095, 1).unwrap();
        assert_eq!(c, 1);
        assert!(h.release(b, 8 * 1024 * 1024, 4096));
        assert!(h.release(a, 1, 1));
        assert!(h.release(c, 4095, 1));
        assert_eq!(h.allocate(16 * 1024 * 1024, 4096), Some(0));
    }
    #[test]
    fn failed_allocation_is_atomic_and_slots_reuse() {
        let mut h = Heap::<2>::new(8192);
        assert_eq!(h.allocate(4096, 4096), Some(0));
        assert_eq!(h.allocate(4096, 4096), Some(4096));
        for (bytes, align) in [(1, 1), (usize::MAX, 4096), (2, 8192), (1, 3), (0, 1)] {
            assert_eq!(h.allocate(bytes, align), None);
        }
        assert_eq!((h.live(), h.used(), h.peak()), (2, 8192, 8192));
        assert!(!h.release(0, 1, 4096));
        assert!(h.release(0, 4096, 4096));
        assert_eq!(h.allocate(4096, 4096), Some(0));
    }
}
