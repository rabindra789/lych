#![no_std]
#![no_main]

mod arch;
mod drivers;

use drivers::uart;
use core::panic::PanicInfo;

/// First Rust func to executed by the kernel
#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    uart::puts("Lych kernel\n");
    uart::puts("Booting...\n");
    let el = arch::cpu::current_el();

    uart::puts("Current EL: ");
    uart::put_digit(el as u8);
    uart::putc(b'\n');
    
    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop{
        core::hint::spin_loop();
    }
}