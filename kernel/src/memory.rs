#[repr(C)]
pub struct MemoryRegion {
    pub start: u64,
    pub end: u64,
}

#[repr(C)]
pub struct Frame {
    pub start: u64,
}

pub struct FrameRange {
    current: u64,
    end: u64,
}

unsafe extern "C" {
    static __text_start: u8;
    static __text_end: u8;

    static __rodata_start: u8;
    static __rodata_end: u8;

    static __data_start: u8;
    static __data_end: u8;

    static __bss_start: u8;
    static __bss_end: u8;

    static __stack_top: u8;
}

fn addr(symbol: &u8) -> u64 {
    symbol as *const u8 as u64
}

const PAGE_SIZE: u64 = 4096;

fn align_up(addr: u64, alignment: u64) -> u64 {
    (addr + alignment - 1) & !(alignment - 1)
}

pub fn usable_memory_start() -> u64 {
    unsafe { align_up(addr(&__stack_top), PAGE_SIZE) }
}

pub fn usable_memory_region() -> MemoryRegion {
    MemoryRegion {
        start: usable_memory_start(),
        end: crate::platform::RAM_END,
    }
}

pub fn is_page_aligned(addr: u64) -> bool {
    addr % PAGE_SIZE == 0
}

pub fn frame_from_address(addr: u64) -> Option<Frame> {
    if !is_page_aligned(addr) {
        return None;
    }

    let region = usable_memory_region();

    if addr < region.start || addr >= region.end {
        return None;
    }

    Some(Frame { start: addr })
}

pub fn frame_count() -> u64 {
    let region = usable_memory_region();
    (region.end - region.start) / PAGE_SIZE
}

impl FrameRange {
    pub fn new(region: MemoryRegion) -> Self {
        debug_assert!(region.start % PAGE_SIZE == 0);
        debug_assert!(region.end % PAGE_SIZE == 0);
        debug_assert!(region.start < region.end);

        Self {
            current: region.start,
            end: region.end,
        }
    }
}

impl Iterator for FrameRange {
    type Item = Frame;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.end {
            return None;
        }

        let frame = Frame {
            start: self.current,
        };

        self.current += PAGE_SIZE;

        Some(frame)
    }
}

pub fn usable_frames() -> FrameRange {
    FrameRange::new(usable_memory_region())
}

pub fn print_layout() {
    use crate::drivers::uart;

    unsafe {
        uart::puts("\nKernel Memory Layout\n");

        uart::puts(".text   : ");
        uart::put_hex(addr(&__text_start));
        uart::puts(" - ");
        uart::put_hex(addr(&__text_end));
        uart::putc(b'\n');

        uart::puts(".rodata : ");
        uart::put_hex(addr(&__rodata_start));
        uart::puts(" - ");
        uart::put_hex(addr(&__rodata_end));
        uart::putc(b'\n');

        uart::puts(".data   : ");
        uart::put_hex(addr(&__data_start));
        uart::puts(" - ");
        uart::put_hex(addr(&__data_end));
        uart::putc(b'\n');

        uart::puts(".bss    : ");
        uart::put_hex(addr(&__bss_start));
        uart::puts(" - ");
        uart::put_hex(addr(&__bss_end));
        uart::putc(b'\n');

        uart::puts("stack   : ");
        uart::put_hex(addr(&__stack_top));
        uart::putc(b'\n');

        uart::puts("RAM     : ");
        uart::put_hex(crate::platform::RAM_BASE);
        uart::puts(" - ");
        uart::put_hex(crate::platform::RAM_END);
        uart::putc(b'\n');

        let usable = usable_memory_region();

        uart::puts("usable  : ");
        uart::put_hex(usable.start);
        uart::puts(" - ");
        uart::put_hex(usable.end);
        uart::putc(b'\n');

        uart::puts("aligned : ");
        uart::put_hex(is_page_aligned(usable.start) as u64);
        uart::putc(b'\n');
    }
}
