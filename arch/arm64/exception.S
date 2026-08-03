/*
 * Exception handling flow:
 *
 * 1. CPU enters the vector table.
 * 2. Save the interrupted CPU context.
 * 3. Read exception registers.
 * 4. Call the Rust exception handler.
 * 5. Restore the CPU context.
 * 6. Return with ERET.
 */

.section ".text"
.global vector_table
.global exception_vectors_init

.macro VECTOR_ENTRY
    .balign 0x80
    b .
.endm

.macro PUSH_PAIR r1, r2
    stp \r1, \r2, [sp, #-16]!
.endm

.macro POP_PAIR r1, r2
    ldp \r1, \r2, [sp], #16
.endm

.balign 0x800
vector_table:
    /* Same EL, using SP_EL0 */
    VECTOR_ENTRY
    VECTOR_ENTRY
    VECTOR_ENTRY
    VECTOR_ENTRY

    /* Same EL, using SP_ELx */
    .balign 0x80
    sync_current_el_spx:
        // Phase 1: Preserve context.
        PUSH_PAIR x0, x1
        PUSH_PAIR x2, x3

        // Build Exception Frame
        sub sp, sp, #16

        // Read exception registers.
        mrs x1, esr_el1
        mrs x2, elr_el1

        // Build ExceptionFrame on the stack.
        str x1, [sp]
        str x2, [sp, #8]

        // Enter Rust
        mov x0, sp
        bl exception_handler
    1:
        b 1b

    VECTOR_ENTRY
    VECTOR_ENTRY
    VECTOR_ENTRY

    /* Lower EL, AArch64 */
    VECTOR_ENTRY
    VECTOR_ENTRY
    VECTOR_ENTRY
    VECTOR_ENTRY

    /* Lower EL, AArch32 */
    VECTOR_ENTRY
    VECTOR_ENTRY
    VECTOR_ENTRY
    VECTOR_ENTRY

exception_vectors_init:
    adr x0, vector_table
    msr vbar_el1, x0
    isb
    ret