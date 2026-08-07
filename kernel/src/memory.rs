unsafe extern "C" {
    static __text_start: u8;
    static __text_end: u8;

    static __rodata_start: u8;
    static __rodata_end: u8;

    static __data_start: u8;
    static __data_end: u8;

    static __bss_start: u8;
    static __bss_end: u8;

    static __stack_top: u8;
}

fn addr(symbol: &u8) -> u64 {
    symbol as *const u8 as u64
}

pub fn print_layout() {
    use crate::drivers::uart;

    unsafe {
        uart::puts("\nKernel Memory Layout\n");

        uart::puts(".text   : ");
        uart::put_hex(addr(&__text_start));
        uart::puts(" - ");
        uart::put_hex(addr(&__text_end));
        uart::putc(b'\n');

        uart::puts(".rodata : ");
        uart::put_hex(addr(&__rodata_start));
        uart::puts(" - ");
        uart::put_hex(addr(&__rodata_end));
        uart::putc(b'\n');

        uart::puts(".data   : ");
        uart::put_hex(addr(&__data_start));
        uart::puts(" - ");
        uart::put_hex(addr(&__data_end));
        uart::putc(b'\n');

        uart::puts(".bss    : ");
        uart::put_hex(addr(&__bss_start));
        uart::puts(" - ");
        uart::put_hex(addr(&__bss_end));
        uart::putc(b'\n');

        uart::puts("stack   : ");
        uart::put_hex(addr(&__stack_top));
        uart::putc(b'\n');
    }
}