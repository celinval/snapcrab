//! Compiler intrinsic shims.
//!
//! Intrinsics that don't have MIR fallback bodies are handled here.
//! This is the irreducible set that neither interpretation nor native calls can provide.

use crate::interpreter::check::{CheckConfig, validate_value};
use crate::value::Value;
use anyhow::{Result, bail};
use rustc_public::mir::mono::Instance;
use rustc_public::ty::{RigidTy, TyKind};
use tracing::debug;

/// Evaluate a compiler intrinsic.
pub fn eval_intrinsic(
    name: &str,
    args: &[Value],
    instance: Instance,
    config: &CheckConfig,
) -> Result<Value> {
    debug!("Intrinsic: {name}");
    match name {
        "assume" => {
            let val = args[0].as_bool().unwrap();
            if !val {
                bail!("Assumption violated in `{}`", instance.name());
            }
            Ok(Value::unit().clone())
        }
        "likely" | "unlikely" => Ok(args[0].clone()),
        "transmute" | "transmute_unchecked" => {
            let result = args[0].clone();
            // Validate that the transmuted value is valid for the target type
            let ret_ty = transmute_return_ty(instance)?;
            validate_value(&result, ret_ty, config)?;
            Ok(result)
        }
        "forget" => Ok(Value::unit().clone()),
        "needs_drop" => Ok(Value::from_bool(false)),
        "black_box" => Ok(args[0].clone()),
        _ => bail!("Unimplemented intrinsic `{name}` in `{}`", instance.name()),
    }
}

/// Extract the return type of a transmute intrinsic from its instance.
fn transmute_return_ty(instance: Instance) -> Result<rustc_public::ty::Ty> {
    let ty = instance.ty();
    match ty.kind() {
        TyKind::RigidTy(RigidTy::FnDef(_, args)) => {
            // transmute<T, U>(src: T) -> U; the second generic arg is the return type
            let ret_ty = *args.0[1].ty().unwrap();
            Ok(ret_ty)
        }
        _ => bail!("Cannot determine return type of transmute"),
    }
}
