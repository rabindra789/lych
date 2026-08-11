use super::Frame;

pub struct FrameAllocator {
    next: u64,
    end: u64,
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

        self.next += super::PAGE_SIZE;

        Some(frame)
    }
}