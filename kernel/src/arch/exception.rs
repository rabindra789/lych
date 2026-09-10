pub const EC_BREAKPOINT: u8 = 0x3C;

pub const EC_INSTRUCTION_ABORT_LOWER_EL: u8 = 0x20;
pub const EC_INSTRUCTION_ABORT_SAME_EL: u8 = 0x21;
pub const EC_DATA_ABORT_LOWER_EL: u8 = 0x24;
pub const EC_DATA_ABORT_SAME_EL: u8 = 0x25;

pub fn exception_class(esr: u64) -> u8 {
    ((esr >> 26) & 0x3f) as u8
}

pub fn exception_name(ec: u8) -> &'static str {
    match ec {
        EC_BREAKPOINT => "Breakpoint (BRK)",

        EC_INSTRUCTION_ABORT_LOWER_EL => "Instruction Abort (lower EL)",
        EC_INSTRUCTION_ABORT_SAME_EL => "Instruction Abort (same EL)",

        EC_DATA_ABORT_LOWER_EL => "Data Abort (lower EL)",
        EC_DATA_ABORT_SAME_EL => "Data Abort (same EL)",

        _ => "Unknown",
    }
}

pub fn fault_status_name(status: u64) -> &'static str {
    match status & 0x3f {
        0x00 => "Address size fault, level 0",
        0x01 => "Address size fault, level 1",
        0x02 => "Address size fault, level 2",
        0x03 => "Address size fault, level 3",

        0x04 => "Translation fault, level 0",
        0x05 => "Translation fault, level 1",
        0x06 => "Translation fault, level 2",
        0x07 => "Translation fault, level 3",

        0x08 => "Access flag fault, level 0",
        0x09 => "Access flag fault, level 1",
        0x0a => "Access flag fault, level 2",
        0x0b => "Access flag fault, level 3",

        0x0c => "Permission fault, level 0",
        0x0d => "Permission fault, level 1",
        0x0e => "Permission fault, level 2",
        0x0f => "Permission fault, level 3",

        0x10 => "Synchronous external abort",
        0x11 => "Synchronous external abort, not TTW",

        0x18 => "Synchronous parity/ECC error",

        0x21 => "Alignment fault",

        0x22 => "Synchronous external abort",

        0x2c => "SError interupt",

        _ => "Unknown fault status",
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
