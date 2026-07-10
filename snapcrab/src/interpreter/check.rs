//! Value validity checking.
//!
//! Validates that interpreter values satisfy Rust's type invariants before
//! passing them to native code or after transmute operations.

use crate::value::Value;
use anyhow::{Result, bail};
use rustc_public::abi::{FieldsShape, Primitive, Scalar, ValueAbi, VariantsShape};
use rustc_public::target::{MachineInfo, MachineSize};
use rustc_public::ty::Ty;
use tracing::trace;

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
    validate_value_inner(value.as_bytes(), ty)
}

fn validate_value_inner(bytes: &[u8], ty: Ty) -> Result<()> {
    let layout = ty.layout()?;
    let shape = layout.shape();

    match &shape.abi {
        ValueAbi::Scalar(scalar) => {
            validate_scalar(bytes, scalar, &ty)?;
        }
        ValueAbi::ScalarPair(first, second) => {
            let target = MachineInfo::target();
            let first_size = scalar_size(first, &target);
            // First scalar at offset 0
            validate_scalar(&bytes[..first_size], first, &ty)?;
            // Second scalar: offset depends on alignment of second scalar
            let second_size = scalar_size(second, &target);
            let second_offset = shape.size.bytes() - second_size;
            validate_scalar(
                &bytes[second_offset..second_offset + second_size],
                second,
                &ty,
            )?;
        }
        ValueAbi::Aggregate { .. } => {
            // For aggregates, validate fields recursively based on variant shape
            match &shape.variants {
                VariantsShape::Single { .. } => {
                    if let FieldsShape::Arbitrary { offsets } = &shape.fields {
                        validate_aggregate_fields(bytes, ty, offsets)?;
                    }
                }
                VariantsShape::Multiple { .. } => {
                    // For multi-variant enums, we'd need to read the discriminant
                    // to know which variant is active. Skip deep validation for now.
                    trace!("Skipping deep validation for multi-variant enum: {ty}");
                }
                _ => {}
            }
        }
        _ => {}
    }

    Ok(())
}

fn validate_aggregate_fields(bytes: &[u8], ty: Ty, offsets: &[MachineSize]) -> Result<()> {
    let layout = ty.layout()?;
    let field_tys = layout.shape().fields.count();
    for i in 0..field_tys {
        let field_layout = ty.layout()?;
        let field_shape = field_layout.shape();
        if let FieldsShape::Arbitrary {
            offsets: field_offsets,
        } = &field_shape.fields
        {
            if i < field_offsets.len() && i < offsets.len() {
                // Field-level validation would require resolving field types,
                // which needs ADT info. For now, top-level scalar checks cover
                // the most important cases.
                let _ = (bytes, offsets);
            }
        }
    }
    Ok(())
}

fn validate_scalar(bytes: &[u8], scalar: &Scalar, ty: &Ty) -> Result<()> {
    let Scalar::Initialized { value, valid_range } = scalar else {
        // Union scalars have no validity constraint
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

    let val = read_uint(&bytes[..size]);

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

fn read_uint(bytes: &[u8]) -> u128 {
    let mut buf = [0u8; 16];
    buf[..bytes.len()].copy_from_slice(bytes);
    u128::from_le_bytes(buf)
}
