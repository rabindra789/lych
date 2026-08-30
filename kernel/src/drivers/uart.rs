const UART0_BASE: usize = 0x0900_0000;

const UART_DR: *mut u32 = UART0_BASE as *mut u32;
const UART_FR: *const u32 = (UART0_BASE + 0x18) as *const u32;

const UART_FR_TXFF: u32 = 1 << 5;

pub fn putc(c: u8) {
    unsafe {
        while UART_FR.read_volatile() & UART_FR_TXFF != 0 {}

        UART_DR.write_volatile(c as u32);
    }
}

pub fn puts(s: &str) {
    for byte in s.bytes() {
        if byte == b'\n' {
            putc(b'\r');
        }
        putc(byte);
    }
}

pub fn put_digit(digit: u8) {
    putc(b'0' + digit)
}

pub fn put_hex(value: u64) {
    puts("0x");

    for shift in (0..16).rev() {
        let nibble = ((value >> (shift * 4)) & 0xF) as u8;

        let c = match nibble {
            0..=9 => b'0' + nibble,
            _ => b'A' + (nibble - 10),
        };

        putc(c);
    }
}