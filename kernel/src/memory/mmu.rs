pub const PAGE_SIZE: u64 = 4096;
pub const ENTRY_COUNT: usize = 512;
pub const L0_SHIFT: u64 = 39;
pub const L1_SHIFT: u64 = 30;
pub const L2_SHIFT: u64 = 21;
pub const L3_SHIFT: u64 = 12;

pub const TABLE_INDEX_MASK: u64 = 0x1ff;

#[repr(C, align(4096))]
pub struct PageTable {
    pub entries: [u64; ENTRY_COUNT],
}

impl PageTable {
    pub const fn new() -> Self {
        Self {
            entries: [0; ENTRY_COUNT],
        }
    }
}

pub fn l0_index(va: u64) -> usize {
    ((va >> L0_SHIFT) & TABLE_INDEX_MASK) as usize
}

pub fn l1_index(va: u64) -> usize {
    ((va >> L1_SHIFT) & TABLE_INDEX_MASK) as usize
}

pub fn l2_index(va: u64) -> usize {
    ((va >> L2_SHIFT) & TABLE_INDEX_MASK) as usize
}

pub fn l3_index(va: u64) -> usize {
    ((va >> L3_SHIFT) & TABLE_INDEX_MASK) as usize
}

pub fn page_offset(va: u64) -> u64 {
    va & 0xfff
}