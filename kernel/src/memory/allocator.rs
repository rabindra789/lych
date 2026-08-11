use super::{Frame, PAGE_SIZE};

pub struct FrameAllocator {
    start: u64,
    end: u64,
    bitmap: Bitmap,
}

pub struct Bitmap {
    bits: &'static mut [u64],
}

impl FrameAllocator {
    pub unsafe fn new(
        start: u64,
        end: u64,
        bitmap_addr: u64,
        bitmap_size: u64,
    ) -> Self {
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