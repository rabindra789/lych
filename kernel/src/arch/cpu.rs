use core::arch::asm;

pub const MAIR_ATTR_DEVICE_NGNRE: u64 = 0x04;
pub const MAIR_ATTR_NORMAL_WBWA: u64 = 0xFF;

pub const MAIR_EL1_VALUE: u64 = MAIR_ATTR_DEVICE_NGNRE | (MAIR_ATTR_NORMAL_WBWA << 8);

unsafe extern "C" {
    fn exception_vectors_init();
}

/// Return the current exception level (EL0-EL3)
pub fn current_el() -> u64 {
    let el: u64;

    unsafe {
        asm!(
            "mrs {}, CurrentEL",
            out(reg) el,
        );
    }

    el >> 2
}

// Read the translation control register for EL1.
pub fn read_tcr_el1() -> u64 {
    let tcr: u64;

    unsafe {
        asm!("mrs {}, TCR_EL1", out(reg) tcr);
    }

    tcr
}

// Read the fault Address Register for EL1
pub fn read_far_el1() -> u64 {
    let far: u64;

    unsafe {
        asm!(
            "mrs {}, FAR_EL1",
            out(reg) far,
        );
    }

    far
}

/// Enable FP and SIMD at EL1 by setting CPACR_EL1.FPEN = 0b11.
pub fn enable_fp_simd() {
    unsafe {
        let cpacr: u64;
        asm!("mrs {}, CPACR_EL1", out(reg) cpacr);
        let cpacr = cpacr | (3 << 20);
        asm!("msr CPACR_EL1, {}", in(reg) cpacr);
    }
}

/// Configure the memory attributes used by the translation tables.
pub fn configure_mair() {
    unsafe {
        asm!(
            "msr MAIR_EL1, {value}",
            "isb",
            value = in (reg) MAIR_EL1_VALUE,
        );
    }
}

/// Configure the EL1 stage-1 translation control register (TCR_EL1).
pub fn configure_tcr() {
    const TCR_T0SZ: u64 = 16; // 48-bit virtual address space
    const TCR_IRGN0_WBWA: u64 = 1 << 8; // Write-back write-allocate cacheable
    const TCR_ORGN0_WBWA: u64 = 1 << 10; // Write-back write-allocate cacheable
    const TCR_SH0_INNER: u64 = 3 << 12; // Inner shareable
    const TCR_TG0_4K: u64 = 0 << 14; // 4KB granule
    const TCR_IPS_40BIT: u64 = 2 << 32; // 40-bit physical address space

    let tcr =
        TCR_T0SZ | TCR_IRGN0_WBWA | TCR_ORGN0_WBWA | TCR_SH0_INNER | TCR_TG0_4K | TCR_IPS_40BIT;

    unsafe {
        asm!(
            "msr TCR_EL1, {value}",
            "isb",
            value = in(reg) tcr,
        );
    }
}

/// Set the EL1 translation table base register.
pub fn set_ttbr0_el1(physical_address: u64) {
    unsafe {
        asm!(
            "msr TTBR0_EL1, {value}",
            "dsb ish",
            "isb",
            value = in(reg) physical_address,
        );
    }
}

pub fn init() {
    enable_fp_simd();
    configure_mair();
    configure_tcr();

    unsafe {
        exception_vectors_init();
    }
}
