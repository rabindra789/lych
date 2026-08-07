/* Define the memory layout of the kernel */

ENTRY(_start)

SECTIONS
{
    /* Load the kernel at 0x40080000. */
    . = 0x40080000;

    .text.boot : ALIGN(4K)
    {
        KEEP(*(.text.boot))
    }

    .text : ALIGN(4K)
    {
        __text_start = .;
        *(.text*)
        __text_end = .;
    }

    .rodata : ALIGN(4K)
    {
        __rodata_start = .;
        *(.rodata*)
        __rodata_end = .;
    }

    .data : ALIGN(4K)
    {
        __data_start = .;
        *(.data*)
        __data_end = .;
    }

    .bss (NOLOAD) : ALIGN(4K)
    {
        __bss_start = .;
        *(.bss*)
        *(COMMON)
        __bss_end = .;
    }

    /* Reverse a 16 KiB boot stack. */
    . = ALIGN(16);
    BOOT_STACK_SIZE = 16K;
    . += BOOT_STACK_SIZE;

    __stack_top = .;
}