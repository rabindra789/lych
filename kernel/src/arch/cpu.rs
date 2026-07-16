use core::arch::asm;

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

pub unsafe fn init_exception() {
    unsafe {
        exception_vectors_init();
    }
}