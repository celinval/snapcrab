//! Value validity checking.
//!
//! Validates that interpreter values satisfy Rust's type invariants before
//! passing them to native code or after transmute operations.

use crate::value::{Value, uint_from_bytes};
use anyhow::{Result, bail};
use rustc_public::abi::{
    FieldsShape, Primitive, Scalar, TagEncoding, ValueAbi, VariantFields, VariantsShape,
};
use rustc_public::target::MachineInfo;
use rustc_public::ty::{RigidTy, Ty, TyKind, VariantIdx};
use rustc_public_bridge::IndexedVal;

/// Which categories of undefined behavior checks to perform.
#[derive(Clone, Debug)]
pub struct CheckConfig {
    pub validity: bool,
    pub alignment: bool,
    pub bounds: bool,
}

impl Default for CheckConfig {
    fn default() -> Self {
        Self {
            validity: true,
            alignment: true,
            bounds: true,
        }
    }
}

impl CheckConfig {
    /// Parse `--skip-check=validity,alignment` style flags.
    pub fn with_skipped(skipped: &[String]) -> Self {
        let mut config = Self::default();
        for s in skipped {
            for item in s.split(',') {
                match item.trim() {
                    "validity" => config.validity = false,
                    "alignment" => config.alignment = false,
                    "bounds" => config.bounds = false,
                    _ => {}
                }
            }
        }
        config
    }
}

/// Validate that a value is valid for its type.
///
/// Checks scalar `valid_range` constraints recursively through the type
/// layout. Returns an error describing the first invalidity found.
pub fn validate_value(value: &Value, ty: Ty, config: &CheckConfig) -> Result<()> {
    if !config.validity {
        return Ok(());
    }
    validate_recursive(value.as_bytes(), ty)
}

fn validate_recursive(bytes: &[u8], ty: Ty) -> Result<()> {
    let layout = ty.layout()?;
    let shape = layout.shape();

    match &shape.abi {
        ValueAbi::Scalar(scalar) => {
            validate_scalar(bytes, scalar, &ty)?;
        }
        ValueAbi::ScalarPair(first, second) => {
            let target = MachineInfo::target();
            let first_size = scalar_size(first, &target);
            validate_scalar(&bytes[..first_size], first, &ty)?;
            let second_size = scalar_size(second, &target);
            let second_offset = shape.size.bytes() - second_size;
            validate_scalar(
                &bytes[second_offset..second_offset + second_size],
                second,
                &ty,
            )?;
        }
        ValueAbi::Aggregate { .. } => {
            validate_aggregate(bytes, ty)?;
        }
        _ => {}
    }

    Ok(())
}

/// Validate an aggregate value by dispatching on its variant shape.
fn validate_aggregate(bytes: &[u8], ty: Ty) -> Result<()> {
    let layout = ty.layout()?;
    let shape = layout.shape();

    match &shape.variants {
        VariantsShape::Single { index } => {
            let field_tys = resolve_field_types(ty, *index);
            let offsets = match &shape.fields {
                FieldsShape::Arbitrary { offsets } => offsets,
                _ => return Ok(()),
            };
            validate_fields(bytes, &field_tys, offsets)?;
        }
        VariantsShape::Multiple {
            tag,
            tag_encoding,
            tag_field,
            variants,
        } => {
            // Validate the tag itself.
            let target = MachineInfo::target();
            let tag_sz = scalar_size(tag, &target);
            let tag_off = match &shape.fields {
                FieldsShape::Arbitrary { offsets } => offsets[*tag_field].bytes(),
                _ => return Ok(()),
            };
            validate_scalar(&bytes[tag_off..tag_off + tag_sz], tag, &ty)?;

            // Determine active variant and validate its fields.
            let active_variant =
                active_variant_idx(&bytes[tag_off..tag_off + tag_sz], tag_encoding, tag_sz);
            if let Some(variant_idx) = active_variant {
                let field_tys = resolve_field_types(ty, variant_idx);
                validate_variant_fields(bytes, &field_tys, variants, variant_idx)?;
            }
        }
        _ => {}
    }

    Ok(())
}

