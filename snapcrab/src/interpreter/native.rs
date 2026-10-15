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
use rustc_public::ty::{RigidTy, TyKind};
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

    check_call_safety(&fn_abi, args.len(), &name)?;

    // Validate arguments before passing to native code.
    for (arg_abi, arg_val) in fn_abi.args.iter().zip(args.iter()) {
        validate_value(arg_val, arg_abi.ty, config)?;
    }

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

/// Bail if a native call is unsupported
///
/// 1. Check if it may leave interpreter-visible memory uninitialized.
///    This happens when the function takes a mutable pointer/reference to a type
///    with padding bytes — the native code may write a value whose padding is
///    uninitialized, and the interpreter will later read those bytes.
/// 2. A function that never returns can only leave by panicking or aborting.
///    Either way it would kill the process because unwind or abort inside the JIT
///    trampoline, which has no unwind information.
/// 3. Implicit argument: This is a defensive check in case new implicit arguments
///    are added to rust-call abi.
fn check_call_safety(fn_abi: &FnAbi, arg_count: usize, fn_name: &str) -> Result<()> {
    if matches!(fn_abi.ret.ty.kind(), TyKind::RigidTy(RigidTy::Never)) {
        bail!(
            "unsupported native call: `{fn_name}` never returns, so calling it would \
             panic or abort without unwinding back into the interpreter"
        );
    }

    // An argument the caller did not supply. The `#[track_caller]` `Location`
    // is appended at the call site, so reaching this means some other kind of
    // implicit argument.
    if fn_abi.args.len() > arg_count {
        bail!(
            "unsupported native call: `{fn_name}` expects {} arguments but got {} \
             (unsupported implicit argument)",
            fn_abi.args.len(),
            arg_count
        );
    }

    for arg_abi in fn_abi.args.iter() {
        if has_mutable_ptr_to_padded(arg_abi.ty) {
            bail!(
                "unsupported native call: `{fn_name}`. The `{}` contains a mutable pointer to a \
                 type with padding. Native code may write uninitialized bytes",
                arg_abi.ty
            );
        }
    }

    if matches!(fn_abi.ret.mode, PassMode::Indirect { .. }) && has_padding(fn_abi.ret.ty) {
        bail!(
            "unsupported native call: `{fn_name}` returns type `{}` with padding via indirect \
             return. Callee may leave padding bytes uninitialized",
            fn_abi.ret.ty
        );
    }

    if has_any_ptr_to_padded(fn_abi.ret.ty) {
        bail!(
            "unsupported native call: `{fn_name}` returns type containing a pointer \
             to a type with padding. Interpreter may read uninitialized bytes"
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
