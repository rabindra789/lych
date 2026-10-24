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
    b exception_sync_current_el_spx

    /* IRQ */
    .balign 0x80
    b exception_irq_current_el_spx

    /* FIQ */
    VECTOR_ENTRY

    /* SError */
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


/*
 * Out-of-line synchronous exception handler.
 *
 * The vector slot above only contains a branch so the full handler does not consume the 128-byte vector slot.
 */
.balign 4
exception_sync_current_el_spx:

    // Phase 1: Preserve context
    PUSH_PAIR x0,  x1
    PUSH_PAIR x2,  x3
    PUSH_PAIR x4,  x5
    PUSH_PAIR x6,  x7
    PUSH_PAIR x8,  x9
    PUSH_PAIR x10, x11
    PUSH_PAIR x12, x13
    PUSH_PAIR x14, x15
    PUSH_PAIR x16, x17
    PUSH_PAIR x18, x19
    PUSH_PAIR x20, x21
    PUSH_PAIR x22, x23
    PUSH_PAIR x24, x25
    PUSH_PAIR x26, x27
    PUSH_PAIR x28, x29

    str x30, [sp, #-8]!

    // Phase 2: Build exception frame
    sub sp, sp, #24

    mrs x1, esr_el1
    mrs x2, elr_el1
    mrs x3, spsr_el1

    str x1, [sp]
    str x2, [sp, #8]
    str x3, [sp, #16]

    // Phase 3: Enter Rust
    mov x0, sp
    bl exception_handler

    // Phase 4: Restore context
    ldr x1, [sp, #8]
    ldr x2, [sp, #16]

    msr elr_el1, x1
    msr spsr_el1, x2

    add sp, sp, #24

    ldr x30, [sp], #8

    POP_PAIR x28, x29
    POP_PAIR x26, x27
    POP_PAIR x24, x25
    POP_PAIR x22, x23
    POP_PAIR x20, x21
    POP_PAIR x18, x19
    POP_PAIR x16, x17
    POP_PAIR x14, x15
    POP_PAIR x12, x13
    POP_PAIR x10, x11
    POP_PAIR x8,  x9
    POP_PAIR x6,  x7
    POP_PAIR x4,  x5
    POP_PAIR x2,  x3
    POP_PAIR x0,  x1

    eret

/*
 * IRQ handler for the current EL using SP_ELx.
 *
 * Preserve the interrupted CPU context, call the Rust handler,
 * restore the context, and return from the exception.
 */
.balign 4
exception_irq_current_el_spx:
    // Save general-purpose registers.
    PUSH_PAIR x0,  x1
    PUSH_PAIR x2,  x3
    PUSH_PAIR x4,  x5
    PUSH_PAIR x6,  x7
    PUSH_PAIR x8,  x9
    PUSH_PAIR x10, x11
    PUSH_PAIR x12, x13
    PUSH_PAIR x14, x15
    PUSH_PAIR x16, x17
    PUSH_PAIR x18, x19
    PUSH_PAIR x20, x21
    PUSH_PAIR x22, x23
    PUSH_PAIR x24, x25
    PUSH_PAIR x26, x27
    PUSH_PAIR x28, x29

    // Reserve 16 bytes to preserve x30 and keep SP aligned.
    str x30, [sp, #-16]!

    // Preserve all SIMD/FP registers because an IRQ can interrupt
    // code that is using them.
    stp q0,  q1,  [sp, #-32]!
    stp q2,  q3,  [sp, #-32]!
    stp q4,  q5,  [sp, #-32]!
    stp q6,  q7,  [sp, #-32]!
    stp q8,  q9,  [sp, #-32]!
    stp q10, q11, [sp, #-32]!
    stp q12, q13, [sp, #-32]!
    stp q14, q15, [sp, #-32]!
    stp q16, q17, [sp, #-32]!
    stp q18, q19, [sp, #-32]!
    stp q20, q21, [sp, #-32]!
    stp q22, q23, [sp, #-32]!
    stp q24, q25, [sp, #-32]!
    stp q26, q27, [sp, #-32]!
    stp q28, q29, [sp, #-32]!
    stp q30, q31, [sp, #-32]!

    // Call the Rust IRQ handler.
    bl irq_handler

    // Restore SIMD/FP registers.
    ldp q30, q31, [sp], #32
    ldp q28, q29, [sp], #32
    ldp q26, q27, [sp], #32
    ldp q24, q25, [sp], #32
    ldp q22, q23, [sp], #32
    ldp q20, q21, [sp], #32
    ldp q18, q19, [sp], #32
    ldp q16, q17, [sp], #32
    ldp q14, q15, [sp], #32
    ldp q12, q13, [sp], #32
    ldp q10, q11, [sp], #32
    ldp q8,  q9,  [sp], #32
    ldp q6,  q7,  [sp], #32
    ldp q4,  q5,  [sp], #32
    ldp q2,  q3,  [sp], #32
    ldp q0,  q1,  [sp], #32

    // Restore general-purpose registers.
    ldr x30, [sp], #16

    POP_PAIR x28, x29
    POP_PAIR x26, x27
    POP_PAIR x24, x25
    POP_PAIR x22, x23
    POP_PAIR x20, x21
    POP_PAIR x18, x19
    POP_PAIR x16, x17
    POP_PAIR x14, x15
    POP_PAIR x12, x13
    POP_PAIR x10, x11
    POP_PAIR x8,  x9
    POP_PAIR x6,  x7
    POP_PAIR x4,  x5
    POP_PAIR x2,  x3
    POP_PAIR x0,  x1

    eret

exception_vectors_init:
    adr x0, vector_table
    msr vbar_el1, x0
    isb
    ret