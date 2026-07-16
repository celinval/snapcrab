//! Native function call support via dynamic symbol resolution.
//!
//! When a function has no MIR body and is not an intrinsic we can shim,
//! we fall back to calling the native compiled version directly. This works
//! because:
//! - The interpreter's memory uses real process addresses (Box<[u8]> buffers)
//! - The std library linked into the compiler process uses the same ABI
//! - The same rustc produced both the MIR and the native code
//!
//! Symbol resolution uses `dlsym(RTLD_DEFAULT, ...)` which searches all
//! libraries loaded with `RTLD_GLOBAL`. This includes std (always present)
//! and any user-supplied libraries loaded via `--native-lib`.
//!
//! The actual call goes through a cranelift JIT'd trampoline (see [`jit`])
//! that flattens interpreter values into a typed byte buffer, passes them
//! to the target function, and writes the return value back to a buffer
//! the interpreter can read. Cranelift handles register allocation and
//! calling convention details.

pub mod jit;

use crate::interpreter::check::{CheckConfig, validate_value};
use crate::ty::{has_any_ptr_to_padded, has_mutable_ptr_to_padded, has_padding};
use crate::value::Value;
use anyhow::{Result, bail};
use rustc_public::abi::{FnAbi, PassMode};
use rustc_public::mir::mono::Instance;
use std::ffi::CString;
use tracing::{debug, trace};

/// Call a native function by resolving its mangled symbol name.
///
/// Validates argument values and checks call safety before invoking the
/// native function through the JIT trampoline.
pub fn call_native(
    instance: Instance,
    args: &[Value],
    config: &CheckConfig,
    jit: &jit::JitEngine,
) -> Result<Value> {
    let mangled = instance.mangled_name();
    let name = instance.name();
    debug!("Native call: {name}");

    let fn_abi = instance.fn_abi()?;
    trace_fn_abi(&name, &mangled, &fn_abi, args.len());

    // Validate arguments before passing to native code.
    for (arg_abi, arg_val) in fn_abi.args.iter().zip(args.iter()) {
        validate_value(arg_val, arg_abi.ty, config)?;
    }

    check_call_safety(&fn_abi, &name)?;

    // Resolve symbol from the current process (cached).
    let symbol_name = mangled.as_str();
    let fn_ptr = jit
        .resolve_symbol(symbol_name)
        .map_err(|e| anyhow::anyhow!("Failed to invoke `{name}`: {e}"))?;
    trace!("Resolved symbol `{symbol_name}` at {fn_ptr:?}");

    // SAFETY: check_call_safety rejected calls violating property 1 (mutable
    // pointers to padded types). Property 2 is upheld by construction (same
    // compiler produces both MIR and native code).
    let result = unsafe { jit.call_native(fn_ptr, &fn_abi, args, &name)? };
    debug!("Native call returned: {name}");
    Ok(result)
}

/// Bail if a native call may leave interpreter-visible memory uninitialized.
///
/// This happens when the function takes a mutable pointer/reference to a type
/// with padding bytes — the native code may write a value whose padding is
/// uninitialized, and the interpreter will later read those bytes.
///
/// Possible future improvements to remove this restrictions:
/// - Track all memory reachable across the interpreter/native boundary.
/// - Change Value to use MaybeUninit<u8> and treat padding as uninitialized.
/// - Sanitize values from native code by zeroing their padding bytes.
fn check_call_safety(fn_abi: &FnAbi, fn_name: &str) -> Result<()> {
    for arg_abi in fn_abi.args.iter() {
        if has_mutable_ptr_to_padded(arg_abi.ty) {
            bail!(
                "unsupported call: `{fn_name}`. The `{}` contains a mutable pointer to a \
                 type with padding — native code may leave uninitialized bytes",
                arg_abi.ty
            );
        }
    }

    // Indirect return: the callee writes the full struct including padding
    // bytes from its native stack over our pre-zeroed buffer. If the return
    // type has padding, those bytes are uninitialized and may propagate via
    // memcpy in the interpreter.
    if matches!(fn_abi.ret.mode, PassMode::Indirect { .. }) && has_padding(fn_abi.ret.ty) {
        bail!(
            "native call `{fn_name}` returns type `{}` with padding via indirect \
             return — callee may leave padding bytes uninitialized",
            fn_abi.ret.ty
        );
    }

    // If the return type contains any pointer to a padded type, the
    // interpreter may later read through it into native-allocated memory
    // with uninitialized padding (regardless of pointer mutability).
    if has_any_ptr_to_padded(fn_abi.ret.ty) {
        bail!(
            "native call `{fn_name}` returns type containing a pointer \
             to a type with padding — interpreter may read uninitialized bytes"
        );
    }

    Ok(())
}

/// Log detailed ABI information at trace level.
fn trace_fn_abi(
    name: &str,
    mangled: &str,
    fn_abi: &rustc_public::abi::FnAbi,
    interpreter_args: usize,
) {
    if !tracing::enabled!(tracing::Level::TRACE) {
        return;
    }
    trace!("  {name} (mangled: {mangled})");
    trace!(
        "  abi args: {}, interpreter args: {interpreter_args}",
        fn_abi.args.len()
    );
    for (i, arg_abi) in fn_abi.args.iter().enumerate() {
        trace!("  arg[{i}]: mode={:?}, ty={}", arg_abi.mode, arg_abi.ty);
    }
    trace!("  ret: mode={:?}, ty={}", fn_abi.ret.mode, fn_abi.ret.ty);
}

/// Look up a symbol in the current process without caching.
pub(crate) fn resolve_symbol(symbol: &str) -> Option<*const ()> {
    let Ok(c_name) = CString::new(symbol) else {
        return None;
    };
    // SAFETY: dlsym with RTLD_DEFAULT is a read-only lookup.
    let ptr = unsafe { libc::dlsym(libc::RTLD_DEFAULT, c_name.as_ptr()) };
    if ptr.is_null() {
        None
    } else {
        Some(ptr.cast())
    }
}
