# Lych

[![CI](https://github.com/rabindra789/lych/actions/workflows/ci.yml/badge.svg)](https://github.com/rabindra789/lych/actions/workflows/ci.yml)

Lych is a monolithic operating system for ARM64, written in Rust from the ground up.

The project focuses on understanding and building every major part of an operating system from first principles instead of treating it as a black box. Every subsystem is implemented step by step with an emphasis on simplicity, maintainability, and clear documentation.

Lych is an open source project developed in India.

## Current

- Boots on QEMU `virt`
- Rust kernel (`no_std`, `no_main`)
- PL011 UART driver
- Exception vector table (VBAR_EL1)
- Synchronous exception handling
- Exception diagnostics
  - `ESR_EL1`
  - `ELR_EL1`
  - `SPSR_EL1`
  - Previous Exception Level
- Exception return with `ERET`
- Resume execution after `BRK`
- Kernel memory layout inspection
  - `.text`, `.rodata`, `.data`, `.bss`, stack, RAM boundaries
  - Usable RAM region computation
- Physical frame abstraction
  - Frame iteration over usable RAM
  - Bitmap-backed frame allocator
    - Bitmap sizing and reservation
    - Frame allocation
    - Frame deallocation
    - Frame reuse

## Roadmap

- Additional synchronous exceptions
- Interrupt handling (IRQ)
- Generic timer
- MMU
- Virtual memory
- Scheduler
- Userspace

## Targets

Current:

- QEMU `virt`

Planned:

- Raspberry Pi
- Generic ARM64 development boards
- ARM laptops
- ARM phones

## License

MIT