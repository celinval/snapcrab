# Next Steps

This is a loose, prioritized TODO list of features we plan to add next. It is
not meant to be comprehensive — a lot of coverage will come as we try more
complex examples.


## Interpreter features

1. **`#[track_caller]` implicit argument support.** Pass the implicit
   `&Location` argument so `#[track_caller]` functions (and the panic
   machinery: `Option::unwrap`, `Result::unwrap`, indexing, …) can be called.
   Requires rustc `1.98.1`+, which exposes the info
   ([rust-lang/rust#159204](https://github.com/rust-lang/rust/pull/159204)), so
   bump `rust-toolchain.toml` first.
2. **`PassMode::Cast` in the native ABI.** Passing/returning small structs and
   arrays by value across the C ABI. The single largest bucket of ignored
   tests. Needs `CastTarget` exposed from rustc_public (see below).
3. **MIR constants with uninitialized bytes.** `evaluate_constant` uses the
   all-or-nothing `raw_bytes`, so a constant with any uninitialized byte (e.g.
   an `Option` in its `None` niche) fails. Copy the initialized bytes and zero
   the rest, mirroring the fix already done for static materialization. Unblocks
   `checked_*` returning `None` and is a prerequisite for `Vec`.
4. **Wrapping / saturating / overflowing arithmetic.** `wrapping_add/sub`,
   `saturating_add/sub`, and the `overflowing_*` family. Today `+`/`-` are
   evaluated as checked and error on overflow.
5. **`Rem` (`%`) and the remaining bit intrinsics** (`bswap`, `bitreverse`,
   rotate). Small, self-contained additions.
6. **`Assume` intrinsic statement** (`StatementKind::Intrinsic(Assume)`), used
   by e.g. `Result::map`/`and_then`.
7. **Float comparison** (`Eq`/ordering on `f32`/`f64`).
8. **`copy_nonoverlapping` statement**, which also unblocks SIMD tests.
9. **Smart-pointer unsizing coercion.** `Rc<T>`/`Arc<T>`/`Pin<&mut T> -> dyn`
   goes through the pointer's own `CoerceUnsized` impl (the data pointer lives
   in a field), which the coercion path does not yet handle.
10. **`Vec` and other collections.** Depends on (3) plus slice-iterator support
    (`ptr::offset`).
11. **Crate loading.**
12. **Threads.** Needs rustc_public support, then making the `Statics`/`Code`
    segments shareable across threads (they are currently single-threaded).

## Native calls

- `PassMode::Cast` support (requires `CastTarget` in rustc_public) — see (2).
- Symbol caching (avoid repeated `dlsym` lookups).
- Trampoline caching (reuse compiled trampolines for identical signatures).
- Function-pointer callbacks (native → interpreter via JIT stubs). A reified
  `fn` pointer maps to an `Instance` we interpret, but its synthetic address is
  not real machine code, so native code cannot call it until we JIT re-entry
  stubs.

## RustC Public

Improvements we would like in rustc_public:

1. Add `PlaceRef` to make processing a `Place` more efficient (would let the
   interpreter avoid recreating places).
2. Add a way to retrieve all mono items.
3. Add a way to retrieve the source for a span (for `annotate_snippets`).
4. Expose `CastTarget` details from `PassMode::Cast` (needed for struct
   pass/return in the C ABI) — see interpreter item (2).
