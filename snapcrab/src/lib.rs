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

mod interpreter;
mod memory;
mod ty;
mod value;

use crate::interpreter::function::invoke_fn;
use crate::memory::ThreadMemory;
use crate::value::{TypedValue, Value};
use anyhow::{Result, bail};
use rustc_public::mir::mono::Instance;
use rustc_public::{CrateDef, CrateItem, entry_fn, local_crate};
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
pub fn run_function(fn_name: &str) -> Result<Value> {
    // Find function definition by name
    let crate_def = local_crate();
    let fn_def = crate_def
        .fn_defs()
        .into_iter()
        .find(|def| def.name() == fn_name)
        .ok_or_else(|| anyhow::anyhow!("Function '{}' not found", fn_name))?;

    info!("Found function: {}", fn_def.name());

    // Convert FnDef to CrateItem using DefId
    let crate_item = CrateItem(fn_def.def_id());

    // Try to convert to instance
    let instance = Instance::try_from(crate_item)
        .map_err(|e| anyhow::anyhow!("Failed to create instance from function: {}", e))?;

    // Check if function takes no arguments
    let body = instance
        .body()
        .ok_or_else(|| anyhow::anyhow!("No body for function"))?;
    let arg_count = body.arg_locals().len();

    if arg_count > 0 {
        bail!(
            "Function '{}' takes {} arguments, only zero-argument functions are supported",
            fn_name,
            arg_count
        );
    }

    // Execute function
    let result = invoke_fn(instance, &mut ThreadMemory::new(), vec![], &mut None)?;

    // Get return type from instance
    let body = instance
        .body()
        .ok_or_else(|| anyhow::anyhow!("No body for function"))?;
    let return_ty = body.ret_local().ty;

    // Create typed value and print
    let typed_result = TypedValue {
        ty: return_ty,
        value: result.as_bytes(),
    };

    info!("Function '{}' returned: {}", fn_name, typed_result);

    Ok(result)
}

pub fn run_main() -> Result<ExitCode> {
    let entry_fn = entry_fn().ok_or_else(|| anyhow::anyhow!("No entry function found"))?;
    info!("Found entry function: {}", entry_fn.name());

    let instance = Instance::try_from(entry_fn)
        .map_err(|e| anyhow::anyhow!("Failed to create instance from entry function: {}", e))?;

    let result = invoke_fn(instance, &mut ThreadMemory::new(), vec![], &mut None)?;

    // Convert the result value to an exit code
    match result {
        val if val.as_type::<u128>() == Some(0) || val.is_unit() => Ok(ExitCode::SUCCESS),
        _ => Ok(ExitCode::FAILURE),
    }
}
