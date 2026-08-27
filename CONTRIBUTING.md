# Contributing

## Requirements

- Rust toolchain (as pinned in `rust-toolchain.toml`)
- `rust-src` component
- `aarch64-unknown-none` target
- QEMU (`qemu-system-aarch64`)
- GDB-multiarch

Install the Rust toolchain, `rust-src`, and the target with `rustup`:

```sh
rustup component add rust-src
rustup target add aarch64-unknown-none
```

Install QEMU and GDB-multiarch with your package manager (e.g. `apt install qemu-system-arm gdb-multiarch`).

## Clone

```sh
git clone https://github.com/rabindra789/lych.git
cd lych
```

## Build

```sh
./scripts/lych build
```

## Run

```sh
./scripts/lych run
```

To stop QEMU, press `Ctrl+C`. The script uses `exec`, so QEMU is the actual process attached to the terminal and `Ctrl+C` terminates it directly.

## Debug

In one terminal, start QEMU paused with a GDB stub:

```sh
./scripts/lych debug
```

In another terminal, attach GDB:

```sh
./scripts/lych gdb
```