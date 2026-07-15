use core::arch::asm;

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