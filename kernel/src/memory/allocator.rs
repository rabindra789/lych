use super::{Frame, PAGE_SIZE};

pub struct PhysicalMemoryManager {
    allocator: FrameAllocator,
}

pub struct FrameAllocator {
    start: u64,
    end: u64,
    bitmap: Bitmap,
}

pub struct Bitmap {
    bits: &'static mut [u64],
}

#[derive(Clone, Copy)]
pub struct PageRange {
    pub start: u64,
    pub count: u64,
}

pub struct PageAllocator {
    memory: PhysicalMemoryManager,
}

impl FrameAllocator {
    pub fn new(
        start: u64,
        end: u64,
        bitmap_addr: u64,
        bitmap_size: u64,
    ) -> Self {
        assert!(start < end);
        assert!(super::is_page_aligned(start));
        assert!(super::is_page_aligned(end));

        assert!(bitmap_size > 0);
        assert!(super::is_page_aligned(bitmap_addr));

        let bitmap = unsafe {
            Bitmap::new(bitmap_addr, bitmap_size)
        };

        Self {
            start,
            end,
            bitmap,
        }
    }

    pub fn clear_all(&mut self) {
        self.bitmap.clear_all();
    }

    pub fn allocate(&mut self) -> Option<Frame> {
        let frame_count = (self.end - self.start) / PAGE_SIZE;

        for index in 0..frame_count {
            if !self.bitmap.is_set(index) {
                self.bitmap.set(index);

                return Some(Frame {
                    start: self.start + index * PAGE_SIZE,
                });
            }
        }

        None
    }

    pub fn deallocate(&mut self, frame: Frame) {
        if frame.start < self.start || frame.start >= self.end {
            return;
        }

        if !super::is_page_aligned(frame.start) {
            return;
        }

        let index = (frame.start - self.start) / PAGE_SIZE;

        self.bitmap.clear(index);
    }
}

pub fn bitmap_size(frame_count: u64) -> u64 {
    let words = (frame_count + 63) / 64;
    words * core::mem::size_of::<u64>() as u64
}

pub fn bitmap_end(start: u64, frame_count: u64) -> u64 {
    let size = bitmap_size(frame_count);

    super::align_up(start + size, super::PAGE_SIZE)
}

impl Bitmap {
    pub unsafe fn new(addr: u64, size: u64) -> Self {
        let words = size / core::mem::size_of::<u64>() as u64;

        let bits = unsafe {
            core::slice::from_raw_parts_mut(
                addr as *mut u64,
                words as usize,
            )
        };

        Self { bits }
    }

    pub fn clear_all(&mut self) {
        for word in self.bits.iter_mut() {
            *word = 0;
        }
    }

    pub fn is_set(&self, index: u64) -> bool {
        let word = index / 64;
        let bit = index % 64;

        (self.bits[word as usize] & (1u64 << bit)) != 0
    }

    pub fn set(&mut self, index: u64) {
        let word = index / 64;
        let bit = index % 64;

        self.bits[word as usize] |= 1u64 << bit;
    }

    pub fn clear(&mut self, index: u64) {
        let word = index / 64;
        let bit = index % 64;

        self.bits[word as usize] &= !(1u64 << bit);
    }
}

impl PhysicalMemoryManager {
    pub fn new (
        start: u64,
        end: u64,
        bitmap_addr: u64,
        bitmap_size: u64,
    ) -> Self {
        let mut allocator = FrameAllocator::new(
            start,
            end,
            bitmap_addr,
            bitmap_size,
        );

        allocator.clear_all();

        Self { allocator }
    }

    pub fn allocate_frame(&mut self) -> Option<Frame> {
        self.allocator.allocate()
    }

    pub fn deallocate_frame(&mut self, frame: Frame) {
        self.allocator.deallocate(frame);
    }
}

impl PageRange {
    pub fn new(start: u64, count: u64) -> Self {
        Self { start, count }
    }

    pub fn end(&self) -> u64 {
        self.start + self.count * PAGE_SIZE
    }

    pub fn contains(&self, addr: u64) -> bool {
        addr >= self.start && addr < self.end()
    }
}

impl PageAllocator {
    pub fn new(memory: PhysicalMemoryManager) -> Self {
        Self { memory }
    }

    pub fn allocate(&mut self, count: u64) -> Option<PageRange> {
        if count == 0 {
            return None;
        }

        let first = self.memory.allocate_frame()?;

        for offset in 1..count {
            let frame = match self.memory.allocate_frame() {
                Some(frame) => frame,
                None => {
                    self.free_run(first.start, offset);
                    return None;
                }
            };

            let expected = first.start + offset * PAGE_SIZE;

            if frame.start != expected {
                self.free_run(first.start, offset);
                return None;
            }
        }

        Some(PageRange::new(first.start, count))
    }

    fn free_run(&mut self, start: u64, count: u64) {
        for offset in 0..count {
            let frame = Frame {
                start: start + offset * PAGE_SIZE,
            };

            self.memory.deallocate_frame(frame);
        }
    }

    pub fn deallocate(&mut self, range: PageRange) {
        self.free_run(range.start, range.count);
    }
}
