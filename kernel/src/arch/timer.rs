use core::arch::asm;

/// Return the Generic Timer frequency in ticks per second.
pub fn frequency() -> u64 {
    let value: u64;

    unsafe {
        asm!("mrs {}, CNTFRQ_EL0", out(reg) value);
    }

    value
}

/// Return the current physical timer counter.
pub fn counter() -> u64 {
    let value: u64;

    unsafe {
        asm!("mrs {}, CNTPCT_EL0", out(reg) value);
    }

    value
}
