# General guidelines

This document captures code conventions for the snapcrab project. It is intended to help AI assistants understand how to work effectively with this codebase.

Inspired by [nextest's AGENTS.md](https://github.com/nextest-rs/nextest/blob/main/AGENTS.md).

## For humans

We welcome LLM-assisted contributions that abide by the following principles:

* **Aim for excellence.** Use LLMs as a quality multiplier. Invest the time savings in improving rigor beyond what you'd do alone. Refactor for clarity. Tackle the tedious parts. Aim for zero bugs.
* **Review like a mentor.** Treat LLM output as you would code from someone you're mentoring. Read every line, question design decisions, and find ways to break it.
* **Write tests.** Use LLMs to produce thorough tests — edge cases, boundary conditions, failure modes. Tests are where AI assistance pays off the most.
* **Improve docs and errors.** Use LLMs to write clear documentation and actionable error messages. Users should never be left wondering what went wrong or what to do next.
* **Your code is your responsibility.** Do not submit a first draft. If your PR shows signs of not being reviewed, we may decline it outright.

## For LLMs

### Correctness over convenience

- Model the full error space. No shortcuts or simplified error handling.
- Handle edge cases, including platform differences and overflow conditions.
- Use the type system to encode correctness constraints.
- Prefer compile-time guarantees over runtime checks where possible.

### Production-grade engineering

- Test comprehensively, including edge cases and boundary conditions.
- Pay attention to what test facilities already exist and reuse them.
- Getting the details right is really important.

### Documentation

- Keep documentation concise. Provide relevant information without overwhelming the reader — don't repeat yourself.
- Doc comments (`///`) must start with a brief one-line summary. This first line is the title — keep it short and descriptive.
- Use inline comments to explain "why," not just "what."
- Don't add narrative comments in function bodies. Only comment what is non-obvious or needs a deeper "why" explanation.
  - Consider pulling the code to a separate function if it becomes too complex.
- Module-level documentation (`//!`) should explain purpose and responsibilities.
- Always use periods at the end of code comments.

## Code style

See [docs/src/developer.md](docs/src/developer.md) for full conventions.

### Import and path conventions

- **Types / Static**: Import and use unqualified unless there's a name collision.
- **Functions**: Use the parent module as qualifier, e.g., `check::validate_value()`.
- Do not write fully qualified paths inline. Import at the top of the module.
- Always import types at the very top of the module. Never import types within function bodies.

### Visibility

- Use `pub(crate)` and `pub(super)` to restrict visibility where possible.
- Only expose what downstream code actually needs.

### Error handling

- Use `anyhow` for error propagation in the interpreter.
- Provide context with `.context()` or `.with_context()` for actionable error messages.
- Error display messages should be lowercase sentence fragments.
- Use `bail!` for early returns with errors.

## Testing practices

We rely heavily on integration tests since most code depends on the compiler context. Unit tests are still important — use them wherever logic can be exercised without the compiler (e.g., value manipulation, binary operations, layout computations).

### Running tests

```bash
# Run all tests
cargo test --quiet

# Run integration tests only
cargo test --quiet --test integration_tests

# Run a specific test
cargo test --quiet --test integration_tests -- test_name

# Run ignored tests (work in progress)
cargo test --quiet --test integration_tests -- --ignored
```

### Test organization

- Integration tests live in `snapcrab/tests/integration_tests.rs` with submodules under `snapcrab/tests/integration_tests/`.
- Test input programs live in `snapcrab/tests/inputs/`, organized by category in subdirectories.
- Use the `check_custom_start!` macro to declare tests. Omit `result` for tests that assert internally.

### Writing test inputs

Test input functions should follow Rust test style: succeed silently (return `()`) if the interpreter is correct, and panic with a clear message if not.
Use `assert!`, `assert_eq!`, or `panic!` / `unreachable!` to signal failure.
The exception is tests specifically exercising panic handling.

Older tests return values and check byte representations in the macro invocation — this predates the panic machinery support. New tests should prefer the assert-and-panic style.

### Adding tests

1. Create or add to an input file under `tests/inputs/<category>/`.
2. Add a `check_custom_start!` entry in the appropriate test module.
3. Use `#[ignore]` attribute inside the macro for tests that aren't passing yet.

## Commit conventions

### Format

Follow [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/):

```
type(scope): brief description

Optional body with more detail.
```

- **Title**: Maximum 50 characters.
- **Body**: Maximum 80 characters per line.
- **Types**: feat, fix, docs, style, refactor, test, chore, perf, ci, build, revert.

### Quality

- **Atomic commits**: Each commit should be a logical unit of change.
- **Bisect-able history**: Every commit must build and pass all checks.
- **Separate concerns**: Format fixes and refactoring should be in separate commits from feature changes.

## Architecture overview

SnapCrab is a Rust interpreter operating at the MIR level. Key modules:

- **`interpreter/`**: Core execution engine.
  - `function.rs`: Three-tier function dispatch (MIR body, intrinsic shim, native call).
  - `rvalue.rs`: Expression evaluation including enum operations.
  - `place.rs`: Memory address resolution with projection handling.
  - `intrinsics.rs`: Compiler intrinsic shims.
  - `native.rs`: Native function calls via `dlsym`.
  - `check.rs`: Value validity checking.
- **`memory/`**: Memory model (stack, heap, statics).
- **`value.rs`**: Runtime value representation.
- **`ty.rs`**: Type system extensions.

## Quick reference

```bash
# Build
cargo build --quiet

# Check (fast, no codegen)
cargo check --quiet --tests

# Format
cargo fmt

# Run all tests
cargo test --quiet

# Run the interpreter
cargo run -- input.rs
cargo run -- --start-fn my_function --skip-check=validity input.rs
```

## Safety restrictions

- Do not push code to remote repositories.
- Do not publish comments or write anything non-local.
- Do not delete files or folders.
- Keep all operations local to the development environment.
- NEVER execute `git add` unless directly prompted.
- NEVER use `git add -A` — always specify files explicitly.
