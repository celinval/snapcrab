//! SnapCrab Interpreter Library
//!
//! A rustc wrapper that leverages `rustc_public` to interpret Rust code at the MIR level.
//!
//! # Warning
//!
//! This library is not meant to be used outside of snapcrab binary.
//! Semantic versioning will only apply snapcrab binary.

#![feature(rustc_private)]
#![doc(hidden)]

extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_public;
extern crate rustc_public_bridge;

mod interpreter;
mod memory;
mod ty;
mod value;

pub use crate::interpreter::check::CheckConfig;
use crate::interpreter::function::invoke_fn;
use crate::memory::ThreadMemory;
use crate::value::TypedValue;
use anyhow::{Context, Result, bail};
use rustc_public::mir::mono::Instance;
use rustc_public::{CrateDef, CrateItem, entry_fn, local_crate};
use std::ffi::{CStr, CString};
use std::path::Path;
use std::process::ExitCode;
use tracing::info;

/// Execute a specific function by name from the current crate.
///
/// This function searches for a function definition with the given name,
/// converts it to an executable instance, and runs it. The function must
/// take no arguments.
///
/// # Arguments
/// * `fn_name` - Name of the function to execute
///
/// # Returns
/// * `Ok(Value)` - Function executed successfully, returns the result value
/// * `Err(anyhow::Error)` - Function not found, has arguments, or execution failed
///
/// # Examples
/// ```ignore
/// // Execute a function named "my_test"
/// let result = run_function("my_test")?;
/// ```
pub fn run_function(
    fn_name: &str,
    check_config: CheckConfig,
    native_libs: &[impl AsRef<Path>],
) -> Result<Vec<u8>> {
    // Load native libraries
    load_native_libs(native_libs)?;

    // Find function definition by name
    let crate_def = local_crate();
    let fn_def = crate_def
        .fn_defs()
        .into_iter()
        .find(|def| def.name().ends_with(fn_name))
        .with_context(|| format!("Function '{}' not found", fn_name))?;

    info!("Found function: {}", fn_def.name());

    // Convert FnDef to CrateItem using DefId
    let crate_item = CrateItem(fn_def.def_id());

    // Try to convert to instance
    let instance = Instance::try_from(crate_item)
        .with_context(|| format!("Failed to create instance from function: {}", fn_name))?;

    // Check if function takes no arguments
    let body = instance.body().context("No body for function")?;
    let arg_count = body.arg_locals().len();

    if arg_count > 0 {
        bail!(
            "Function '{}' takes {} arguments, only zero-argument functions are supported",
            fn_name,
            arg_count
        );
    }

    // Execute function
    let mut memory = ThreadMemory::new();
    memory.check_config = check_config;
    let result = invoke_fn(instance, &mut memory, vec![], &mut None)?;

    // Get return type from instance
    let body = instance.body().context("No body for function")?;
    let return_ty = body.ret_local().ty;

    // Create typed value and print
    let typed_result = TypedValue {
        ty: return_ty,
        value: result.as_bytes(),
    };

    info!("Function '{}' returned: {}", fn_name, typed_result);

    Ok(result.as_bytes().to_vec())
}

/// Load native shared libraries so their symbols are available to the interpreter.
///
/// Uses `RTLD_GLOBAL` so symbols are visible to `dlsym(RTLD_DEFAULT, ...)`,
/// which is how the interpreter resolves native function addresses.
/// `RTLD_LOCAL` (the default) would hide symbols from `RTLD_DEFAULT` lookups.
fn load_native_libs(paths: &[impl AsRef<Path>]) -> Result<()> {
    for path in paths {
        let path = path.as_ref();
        let c_path = CString::new(path.to_str().context("non-UTF8 library path")?)
            .context("library path contains null byte")?;
        // SAFETY: dlopen with RTLD_NOW | RTLD_GLOBAL loads the library and makes
        // its symbols visible to subsequent dlsym(RTLD_DEFAULT, ...) calls.
        let handle = unsafe { libc::dlopen(c_path.as_ptr(), libc::RTLD_NOW | libc::RTLD_GLOBAL) };
        if handle.is_null() {
            let err = unsafe { CStr::from_ptr(libc::dlerror()) };
            bail!(
                "failed to load native library '{}': {}",
                path.display(),
                err.to_string_lossy()
            );
        }
        info!("Loaded native library: {}", path.display());
    }
    Ok(())
}

pub fn run_main(check_config: CheckConfig, native_libs: &[impl AsRef<Path>]) -> Result<ExitCode> {
    load_native_libs(native_libs)?;
    let entry_fn = entry_fn().context("No entry function found")?;
    info!("Found entry function: {}", entry_fn.name());

    let instance =
        Instance::try_from(entry_fn).context("Failed to create instance from entry function")?;

    let mut memory = ThreadMemory::new();
    memory.check_config = check_config;
    let result = invoke_fn(instance, &mut memory, vec![], &mut None)?;

    // Convert the result value to an exit code
    match result {
        val if val.as_type::<u128>() == Some(0) || val.is_unit() => Ok(ExitCode::SUCCESS),
        _ => Ok(ExitCode::FAILURE),
    }
}
