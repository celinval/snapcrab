//! Compiler intrinsic shims.
//!
//! Intrinsics that don't have MIR fallback bodies are handled here.
//! This is the irreducible set that neither interpretation nor native calls can provide.

use crate::value::Value;
use anyhow::{Result, bail};
use rustc_public::mir::mono::Instance;
use tracing::debug;

/// Evaluate a compiler intrinsic.
pub fn eval_intrinsic(name: &str, args: &[Value], instance: Instance) -> Result<Value> {
    debug!("Intrinsic: {name}");
    match name {
        "assume" => Ok(Value::unit().clone()),
        "likely" | "unlikely" => Ok(args[0].clone()),
        "transmute" | "transmute_unchecked" => Ok(args[0].clone()),
        "forget" => Ok(Value::unit().clone()),
        "needs_drop" => Ok(Value::from_bool(false)),
        "black_box" => Ok(args[0].clone()),
        _ => bail!("Unimplemented intrinsic `{name}` in `{}`", instance.name()),
    }
}
