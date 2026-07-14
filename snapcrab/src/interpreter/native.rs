//! Native function call support via dynamic symbol resolution.
//!
//! When a function has no MIR body and is not an intrinsic we can shim,
//! we fall back to calling the native compiled version directly. This works
//! because:
//! - The interpreter's memory uses real process addresses
//! - The std library linked into the compiler process uses the same ABI
//! - The same rustc produced both the MIR and the native code
//!
//! The call uses an inline assembly trampoline that loads all argument
//! registers and calls the target. Since the trampoline is a normal Rust
//! frame with compiler-generated unwind info, panic unwinding propagates
//! correctly.

#[cfg(all(target_arch = "x86_64", unix))]
mod x86_64;
#[cfg(all(target_arch = "x86_64", unix))]
use x86_64 as platform;

#[cfg(not(all(target_arch = "x86_64", unix)))]
use unsupported as platform;

use crate::interpreter::check::{CheckConfig, validate_value};
use crate::value::Value;
use anyhow::{Result, bail};
use rustc_public::abi::{Primitive, Scalar, ValueAbi};
use rustc_public::mir::mono::Instance;
use std::ffi::CString;
use tracing::debug;

/// Call a native function by resolving its mangled symbol name.
///
/// Validates argument values before the call to prevent UB from invalid
/// values crossing the native boundary.
pub fn call_native(instance: Instance, args: &[Value], config: &CheckConfig) -> Result<Value> {
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

    // Delegate to platform-specific calling convention handler.
    platform::call(fn_ptr.cast(), &fn_abi, args, &name)
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

/// Check if a ValueAbi represents a float type.
fn is_float_abi(abi: &ValueAbi) -> bool {
    match abi {
        ValueAbi::Scalar(scalar) => is_float_scalar(scalar),
        _ => false,
    }
}

/// Check if a Scalar represents a float primitive.
fn is_float_scalar(scalar: &Scalar) -> bool {
    let prim = match scalar {
        Scalar::Initialized { value, .. } | Scalar::Union { value } => *value,
    };
    matches!(prim, Primitive::Float { .. })
}

#[cfg(not(all(target_arch = "x86_64", unix)))]
mod unsupported {
    use crate::value::Value;
    use anyhow::{Result, bail};
    use rustc_public::abi::FnAbi;

    pub fn call(
        _fn_ptr: *const (),
        _fn_abi: &FnAbi,
        _args: &[Value],
        fn_name: &str,
    ) -> Result<Value> {
        bail!(
            "Failed to invoke `{fn_name}`: native calls are not supported on this platform \
             (only x86-64 Unix is currently supported)"
        )
    }
}
