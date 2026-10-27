
/* Primary and secondary CPU entry points. */

.section ".text.boot"
.global _start
.global secondary_entry

.extern kernel_main
.extern secondary_rust_entry
.extern __boot_stack_top
.extern __secondary_stacks_bottom
.extern __bss_start
.extern __bss_end

_start:
    /* Only the primary CPU initializes the boot stack and BSS. */
    ldr x0, =__boot_stack_top
    mov sp, x0

    ldr x0, =__bss_start
    ldr x1, =__bss_end

1:
    cmp x0, x1
    b.hs 2f
    str xzr, [x0], #8
    b 1b

2:
    bl kernel_main

3:
    wfe
    b 3b

/*
 * PSCI starts a secondary CPU here.
 * x0 contains the logical CPU index passed as context_id.
 * The secondary must never clear BSS or reuse the primary stack.
 */
.balign 16
secondary_entry:
    mov x9, x0

    /* Select the stack belonging to this CPU index. */
    ldr x1, =SECONDARY_STACK_SIZE
    ldr x2, =__secondary_stacks_bottom
    madd x2, x9, x1, x2
    add x2, x2, x1
    mov sp, x2

    /* Pass the CPU index to the Rust secondary entry. */
    mov x0, x9
    bl secondary_rust_entry

4:
    wfe
    b 4b
