# Architecture

SnapCrab will consist of two main components that work together to provide fast Rust code interpretation:
the interpreter and the cargo driver.

## Interpreter

The interpreter will be a rustc wrapper that leverages `rustc_public` to interpret the target crate's MIR (Mid-level Intermediate Representation).

Key responsibilities:
- Parse and analyze Rust source code using rustc's frontend
- Generate MIR for the target crate
- Execute MIR instructions directly without code generation
- Provide runtime environment for interpreted execution
- Handle function calls, control flow, and memory operations
- Dynamically load libraries and invoke native code for cross-language interoperability
- Support potential JIT compilation strategies for performance optimization

By operating at the MIR level,
the interpreter will execute Rust code without the overhead of LLVM code generation and linking,
significantly reducing iteration time during development.
The ability to dynamically load libraries will enable seamless integration with existing native code,
while the foundation for JIT compilation will allow for performance improvements in hot code paths.

## Cargo Driver

The cargo driver will handle dependency management and compilation coordination.
It will compile the crate and its dependencies,
then trigger the interpreter for the target code.

Key responsibilities:
- Compile external dependencies using standard rustc
- Coordinate between compiled dependencies and interpreted target code
- Manage the build process and dependency resolution
- Interface between cargo's build system and the interpreter
- Handle mixed compilation scenarios (compiled deps + interpreted target)

This approach will allow SnapCrab to leverage the existing Rust ecosystem while providing fast execution for the code under development.
