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

## Three-tier dispatch

```
fn invoke_fn(instance, args) {
    1. if instance.has_body()  → interpret MIR
    2. if instance.intrinsic() → shim (assume, transmute, etc.)
    3. otherwise               → native call via dlsym
}
```

## Supported architectures

Currently, only **Unix x86-64** (the x86-64 System V ABI) is supported for native
calls. The inline assembly trampolines are platform-specific, and other
architectures (e.g., aarch64) would require equivalent implementations.

### x86-64 System V calling convention

This section summarizes the calling convention as defined by the
[System V Application Binary Interface — AMD64 Architecture Processor
Supplement][abi-amd64] (referenced normatively by the
[LSB Core Specification for AMD64][lsb-amd64-calling]).

[abi-amd64]: https://refspecs.linuxfoundation.org/elf/x86_64-abi-0.95.pdf
[lsb-amd64-calling]: https://refspecs.linuxfoundation.org/LSB_5.0.0/LSB-Core-AMD64/LSB-Core-AMD64.html#CALLINGSEQUENCE

#### Register usage

| Register | Purpose | Preserved across calls |
|----------|---------|----------------------|
| `%rax` | Return value (1st); SSE register count for varargs | No |
| `%rbx` | Callee-saved; optional base pointer | Yes |
| `%rcx` | 4th integer argument | No |
| `%rdx` | 3rd integer argument; 2nd return register | No |
| `%rsp` | Stack pointer | Yes |
| `%rbp` | Callee-saved; optional frame pointer | Yes |
| `%rsi` | 2nd integer argument | No |
| `%rdi` | 1st integer argument | No |
| `%r8` | 5th integer argument | No |
| `%r9` | 6th integer argument | No |
| `%r10` | Temporary; static chain pointer | No |
| `%r11` | Temporary | No |
| `%r12`–`%r15` | Callee-saved | Yes |
| `%xmm0`–`%xmm1` | Float arguments and return values | No |
| `%xmm2`–`%xmm7` | Float arguments | No |
| `%xmm8`–`%xmm15` | Temporaries | No |

#### Argument classification

Each argument is classified into one of the following classes, which determine
how it is passed:

- **INTEGER** — integral types and pointers that fit in a general-purpose
  register.
- **SSE** — `float`, `double`, and `__m64` types that fit in an SSE register.
- **SSEUP** — upper half of a type that spans two SSE eightbytes (e.g.,
  `__m128`).
- **X87 / X87UP** — `long double` (80-bit extended precision).
- **COMPLEX_X87** — `complex long double`.
- **MEMORY** — passed on the stack (types larger than 16 bytes, or unaligned
  aggregates).
- **NO_CLASS** — padding and empty fields (not passed).

For aggregates (structs, arrays, unions):

1. If the size exceeds two eightbytes (16 bytes), or the type contains unaligned
   fields, it is classified as MEMORY.
2. Otherwise each eightbyte is classified independently by merging field
   classes: INTEGER dominates over SSE, and MEMORY dominates over
   everything.
3. Post-merge cleanup: if any eightbyte is MEMORY, the whole argument
   becomes MEMORY. If SSEUP is not preceded by SSE, it is converted to
   SSE.

#### Parameter passing

Once classified, arguments are assigned left-to-right:

1. **MEMORY** — passed on the stack.
2. **INTEGER** — next available register from `%rdi`, `%rsi`, `%rdx`, `%rcx`,
   `%r8`, `%r9` (6 total).
3. **SSE** — next available register from `%xmm0`–`%xmm7` (8 total).
4. **SSEUP** — upper half of the last used SSE register.

If no register is available for any eightbyte of an argument, the *entire*
argument spills to the stack (previously assigned registers for that argument are
reverted). Stack arguments are pushed in right-to-left order and each slot is
eightbyte-aligned.

The integer and SSE register counters are independent — a function can use
all 6 integer *and* all 8 SSE register slots simultaneously.

#### Return values

1. **MEMORY** — caller passes a hidden pointer in `%rdi` (as if it were the
   first argument); `%rax` returns that pointer.
2. **INTEGER** — next available from `%rax`, `%rdx`.
3. **SSE** — next available from `%xmm0`, `%xmm1`.
4. **X87 / X87UP** — returned in `%st0`.
5. **COMPLEX_X87** — real in `%st0`, imaginary in `%st1`.

#### Stack frame

