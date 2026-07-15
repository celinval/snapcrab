# Native function calls

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

## Validation

Before calling native code, we validate all argument values against their
type's `valid_range` (scalar constraints from the layout). This catches
invalid bool, NonZero, enum discriminant, and pointer values before they
cross the boundary.

## Limitations

- `PassMode::Cast` not yet supported (requires `CastTarget` in `rustc_public`)
- `#[track_caller]` functions add an implicit `&Location` argument not
  visible in MIR — detected and reported as an error
- Float equality comparisons not supported in the interpreter (unrelated to
  native calls, but surfaces in tests)
- Symbol resolution is not cached (`dlsym` called on every invocation)

