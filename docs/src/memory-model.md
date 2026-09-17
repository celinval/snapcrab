# Memory model

This chapter describes how memory is modeled in Snapcrab.
It's worth noting that Rust doesn't have a formal memory model.

## Real addresses

Snapcrab interpreter uses real process addresses, which makes is possible
to interpret safe programs that may use pointer arithmetic, casts, and comparisons.
Every allocation the interpreted program sees is backed by an actual host
allocation, and the "pointer" the program manipulates is that allocation's real
address. Values are stored in host-native byte order and pointers are
host-sized, which is why the interpreted target must match the host's
endianness and pointer width (see [Architecture](./architecture.md)).

## 0-Byte Initialization

Snapcrab data is always initialized to zero bytes before use.
This simplifies some operations since data can always be trated as
a slice of bytes, avoiding extra overhead handling padding bytes,
and it also allow us to avoid UB even when the program may access
uninitialized memory. We may reconsider this decision in the future, since this
invariant cannot be easily upheld when crossing FFI boundary.

The only segment that does not follow this pattern is the Code Segment.
This segment holds an actual `Instance` object that is used to interpret
a function pointer call.

## Segments

Memory is split into segments, each owning its allocations and validating its
own accesses. A `ThreadMemory` structure holds them:

| Segment    | Holds                                             | Lifetime              |
|------------|---------------------------------------------------|-----------------------|
| **Stack**  | one contiguous frame per active call              | per call              |
| **Heap**   | the program's `Box`/`Vec`/… allocations           | until freed           |
| **Statics**| materialized globals: consts, statics, vtables    | whole interpreter run |
| **Code**   | reified functions (the `fn`-pointer registry)     | whole interpreter run |

Each segment carries a `MemorySanitizer` that records `address → size` for its
live allocations.
The sanitizer does **bounds checking only**, and does not track provenance or ownership.
The goal is speed and catching invalid memory accesses such as out-of-bounds or use-after-free.
A program that exhibit UB by violating Rust's aliasing model will also exhibit UB in the interpreter.
For thorough UB detection, use [Miri](https://github.com/rust-lang/miri/).

### Reads and writes

`ThreadMemory::read_addr` / `write_addr` are the entry points. A read:

1. returns the unit value immediately for a zero-sized type (no address touched);
2. checks the address is aligned for the type;
3. tries each data segment in turn — **stack → heap → statics** — and returns an
   owned `Value` copied out of the first that contains the range.

Reads return an *owned* `Value` (bytes copied while the segment's lock and
allocation are live) rather than a borrow, so a value can't dangle if the
underlying block is later freed. Distinct live allocations never overlap, so a
heap address simply misses the stack segment and is found in the heap.

### Aligned backing

A segment's backing store must honor the alignment of what it holds.
All memory segments that hold data use `AlignedBuf`, which is aligned
*by construction* rather than relying on whatever the system allocator happens
to return.
Stack frames are aligned to their largest local; static allocations to the allocation's declared alignment.

## Stack

Each call frame is a single `AlignedBuf` holding all locals in a contiguous block, with a table of per-local offsets.
This mirrors a real stack frame and avoids a separate allocation per local.
The frame is registered in the stack sanitizer for its duration and removed on return.

## Heap

Heap allocations, such as the ones done by `Box`, `Vec`, `String`, end in calls to allocator shims `__rust_alloc`, `__rust_alloc_zeroed`, `__rust_dealloc`, and `__rust_realloc`.
The interpreter intercepts those symbols and services them from the `Heap` segment instead of letting them reach a native `malloc` whose pointer it could not track.

Each allocation is zero-initialized, and kept alive in a map keyed by its base address and properly aligned.
Zero-sized allocations are rejected as UB (ZSTs use a dangling pointer
and never reach `__rust_alloc`). Slices whose `len * size_of::<T>()` would
exceed `isize::MAX` are also rejected.

> The interpreted program's own `#[global_allocator]` is **not** honored — all
> allocation goes through the host allocator via this interception.

## Statics

`Statics` materializes the compiler's global allocations
(`rustc_public::mir::alloc::GlobalAlloc`) into memory on first use:

- **Memory / Static** — const data and `static`s. Bytes are copied into an
  `AlignedBuf`; only the *initialized* bytes are copied, leaving padding zeroed.
- **Provenance** — pointer-sized holes that point at other allocations are
  patched with the resolved real addresses (recursively), so a `&str` constant's
  data pointer, a vtable's method slots, and so on all become valid addresses.
- **VTable** — resolved to its backing memory allocation (drop pointer, size,
  align, method pointers).
- **Function** — reified to a callable address in the Code segment (below).

## Code segment and function pointers

Interpreted functions have no real machine address, yet a program can turn a
function into a value: `f as fn()`, a vtable slot, a `const` fn pointer. The
`CodeSegment` is a registry that gives each function a **synthetic but real**
address:

- Reifying an `Instance` boxes a small `FunctionInfo { instance }` and returns
  its heap address. The map is keyed by `Instance`, so the same function always
  reifies to the same address — which makes `fn`-pointer equality and
  `as usize` behave. Because the address is a real `Box`, it never collides with
  data allocations.
- Resolving an address validates it against the code sanitizer (an exact,
  correctly-aligned base) and reads the `Instance` back.

The Code segment can not be accessed via normal read / write operations since
it is not modeled as raw data bytes. Instead, it offers its own resolution interface.

Rust guarantees no alignment for the code a `fn` pointer points at, so
`FunctionInfo` imposes none — a reified address is just where its `Box` lands.
Resolution still accepts only an exact base by checking the full `FunctionInfo`
size (an interior address would run past the one-function allocation).

## Limitations

- **No provenance or aliasing model** — bounds checking only; use Miri for UB.
- **Synthetic function addresses are not real machine addresses**, so a `fn`
  pointer or vtable entry handed to native code and *called there* will not work
  until re-entry stubs are JITted.
- **The program's `#[global_allocator]` is ignored**; allocation uses the host
  allocator.
- **Cross-target interpretation is unsupported** — host and target must share
  endianness and pointer width.
- **Fn pointers cannot cross thread boundaries:** Snapcrab doesn't yet support threads.
  We would need rustc_public support first, then we'd need to convert the CodeSegment
  into something that could be shared across threads.
