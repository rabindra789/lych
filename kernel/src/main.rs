#![no_std]
#![no_main]

extern crate alloc;

mod arch;
mod drivers;
mod memory;
mod platform;

use core::panic::PanicInfo;
use drivers::uart;

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

    arch::cpu::init();

    let timer_frequency = arch::timer::frequency();
    let start = arch::timer::counter();

    for _ in 0..100_000 {
        core::hint::spin_loop();
    }

    let end = arch::timer::counter();

    uart::puts("Timer frequency: ");
    uart::put_hex(timer_frequency);
    uart::putc(b'\n');

    uart::puts("Timer start: ");
    uart::put_hex(start);
    uart::putc(b'\n');

    uart::puts("Timer end: ");
    uart::put_hex(end);
    uart::putc(b'\n');

    assert!(timer_frequency > 0);
    assert!(end > start);

    memory::init();
    memory::init_heap();
    memory::mmu::install_page_table_root();
    arch::cpu::enable_mmu();

    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn exception_handler(frame: &mut ExceptionFrame) {
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

    match ec {
        arch::exception::EC_INSTRUCTION_ABORT_LOWER_EL
        | arch::exception::EC_INSTRUCTION_ABORT_SAME_EL => {
            uart::puts("FAR_EL1   : ");
            uart::put_hex(arch::cpu::read_far_el1());
            uart::putc(b'\n');

            uart::puts("IFSC      : ");
            uart::put_hex(frame.esr & 0x3f);
            uart::putc(b'\n');

            uart::puts("Fault     : ");
            uart::puts(arch::exception::fault_status_name(frame.esr));
            uart::putc(b'\n');
        }

        arch::exception::EC_DATA_ABORT_LOWER_EL | arch::exception::EC_DATA_ABORT_SAME_EL => {
            uart::puts("FAR_EL1   : ");
            uart::put_hex(arch::cpu::read_far_el1());
            uart::putc(b'\n');

            uart::puts("DFSC      : ");
            uart::put_hex(frame.esr & 0x3f);
            uart::putc(b'\n');

            uart::puts("Fault     : ");
            uart::puts(arch::exception::fault_status_name(frame.esr));
            uart::putc(b'\n');
        }

        _ => {}
    }

    if ec == arch::exception::EC_BREAKPOINT {
        // Skip over the BRK instruction when returning.
        // ARM64 instructions are fixed-width (4 bytes).
        frame.elr += 4;
    }
}
