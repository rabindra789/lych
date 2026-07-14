// First code executed by the CPU.

.section ".text.boot"
.global _start

.extern kernel_main
.extern __stack_top

_start:
    // Initialize the stack pointer
    ldr x0, =__stack_top
    mov sp, x0

    // Jump into Rust
    bl kernel_main

1:
    wfe
    b 1b