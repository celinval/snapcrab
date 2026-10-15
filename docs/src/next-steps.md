# Next Steps

This is a loose, prioritized TODO list of features we plan to add next. It is
not meant to be comprehensive — a lot of coverage will come as we try more
complex examples.


## Interpreter features

Roughly in priority order (top first):

- **`PassMode::Cast` in the native ABI.** Passing/returning small structs and
  arrays by value across the C ABI. The single largest bucket of ignored
  tests. Needs `CastTarget` exposed from rustc_public (see below).
- **Atomic intrinsics** (`atomic_load`/`atomic_store`/`atomic_xsub`, …). Needed
  by anything with an atomic refcount or an interior-mutable `static`: `Arc`
  coerces and dispatches correctly but cannot be dropped, and
  `AtomicU32::store` is unsupported.
- **Float comparison** (`Eq`/ordering on `f32`/`f64`).
- **Calling a `fn` pointer through the `Fn` traits.** The `FnOnce`/`FnMut`
  impls for `fn` pointers are shims without bodies, so passing a function
  (rather than a closure) where a closure is expected fails with an argument
  count mismatch — e.g. `iter().max()`, which calls `Ord::cmp` through one.
- **Other collections** (`String` growth beyond the basics, maps, sets).
  `Vec` works; the rest are untested.
- **Crate loading.**
- **Unwinding.** `UnwindAction` is ignored, so a panic ends interpretation
  without running cleanup and destructors do not run on the panic path.
- **`Assert` messages.** `AssertMessage::description` drops the operands, so a
  failed bounds check reports `index out of bounds` where rustc reports
  `index out of bounds: the len is 2 but the index is 5`.
- **Rendering formatted panic messages.** `panic!("{x}")`, `assert_eq!` and
  libcore's UB checks build a `core::fmt::Arguments`, and `Result::unwrap`
  formats the `Err` through `dyn Debug`. Rendering them needs `dyn Debug`
  dispatch through vtables whose slots hold our synthetic reified addresses.
- **Threads.** Needs rustc_public support, then making the `Statics`/`Code`
  segments shareable across threads (they are currently single-threaded).

## Native calls

- `PassMode::Cast` support (requires `CastTarget` in rustc_public) — see the
  `PassMode::Cast` interpreter item above.
- Symbol caching (avoid repeated `dlsym` lookups).
- Trampoline caching (reuse compiled trampolines for identical signatures).
- Function-pointer callbacks (native → interpreter via JIT stubs). A reified
  `fn` pointer maps to an `Instance` we interpret, but its synthetic address is
  not real machine code, so native code cannot call it until we JIT re-entry
  stubs.
- Handle uninitialized padding bytes written in native code. Possible changes:
  - Track all memory reachable across the interpreter/native boundary.
  - Change Value to use MaybeUninit<u8> and treat padding as uninitialized.
  - Sanitize values from native code by zeroing their padding bytes.

## RustC Public

Improvements we would like in rustc_public:

- Add `PlaceRef` to make processing a `Place` more efficient (would let the
  interpreter avoid recreating places).
- Add a way to retrieve all mono items.
- Add a way to retrieve the source for a span (for `annotate_snippets`).
- Expose `CastTarget` details from `PassMode::Cast` (needed for struct
  pass/return in the C ABI) — see the `PassMode::Cast` interpreter item.
- Build the caller location from the call terminator's `fn_span`, as
  `rustc_codegen_ssa` does, rather than from `source_info.span`. The two differ
  for a method call: rustc and Miri report `x.unwrap()` at the `unwrap`
  segment, while `Body::caller_location` reports the start of the receiver, so
  a panic's column does not match. `TerminatorKind::Call` would also need to
  expose `fn_span`.
