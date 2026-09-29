use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;

pub struct KernelHeap {
    start: usize,
    end: usize,
    next: usize,
    limit: usize,
}

impl KernelHeap {
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            next: 0,
            limit: 0,
        }
    }

    pub fn init(&mut self, start: usize, size: usize, max_size: usize) {
        assert!(self.start == 0);
        assert!(size > 0);
        assert!(max_size >= size);

        let end = start.checked_add(size).expect("Heap range overflow");
        let limit = start.checked_add(max_size).expect("Heap limit overflow");

        self.start = start;
        self.end = end;
        self.next = start;
        self.limit = limit;
    }

    fn grow_pages(&mut self, pages: usize) -> bool {
        if pages == 0 {
            return true;
        }

        let growth = match pages.checked_mul(4096) {
            Some(value) => value,
            None => return false,
        };

        let new_end = match self.end.checked_add(growth) {
            Some(value) => value,
            None => return false,
        };

        if new_end > self.limit {
            return false;
        }

        self.end = new_end;
        true
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
            let missing = allocation_end - self.end;

            let pages = missing / 4096 + if missing % 4096 != 0 { 1 } else { 0 };

            if !self.grow_pages(pages) {
                return None;
            }
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

pub struct GlobalHeap {
    heap: UnsafeCell<KernelHeap>,
}

impl GlobalHeap {
    pub const fn new() -> Self {
        Self {
            heap: UnsafeCell::new(KernelHeap::new()),
        }
    }

    pub fn init(&self, start: usize, size: usize, max_size: usize) {
        unsafe {
            (*self.heap.get()).init(start, size, max_size);
        }
    }
}

// Lych is currently single core and the heap is not accessed concurrently. This will need to be replaced with real synchronization when SMP/preemption is added.
unsafe impl Sync for GlobalHeap {}

unsafe impl GlobalAlloc for GlobalHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe {
            match (*self.heap.get()).allocate(layout) {
                Some(ptr) => ptr,
                None => core::ptr::null_mut(),
            }
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // The current bump allocator does not reuse freed memory.
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

#[global_allocator]
pub static KERNEL_ALLOCATOR: GlobalHeap = GlobalHeap::new();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocations_are_aligned_and_sequential() {
        let mut heap = KernelHeap::new();

        heap.init(0x1000, 0x1000, 0x2000);

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

        heap.init(0x1000, 32, 64);

        assert!(
            heap.allocate(Layout::from_size_align(32, 8).unwrap())
                .is_some()
        );
        assert!(
            heap.allocate(Layout::from_size_align(1, 1).unwrap())
                .is_none()
        );
    }

    #[test]
    fn allocation_grows_heap_when_needed() {
        let mut heap = KernelHeap::new();

        heap.init(0x1000, 32, 8192);

        let first = heap.allocate(Layout::from_size_align(32, 8).unwrap());
        assert!(first.is_some());
        assert_eq!(heap.remaining(), 0);

        let second = heap.allocate(Layout::from_size_align(1, 1).unwrap());
        assert!(second.is_some());

        assert_eq!(second.unwrap() as usize, 0x1020);
        assert_eq!(heap.remaining(), 4095);
    }
}
