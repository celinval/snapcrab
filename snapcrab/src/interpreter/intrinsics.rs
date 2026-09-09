//! Compiler intrinsic shims.
//!
//! Intrinsics that don't have MIR fallback bodies are handled here.
//! This is the irreducible set that neither interpretation nor native calls can provide.

use std::mem;

use crate::interpreter::check::{CheckConfig, validate_value};
use crate::ty::MonoType;
use crate::value::Value;
use anyhow::{Context, Result, bail};
use rustc_public::mir::mono::Instance;
use rustc_public::ty::{AdtKind, GenericArgs, RigidTy, Ty, TyKind, VariantDef};
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
        // Guards against instantiating an uninhabited type (e.g. via
        // `MaybeUninit::assume_init`). Codegen would emit an abort here; since
        // we skip codegen, perform the check ourselves.
        "assert_inhabited" => {
            let ty = intrinsic_type_arg(instance, 0)?;
            if is_uninhabited(ty)? {
                bail!("attempted to instantiate uninhabited type `{ty}`");
            }
            Ok(Value::unit().clone())
        }
        // `size_of_val::<T>(ptr)` / `min_align_of_val::<T>(ptr)`: for sized `T`
        // the pointer is irrelevant; for unsized `T` the size comes from the
        // pointer's metadata (slice length, `str` byte length).
        "size_of_val" => {
            let ty = intrinsic_type_arg(instance, 0)?;
            Ok(Value::from_type(size_of_val(ty, &args[0])?))
        }
        "align_of_val" | "min_align_of_val" => {
            let ty = intrinsic_type_arg(instance, 0)?;
            Ok(Value::from_type(align_of_val(ty, &args[0])?))
        }
        // count ones -- read_uint extend integer with "0" bits, then count ones
        "ctpop" => Ok(Value::from_type(args[0].read_uint().count_ones())),

        "cttz" | "cttz_nonzero" => {
            let x = args[0].read_uint();
            if x == 0 {
                if name == "cttz_nonzero" {
                    bail!("`intrinsics::cttz_nonzero` called with zero, which is UB");
                }
                let bits = (args[0].len() * 8) as u32;
                Ok(Value::from_type(bits))
            } else {
                Ok(Value::from_type(x.trailing_zeros()))
            }
        }
        "ctlz" | "ctlz_nonzero" => {
            let bits = (args[0].len() * 8) as u32;
            let significant = u128::BITS - args[0].read_uint().leading_zeros();
            let leading_zeros = bits - significant;
            if leading_zeros == bits && name == "ctlz_nonzero" {
                bail!("`intrinsics::ctlz_nonzero` called with zero, which is UB");
            }
            Ok(Value::from_type(leading_zeros))
        }
        "needs_drop" => {
            // Per the spec, this should be resolved statically.
            // https://doc.rust-lang.org/std/intrinsics/fn.needs_drop.html
            unreachable!("internal error: unexpected `intrinsics::needs_drop` call.");
        }
        "black_box" => Ok(args[0].clone()),
        _ => bail!("Unimplemented intrinsic `{name}` in `{}`", instance.name()),
    }
}

/// Extract the return type of a transmute intrinsic from its instance.
fn transmute_return_ty(instance: Instance) -> Result<rustc_public::ty::Ty> {
    // transmute<T, U>(src: T) -> U; the second generic arg is the return type.
    intrinsic_type_arg(instance, 1)
}

/// Extract the `n`th generic type argument of an intrinsic instance.
fn intrinsic_type_arg(instance: Instance, n: usize) -> Result<rustc_public::ty::Ty> {
    let ty = instance.ty();
    let TyKind::RigidTy(RigidTy::FnDef(_, args)) = ty.kind() else {
        bail!("cannot read generic args of `{}`", instance.name());
    };
    let arg = args
        .0
        .get(n)
        .with_context(|| format!("`{}` has no generic arg {n}", instance.name()))?;
    arg.ty()
        .cloned()
        .with_context(|| format!("generic arg {n} of `{}` is not a type", instance.name()))
}

