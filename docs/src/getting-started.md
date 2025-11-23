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

### Running Functions

Execute a specific function by name:

```bash
cargo run -- function <function_name>
```

### Running Tests

Execute unit tests instantly:

```bash
cargo run -- test
```

## Limitations

Current limitations in the early development phase:
- Limited subset of Rust syntax supported
- Small binary programs only
- Basic language constructs
- Linux x86-64 target only

Future expansion will include external dependencies and broader Rust feature support.
