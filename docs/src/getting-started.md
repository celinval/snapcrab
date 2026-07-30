# Getting Started

This guide will help you get started with SnapCrab for faster Rust development workflows.

## Installation

SnapCrab is currently in early development. To build from source:

```bash
git clone <repository-url>
cd snapcrab
cargo build --release
```

## Usage

SnapCrab is designed to execute small Rust programs and unit tests without compilation overhead.

### Standalone: a single source file

Interpret the `main` function of a Rust source file:

```bash
snapcrab run <file.rs>
```

Interpret a specific function by name (requires its fully qualified name):

```bash
snapcrab run --start-fn <function_name> <file.rs>
```

### Cargo projects: the cargo-snap driver

For a real cargo project (with dependencies), use the `cargo snap` subcommand,
which builds the crate and its dependencies with MIR encoded, then interprets:

```bash
# Interpret the crate's `main`.
cargo snap run

# Discover and interpret tests (test support is a work in progress).
cargo snap test --filter <substring>
```

## Requirements

- A **little-endian** host machine (e.g., x86-64, AArch64). SnapCrab will not compile on big-endian hosts.
- The interpreted code must target the **same machine** as the host (same endianness and pointer width). Cross-interpretation is not supported.

## Limitations

Current limitations in the early development phase:
- Limited subset of Rust syntax supported
- `cargo snap run` works; `cargo snap test` is still a work in progress
- Basic language constructs
- Little-endian host only; no cross-target interpretation

Future expansion will include external dependencies and broader Rust feature support.
