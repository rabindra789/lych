#![no_std]
#![no_main]

mod arch;
mod drivers;

use drivers::uart;
use core::panic::PanicInfo;

use crate::arch::exception::ExceptionFrame;

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
pub extern "C" fn exception_handler(frame: &ExceptionFrame) -> ! {
    let ec = arch::exception::exception_class(frame.esr);

    uart::puts("\n");
    uart::puts("Lych Kernel Exception\n");
    uart::puts("-------------------------------\n");

    uart::puts("Exception : ");
    uart::puts(arch::exception::exception_name(ec));
    uart::putc(b'\n');

    uart::puts("ESR_EL1   : ");
    uart::put_hex(frame.esr);
    uart::putc(b'\n');

    uart::puts("ELR_EL1   : ");
    uart::put_hex(frame.elr);
    uart::putc(b'\n');

    uart::puts("SPSR_EL1  : ");
    uart::put_hex(frame.spsr);
    uart::putc(b'\n');
    
    let previous_el = arch::exception::previous_exception_level(frame.spsr);
    
    uart::puts("Previous EL: ");
    uart::puts(arch::exception::exception_level_name(previous_el));
    uart::putc(b'\n');

    uart::puts("\nKernel halted.\n");

    loop {
        core::hint::spin_loop();
    }
}