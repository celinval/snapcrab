# SnapCrab
Experimental Rust interpreter

An experimental Rust interpreter designed to speed up local development by executing Rust code without compilation and linking overhead.

## Goals

- Enable rapid testing and development iteration
- Execute small Rust programs directly from source
- Support unit tests (`#[test]` functions) without full compilation
- Target Linux x86-64 initially

## Current Status

Early development phase. Starting with a limited subset of Rust syntax to evaluate project feasibility:
- Small binary programs
- Basic language constructs
- Future expansion to external dependencies and broader Rust feature support

## Target Use Case

Developers who want to quickly execute tests and small programs during development without waiting for the full Rust compilation pipeline.
