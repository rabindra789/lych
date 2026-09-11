# Contributing

Thanks for your interest in contributing to Lych. This page describes how to set up a development environment, build the kernel, and run it under QEMU.

## Prerequisites

- A stable Rust toolchain as pinned in `rust-toolchain.toml`
- The `rust-src` component and the `aarch64-unknown-none` target
- QEMU (`qemu-system-aarch64`)
- `gdb-multiarch`

Install the Rust toolchain, `rust-src`, and the target:

```sh
rustup component add rust-src
rustup target add aarch64-unknown-none
```

Install QEMU and GDB with your system package manager:

```sh
apt install qemu-system-arm gdb-multiarch
```

> **Windows:** use Git Bash and the `./scripts/lych` commands below, or PowerShell with `.\scripts\lych.ps1` instead. The two scripts expose the same commands.

## Getting the Source

```sh
git clone https://github.com/rabindra789/lych.git
cd lych
```

## Building

```sh
./scripts/lych build
```

## Running

```sh
./scripts/lych run
```

QEMU attaches the serial console to the terminal. Stop it with `Ctrl+C` — the script `exec`s QEMU, so `Ctrl+C` stops the process directly.

## Debugging

In one terminal, start QEMU paused with a GDB stub:

```sh
./scripts/lych debug
```

In another terminal, attach GDB:

```sh
./scripts/lych gdb
```

## Guidelines

- Format code with `cargo fmt` before committing.
- The kernel is `#![no_std]` and `#![no_main]`. Keep it dependency-free unless a dependency is unavoidable.
- Restrict `unsafe` to the minimum required for the hardware access in question, and comment the safety justification.
- Follow the phase structure in the README: subsystem changes should be scoped, self-contained, and documented.