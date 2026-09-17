//! Compiler intrinsic shims.
//!
//! Intrinsics that don't have MIR fallback bodies are handled here.
//! This is the irreducible set that neither interpretation nor native calls can provide.

use std::mem;

use crate::interpreter::check::validate_value;
use crate::memory::{ThreadMemory, pointer_width};
use crate::ty::MonoType;
use crate::value::Value;
use anyhow::{Context, Result, bail};
use rustc_public::abi::FieldsShape;
use rustc_public::mir::mono::Instance;
use rustc_public::ty::{AdtKind, GenericArgs, RigidTy, Ty, TyKind, VariantDef};
use tracing::debug;

/// Evaluate a compiler intrinsic.
pub fn eval_intrinsic(
    name: &str,
    args: &[Value],
    instance: Instance,
    memory: &ThreadMemory,
) -> Result<Value> {
    debug!("Intrinsic: {name}");
    let config = &memory.check_config;
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
            Ok(Value::from_type(
                size_and_align_of_val(ty, &args[0], memory)?.0,
            ))
        }
        "align_of_val" | "min_align_of_val" => {
            let ty = intrinsic_type_arg(instance, 0)?;
            Ok(Value::from_type(
                size_and_align_of_val(ty, &args[0], memory)?.1,
            ))
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
fn transmute_return_ty(instance: Instance) -> Result<Ty> {
    // transmute<T, U>(src: T) -> U; the second generic arg is the return type.
    intrinsic_type_arg(instance, 1)
}

/// Extract the `n`th generic type argument of an intrinsic instance.
fn intrinsic_type_arg(instance: Instance, n: usize) -> Result<Ty> {
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

/// The maximum size of a Rust value in bytes: allocations and objects must not
/// exceed `isize::MAX`.
const MAX_OBJECT_SIZE: usize = isize::MAX as usize;

/// Compute the runtime `(size, align)` in bytes of the value `ty` points to.
///
/// Sized types ignore the pointer. Unsized types read from the pointer's
/// metadata: slices/`str` from the element count, `dyn Trait` from its vtable,
/// and an ADT/tuple with an unsized tail from that tail, recursively.
fn size_and_align_of_val(ty: Ty, ptr: &Value, memory: &ThreadMemory) -> Result<(usize, usize)> {
    match ty.kind() {
        TyKind::RigidTy(RigidTy::Slice(elem)) => {
            let len = ptr.ptr_metadata()?.read_uint() as usize;
            // The length comes from the interpreted program, possibly from
            // unsafe code, so validate the safety requirement: the total size
            // `len * size_of::<T>()` must not exceed `isize::MAX`. A `usize`
            // `checked_mul` alone would miss products in
            // `(isize::MAX, usize::MAX]`.
            let size = len
                .checked_mul(elem.size()?)
                .filter(|&size| size <= MAX_OBJECT_SIZE)
                .with_context(|| {
                    format!("slice of {len} `{elem}` elements exceeds the maximum object size")
                })?;
            Ok((size, elem.alignment()?))
        }
        TyKind::RigidTy(RigidTy::Str) => Ok((
            ptr.ptr_metadata()?.read_uint() as usize,
            mem::align_of::<u8>(),
        )),
        // `dyn Trait`: the metadata is a vtable pointer whose header holds the
        // erased type's size and alignment.
        TyKind::RigidTy(RigidTy::Dynamic(..)) => {
            size_align_from_vtable(ptr.ptr_metadata()?.read_uint() as usize, memory)
        }
        // An ADT or tuple whose last field is unsized.
        _ if ty.is_unsized()? => unsized_tail_size_and_align(ty, ptr, memory),
        _ => Ok((ty.size()?, ty.alignment()?)),
    }
}

/// Read the erased type's `(size, align)` from a vtable.
///
/// A vtable's header is `[drop_in_place, size, align, ...methods]`, so the size
/// and alignment sit one and two pointer widths past the base.
fn size_align_from_vtable(vtable: usize, memory: &ThreadMemory) -> Result<(usize, usize)> {
    let word = pointer_width();
    let read = |offset: usize| -> Result<usize> {
        Ok(memory
            .read_addr(vtable + offset, Ty::usize_ty())?
            .read_uint() as usize)
    };
    Ok((read(word)?, read(2 * word)?))
}

/// Compute `(size, align)` for an aggregate (`struct`/tuple) with an unsized
/// tail, following rustc's rule: the tail sits at its field offset, the whole
/// alignment is the larger of the sized prefix's and the tail's, and the size
/// is the tail's end rounded up to that alignment.
fn unsized_tail_size_and_align(
    ty: Ty,
    ptr: &Value,
    memory: &ThreadMemory,
) -> Result<(usize, usize)> {
    let shape = ty.layout()?.shape();
    let FieldsShape::Arbitrary { offsets } = &shape.fields else {
        bail!("unsized type `{ty}` has no field layout");
    };
    let tail_offset = offsets
        .last()
        .with_context(|| format!("unsized type `{ty}` has no fields"))?
        .bytes();

    let (tail_size, tail_align) = size_and_align_of_val(unsized_tail_ty(ty)?, ptr, memory)?;
    let align = (shape.abi_align as usize).max(tail_align);

    // Round the tail's end up to the aggregate's alignment, then enforce the
    // `isize::MAX` object-size limit.
    let end = tail_offset
        .checked_add(tail_size)
        .and_then(|end| end.checked_add(align - 1))
        .map(|end| end & !(align - 1))
        .filter(|&size| size <= MAX_OBJECT_SIZE)
        .with_context(|| format!("size of `{ty}` exceeds the maximum object size"))?;
    Ok((end, align))
}

/// The type of an aggregate's last (unsized) field.
fn unsized_tail_ty(ty: Ty) -> Result<Ty> {
    match ty.kind() {
        TyKind::RigidTy(RigidTy::Tuple(fields)) => fields
            .last()
            .copied()
            .with_context(|| format!("tuple `{ty}` has no fields")),
        TyKind::RigidTy(RigidTy::Adt(def, args)) => {
            let variant = def
                .variants_iter()
                .next()
                .with_context(|| format!("`{ty}` has no variant"))?;
            let field = variant
                .fields()
                .last()
                .cloned()
                .with_context(|| format!("`{ty}` has no fields"))?;
            Ok(field.ty_with_args(&args))
        }
        _ => bail!("cannot determine the unsized tail of `{ty}`"),
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
