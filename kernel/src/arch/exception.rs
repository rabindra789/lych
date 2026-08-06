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
