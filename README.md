<p align="center">
  <img src="assets/lych-logo.svg" alt="Lych OS Logo" width="140">
</p>

<h1 align="center">Lych</h1>

<p align="center">
  A monolithic ARM64 operating system written in Rust.
</p>

[![CI](https://github.com/rabindra789/lych/actions/workflows/ci.yml/badge.svg)](https://github.com/rabindra789/lych/actions/workflows/ci.yml)
[![License](https://img.shields.io/github/license/rabindra789/lych)](https://github.com/rabindra789/lych/blob/main/LICENSE)
[![Rust](https://img.shields.io/badge/language-Rust-orange?logo=rust)](https://www.rust-lang.org/)
[![Architecture](https://img.shields.io/badge/arch-ARM64-blue)](https://github.com/rabindra789/lych)

Lych is an open-source ARM64 operating system written in Rust. This repository contains the kernel, the platform bootstrap code, and the tooling and documentation needed to build, run, and debug the system.

The kernel is developed in small, self-contained pieces: boot, exception handling, memory management, and later timing, processes, userspace, and drivers. Each piece belongs to a documented phase and is kept small enough to follow and structured enough to extend.

The kernel currently boots on QEMU `virt` with the EL1 MMU enabled and an identity mapping of RAM.

## Getting Started

### Prerequisites

- A stable Rust toolchain (as pinned in `rust-toolchain.toml`), with the `rust-src` component and the `aarch64-unknown-none` target.
- QEMU with an AArch64 system emulator (`qemu-system-aarch64`).
- `gdb-multiarch` for debugging.

Install the Rust prerequisites:

```sh
rustup component add rust-src
rustup target add aarch64-unknown-none
```

Install QEMU and GDB through your system package manager, e.g.:

```sh
apt install qemu-system-arm gdb-multiarch
```

> **Windows:** use the `./scripts/lych` commands from Git Bash, or `.\scripts\lych.ps1` from PowerShell. The two scripts expose the same commands.

### Build

```sh
./scripts/lych build
```

This produces `target/aarch64-unknown-none/release/kernel`, an ELF binary suitable for QEMU's `-kernel` option.

### Run

```sh
./scripts/lych run
```

QEMU starts under the `virt` machine model with the CPU set to `cortex-a72`, and the serial console is connected to the terminal. Stop it with `Ctrl+C`.

### Debug

Terminal 1:

```sh
./scripts/lych debug
```

Terminal 2:

```sh
./scripts/lych gdb
```

The first starts QEMU paused with a GDB stub on `tcp::1234`; the second attaches `gdb-multiarch` to it.

### Format

```sh
./scripts/lych fmt
```

Check formatting without changing files:

```sh
./scripts/lych fmt-check
```

### Clean

```sh
./scripts/lych clean
```

All development flows go through the scripts in `scripts/`: `./scripts/lych` (Git Bash, Linux, macOS) or `.\scripts\lych.ps1` (Windows PowerShell). The QEMU arguments, the CPU model, and the debug options are kept in these two files so they change in only one place when the platform or workflow changes.

## Repository Layout

| Path                  | Purpose                                            |
|-----------------------|----------------------------------------------------|
| `arch/arm64/`         | Startup assembly and exception-entry stubs        |
| `boot/`               | Kernel linker script                               |
| `kernel/`             | The kernel crate (`no_std`, `no_main`)            |
| `kernel/src/arch/`    | CPU and exception-handling primitives             |
| `kernel/src/drivers/` | Device drivers (currently the PL011 UART)         |
| `kernel/src/memory/`  | Memory management (frame allocator, page tables)  |
| `scripts/`            | Build, run, and debug entry points                |
| `docs/`               | Development notes                                  |

## Current Status

The kernel boots on QEMU `virt` and currently provides:

- ARM64 boot code in `arch/arm64/boot.S`
- Rust kernel (`no_std`, `no_main`)
- PL011 UART driver with early boot output
- Exception vector table (`VBAR_EL1`)
- Synchronous exception handling
- Exception diagnostics: `ESR_EL1`, `ELR_EL1`, `SPSR_EL1`, previous exception level
- Exception return via `ERET`
- Resume execution after `BRK`
- Kernel memory layout inspection (`.text`, `.rodata`, `.data`, `.bss`, stack, RAM)
- Usable RAM region computation
- Physical frame abstraction
- Bitmap-backed frame allocator (allocation, deallocation, reuse)
- Page-table hierarchy (L0-L3) for the EL1 stage-1 translation
- Identity mapping of RAM on QEMU `virt`
- UART device mapping at `0x0900_0000`
- Memory attributes configured in `MAIR_EL1`
- Translation control configured in `TCR_EL1` (4 KB granule, 48-bit virtual, 40-bit physical address space)
- Page-table root installed in `TTBR0_EL1`
- EL1 MMU enabled (`SCTLR_EL1.M`)

## Roadmap

Development is organized into phases. Each phase is implemented and documented before the next begins.

### Phase 1: Boot and Exceptions
- Complete

### Phase 2: Memory
- Kernel memory layout (done)
- Physical frame allocator (done)
- Page tables and identity mapping (done)
- EL1 MMU enabled on QEMU `virt` (done)
- Heap (pending)

### Phase 3: Time and Interrupts
- Generic timer
- IRQ
- Tick counter
- Sleep

### Phase 4: Processes
- CPU context
- Context switching
- Scheduler
- Idle task
- Multiple kernel threads

### Phase 5: Virtual Memory
- User address spaces
- Page faults
- Copy-on-write
- Memory protection

### Phase 6: Userspace
- EL0
- System calls
- Process loader
- ELF loader
- User applications

### Phase 7: Drivers
- Framebuffer
- Keyboard
- Storage
- Filesystem

### Phase 8: Networking
- Ethernet
- TCP/IP
- DHCP
- Ping
- HTTP

### Phase 9: Multiprocessor
- Secondary cores
- Spinlocks
- SMP scheduler

## Platforms

Current:

- QEMU `virt` (arm64)

Planned:

- Raspberry Pi
- Generic ARM64 development boards
- ARM laptops
- ARM phones

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for environment setup, build, and debugging instructions. The roadmap above lists the areas currently in scope.

## License

Lych is released under the MIT License. See [LICENSE](LICENSE).