pub const PAGE_SIZE: u64 = 4096;
pub const ENTRY_COUNT: usize = 512;
pub const L0_SHIFT: u64 = 39;
pub const L1_SHIFT: u64 = 30;
pub const L2_SHIFT: u64 = 21;
pub const L3_SHIFT: u64 = 12;

pub const TABLE_INDEX_MASK: u64 = 0x1ff;

pub const DESC_VALID: u64 = 1 << 0;
pub const DESC_TABLE: u64 = 1 << 1;

pub const ATTR_DEVICE: u64 = 0;
pub const ATTR_NORMAL: u64 = 1 << 2;

pub const AP_RW_EL1: u64 = 0 << 6;
pub const AP_RO_EL1: u64 = 1 << 7;

pub const SH_INNER: u64 = 3 << 8;
pub const ACCESS_FLAG: u64 = 1 << 10;

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

pub unsafe fn from_frame(frame: crate::memory::Frame) -> &'static mut PageTable {
    &mut *(frame.start as *mut PageTable)
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

pub fn page_descriptor(physical: u64) -> u64 {
    (physical & 0x0000_FFFF_FFFF_F000)
        | DESC_VALID
        | ATTR_NORMAL
        | AP_RO_EL1
        | SH_INNER
        | ACCESS_FLAG
}

pub fn table_descriptor(physical: u64) -> u64 {
    (physical & 0x0000_FFFF_FFFF_F000) | DESC_VALID | DESC_TABLE
}

pub fn table_entry(table: u64) -> u64 {
    table_descriptor(table)
}

pub fn zero_table(table: &mut PageTable) {
    for entry in table.entries.iter_mut() {
        *entry = 0;
    }
}
