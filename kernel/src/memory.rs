pub mod allocator;
pub mod mmu;

use allocator::PhysicalMemoryManager;

#[repr(C)]
pub struct MemoryRegion {
    pub start: u64,
    pub end: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
#[allow(dead_code)] // Used by frame-management and page-table interfaces.
pub struct Frame {
    pub start: PhysAddr,
}

#[allow(dead_code)] // Used by memory scanning once frame enumeration is wired in.
pub struct FrameRange {
    current: u64,
    end: u64,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PhysAddr(pub u64);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct VirtAddr(pub u64);

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

impl PhysAddr {
    pub const fn new(addr: u64) -> Self {
        Self(addr)
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl VirtAddr {
    pub const fn new(addr: u64) -> Self {
        Self(addr)
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

pub const PAGE_SIZE: u64 = 4096;

fn align_up(addr: u64, alignment: u64) -> u64 {
    (addr + alignment - 1) & !(alignment - 1)
}

pub fn usable_memory_start() -> u64 {
    unsafe { align_up(addr(&__stack_top), PAGE_SIZE) }
}

pub fn usable_memory_region() -> MemoryRegion {
    let start = allocator::bitmap_end(bitmap_start(), frame_count());

    MemoryRegion {
        start,
        end: crate::platform::RAM_END,
    }
}

pub fn raw_memory_region() -> MemoryRegion {
    MemoryRegion {
        start: usable_memory_start(),
        end: crate::platform::RAM_END,
    }
}

pub fn is_page_aligned(addr: u64) -> bool {
    addr % PAGE_SIZE == 0
}

#[allow(dead_code)] // Used by physical-frame lookup helpers.
pub fn frame_from_address(addr: u64) -> Option<Frame> {
    if !is_page_aligned(addr) {
        return None;
    }

    let region = usable_memory_region();

    if addr < region.start || addr >= region.end {
        return None;
    }

    Some(Frame {
        start: PhysAddr::new(addr),
    })
}

pub fn frame_count() -> u64 {
    let region = raw_memory_region();
    (region.end - region.start) / PAGE_SIZE
}

#[allow(dead_code)] // Used by memory scanning once frame enumeration is wired in.
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
            start: PhysAddr::new(self.current),
        };

        self.current += PAGE_SIZE;

        Some(frame)
    }
}

#[allow(dead_code)] // Used when boot code needs to iterate all usable frames.
pub fn usable_frames() -> FrameRange {
    FrameRange::new(usable_memory_region())
}

pub fn bitmap_start() -> u64 {
    usable_memory_start()
}

pub fn bitmap_size() -> u64 {
    allocator::bitmap_size(frame_count())
}

static mut PHYSICAL_MEMORY: Option<PhysicalMemoryManager> = None;

/// Initialize the physical frame allocator over the usable memory region.
///
/// The bitmap itself lives just below the allocatable region and is excluded
/// from the allocator's range, so it can never be handed out as a frame.
pub fn init() {
    let region = usable_memory_region();

    let manager =
        PhysicalMemoryManager::new(region.start, region.end, bitmap_start(), bitmap_size());

    unsafe {
        PHYSICAL_MEMORY = Some(manager);
    }
}

#[allow(dead_code)] // Used by page-table and allocation callers.
pub fn with_physical_memory<R>(f: impl FnOnce(&mut PhysicalMemoryManager) -> R) -> R {
    unsafe {
        let slot = core::ptr::addr_of_mut!(PHYSICAL_MEMORY);

        match (*slot).as_mut() {
            Some(manager) => f(manager),
            None => panic!("physical memory manager not initialized"),
        }
    }
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

pub fn phys_to_virt(addr: PhysAddr) -> VirtAddr {
    VirtAddr::new(addr.as_u64())
}

#[allow(dead_code)] // Used once descriptors need phys addresses from virt addresses.
pub fn virt_to_phys(addr: VirtAddr) -> PhysAddr {
    PhysAddr::new(addr.as_u64())
}

pub fn test_page_table_mappings() {
    let hierarchy =
        mmu::PageTableHierarchy::new().expect("failed to allocate page-table hierarchy");

    hierarchy.zero();
    hierarchy.link_tables();

    // Verify RAM identity mapping.
    mmu::with_page_table(hierarchy.l2, |table| {
        for index in 0..64 {
            let expected_physical = 0x4000_0000 + (index as u64 * 0x20_0000);

            let entry = table.entries[index];

            assert_eq!(entry & mmu::DESC_VALID, mmu::DESC_VALID);
            assert_eq!(entry & mmu::DESC_TABLE, 0);

            let physical = entry & 0x0000_FFFF_FFE0_0000;

            assert_eq!(physical, expected_physical);
        }

        // Everything after the 64 RAM blocks must remain unused.
        for index in 64..mmu::ENTRY_COUNT {
            assert_eq!(table.entries[index], 0);
        }
    });

    // Verify UART mapping.
    mmu::with_page_table(hierarchy.uart_l2, |table| {
        let entry = table.entries[72];

        assert_eq!(entry & mmu::DESC_VALID, mmu::DESC_VALID);
        assert_eq!(entry & mmu::DESC_TABLE, 0);

        let physical = entry & 0x0000_FFFF_FFE0_0000;

        assert_eq!(physical, 0x0900_0000);

        // No unexpected UART mappings.
        for index in 0..mmu::ENTRY_COUNT {
            if index != 72 {
                assert_eq!(table.entries[index], 0);
            }
        }
    });

    crate::drivers::uart::puts("Identity mappings validated successfully\n");
}
