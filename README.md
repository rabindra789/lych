<p align="center">
  <img src="assets/lych-logo.svg" alt="Lych OS Logo" width="140">
</p>

<h1 align="center">Lych</h1>

<p align="center">
  Lych is a monolithic operating system for ARM64, written in Rust from the ground up.
</p>

[![CI](https://github.com/rabindra789/lych/actions/workflows/ci.yml/badge.svg)](https://github.com/rabindra789/lych/actions/workflows/ci.yml)
[![License](https://img.shields.io/github/license/rabindra789/lych)](https://github.com/rabindra789/lych/blob/main/LICENSE)
[![Rust](https://img.shields.io/badge/language-Rust-orange?logo=rust)](https://www.rust-lang.org/)
[![Architecture](https://img.shields.io/badge/arch-ARM64-blue)](https://github.com/rabindra789/lych)

The project focuses on understanding and building every major part of an operating system from first principles instead of treating it as a black box. Every subsystem is implemented step by step with an emphasis on simplicity, maintainability, and clear documentation.

Lych is an open source project developed in India.

## Quick Start

### Requirements

- Rust
- QEMU AArch64
- `gdb-multiarch` for debugging

### Build

```bash
./scripts/lych build
```

### Run

```bash
./scripts/lych run
```

Stop QEMU with `Ctrl+C`.

### Debug

Terminal 1:

```bash
./scripts/lych debug
```

Terminal 2:

```bash
./scripts/lych gdb
```

### Clean

```bash
./scripts/lych clean
```

All development flows go through `./scripts/lych` instead of raw `qemu-system-aarch64` / `gdb-multiarch` commands. When QEMU arguments, the CPU model, debug options, or kernel image location change, only the script needs updating and everyone's workflow stays consistent.

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

### Phase 1 — Boot & Exceptions
- Complete

### Phase 2 — Memory
- Kernel memory
- Physical memory
- MMU
- Heap

### Phase 3 — Time & Interrupts
- Generic timer
- IRQ
- Tick counter
- Sleep

### Phase 4 — Processes
- CPU context
- Context switching
- Scheduler
- Idle task
- Multiple kernel threads

### Phase 5 — Virtual Memory
- User address spaces
- Page faults
- Copy-on-write
- Memory protection

### Phase 6 — Userspace
- EL0
- System calls
- Process loader
- ELF loader
- User applications

### Phase 7 — Drivers
- UART
- Framebuffer
- Keyboard
- Storage
- Filesystem

### Phase 8 — Networking
- Ethernet
- TCP/IP
- DHCP
- Ping
- HTTP

### Phase 9 — Multiprocessor
- Secondary cores
- Spinlocks
- SMP scheduler

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