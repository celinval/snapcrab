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
use crate::value::Value;
use anyhow::Result;
use rustc_public::mir::mono::Instance;
use tracing::{debug, trace};

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
    debug!("Native call: {name}");

    let fn_abi = instance.fn_abi()?;
    trace_fn_abi(&name, &mangled, &fn_abi, args.len());

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

    // Call via JIT'd trampoline.
    let result = jit.call_native(fn_ptr, &fn_abi, args, &name)?;
    debug!("Native call returned: {name}");
    Ok(result)
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
