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

### Running the Main Function

Execute the main function of a Rust source file:

```bash
snapcrab <file.rs>
```

### Running a Specific Function

Execute a specific function by name (requires fully qualified name):

```bash
snapcrab --start-fn <function_name> <file.rs>
```

## Limitations

Current limitations in the early development phase:
- Limited subset of Rust syntax supported
- Small binary programs only / no cargo support yet
- Basic language constructs
- Linux x86-64 target only

Future expansion will include external dependencies and broader Rust feature support.
