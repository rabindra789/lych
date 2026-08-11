use super::{Frame, PAGE_SIZE};

pub struct FrameAllocator {
    next: u64,
    end: u64,
}

pub struct Bitmap {
    bits: &'static mut [u64],
}

impl FrameAllocator {
    pub fn new(start: u64, end: u64) -> Self {
        Self {
            next: start,
            end,
        }
    }

    pub fn allocate(&mut self) -> Option<Frame> {
        if self.next >= self.end {
            return None;
        }

        let frame = Frame {
            start: self.next,
        };

        self.next += PAGE_SIZE;

        Some(frame)
    }

    pub fn deallocate(&mut self, frame:Frame) {
        // Temp implementation
        // Proper reusable-frame tracking comes with the bitmap allocator.
        let _ = frame;
    }
}

pub fn bitmap_size(frame_count: u64) -> u64 {
    let words = (frame_count + 63) / 64;
    words * core::mem::size_of::<u64>() as u64
}