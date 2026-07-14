//! Native function call support via dynamic symbol resolution.
//!
//! When a function has no MIR body and is not an intrinsic we can shim,
//! we fall back to calling the native compiled version directly. This works
//! because:
//! - The interpreter's memory uses real process addresses
//! - The std library linked into the compiler process uses the same ABI
//! - The same rustc produced both the MIR and the native code
//!
//! The call uses a cranelift JIT'd trampoline that loads typed arguments
//! and calls the target with the correct ABI.

pub mod jit;

use crate::interpreter::check::{CheckConfig, validate_value};
use crate::value::Value;
use anyhow::{Result, bail};
use rustc_public::mir::mono::Instance;
use std::ffi::CString;
use tracing::debug;

/// Call a native function by resolving its mangled symbol name.
///
/// Validates argument values before the call to prevent UB from invalid
/// values crossing the native boundary.
pub fn call_native(
    instance: Instance,
    args: &[Value],
    config: &CheckConfig,
    jit: &jit::JitEngine,
) -> Result<Value> {
    let mangled = instance.mangled_name();
    let name = instance.name();
    debug!("Native call: {name} ({mangled})");

    let fn_abi = instance.fn_abi()?;
    debug_fn_abi(&name, &mangled, &fn_abi, args.len());

    // Validate arguments before passing to native code.
    for (arg_abi, arg_val) in fn_abi.args.iter().zip(args.iter()) {
        validate_value(arg_val, arg_abi.ty, config)?;
    }

    // Resolve symbol from the current process via dlsym(RTLD_DEFAULT, ...).
    // TODO: cache resolved symbols to avoid repeated linear searches.
    let symbol_name = mangled.as_str();
    let c_name = CString::new(symbol_name).expect("Symbol name should not contain null bytes");
    // SAFETY: dlsym with RTLD_DEFAULT searches the current process's loaded symbols.
    // The returned pointer is valid for the process lifetime (std is always loaded).
    let fn_ptr = unsafe { libc::dlsym(libc::RTLD_DEFAULT, c_name.as_ptr()) };
    if fn_ptr.is_null() {
        bail!("Failed to invoke `{name}`: symbol `{symbol_name}` not found in current process");
    }

    // Call via JIT'd trampoline.
    jit.call_native(fn_ptr.cast(), &fn_abi, args, &name)
}

/// Log detailed ABI information for a native call.
fn debug_fn_abi(
    name: &str,
    mangled: &str,
    fn_abi: &rustc_public::abi::FnAbi,
    interpreter_args: usize,
) {
    if !tracing::enabled!(tracing::Level::DEBUG) {
        return;
    }
    debug!("  {name} ({mangled})");
    debug!(
        "  abi args: {}, interpreter args: {interpreter_args}",
        fn_abi.args.len()
    );
    for (i, arg_abi) in fn_abi.args.iter().enumerate() {
        debug!("  arg[{i}]: mode={:?}, ty={}", arg_abi.mode, arg_abi.ty);
    }
    debug!("  ret: mode={:?}, ty={}", fn_abi.ret.mode, fn_abi.ret.ty);
}
