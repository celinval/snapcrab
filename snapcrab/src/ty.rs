//! Module with type extensions.
use anyhow::Result;
use rustc_public::abi::{Primitive, Scalar, ValueAbi};
use rustc_public::mir::Mutability;
use rustc_public::ty::{AdtDef, GenericArgs, RigidTy, Ty, TyKind};

pub trait MonoType {
    /// Return the size of the type in bytes.
    fn size(&self) -> Result<usize>;

    /// Return the alignment of the type in bytes.
    fn alignment(&self) -> Result<usize>;

    /// Check if this is a thin pointer (single usize).
    fn is_thin_ptr(&self) -> bool;

    /// Check if this is a wide pointer (two usize values).
    #[allow(dead_code)]
    fn is_wide_ptr(&self) -> bool;
}

impl MonoType for Ty {
    /// Get the size in bytes of a type
    fn size(&self) -> Result<usize> {
        Ok(self.layout().map(|layout| layout.shape().size.bytes())?)
    }

    /// Get the alignment in bytes of a type
    fn alignment(&self) -> Result<usize> {
        Ok(self
            .layout()
            .map(|layout| layout.shape().abi_align as usize)?)
    }

    fn is_thin_ptr(&self) -> bool {
        let ptr_size = crate::memory::pointer_width();
        self.kind().is_any_ptr() && self.size().ok() == Some(ptr_size)
    }

    fn is_wide_ptr(&self) -> bool {
        let ptr_size = crate::memory::pointer_width();
        self.kind().is_any_ptr() && self.size().ok() == Some(2 * ptr_size)
    }
}

/// Check if a type contains a mutable pointer (`&mut T` or `*mut T`) where
/// the pointee `T` has padding bytes.
///
/// Traverses struct/tuple/array fields recursively. Does not follow through
/// pointer indirections (only checks the structure of the type itself).
pub fn has_mutable_ptr_to_padded(ty: Ty) -> bool {
    match ty.kind() {
        TyKind::RigidTy(RigidTy::Ref(_, pointee, Mutability::Mut)
        | RigidTy::RawPtr(pointee, Mutability::Mut)) => {
            has_padding(pointee) || has_mutable_ptr_to_padded(pointee)
        }
        TyKind::RigidTy(RigidTy::Ref(_, pointee, Mutability::Not)
        | RigidTy::RawPtr(pointee, Mutability::Not)) => {
            has_mutable_ptr_to_padded(pointee)
        }
        TyKind::RigidTy(RigidTy::Adt(adt_def, args)) => {
            adt_fields_contain_mutable_ptr_to_padded(adt_def, &args)
        }
        TyKind::RigidTy(RigidTy::Tuple(fields)) => {
            fields.iter().any(|f| has_mutable_ptr_to_padded(*f))
        }
        TyKind::RigidTy(RigidTy::Array(elem, _) | RigidTy::Slice(elem)) => {
            has_mutable_ptr_to_padded(elem)
        }
        // Scalars, function pointers, closures, dyn, str, never — cannot
        // contain a mutable pointer to a padded type in their field layout.
        TyKind::RigidTy(
            RigidTy::Bool
            | RigidTy::Char
            | RigidTy::Int(_)
            | RigidTy::Uint(_)
            | RigidTy::Float(_)
            | RigidTy::Str
            | RigidTy::Never
            | RigidTy::Foreign(_)
            | RigidTy::FnDef(..)
            | RigidTy::FnPtr(_)
            | RigidTy::Dynamic(..)
            | RigidTy::Closure(..)
            | RigidTy::Coroutine(..)
            | RigidTy::CoroutineClosure(..)
            | RigidTy::CoroutineWitness(..)
            | RigidTy::Pat(..),
        )
        // Non-rigid types (aliases, params, bound vars) are not expected in
        // monomorphized code
        | TyKind::Alias(..)
        | TyKind::Param(_)
        | TyKind::Bound(..) => false,
    }
}

/// Check if a type's layout may have padding bytes.
///
/// Only `Scalar` and `Vector` are guaranteed padding-free. `ScalarPair` has
/// padding if the two scalars don't evenly split the total size (e.g.,
/// `(u8, u64)` has 7 bytes of padding). For aggregates, we check the field
/// shape: arrays have padding if stride > element size; structs are checked
/// by comparing offsets against total size.
pub fn has_padding(ty: Ty) -> bool {
    use rustc_public::abi::FieldsShape;

    let Ok(layout) = ty.layout() else {
        return false;
    };
    let shape = layout.shape();
    let total = shape.size.bytes();
    if total == 0 {
        return false;
    }
    match &shape.abi {
        ValueAbi::Scalar(_) | ValueAbi::Vector { .. } => false,
        ValueAbi::ScalarPair(first, second) => {
            let target = crate::memory::machine_info();
            let first_size = scalar_primitive(first).size(target).bytes();
            let second_size = scalar_primitive(second).size(target).bytes();
            first_size + second_size != total
        }
        _ => match &shape.fields {
            // Arrays are always tightly packed — only check the element type.
            FieldsShape::Array { .. } => match ty.kind() {
                TyKind::RigidTy(RigidTy::Array(elem, _)) => has_padding(elem),
                _ => false,
            },
            FieldsShape::Arbitrary { .. } => composite_has_padding(ty, total),
            _ => true,
        },
    }
}

fn adt_fields_contain_mutable_ptr_to_padded(adt_def: AdtDef, args: &GenericArgs) -> bool {
    for variant in adt_def.variants() {
        for field in variant.fields() {
            let field_ty = field.ty_with_args(args);
            if has_mutable_ptr_to_padded(field_ty) {
                return true;
            }
        }
    }
    false
}

/// Check if a composite type (struct, enum, union, tuple) has padding.
///
/// A variant has padding if the sum of its field sizes is less than `total`,
/// or if any field itself has padding.
fn composite_has_padding(ty: Ty, total: usize) -> bool {
    match ty.kind() {
        TyKind::RigidTy(RigidTy::Tuple(fields)) => {
            let sum: usize = fields
                .iter()
                .filter_map(|t| t.layout().ok())
                .map(|l| l.shape().size.bytes())
                .sum();
            sum != total || fields.iter().any(|t| has_padding(*t))
        }
        TyKind::RigidTy(RigidTy::Adt(adt_def, args)) => adt_def.variants().iter().any(|variant| {
            let field_tys: Vec<_> = variant
                .fields()
                .iter()
                .map(|f| f.ty_with_args(&args))
                .collect();
            let sum: usize = field_tys
                .iter()
                .filter_map(|t| t.layout().ok())
                .map(|l| l.shape().size.bytes())
                .sum();
            sum < total || field_tys.iter().any(|t| has_padding(*t))
        }),
        _ => true,
    }
}

fn scalar_primitive(scalar: &Scalar) -> Primitive {
    match scalar {
        Scalar::Initialized { value, .. } | Scalar::Union { value } => *value,
    }
}
