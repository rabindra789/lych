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
        *(.text*)
    }

    .rodata : ALIGN(4K)
    {
        *(.rodata*)
    }

    .data : ALIGN(4K)
    {
        *(.data*)
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