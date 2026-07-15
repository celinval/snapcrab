# Next Steps

This is a loose TODO list with things we're planning to add next.
This is not meant to be a compreheensive list.
A lot of the coverage we will add as we try more complex examples.

- Reference handling
- Heap allocation
- Drop semantics
- ADTs
- Intrinsics
- DST (partial — slices work, `dyn Trait` not yet supported)
- Crate loading


## Native calls

- `PassMode::Cast` support (requires `CastTarget` in rustc_public)
- Symbol caching (avoid repeated `dlsym` lookups)
- Trampoline caching (reuse compiled trampolines for identical signatures)
- Function pointer callbacks (native → interpreter via JIT stubs)
- `#[track_caller]` implicit argument support

## RustC Public

Here is a list of things that we can improve in rustc_public:

1. Add `PlaceRef`. Make it more efficient to process Place.
   - Once we do this, we can remove place creation in the interpreter
2. Add a way to retrieve all mono items
3. Add a way to retrieve source for a span so we can use annotate_snippets.
4. Expose `CastTarget` details from `PassMode::Cast` (needed for struct
   pass/return in C ABI)
5. Expose `#[track_caller]` implicit argument info
   (see https://github.com/rust-lang/rust/pull/159204)
