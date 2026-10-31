use core::sync::atomic::{AtomicBool, Ordering};

pub const UART0_BASE: usize = 0x0900_0000;

const UART_DR: *mut u32 = UART0_BASE as *mut u32;
const UART_FR: *const u32 = (UART0_BASE + 0x18) as *const u32;

const UART_FR_TXFF: u32 = 1 << 5;

static UART_TX_LOCK: AtomicBool = AtomicBool::new(false);

struct UartGuard {
    saved_daif: u64,
}

impl UartGuard {
    fn acquire() -> Self {
        let saved_daif: u64;

        unsafe {
            // Mask IRQs on this CPU before acquiring the lock.
            core::arch::asm!(
                "mrs {0}, daif",
                "msr daifset, #2",
                out(reg) saved_daif,
                options(nostack, preserves_flags)
            );
        }

        while UART_TX_LOCK
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }

        Self { saved_daif }
    }
}

impl Drop for UartGuard {
    fn drop(&mut self) {
        // Release the lock before restoring local IRQ state.
        UART_TX_LOCK.store(false, Ordering::Release);

        unsafe {
            core::arch::asm!(
                "msr daif, {0}",
                in(reg) self.saved_daif,
                options(nostack, preserves_flags)
            );
        }
    }
}

/// Raw UART transmission. Caller must hold UART_TX_LOCK.
fn putc_raw(c: u8) {
    unsafe {
        while UART_FR.read_volatile() & UART_FR_TXFF != 0 {
            core::hint::spin_loop();
        }

        UART_DR.write_volatile(c as u32);
    }
}

/// Writer used while the UART lock is held.
pub struct UartWriter;

impl UartWriter {
    pub fn putc(&mut self, c: u8) {
        putc_raw(c);
    }

    pub fn puts(&mut self, s: &str) {
        for byte in s.bytes() {
            if byte == b'\n' {
                putc_raw(b'\r');
            }

            putc_raw(byte);
        }
    }

    pub fn put_hex(&mut self, value: u64) {
        self.puts("0x");

        for shift in (0..16).rev() {
            let nibble = ((value >> (shift * 4)) & 0xF) as u8;

            let c = match nibble {
                0..=9 => b'0' + nibble,
                _ => b'A' + (nibble - 10),
            };

            putc_raw(c);
        }
    }

    pub fn put_digit(&mut self, digit: u8) {
        putc_raw(b'0' + digit);
    }
}

/// Execute a complete UART operation under one lock.
pub fn write(f: impl FnOnce(&mut UartWriter)) {
    let _guard = UartGuard::acquire();
    let mut writer = UartWriter;

    f(&mut writer);
}

// Compatibility functions for existing callers.

pub fn putc(c: u8) {
    write(|uart| uart.putc(c));
}

pub fn puts(s: &str) {
    write(|uart| uart.puts(s));
}

#[allow(dead_code)]
pub fn put_digit(digit: u8) {
    write(|uart| uart.put_digit(digit));
}

pub fn put_hex(value: u64) {
    write(|uart| uart.put_hex(value));
}
