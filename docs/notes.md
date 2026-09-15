# Development Notes

The kernel is currently developed and tested using release builds.

Debug builds may enable additional runtime checks from `core` that are not yet suitable during early bare-metal bring-up.

## Memory System

### Physical RAM layout

The platform defines `RAM_BASE` at `0x4000_0000` and `RAM_SIZE` of 128 MiB, so `RAM_END` is `0x4000_0000 + 128MiB = 0x4800_0000`. The kernel runs on QEMU `virt` with this physical RAM region available.

### Kernel memory layout

The kernel divides the usable portion of RAM into several regions. The area from `RAM_BASE` upward is split into:

- A bitmap that tracks which frames are in use.
- The physical frame allocator that hands out free frames.
- Page tables that map RAM and devices.
- A reserved heap region from `0x4100_0000` to `0x4110_0000`.
- Empty space for future kernel heap usage.

### Usable memory calculation

Usable memory starts at the first page-aligned address above the stack top. The function `usable_memory_start()` returns `align_up(addr(__stack_top), PAGE_SIZE)` where `PAGE_SIZE` is 4096 bytes. The stack pointer is placed just above the kernel image, so usable memory begins where the stack ends.

The usable memory region runs from that start address up to `RAM_END`. The function `usable_memory_region()` returns a `MemoryRegion` describing this range.

The bitmap that tracks frame allocation lives at the beginning of the usable region. The area covered by the bitmap is excluded from the allocatable frame range, so the frame allocator never hands out its own bits.

### Page alignment

The kernel uses 4 KiB pages throughout. The function `is_page_aligned(addr)` checks whether `addr % PAGE_SIZE == 0`. The helper `align_up(addr, alignment)` rounds an address up to the next multiple of `alignment`, using the formula `(addr + alignment - 1) & !(alignment - 1)`. All frame addresses and bitmap positions must be page-aligned.

### Frame and PhysAddr/VirtAddr types

The kernel defines three core types for physical memory reasoning:

- `PhysAddr(u64)` wraps a physical address. It provides `new(addr)` and `as_u64()`.
- `VirtAddr(u64)` wraps a virtual address with the same accessors.
- `Frame` represents a single physical frame. It holds a `PhysAddr start` field.

These types are simple data containers with no behavior beyond holding a 64-bit address value.

### FrameRange and usable frame iteration

`FrameRange` iterates over frames in a `MemoryRegion`. It stores a `current` position and an `end` boundary. The `Iterator` implementation returns one `Frame` per call, advancing `current` by `PAGE_SIZE` each time. When `current >= end`, iteration stops.

The function `usable_frames()` returns a `FrameRange` over the usable memory region, enabling callers to scan all frames that the frame allocator can potentially manage.

### Bitmap location and bitmap size calculation

The bitmap tracks which frames are in use. Its size depends on the total frame count: `bitmap_size(frame_count)` computes `(frame_count + 63) / 64` words, each word being a `u64`, so the total size is `words * 8 bytes`. The bitmap starts at `bitmap_start()`, which returns `usable_memory_start()`. The bitmap ends at `bitmap_end(start, frame_count)`, which calculates `align_up(start + size, PAGE_SIZE)`. This aligns the bitmap's end to a page boundary.

The bitmap is placed at the very beginning of usable memory. Because it occupies physical frames, those frames are marked as used in the bitmap and are excluded from the allocatable range. The frame allocator never returns a frame that the bitmap says is allocated.

### Why the bitmap is excluded from allocatable memory

The bitmap lives at `usable_memory_start()` and extends upward for `bitmap_size(frame_count)` bytes, aligned to `PAGE_SIZE`. The frame allocator's range runs from the bitmap's aligned end up to `usable_memory_region().end`. Any frame whose index falls below the bitmap's range is already marked as used, so the allocator skips it. This design prevents the allocator from handing out its own tracking data.

### PhysicalMemoryManager

`PhysicalMemoryManager` wraps a `FrameAllocator` and exposes high-level operations. It is initialized by `memory::init()`, which computes the usable region, creates a `PhysicalMemoryManager` with the bitmap parameters, and then reserves the heap range so those frames are marked as used. The manager delegates `allocate_frame()`, `deallocate_frame()`, `reserve_range()`, and `is_allocated()` to the inner `FrameAllocator`.

