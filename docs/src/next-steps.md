# Next Steps

This is a loose, prioritized TODO list of features we plan to add next. It is
not meant to be comprehensive — a lot of coverage will come as we try more
complex examples.


## Interpreter features

1. **`PassMode::Cast` in the native ABI.** Passing/returning small structs and
   arrays by value across the C ABI. The single largest bucket of ignored
   tests. Needs `CastTarget` exposed from rustc_public (see below).
2. **MIR constants with uninitialized bytes.** `evaluate_constant` uses the
   all-or-nothing `raw_bytes`, so a constant with any uninitialized byte (e.g.
   an `Option` in its `None` niche) fails. Copy the initialized bytes and zero
   the rest, mirroring the fix already done for static materialization. Unblocks
   `checked_*` returning `None` and is a prerequisite for `Vec`.
3. **Wrapping / saturating / overflowing arithmetic.** `wrapping_add/sub`,
   `saturating_add/sub`, and the `overflowing_*` family. Today `+`/`-` are
   evaluated as checked and error on overflow.
4. **`Rem` (`%`) and the remaining bit intrinsics** (`bswap`, `bitreverse`,
   rotate). Small, self-contained additions.
5. **`Assume` intrinsic statement** (`StatementKind::Intrinsic(Assume)`), used
   by e.g. `Result::map`/`and_then`.
6. **Float comparison** (`Eq`/ordering on `f32`/`f64`).
7. **`copy_nonoverlapping` statement**, which also unblocks SIMD tests.
8. **Smart-pointer unsizing coercion.** `Rc<T>`/`Arc<T>`/`Pin<&mut T> -> dyn`
   goes through the pointer's own `CoerceUnsized` impl (the data pointer lives
   in a field), which the coercion path does not yet handle.
9. **`Vec` and other collections.** Depends on (2) plus slice-iterator support
   (`ptr::offset`).
10. **Crate loading.**
11. **`#[track_caller]` support (blocked on rustc 1.99).** Report the caller
    `Location` for `#[track_caller]` functions and the panic machinery
    (`Option::unwrap`, `Result::unwrap`, indexing, …). The pinned `1.98.1`
    exposes the implicit trailing `&Location` ABI argument, but not the API to
    resolve the location value. Clean support uses the public
    `Instance::requires_caller_location` and `Body::caller_location`
    ([rust-lang/rust#159204](https://github.com/rust-lang/rust/pull/159204)),
    which land in rustc `1.99`. Deferred until `rust-toolchain.toml` moves to
    `1.99`; `Body::caller_location` also handles inlining and propagation
    through nested `#[track_caller]` callers.
12. **Threads.** Needs rustc_public support, then making the `Statics`/`Code`
    segments shareable across threads (they are currently single-threaded).

## Native calls

- `PassMode::Cast` support (requires `CastTarget` in rustc_public) — see (1).
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
