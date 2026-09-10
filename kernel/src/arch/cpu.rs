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

pub fn init() {
    enable_fp_simd();
    configure_mair();

    unsafe {
        exception_vectors_init();
    }
}
