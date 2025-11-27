# Welcome to SnapCrab

SnapCrab is an experimental Rust interpreter designed to accelerate local
development by executing Rust code without the overhead of compilation and linking.

Traditional Rust development requires a full compilation and linking cycle for every code change,
which can slow down the development process.
SnapCrab aims to solve this by interpreting Rust code directly,
enabling rapid iteration during development.

## Key Features

- **Fast execution**: Skip compilation and linking overhead
- **Test-focused**: Execute unit tests (`#[test]` functions) instantly
- **Development-oriented**: Optimized for quick feedback during coding
- **Linux x86-64 target**: Initial platform support

## Current Status

SnapCrab is in early development,
starting with a limited subset of Rust syntax to evaluate project feasibility.
The initial focus is on small binary programs and basic language constructs,
with plans to expand support for external dependencies and broader Rust features.

## Getting Started

This documentation will guide you through using SnapCrab for faster Rust development workflows.

Beyond providing a fast interpreter, SnapCrab serves as a tool to identify gaps in the `rustc_public` APIs and as a practical example demonstrating how to use these APIs for building Rust tooling. The project acts as a testbed for exploring compiler interface improvements.