/// Determine whether `ty` is uninhabited (has no valid values).
///
/// The stable `LayoutShape` does not expose rustc's `uninhabited` flag, so we
/// reproduce the inhabitedness rule structurally: the never type is
/// uninhabited, a non-empty array of an uninhabited element is, a struct is
/// when any field is, and an enum is when every variant is (which also covers
/// zero-variant enums). Pointers and references to uninhabited types stay
/// inhabited. All fields are treated as visible, matching rustc within a
/// self-contained crate.
fn is_uninhabited(ty: Ty) -> Result<bool> {
    let TyKind::RigidTy(rigid) = ty.kind() else {
        // Non-rigid types (params, aliases) never reach a monomorphized
        // intrinsic; treat them as inhabited.
        return Ok(false);
    };
    match rigid {
        RigidTy::Never => Ok(true),
        RigidTy::Tuple(fields) => {
            for field in fields {
                if is_uninhabited(field)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        RigidTy::Array(elem, len) => Ok(len.eval_target_usize()? > 0 && is_uninhabited(elem)?),
        RigidTy::Adt(def, args) => match def.kind() {
            // A union with fields can always be left in a valid (untouched)
            // state, so it is inhabited.
            AdtKind::Union => Ok(false),
            // A struct is a single-variant case of the enum rule: uninhabited
            // when no variant can be inhabited.
            AdtKind::Struct | AdtKind::Enum => {
                for variant in def.variants_iter() {
                    if !variant_uninhabited(&variant, &args)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
        },
        _ => Ok(false),
    }
}

/// Compute `size_of_val` for `ty` given a (possibly wide) pointer to it.
///
/// Sized types ignore the pointer; unsized slices and `str` read their length
/// from the pointer's metadata.
///
/// TODO: Other unsized types (e.g. `dyn Trait`) are not yet supported.
fn size_of_val(ty: Ty, ptr: &Value) -> Result<usize> {
    match ty.kind() {
        TyKind::RigidTy(RigidTy::Slice(elem)) => {
            let len = ptr.ptr_metadata()?.read_uint() as usize;
            // The length comes from the interpreted program, so an absurd
            // slice metadata must not wrap into a plausible size.
            len.checked_mul(elem.size()?).with_context(|| {
                format!("slice of {len} `{elem}` elements overflows the address space")
            })
        }
        TyKind::RigidTy(RigidTy::Str) => Ok(ptr.ptr_metadata()?.read_uint() as usize),
        TyKind::RigidTy(RigidTy::Dynamic(..)) => {
            bail!("`intrinsics::size_of_val` does not yet support `dyn Trait` types")
        }
        _ => ty.size(),
    }
}

/// Compute `align_of_val` for `ty` given a (possibly wide) pointer to it.
///
/// Sized types ignore the pointer; unsized slices and `str` return the alignment of their elements.
///
/// TODO: Other unsized types (e.g. `dyn Trait`) are not yet supported.
fn align_of_val(ty: Ty, _ptr: &Value) -> Result<usize> {
    match ty.kind() {
        TyKind::RigidTy(RigidTy::Slice(elem)) => elem.alignment(),
        TyKind::RigidTy(RigidTy::Str) => Ok(mem::align_of::<u8>()),
        TyKind::RigidTy(RigidTy::Dynamic(..)) => {
            bail!("`intrinsics::align_of_val` does not yet support `dyn Trait` types")
        }
        _ => ty.alignment(),
    }
}

/// A variant is uninhabited when any of its fields is uninhabited.
fn variant_uninhabited(variant: &VariantDef, args: &GenericArgs) -> Result<bool> {
    for field in variant.fields() {
        if is_uninhabited(field.ty_with_args(args))? {
            return Ok(true);
        }
    }
    Ok(false)
}
