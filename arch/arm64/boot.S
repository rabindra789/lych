// First code executed by the CPU.

.section ".text.boot"
.global _start

.extern kernel_main
.extern __stack_top
.extern __bss_start
.extern __bss_end

_start:
    // Initialize the stack pointer
    ldr x0, =__stack_top
    mov sp, x0

    // Clear the .bss section.
    ldr x0, =__bss_start
    ldr x1, =__bss_end

1:
    cmp x0, x1
    b.ge 2f

    str xzr, [x0], #8
    b 1b

2:
    // Jump into Rust.
    bl kernel_main