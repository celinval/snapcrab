# Native function calls

> **Status: Work in progress.** This feature is very experimental and has known
> safety limitations. Yes, snapcrab is experimental, which means this is super unstable and **potentially unsafe* even if the interpreted code is safe. See [Safety](#safety) below.

## Overview

When the interpreter encounters a function without a MIR body (and it's not a
shimmed intrinsic), it falls back to calling the native compiled version
directly from the current process.

This works because:
- The interpreter's memory uses real process addresses (stack frames are
  `Box<[u8]>` whose heap pointers are the actual addresses used by the
  interpreter).
- The std library linked into the compiler process was compiled by the same
  rustc, so the ABI matches exactly.
- We resolve the function's mangled symbol via `dlsym(RTLD_DEFAULT, ...)`.

User-supplied native libraries (`.so` files) can be loaded via the
`--native-lib` flag. They are opened with `dlopen(RTLD_NOW | RTLD_GLOBAL)`
so their symbols become visible to `RTLD_DEFAULT` lookups.

## Three-tier dispatch

```
fn invoke_fn(instance, args) {
    1. if instance.has_body()  → interpret MIR
    2. if instance.intrinsic() → shim (assume, transmute, etc.)
    3. otherwise               → native call via dlsym
}
```

## Implementation: cranelift JIT trampolines

We use [cranelift](https://cranelift.dev/) to generate trampolines at runtime.
Each trampoline has a fixed signature:

```text
extern "C" fn(fn_ptr: *const (), args_buf: *const u8, ret_buf: *mut MaybeUninit<u8>)
```

The trampoline body:
1. Loads typed arguments from `args_buf` at their recorded offsets.
2. Calls `fn_ptr` with those arguments using the target's calling convention.
3. Stores the return value(s) into `ret_buf`.

Cranelift handles register allocation, calling convention details, and unwind
info generation. We only need to describe the function signature (argument
types and return types) using cranelift IR types derived from `fn_abi()`.

### Why cranelift?

- **Platform-independent**: no hand-written assembly per architecture.
- **Correct unwind info**: cranelift generates proper `.eh_frame` entries, so
  panics in native code can unwind through the trampoline correctly.
- **No register manipulation**: we describe the signature declaratively and
  cranelift handles the rest.

### JitEngine

The `JitEngine` struct (in `interpreter/native/jit.rs`) wraps a cranelift
`JITModule` in `Arc<Mutex<>>` so it can be shared across threads when
`ThreadMemory` is cloned. It compiles a new trampoline for each unique
function signature encountered.

The lock is released before invoking the trampoline to avoid holding it
during the native call (which could re-enter the interpreter).

## PassMode handling

From `fn_abi()` we get each argument's `PassMode`, which determines how
values are passed to the trampoline:

| PassMode | Argument handling | Return handling |
|----------|------------------|-----------------|
| **Ignore** | Not passed (ZST) | No return value |
| **Direct** | Single typed value (Scalar or Vector) | Single register |
| **Pair** | Two scalars from ScalarPair layout | Two registers |
| **Indirect** | Pointer to the value's bytes | Hidden first-arg pointer |
| **Cast** | *(not yet supported)* | *(not yet supported)* |

### Cast (TODO)

`PassMode::Cast` is used by the C ABI for small aggregates (structs, arrays)
that fit in registers. The `CastTarget` (currently `Opaque` in `rustc_public`)
describes how to split the struct bytes into register-sized pieces and whether
each piece is INTEGER or SSE class.

Until `rustc_public` exposes `CastTarget` details, Cast is not supported.
This affects:
- Passing `#[repr(C)]` structs by value
- Returning small structs from `extern "C"` functions
- Passing arrays by value

### Limitations of the `rustc_public` ABI API

The `FnAbi`/`PassMode` information exposed by `rustc_public` may not be
sufficient for building fully correct native call sequences without LLVM.
As discussed in [rust-lang/rust#159359](https://github.com/rust-lang/rust/pull/159359),
`PassMode::Direct` with `BackendRepr::Scalar` does not guarantee the value
is actually passed directly — LLVM's backend can interpret IR patterns
differently depending on the target, argument position, and other factors.

The internal ABI representations were originally designed for a single-backend
context (LLVM) and are essentially "ABI modulo LLVM." Building correct call
sequences without LLVM would require reimplementing LLVM's exact ABI lowering
logic — which is what cranelift does independently for its own targets.

In practice, SnapCrab works correctly for the common cases (scalars, pairs,
indirect) because these map straightforwardly to cranelift IR. The risk is
in edge cases where LLVM and cranelift disagree on ABI lowering for the same
`PassMode` description.

## Validation

Before calling native code, we validate all argument values against their
type's `valid_range` (scalar constraints from the layout). This catches
invalid bool, NonZero, enum discriminant, and pointer values before they
cross the boundary.

Additionally, `check_call_safety` rejects calls that could leave
interpreter-visible memory uninitialized:

- Arguments containing a mutable pointer (`&mut T` or `*mut T`) where `T`
  has padding bytes — native code may write a struct with uninitialized
  padding through the pointer.
- Indirect returns where the return type has padding — the callee writes
  the full struct (including padding from its native stack) into the
  interpreter's buffer.
- Return types containing any pointer (mutable or const) to a padded
  type — native code may return a pointer to memory it allocated with
  uninitialized padding, and the interpreter would read through it.

The check traverses types recursively through struct fields, tuples, arrays,
and pointer indirections to find mutable pointers to padded types anywhere
in the argument or return type.

## Safety

Native calls are inherently unsafe. The interpreter currently assumes all
memory buffers are fully initialized, but native code can write uninitialized
bytes (e.g., struct padding) into any buffer it has access to. When the
interpreter subsequently reads those bytes, this is technically undefined
behavior.

The `check_call_safety` check mitigates this by rejecting calls where we can
statically determine that uninitialized padding may be introduced.

Possible future improvements:
- Track all memory reachable across the interpreter/native boundary.
- Change `Value` to use `MaybeUninit<u8>` and treat padding as uninitialized.
- Sanitize values from native code by zeroing their padding bytes.
- Handle static objects shared across the interpreter/native boundary.

## Limitations

- `PassMode::Cast` not yet supported (requires `CastTarget` in `rustc_public`)
- `#[track_caller]` functions add an implicit `&Location` argument not
  visible in MIR — detected and reported as an error
- Native calls with mutable pointers to padded types are rejected
- Native calls returning pointers to padded types are rejected
  (see [Validation](#validation))
