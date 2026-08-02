# Lych

Lych is a monolithic operating system for ARM64, written in Rust from the ground up.

The project is focused on understanding and building every major part of an operating system instead of treating it as a black box. Every subsystem is implemented step by step, with the goal of creating a clean, maintainable, and well-documented codebase.

Lych is being developed in India as an open source project.

## Current

- Boots on QEMU `virt`
- Rust kernel (`no_std`, `no_main`)
- PL011 UART driver
- Exception vector table
- Basic exception handling
- Exception diagnostics (`ESR_EL1`, `ELR_EL1`)

## Roadmap

- Complete exception handling
- Memory management
- MMU
- Timer and interrupts
- Scheduler
- Virtual memory
- Userspace

## Targets

Current:
- QEMU `virt`

Future:
- Raspberry Pi
- Generic ARM64 boards
- ARM laptops
- ARM phones

## License

MIT