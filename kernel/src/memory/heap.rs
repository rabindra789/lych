use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;

#[repr(C)]
struct FreeBlock {
    size: usize,
    next: *mut FreeBlock,
}

const FREE_BLOCK_HEADER_SIZE: usize = core::mem::size_of::<FreeBlock>();

pub struct KernelHeap {
    start: usize,
    end: usize,
    limit: usize,
    free_list: *mut FreeBlock,
    allocated: usize,
}

impl KernelHeap {
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            limit: 0,
            free_list: core::ptr::null_mut(),
            allocated: 0,
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
        self.limit = limit;

        self.allocated = 0;

        let first_block = start as *mut FreeBlock;

        unsafe {
            (*first_block).size = size;
            (*first_block).next = core::ptr::null_mut();
        }

        self.free_list = first_block;
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

        let required = FREE_BLOCK_HEADER_SIZE.checked_add(size)?;

        let mut prev: *mut FreeBlock = core::ptr::null_mut();
        let mut current = self.free_list;

        while !current.is_null() {
            let block_addr = current as usize;
            let block_size = unsafe { (*current).size };
            let next_block = unsafe { (*current).next };

            let mut user_addr = align_up(block_addr.checked_add(FREE_BLOCK_HEADER_SIZE)?, align)?;
            let mut alloc_addr = user_addr.checked_sub(FREE_BLOCK_HEADER_SIZE)?;
            let mut prefix = alloc_addr.checked_sub(block_addr)?;

            // If the prefix is too small to hold a free-block header, move the allocation forward so the prefix can remain usable.
            if prefix != 0 && prefix < FREE_BLOCK_HEADER_SIZE {
                user_addr = user_addr.checked_add(align)?;
                alloc_addr = user_addr.checked_sub(FREE_BLOCK_HEADER_SIZE)?;
                prefix = alloc_addr.checked_sub(block_addr)?;
            }

            let available = match block_size.checked_sub(prefix) {
                Some(value) => value,
                None => {
                    prev = current;
                    current = next_block;
                    continue;
                }
            };

            if available < required {
                prev = current;
                current = next_block;
                continue;
            }

            let suffix = available - required;

            let suffix_block = if suffix >= FREE_BLOCK_HEADER_SIZE {
                let suffix_addr = alloc_addr.checked_add(required)?;
                let suffix_block = suffix_addr as *mut FreeBlock;

                unsafe {
                    (*suffix_block).size = suffix;
                    (*suffix_block).next = next_block;
                }

                Some(suffix_block)
            } else {
                None
            };

            // Keep a prefix free block when possible.
            if prefix >= FREE_BLOCK_HEADER_SIZE {
                unsafe {
                    (*current).size = prefix;
                    (*current).next = suffix_block.unwrap_or(next_block);
                }
            } else {
                // Prefix is zero, so replace the current free block with the optional suffix block.
                let replacement = suffix_block.unwrap_or(next_block);

                if prev.is_null() {
                    self.free_list = replacement;
                } else {
                    unsafe {
                        (*prev).next = replacement;
                    }
                }
            }

            // Reuse the beginning of the allocated block as its metadata.
            let allocation = alloc_addr as *mut FreeBlock;

            let allocation_size = if suffix >= FREE_BLOCK_HEADER_SIZE {
                required
            } else {
                available
            };

            unsafe {
                (*allocation).size = allocation_size;
                (*allocation).next = core::ptr::null_mut();
            }

            self.allocated = self.allocated.saturating_add(allocation_size);

            return Some(user_addr as *mut u8);
        }

        // No existing free block can satisfy the allocation.
        // Grow the heap enough for the header, allocation, and alignment padding.
        let growth_needed = required.checked_add(align - 1)?;

        let pages = growth_needed.checked_add(4095)? / 4096;

        let growth_bytes = pages.checked_mul(4096)?;
        let old_end = self.end;

        if !self.grow_pages(pages) {
            return None;
        }

        let new_block = old_end as *mut FreeBlock;

        unsafe {
            (*new_block).size = growth_bytes;
            (*new_block).next = self.free_list;
        }

        self.free_list = new_block;

        self.allocate(layout)
    }

    pub fn used(&self) -> usize {
        self.allocated
    }

    pub fn remaining(&self) -> usize {
        let mut sum: usize = 0;
        let mut current = self.free_list;
        while !current.is_null() {
            sum = sum.saturating_add(unsafe { (*current).size });
            current = unsafe { (*current).next };
        }
        sum
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
    fn allocations_are_aligned_and_non_overlapping() {
        let mut heap = KernelHeap::new();

        heap.init(0x1000, 0x1000, 0x2000);

        let first = heap.allocate(Layout::from_size_align(16, 8).unwrap());
        let second = heap.allocate(Layout::from_size_align(32, 16).unwrap());

        let first_addr = first.unwrap() as usize;
        let second_addr = second.unwrap() as usize;

        assert_eq!(first_addr % 8, 0);
        assert_eq!(second_addr % 16, 0);

        let first_end = first_addr + 16;
        let second_end = second_addr + 32;

        assert!(first_end <= second_addr || second_end <= first_addr);

        assert!(heap.used() > 0);
        assert!(heap.remaining() > 0);
        assert_eq!(heap.used() + heap.remaining(), heap.end - heap.start);
    }

    #[test]
    fn allocation_fails_when_heap_is_full() {
        let mut heap = KernelHeap::new();

        heap.init(0x1000, 64, 64);

        assert!(
            heap.allocate(Layout::from_size_align(32, 8).unwrap())
                .is_some()
        );

        assert!(
            heap.allocate(Layout::from_size_align(32, 8).unwrap())
                .is_none()
        );
    }

    #[test]
    fn allocation_grows_heap_when_needed() {
        let mut heap = KernelHeap::new();

        heap.init(0x1000, 32, 8192);

        let first = heap.allocate(Layout::from_size_align(32, 8).unwrap());
        assert!(first.is_some());

        let second = heap.allocate(Layout::from_size_align(1, 1).unwrap());
        assert!(second.is_some());

        assert!(heap.end > 0x1020);
        assert!(heap.remaining() > 0);
    }
}
