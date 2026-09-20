use core::alloc::Layout;

pub struct KernelHeap {
    start: usize,
    end: usize,
    next: usize,
}

impl KernelHeap {
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            next: 0,
        }
    }

    pub fn init(&mut self, start: usize, size: usize) {
        assert!(size > 0);

        self.start = start;
        self.end = start.checked_add(size).expect("Heap range overflow");
        self.next = start;
    }

    pub fn allocate(&mut self, layout: Layout) -> Option<*mut u8> {
        let size = layout.size();
        let align = layout.align();

        if size == 0 {
            return None;
        }

        let aligned_start = align_up(self.next, align)?;

        let allocation_end = aligned_start.checked_add(size)?;

        if allocation_end > self.end {
            return None;
        }

        self.next = allocation_end;

        Some(aligned_start as *mut u8)
    }

    pub fn used(&self) -> usize {
        self.next - self.start
    }

    pub fn remaining(&self) -> usize {
        self.end - self.next
    }
}

const fn align_up(value: usize, align: usize) -> Option<usize> {
    if align == 0 || !align.is_power_of_two() {
        return None;
    }

    let mask = align - 1;

    match value.checked_add(mask) {
        Some(value) => Some(value & !mask),
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocations_are_aligned_and_sequential() {
        let mut heap = KernelHeap::new();

        heap.init(0x1000, 0x1000);

        let first = heap.allocate(Layout::from_size_align(16, 8).unwrap());
        let second = heap.allocate(Layout::from_size_align(32, 16).unwrap());

        assert_eq!(first.unwrap() as usize, 0x1000);
        assert_eq!(second.unwrap() as usize, 0x1010);
        assert_eq!(heap.used(), 0x30);
        assert_eq!(heap.remaining(), 0xFD0);
    }

    #[test]
    fn allocation_fails_when_heap_is_full() {
        let mut heap = KernelHeap::new();

        heap.init(0x1000, 32);

        assert!(
            heap.allocate(Layout::from_size_align(32, 8).unwrap())
                .is_some()
        );
        assert!(
            heap.allocate(Layout::from_size_align(1, 1).unwrap())
                .is_none()
        );
    }
}
