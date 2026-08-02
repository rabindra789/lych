pub fn exception_class(esr: u64) -> u8 {
    ((esr >> 26) & 0x3f) as u8
}