### FrameAllocator

`FrameAllocator` holds a start address, an end address, and a `Bitmap`. It provides:

- `allocate()` - scans the bitmap for the first clear bit, sets it, and returns the corresponding `Frame`.
- `deallocate(frame)` - clears the bitmap bit for the given frame, unless the frame is outside the allocator's range or unaligned.
- `reserve_range(start, end)` - marks all frames in `[start, end)` as used by setting their bitmap bits. This is how the heap region is claimed upfront.
- `is_allocated(frame)` - checks whether a given frame's bitmap bit is set.

### Bitmap operations

The `Bitmap` type holds a `&'static mut [u64]` slice. Operations work on individual bits within that slice:

- `is_set(index)` - returns true if the bit at `index` is 1.
- `set(index)` - sets the bit at `index` to 1.
- `clear(index)` - sets the bit at `index` to 0.
- `clear_all()` - zeroes every word in the slice.

The word index is `index / 64` and the bit within the word is `index % 64`, using a `1u64 << bit` mask.

### allocate_frame()

`PhysicalMemoryManager::allocate_frame()` delegates to `self.allocator.allocate()`, which scans from the allocator's start range, finds the first unset bitmap bit, sets it, and returns a `Frame` with the corresponding physical address. If no free frames remain, it returns `None`.

### deallocate_frame()

`PhysicalMemoryManager::deallocate_frame(frame)` delegates to `self.allocator.deallocate(frame)`. It checks that the frame's address is within `[start, end)` and page-aligned. If valid, it computes the bitmap index as `(frame_start - start) / PAGE_SIZE` and clears that bit.

### PageAllocator and PageRange

`PageRange` holds a `start` address and a `count` of pages. `PageAllocator` operates over the physical memory manager's allocatable range, scanning the bitmap for a run of `count` consecutive free frames. When found, it sets those bits and returns the `PageRange`. It also provides `deallocate(range)` to free a previously allocated range.

Page allocation is a higher-level path that may be used for larger contiguous allocations, such as later kernel heap setup or device buffers.

### page table allocation

The page-table hierarchy (L0 through L3, plus a separate L2 for UART) is allocated entirely through the physical frame allocator. `PageTableHierarchy::new()` calls `with_physical_memory(|memory| memory.allocate_frame())` five times to obtain frames for each level. If any allocation fails, previously acquired frames are freed and `None` is returned.

Each page-table frame is then populated with entries. The L2 table contains block descriptors that identity-map 64 RAM blocks at `0x4000_0000`, each block 2 MiB wide (64 entries × 2 MiB = 128 MiB total). The UART gets a separate device-type descriptor at `0x0900_0000`.

### page table hierarchy

The hierarchy consists of four levels (L0-L3) with a 4 KiB granule. L0 is the top-level table, L1 is second, L2 covers the 1 GiB RAM region (64 × 2 MiB blocks), and L3 provides fine-grained 4 KiB mapping within each 2 MiB block. A fifth `uart_l2` table maps the UART device at `0x0900_0000`. The hierarchy is created once during early init and installed via `TTBR0_EL1`.

### identity mapping

The identity map means that virtual addresses equal physical addresses for the RAM region. The L2 table maps physical `0x4000_0000 + i * 0x20_0000` to the same virtual address for `i` in `0..64`. This covers the 128 MiB RAM region from `0x4000_0000` to `0x4800_0000`. Because the map is identity, the kernel can access any RAM address through the same numeric value as a virtual address.

### RAM and UART mappings

The L2 table's first entry maps the RAM identity region. Entry 72 (index 72) in the UART L2 table maps `0x0900_0000` with device attributes. All other L2 entries are zero (unused). The attributes use `ATTR_NORMAL`, `AP_RW_EL1`, `SH_INNER`, and the access flag.

### MAIR_EL1

The Memory Attribute Indicator Register `MAIR_EL1` defines memory types and attributes. The kernel configures normal memory as `Outer Shareable` with Write-Back, Write-Allocate policies, and device memory as Device-nGnRnE. These attributes affect how the cache and buffer policies behave for each memory type. The exact `MAIR_EL1` value is set in the platform bring-up code and is not directly manipulated by the memory module.

