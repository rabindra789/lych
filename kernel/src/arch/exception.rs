pub const EC_BREAKPOINT: u8 = 0x3C;

pub fn exception_class(esr: u64) -> u8 {
    ((esr >> 26) & 0x3f) as u8
}

pub fn exception_name(ec: u8) -> &'static str {
    match ec {
        0x3C => "Breakpoint (BRK)",
        _ => "Unknown",
    }
}


#[repr(C)]
pub struct ExceptionFrame {
    /// Exception Syndrome Register
    pub esr: u64,
    /// Exception Link Register
    pub elr: u64,
    /// Saved Program Status Register
    pub spsr: u64,
}

/// Returns the previous exception level encoded in SPSR_EL1.
pub fn previous_exception_level(spsr: u64) -> u8 {
    ((spsr >> 2) & 0b11) as u8
}

pub fn exception_level_name(el: u8) -> &'static str {
    match el {
        0 => "EL0",
        1 => "EL1",
        2 => "EL2",
        3 => "EL3",
        _ => "Unknown",
    }
}