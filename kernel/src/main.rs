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
    uart::put_hex(el);
    uart::putc(b'\n');

    unsafe {
        arch::cpu::init_exception();
    }

    unsafe {
        core::arch::asm!("brk #0")
    }
    
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

#[unsafe(no_mangle)]
pub extern "C" fn exception_handler(esr: u64, elr: u64) -> ! {
    uart::puts("\n=== Exception ===\n");

    uart::puts("ESR_EL1: ");
    uart::put_hex(esr);
    uart::putc(b'\n');

    let ec = arch::exception::exception_class(esr);

    uart::puts("EC: ");
    uart::puts(arch::exception::exception_name(ec));
    uart::putc(b'\n');

    uart::puts("ELR_EL1: ");
    uart::put_hex(elr);
    uart::putc(b'\n');

    loop {
        core::hint::spin_loop();
    }
}