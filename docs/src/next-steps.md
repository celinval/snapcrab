# Next Steps

This is a loose TODO list with things we're planning to add next.
This is not meant to be a compreheensive list.
A lot of the coverage we will add as we try more complex examples.

- Reference handling
- Heap allocation
- Static objects
- Drop semantics
- ADTs
- Intrinsics
- DST

- Crate loading
- FFI support
- Standard library


## RustC Public

Here is a list of things that we can improve in rustc_public:

1. Add `PlaceRef`. Make it more efficient to process Place.
   - Once we do this, we can remove place creation in the interpreter
2. Add a way to retrieve all mono items
3. Add a way to retrieve source for a span so we can use annotate_snippets.