- The stack grows downward.
- The stack must be **16-byte aligned** before a `call` instruction (i.e.,
  `%rsp - 8` is 16-byte aligned at function entry due to the pushed return
  address).
- A **128-byte red zone** below `%rsp` is reserved for leaf functions and is
  not clobbered by signal handlers.

### How SnapCrab maps Rust ABI to System V

From `fn_abi()` we get each argument's `PassMode`:
- **Direct** → one register slot (int or float)
- **Pair** → two register slots (e.g., fat pointer = data ptr + len)
- **Indirect** → caller allocates, passes pointer in int register
- **Ignore** → ZST, not passed

### Spill behavior

When registers are exhausted, remaining arguments spill to the stack. The
behavior differs between `Pair` and multi-eightbyte `Direct` types:

- **`Pair` (e.g., `&str`, `&[T]`)**: each half is assigned independently.
  If only one register remains, the first half goes in that register and the
  second spills to the stack. The pair is *split* across register and stack.

- **Multi-eightbyte `Direct` (e.g., `u128`)**: treated as an indivisible
  unit. If both register slots aren't available, the *entire* value spills
  to the stack. Remaining registers may be used by subsequent arguments.

For example, with `fn(u64, u64, u64, u64, u64, u128, u64)`:
- Args 1–5 fill rdi–r8.
- `u128` needs 2 slots but only r9 is free → spills entirely to stack.
- Final `u64` takes r9 (the register skipped by u128).

This is all determined by the compiler and reflected in `fn_abi()`. Our code
simply follows what `PassMode` says — the spill decisions are already made.

## Implementation: inline assembly trampolines

We use `#[inline(never)]` Rust functions containing inline `asm!` that:
1. Load all int args into rdi/rsi/rdx/rcx/r8/r9
2. Load all float args into xmm0–xmm7
3. `call fn_ptr`
4. Capture the return value from rax/xmm0/etc.

Key insight: since the trampoline is a regular Rust function with compiler-
generated unwind info (`.cfi_startproc`/`.cfi_endproc`), panics in the callee
can unwind through it correctly. This was verified with standalone tests.

We have 4 trampoline variants based on return shape:
- `trampoline_void` — no return
- `trampoline_int` — rax
- `trampoline_float` — xmm0
- `trampoline_pair` — rax + rdx

## Why not libffi?

libffi inserts a C assembly trampoline frame that lacks Rust unwind info.
When a native function panics (e.g., `unwrap_failed`), the unwind can't
propagate through libffi's frame, causing hangs or undefined behavior in
`cargo test` (where output capture interacts with panic hooks).

## Why not `extern "C"` transmute?

Same ABI, but declaring the function as `extern "C"` makes unwinding through
it undefined behavior. `extern "C-unwind"` would fix this, but we'd still
need compile-time type combinatorics.

## Current status

- The inline asm trampoline generates correct assembly (verified via `--emit
  asm`) and works for non-`#[track_caller]` functions.
- **Blocker**: `#[track_caller]` functions (like `unwrap_failed`) receive an
  implicit `&Location` argument. The MIR-level call includes this argument,
  but it's not reaching the native call path correctly. Investigating whether
  our interpreter passes the location value through `args` or if it's being
  dropped somewhere.

## Validation

Before calling native code, we validate all argument values against their
type's `valid_range` (scalar constraints from the layout). This catches
invalid bool, NonZero, enum discriminant, and pointer values before they
cross the boundary.

## Next steps

1. **Debug `#[track_caller]` argument passing**: Add debug output to
   `call_native` to print the number of `fn_abi.args` vs `args.len()` for
   `unwrap_failed`. Determine whether the implicit `&Location` is in `args`
   from the interpreter or if it needs to be synthesized.
2. **Stack spill support**: For functions with >6 int or >8 float args,
   allocate stack space in the trampoline before calling.
3. **aarch64 support**: Write equivalent trampolines using aarch64 registers
   (x0–x7 for int, v0–v7 for float).
4. **Symbol caching**: Cache `dlsym` results in a `HashMap<Symbol, *const ()>`
   to avoid repeated linear searches.
5. **Remove libffi dependency**: Once the inline asm path is fully working,
   drop the `libffi` crate.

## Limitations

- x86-64 System V ABI only (see [Supported architectures](#supported-architectures))
- Max 6 integer + 8 float register args (stack spill not supported)
- `PassMode::Cast` not supported
- Native panics require proper unwind propagation (verified working)
