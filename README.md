# SnapCrab
Experimental Rust interpreter

An experimental Rust interpreter designed to speed up local development by executing Rust code without compilation and linking overhead.

## Requirements

- **Little-endian host machine** (e.g., x86-64, AArch64). SnapCrab cannot be built on big-endian hosts.
- **Matching target**: The interpreted code must target the same machine as the host. Cross-interpretation (e.g., interpreting 32-bit code on a 64-bit host) is not supported.
- **Native call support**: On x86-64 Unix, the interpreter can call pre-compiled functions (some std internals, `extern "C"` functions) that lack MIR bodies. On other architectures (e.g., AArch64), these calls produce a runtime error — most pure-Rust code still works since it has MIR bodies available.

## Goals

- Enable rapid testing and development iteration
- Execute small Rust programs directly from source
- Support unit tests (`#[test]` functions) without full compilation
- Target Linux x86-64 initially

As a developer of `rustc_public`, I plan to also use this experiment for:

- Identifying gaps in the `rustc_public` APIs
- Serve as a practical example of using `rustc_public` APIs for building Rust tooling

## Current Status

Early development phase. Starting with a limited subset of Rust syntax to evaluate project feasibility:
- Small binary programs
- Basic language constructs
- Future expansion to external dependencies and broader Rust feature support

## Target Use Case

Developers who want to quickly execute tests and small programs during development without waiting for the full Rust compilation pipeline.

## Contributing

We welcome contributions, including those assisted by AI coding agents.
AI agents are tools like any other — there's no need to disclose their use.
What matters is the quality of the final code.
Contributors are responsible for everything they submit, regardless of how it was produced.

See the [developer guide](https://celinval.github.io/snapcrab/developer.html) to get started, and [AGENTS.md](AGENTS.md) for AI-assisted contribution guidelines.
