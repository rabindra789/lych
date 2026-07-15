#![no_std]
#![no_main]

mod arch;
mod uart;

use core::panic::PanicInfo;

/// First Rust func to executed by the kernel
#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    uart::puts("Lych kernel\n");
    
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