/// Validate fields of a struct-like layout.
fn validate_fields(
    bytes: &[u8],
    field_tys: &[Ty],
    offsets: &[rustc_public::target::MachineSize],
) -> Result<()> {
    for (i, field_ty) in field_tys.iter().enumerate() {
        if i >= offsets.len() {
            break;
        }
        let offset = offsets[i].bytes();
        let field_size = field_ty
            .layout()
            .map(|l| l.shape().size.bytes())
            .unwrap_or(0);
        if field_size == 0 {
            continue;
        }
        if offset + field_size <= bytes.len() {
            validate_recursive(&bytes[offset..offset + field_size], *field_ty)?;
        }
    }
    Ok(())
}

/// Validate fields of a specific variant in a multi-variant enum.
fn validate_variant_fields(
    bytes: &[u8],
    field_tys: &[Ty],
    variants: &[VariantFields],
    variant_idx: VariantIdx,
) -> Result<()> {
    let idx = variant_idx.to_index();
    if idx >= variants.len() {
        return Ok(());
    }
    let variant = &variants[idx];
    for (i, field_ty) in field_tys.iter().enumerate() {
        if i >= variant.offsets.len() {
            break;
        }
        let offset = variant.offsets[i].bytes();
        let field_size = field_ty
            .layout()
            .map(|l| l.shape().size.bytes())
            .unwrap_or(0);
        if field_size == 0 {
            continue;
        }
        if offset + field_size <= bytes.len() {
            validate_recursive(&bytes[offset..offset + field_size], *field_ty)?;
        }
    }
    Ok(())
}

/// Resolve field types for a given variant of a type.
fn resolve_field_types(ty: Ty, variant_idx: VariantIdx) -> Vec<Ty> {
    match ty.kind() {
        TyKind::RigidTy(RigidTy::Adt(def, ref args)) => {
            let Some(variant) = def.variant(variant_idx) else {
                return vec![];
            };
            variant
                .fields()
                .iter()
                .map(|f| f.ty_with_args(args))
                .collect()
        }
        TyKind::RigidTy(RigidTy::Tuple(fields)) => fields.clone(),
        _ => vec![],
    }
}

/// Determine which variant is active from the tag bytes.
fn active_variant_idx(
    tag_bytes: &[u8],
    encoding: &TagEncoding,
    tag_sz: usize,
) -> Option<VariantIdx> {
    let tag_val = uint_from_bytes(&tag_bytes[..tag_sz]);
    match encoding {
        TagEncoding::Direct => Some(VariantIdx::to_val(tag_val as usize)),
        TagEncoding::Niche {
            untagged_variant,
            niche_variants,
            niche_start,
        } => {
            let niche_start_idx = niche_variants.start().to_index();
            let niche_end_idx = niche_variants.end().to_index();
            let variant_count = niche_end_idx - niche_start_idx + 1;
            let max_tag = u128::MAX >> (128 - tag_sz * 8);
            let relative = tag_val.wrapping_sub(*niche_start) & max_tag;
            if relative < variant_count as u128 {
                Some(VariantIdx::to_val(niche_start_idx + relative as usize))
            } else {
                Some(*untagged_variant)
            }
        }
    }
}

fn validate_scalar(bytes: &[u8], scalar: &Scalar, ty: &Ty) -> Result<()> {
    let Scalar::Initialized { value, valid_range } = scalar else {
        return Ok(());
    };

    let target = MachineInfo::target();
    let size = value.size(&target).bytes();
    if bytes.len() < size {
        bail!(
            "Value too small for type `{ty}`: expected {size} bytes, got {}",
            bytes.len()
        );
    }

    let val = uint_from_bytes(&bytes[..size]);

    if !valid_range.contains(val) {
        let kind = match value {
            Primitive::Int { signed: true, .. } => "integer",
            Primitive::Int { signed: false, .. } => "unsigned integer",
            Primitive::Pointer(_) => "pointer",
            Primitive::Float { .. } => return Ok(()),
        };
        bail!(
            "Invalid {kind} value {val:#x} for type `{ty}` \
             (valid range: {valid_range:?})"
        );
    }

    Ok(())
}

fn scalar_size(scalar: &Scalar, target: &MachineInfo) -> usize {
    let prim = match scalar {
        Scalar::Initialized { value, .. } | Scalar::Union { value } => *value,
    };
    prim.size(target).bytes()
}
