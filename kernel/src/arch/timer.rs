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

/// Set the physical timer to fire after the given number of ticks.
///
/// `CNTP_TVAL_EL0` is a 32-bit register.
pub fn set_physical_timer(ticks: u32) {
    unsafe {
        core::arch::asm!(
            "msr CNTP_TVAL_EL0, {}",
            in(reg) u64::from(ticks),
        );
    }
}

/// Enable the physical timer.
pub fn enable_physical_timer() {
    unsafe {
        core::arch::asm!(
            "msr CNTP_CTL_EL0, {value}",
            value = in(reg) 1u64,
        );
    }
}

/// Disable the physical timer.
pub fn disable_physical_timer() {
    unsafe {
        core::arch::asm!(
            "msr CNTP_CTL_EL0, {value}",
            value = in(reg) 0u64,
        );
    }
}