### TCR_EL1

The Translation Control Register `TCR_EL1` controls the stage-1 translation regime. The kernel sets it for a 4 KiB page granule, a 48-bit virtual address space, and a 40-bit physical address space. The `TCR_EL1` value also enables the HW walker and configures the TTL (translation table level) parameters. This register is written during page-table initialization before the MMU is enabled.

### TTBR0_EL1

The Top-Level Border Register 0 `TTBR0_EL1` holds the physical address of the L0 page table. After the page-table hierarchy is created and linked, `install_page_table_root()` writes the L0 frame's physical address into `TTBR0_EL1`. This tells the MMU where to start walking the translation tables.

### MMU enable

The Microcontroller Unit (MMU) is enabled by setting `SCTLR_EL1.M`. After `TTBR0_EL1` is set, page tables are linked, and `SCTLR_EL1.M` is 1, the kernel can translate virtual to physical addresses. The function `arch::cpu::enable_mmu()` performs this write. After enable, all subsequent memory accesses go through the stage-1 translation tables.

### exception vector coverage under the identity map

The identity map covers the RAM region from `0x4000_0000` to `0x4800_0000`. Exception vectors live at the low end of the virtual address space, which is not covered by the identity map. However, the vectors are physical addresses that the hardware vectors table (`VBAR_EL1`) points to, so the CPU jumps to the correct physical location regardless of the identity map. The identity map ensures that the RAM itself is accessible through virtual addresses, while the vector table operates through physical addresses.

### kernel heap reservation

The kernel heap occupies the range `HEAP_START = 0x4100_0000` through `HEAP_END = 0x4110_0000` (1 MiB). This entire region already falls within the identity-mapped RAM (`0x4000_0000` to `0x4800_0000`), so no new page-table mappings are needed for the heap to be physically accessible.

The heap is reserved from the physical frame allocator by calling `manager.reserve_range(HEAP_START, HEAP_END)` during initialization. This marks all frames in that range as used in the bitmap, so the frame allocator will never hand them out. The heap allocator (a simple bump-pointer or similar) then uses those reserved frames as a static heap, since they are guaranteed to be available and mapped.

### HEAP_START, HEAP_SIZE, and HEAP_END

- `HEAP_START = 0x4100_0000` marks the first address of the heap region.
- `HEAP_SIZE = 0x0010_0000` (1 MiB) is the total heap extent.
- `HEAP_END = HEAP_START + HEAP_SIZE = 0x4110_0000` marks the first address after the heap.

These constants are defined in `kernel/src/memory.rs` and used both by the physical frame allocator (to reserve the region) and by any code that needs to know where the kernel heap begins and ends.

### why the heap is only reserved, not yet a dynamic allocator

The current implementation reserves the 1 MiB heap region from the physical frame allocator by marking its frames as used in the bitmap. This guarantees that the heap area will not be given to other allocations. However, the kernel does not yet have a dynamic heap allocator (such as a bump-pointer or slab allocator) that actually hands out sub-ranges within the reserved region. The reservation step comes first: it secures the memory so that no other code can claim those frames. A future phase will implement the dynamic allocator that actually services `allocate`/`deallocate` calls within the reserved heap area.

### Component relationship

```
RAM (0x4000_0000 .. 0x4800_0000)
  └─ bitmap (at usable_memory_start, excluded from allocator range)
  └─ physical frame allocator (bitmap-backed, hands out free frames)
  └─ page tables (L0-L3, identity-map RAM, map UART at 0x0900_0000)
  └─ reserved heap region (0x4100_0000 .. 0x4110_0000, marked used in bitmap)
  └─ future kernel heap allocator (will hand out sub-ranges within reserved region)
```

The bitmap sits right at the start of usable memory. The frame allocator uses the remaining frames above the bitmap. Page tables occupy some frames above the allocator, and the heap region is claimed on top of that. The order matters: bitmap first, then allocator, then page tables, then reserved heap, with the future heap allocator sitting inside the reserved region.
