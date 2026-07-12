use crate::ty::MonoType;
use crate::value::Value;
use anyhow::{Context, Result, bail};
use num_traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedNeg, CheckedSub, Zero};
use rustc_public::abi::{TagEncoding, VariantsShape};
use rustc_public::mir::{AggregateKind, BinOp, CastKind, Operand, PointerCoercion, Rvalue, UnOp};
use rustc_public::ty::{AdtDef, IntTy, RigidTy, Ty, TyKind, TypeAndMut, UintTy, VariantIdx};
use rustc_public_bridge::IndexedVal;
use zerocopy::{FromBytes, Immutable, IntoBytes};

use super::function::FnInterpreter;

/// Trait for evaluating binary operations on values.
pub trait BinaryEval {
    /// Evaluates a binary operation on two values.
    ///
    /// # Arguments
    /// * `left` - Left operand value
    /// * `right` - Right operand value
    /// * `operand_type` - Type of the operands
    ///
    /// # Returns
    /// * `Ok(Value)` - Result of the operation
    /// * `Err(anyhow::Error)` - If operation fails or is unsupported
    fn eval(&self, left: &Value, right: &Value, operand_type: RigidTy) -> Result<Value>;
}

/// Trait for evaluating unary operations on values.
pub trait UnaryEval {
    /// Evaluates a unary operation on a value.
    ///
    /// # Arguments
    /// * `operand` - The value to operate on
    /// * `result_type` - Expected type of the result
    ///
    /// # Returns
    /// * `Ok(Value)` - Result of the operation
    /// * `Err(anyhow::Error)` - If operation fails or is unsupported
    fn eval(&self, operand: &Value, result_type: RigidTy) -> Result<Value>;
}

impl BinaryEval for BinOp {
    fn eval(&self, left: &Value, right: &Value, operand_type: RigidTy) -> Result<Value> {
        match operand_type {
            RigidTy::Int(int_ty) => match int_ty {
                IntTy::I8 => eval_int_binop::<i8>(*self, left, right),
                IntTy::I16 => eval_int_binop::<i16>(*self, left, right),
                IntTy::I32 => eval_int_binop::<i32>(*self, left, right),
                IntTy::I64 => eval_int_binop::<i64>(*self, left, right),
                IntTy::I128 => eval_int_binop::<i128>(*self, left, right),
                IntTy::Isize => eval_int_binop::<isize>(*self, left, right),
            },
            RigidTy::Uint(uint_ty) => match uint_ty {
                UintTy::U8 => eval_int_binop::<u8>(*self, left, right),
                UintTy::U16 => eval_int_binop::<u16>(*self, left, right),
                UintTy::U32 => eval_int_binop::<u32>(*self, left, right),
                UintTy::U64 => eval_int_binop::<u64>(*self, left, right),
                UintTy::U128 => eval_int_binop::<u128>(*self, left, right),
                UintTy::Usize => eval_int_binop::<usize>(*self, left, right),
            },
            RigidTy::Bool => eval_bool_binop(*self, left, right),
            RigidTy::Char => eval_int_binop::<u32>(*self, left, right),
            RigidTy::RawPtr(_, _) | RigidTy::Ref(_, _, _) => {
                let ty = Ty::from_rigid_kind(operand_type);
                if !ty.is_thin_ptr() {
                    bail!("Wide pointers not supported");
                }
                eval_int_binop::<usize>(*self, left, right)
            }
            _ => bail!(
                "Unsupported binary operation `{self:?}` on `{}` type",
                Ty::from_rigid_kind(operand_type)
            ),
        }
    }
}

impl UnaryEval for UnOp {
    fn eval(&self, operand: &Value, result_type: RigidTy) -> Result<Value> {
        match self {
            UnOp::Not | UnOp::Neg => match result_type {
                RigidTy::Int(int_ty) => match int_ty {
                    IntTy::I8 => eval_int_unop::<i8>(*self, operand),
                    IntTy::I16 => eval_int_unop::<i16>(*self, operand),
                    IntTy::I32 => eval_int_unop::<i32>(*self, operand),
                    IntTy::I64 => eval_int_unop::<i64>(*self, operand),
                    IntTy::I128 => eval_int_unop::<i128>(*self, operand),
                    IntTy::Isize => eval_int_unop::<isize>(*self, operand),
                },
                RigidTy::Bool => eval_bool_unop(*self, operand),
                RigidTy::Uint(_) => bail!("Unary operations on unsigned integers not supported"),
                _ => bail!(
                    "Unsupported operation `{self:?}` on `{}` type",
                    Ty::from_rigid_kind(result_type)
                ),
            },
            UnOp::PtrMetadata => {
                // Extract metadata from wide pointer
                operand.ptr_metadata()
            }
        }
    }
}

/// Evaluates a binary operation on signed integers.
fn eval_int_binop<T>(op: BinOp, l: &Value, r: &Value) -> Result<Value>
where
    T: FromBytes
        + IntoBytes
        + Immutable
        + CheckedAdd
        + CheckedDiv
        + CheckedMul
        + CheckedSub
        + PartialEq
        + PartialOrd
        + Zero
        + std::ops::BitAnd<Output = T>
        + std::ops::BitOr<Output = T>
        + std::ops::BitXor<Output = T>,
{
    let left = l.as_type::<T>().unwrap();
    let right = r.as_type::<T>().unwrap();
    match op {
        BinOp::Add => left
            .checked_add(&right)
            .map(Value::from_type)
            .with_context(|| format!("Attempt to {op:?} with overflow")),
        BinOp::Sub => left
            .checked_sub(&right)
            .map(Value::from_type)
            .with_context(|| format!("Attempt to {op:?} with overflow")),
        BinOp::Mul => left
            .checked_mul(&right)
            .map(Value::from_type)
            .with_context(|| format!("Attempt to {op:?} with overflow")),
        BinOp::Div => {
            if right == <T as Zero>::zero() {
                bail!("Division by zero");
            }
            left.checked_div(&right)
                .map(Value::from_type)
                .with_context(|| format!("Attempt to {op:?} with overflow"))
        }
        BinOp::BitAnd => Ok(Value::from_type(left & right)),
        BinOp::BitOr => Ok(Value::from_type(left | right)),
        BinOp::BitXor => Ok(Value::from_type(left ^ right)),
        BinOp::Eq => Ok(Value::from_bool(left == right)),
        BinOp::Ne => Ok(Value::from_bool(left != right)),
        BinOp::Lt => Ok(Value::from_bool(left < right)),
        BinOp::Le => Ok(Value::from_bool(left <= right)),
        BinOp::Gt => Ok(Value::from_bool(left > right)),
        BinOp::Ge => Ok(Value::from_bool(left >= right)),
        _ => bail!("Unsupported integer binary operation: {:?}", op),
    }
}

/// Evaluates a binary operation on boolean values.
fn eval_bool_binop(op: BinOp, l: &Value, r: &Value) -> Result<Value> {
    let left = l.as_bool().unwrap();
    let right = r.as_bool().unwrap();
    let result = match op {
        BinOp::BitAnd => left & right,
        BinOp::BitOr => left | right,
        BinOp::Eq => left == right,
        BinOp::Ne => left != right,
        _ => bail!("Unsupported boolean binary operation: {:?}", op),
    };
    Ok(Value::from_bool(result))
}

/// Evaluates a unary operation on a signed integer.
fn eval_int_unop<T>(op: UnOp, v: &Value) -> Result<Value>
where
    T: FromBytes + IntoBytes + Immutable + CheckedNeg,
{
    let val = v.as_type::<T>().unwrap();
    match op {
        UnOp::Neg => val
            .checked_neg()
            .map(Value::from_type)
            .context("Integer overflow in negation"),
        _ => bail!("Unsupported integer unary operation: {:?}", op),
    }
}

/// Evaluates a unary operation on a boolean value.
fn eval_bool_unop(op: UnOp, v: &Value) -> Result<Value> {
    let val = v.as_bool().unwrap();
    match op {
        UnOp::Not => Ok(Value::from_bool(!val)),
        _ => bail!("Unsupported boolean unary operation: {:?}", op),
    }
}

impl<'a> FnInterpreter<'a> {
    /// Evaluates an rvalue (right-hand side value) expression.
    ///
    /// # Arguments
    /// * `rvalue` - The rvalue to evaluate (binary op, unary op, use, etc.)
    ///
    /// # Returns
    /// * `Ok(Value)` - The computed value
    /// * `Err(anyhow::Error)` - If evaluation fails or rvalue type is unsupported
    pub(super) fn evaluate_rvalue(&self, rvalue: &Rvalue) -> Result<Value> {
        match rvalue {
            Rvalue::BinaryOp(op, left, right) => {
                let left_val = self.evaluate_operand(left)?;
                let right_val = self.evaluate_operand(right)?;
                let operand_type = left.ty(self.locals())?.kind().rigid().unwrap().clone();
                op.eval(&left_val, &right_val, operand_type)
            }
            Rvalue::UnaryOp(op, operand) => {
                let val = self.evaluate_operand(operand)?;
                let result_type = rvalue.ty(self.locals())?.kind().rigid().unwrap().clone();
                op.eval(&val, result_type)
            }
            Rvalue::Use(operand) => self.evaluate_operand(operand),
            Rvalue::Ref(_, _, place) => {
                let ty = rvalue.ty(self.locals())?;
                if ty.is_wide_ptr() {
                    self.read_wide_ptr_from_place(place)
                } else {
                    let address = self.resolve_place_addr(place)?;
                    Ok(Value::from_type(address))
                }
            }
            Rvalue::AddressOf(_, place) => {
                let ty = rvalue.ty(self.locals())?;
                if ty.is_wide_ptr() {
                    self.read_wide_ptr_from_place(place)
                } else {
                    let address = self.resolve_place_addr(place)?;
                    Ok(Value::from_type(address))
                }
            }
            Rvalue::Cast(cast_kind, operand, target_ty) => {
                let value = self.evaluate_operand(operand)?;
                let source_ty = operand.ty(self.locals())?;
                self.perform_cast(cast_kind, value, source_ty, *target_ty)
            }
            Rvalue::CheckedBinaryOp(op, left, right) => {
                let left_val = self.evaluate_operand(left)?;
                let right_val = self.evaluate_operand(right)?;
                let operand_type = left.ty(self.locals())?.kind().rigid().unwrap().clone();
                let result_ty = rvalue.ty(self.locals())?;
                match op.eval(&left_val, &right_val, operand_type) {
                    Ok(val) => {
                        Value::from_tuple_with_layout(&[val, Value::from_bool(false)], result_ty)
                    }
                    Err(_) => {
                        let zero = Value::with_size(left_val.len());
                        Value::from_tuple_with_layout(&[zero, Value::from_bool(true)], result_ty)
                    }
                }
            }
            Rvalue::Discriminant(place) => {
                let enum_ty = place.ty(self.locals())?;
                let enum_val = self.read_from_place(place)?;
                read_discriminant(&enum_val, enum_ty)
            }
            Rvalue::Aggregate(kind, operands) => self.eval_aggregate(rvalue, kind, operands),
            Rvalue::Repeat(operand, count) => {
                let value = self.evaluate_operand(operand)?;
                let count_val = count.eval_target_usize()? as usize;
                Ok(Value::from_repeated(&value, count_val))
            }
            _ => {
                bail!("Unsupported rvalue: {:?}", rvalue);
            }
        }
    }

    fn eval_aggregate(
        &self,
        rvalue: &Rvalue,
        kind: &AggregateKind,
        operands: &[Operand],
    ) -> std::result::Result<Value, anyhow::Error> {
        match kind {
            AggregateKind::Adt(def, _, _, _, Some(_field)) => {
                debug_assert!(def.kind().is_union());
                debug_assert_eq!(operands.len(), 1);
                let value = self.evaluate_operand(&operands[0])?;
                let ty = rvalue.ty(self.locals())?;
                Ok(Value::from_val_with_padding(&value, ty.size()?))
            }
            AggregateKind::Adt(def, variant_idx, _, _, _) if def.kind().is_enum() => {
                let ty = rvalue.ty(self.locals())?;
                let mut values = Vec::new();
                for operand in operands {
                    values.push(self.evaluate_operand(operand)?);
                }
                build_enum_variant(&values, ty, *def, *variant_idx)
            }
            AggregateKind::Tuple | AggregateKind::Adt(..) | AggregateKind::Closure(..) => {
                let mut values = Vec::new();
                for operand in operands {
                    values.push(self.evaluate_operand(operand)?);
                }
                let ty = rvalue.ty(self.locals())?;
                Value::from_tuple_with_layout(&values, ty)
            }
            AggregateKind::Array(_) => {
                let mut values = Vec::new();
                for operand in operands {
                    values.push(self.evaluate_operand(operand)?);
                }
                Ok(Value::from_array(&values))
            }
            _ => bail!("Unsupported aggregate kind: {:?}", kind),
        }
    }

    /// Performs a cast operation
    fn perform_cast(
        &self,
        cast_kind: &rustc_public::mir::CastKind,
        value: Value,
        source_ty: Ty,
        target_ty: Ty,
    ) -> Result<Value> {
        match cast_kind {
            CastKind::IntToInt => {
                let target_size = target_ty.size()?;
                let source_size = source_ty.size()?;

                if target_size == source_size {
                    Ok(value)
                } else if target_size > source_size {
                    // Check if source is signed for sign extension
                    let is_signed = matches!(source_ty.kind().rigid(), Some(RigidTy::Int(_)));

                    if is_signed {
                        Ok(value.sign_extend(target_size))
                    } else {
                        // Zero extend for unsigned
                        Ok(Value::from_val_with_padding(&value, target_size))
                    }
                } else {
                    // Truncate to target size
                    Ok(value.as_bytes()[..target_size].into())
                }
            }
            CastKind::PtrToPtr => {
                if target_ty.is_wide_ptr() {
                    bail!("Expected cast to thin pointer, but found: `{target_ty}")
                } else if source_ty.is_wide_ptr() {
                    value.to_data_addr()
                } else {
                    Ok(value)
                }
            }
            CastKind::PointerCoercion(PointerCoercion::Unsize) => {
                perform_unsized_coercion(value, source_ty, target_ty)
            }
            CastKind::Transmute => {
                super::check::validate_value(&value, target_ty, &self.memory.check_config)?;
                Ok(value)
            }
            _ => bail!("Unsupported cast kind: {:?}", cast_kind),
        }
    }
}

/// Perform unsized coercion of a pointer/reference value, e.g., `&[T; n]` to `&[T]`.
///
/// Note the source could be a thin or wide pointer, while the target type is
/// always a wide pointer.
///
/// This includes converting:
///   - Thin pointers to wide pointers. E.g.: Array to slice, object to dyn trait.
///   - Structs containing thin pointers to structs containing wide pointers
///   - Conversion between wide pointers.
///       - E.g.: `&(dyn Any + Send)` to `&dyn Any`.
fn perform_unsized_coercion(value: Value, src_ptr_ty: Ty, dst_ptr_ty: Ty) -> Result<Value> {
    let src_pointee_kind = src_ptr_ty
        .kind()
        .builtin_deref(true)
        .map(|TypeAndMut { ty, .. }| ty.kind())
        .context("Expected pointer coercion, found source `{src_ptr_ty}`")?;
    let dst_pointee_kind = dst_ptr_ty
        .kind()
        .builtin_deref(true)
        .map(|TypeAndMut { ty, .. }| ty.kind())
        .context("Expected pointer coercion, found target `{src_ptr_ty}`")?;

    if src_pointee_kind == dst_pointee_kind {
        // In case of redundant cast
        Ok(value)
    } else if dst_pointee_kind.is_slice() {
        // [T; N] -> &[T]
        let data_ptr = value.as_type::<usize>().context("Expected pointer value")?;
        let TyKind::RigidTy(RigidTy::Array(_, len_const)) = src_pointee_kind else {
            bail!("Expected array for coercion to slice, but found {dst_ptr_ty}")
        };
        let len = len_const.eval_target_usize()? as usize;

        Ok(Value::new_wide_ptr(data_ptr, len))
    } else {
        // TODO: support container coercion (e.g., &Wrapper<[u8; N]> -> &Wrapper<[u8]>)
        // and trait object coercion (e.g., &Wrapper<T> -> &Wrapper<dyn Trait>).
        // See test_wrapper_slice_len, test_wrapper_dyn_debug.
        bail!("Unsupported coercion {src_ptr_ty} -> {dst_ptr_ty}")
    }
}

fn tag_scalar_size(
    tag: &rustc_public::abi::Scalar,
    target: &rustc_public::target::MachineInfo,
) -> usize {
    let prim = match tag {
        rustc_public::abi::Scalar::Initialized { value, .. }
        | rustc_public::abi::Scalar::Union { value } => *value,
    };
    prim.size(target).bytes()
}

/// Read the discriminant value from an enum's in-memory representation.
pub(super) fn read_discriminant(enum_val: &Value, enum_ty: Ty) -> Result<Value> {
    let layout = enum_ty.layout()?;
    let shape = layout.shape();
    let discr_ty = enum_ty.kind().discriminant_ty().unwrap();
    let discr_size = discr_ty.size()?;

    match &shape.variants {
        VariantsShape::Single { index } => {
            // Single-variant enum: discriminant is always that variant's value
            let TyKind::RigidTy(RigidTy::Adt(def, _)) = enum_ty.kind() else {
                return Ok(Value::with_size(discr_size));
            };
            let discr = def.discriminant_for_variant(*index);
            Ok(discr_value_to_bytes(discr.val, discr_size))
        }
        VariantsShape::Multiple {
            tag,
            tag_encoding,
            tag_field,
            ..
        } => {
            let target = rustc_public::target::MachineInfo::target();
            let tag_sz = tag_scalar_size(tag, &target);
            let tag_off = match &shape.fields {
                rustc_public::abi::FieldsShape::Arbitrary { offsets } => {
                    offsets[*tag_field].bytes()
                }
                _ => bail!("Unexpected field shape for enum"),
            };
            let tag_bytes = &enum_val.as_bytes()[tag_off..tag_off + tag_sz];
            let tag_val = read_uint(tag_bytes);

            let TyKind::RigidTy(RigidTy::Adt(def, _)) = enum_ty.kind() else {
                bail!("Expected ADT for discriminant read");
            };

            let discr_val = match tag_encoding {
                TagEncoding::Direct => tag_val,
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
                        let variant_idx = VariantIdx::to_val(niche_start_idx + relative as usize);
                        def.discriminant_for_variant(variant_idx).val
                    } else {
                        def.discriminant_for_variant(*untagged_variant).val
                    }
                }
            };
            Ok(discr_value_to_bytes(discr_val, discr_size))
        }
        _ => Ok(Value::with_size(discr_size)),
    }
}

/// Write a discriminant tag into an enum value's bytes.
pub(super) fn write_discriminant(
    enum_val: &mut [u8],
    enum_ty: Ty,
    variant_idx: VariantIdx,
) -> Result<()> {
    let layout = enum_ty.layout()?;
    let shape = layout.shape();

    match &shape.variants {
        VariantsShape::Single { .. } => {
            // Nothing to write for single-variant enums
            Ok(())
        }
        VariantsShape::Multiple {
            tag,
            tag_encoding,
            tag_field,
            ..
        } => {
            let target = rustc_public::target::MachineInfo::target();
            let tag_sz = tag_scalar_size(tag, &target);
            let tag_off = match &shape.fields {
                rustc_public::abi::FieldsShape::Arbitrary { offsets } => {
                    offsets[*tag_field].bytes()
                }
                _ => bail!("Unexpected field shape for enum"),
            };

            let TyKind::RigidTy(RigidTy::Adt(def, _)) = enum_ty.kind() else {
                bail!("Expected ADT for SetDiscriminant");
            };

            let tag_val = match tag_encoding {
                TagEncoding::Direct => def.discriminant_for_variant(variant_idx).val,
                TagEncoding::Niche {
                    untagged_variant,
                    niche_variants,
                    niche_start,
                } => {
                    if variant_idx == *untagged_variant {
                        // Untagged variant: don't write anything (payload determines it)
                        return Ok(());
                    }
                    let niche_start_idx = niche_variants.start().to_index();
                    let relative = variant_idx.to_index() - niche_start_idx;
                    niche_start.wrapping_add(relative as u128)
                }
            };
            write_uint(&mut enum_val[tag_off..tag_off + tag_sz], tag_val);
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Build an enum variant value with fields placed at the correct offsets.
fn build_enum_variant(
    field_values: &[Value],
    enum_ty: Ty,
    _def: AdtDef,
    variant_idx: VariantIdx,
) -> Result<Value> {
    let total_size = enum_ty.size()?;
    let layout = enum_ty.layout()?;
    let shape = layout.shape();

    let mut result = Value::with_size(total_size);

    let offsets = match &shape.variants {
        VariantsShape::Multiple { variants, .. } => &variants[variant_idx.to_index()].offsets,
        VariantsShape::Single { .. } => match &shape.fields {
            rustc_public::abi::FieldsShape::Arbitrary { offsets } => offsets,
            _ => return Ok(result),
        },
        _ => return Ok(result),
    };

    // Place field values at their offsets
    for (i, val) in field_values.iter().enumerate() {
        if val.len() > 0 {
            let offset = offsets[i].bytes();
            let end = offset + val.len();
            result.as_bytes_mut()[offset..end].copy_from_slice(val.as_bytes());
        }
    }

    // Write the discriminant tag
    write_discriminant(result.as_bytes_mut(), enum_ty, variant_idx)?;

    Ok(result)
}

fn discr_value_to_bytes(val: u128, size: usize) -> Value {
    let bytes = val.to_le_bytes();
    Value::from_bytes(&bytes[..size])
}

fn read_uint(bytes: &[u8]) -> u128 {
    let mut buf = [0u8; 16];
    buf[..bytes.len()].copy_from_slice(bytes);
    u128::from_le_bytes(buf)
}

fn write_uint(dest: &mut [u8], val: u128) {
    let bytes = val.to_le_bytes();
    dest.copy_from_slice(&bytes[..dest.len()]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_binary_operations() {
        let result = BinOp::Add
            .eval(
                &Value::from_type(5i128),
                &Value::from_type(3i128),
                RigidTy::Int(IntTy::I128),
            )
            .unwrap();
        assert_eq!(result, Value::from_type(8i128));
    }

    #[test]
    fn test_int_overflow() {
        let result = BinOp::Add.eval(
            &Value::from_type(i128::MAX),
            &Value::from_type(1i128),
            RigidTy::Int(IntTy::I128),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_uint_binary_operations() {
        assert_eq!(
            BinOp::Add
                .eval(
                    &Value::from_type(10u128),
                    &Value::from_type(5i128),
                    RigidTy::Uint(UintTy::U128)
                )
                .unwrap(),
            Value::from_type(15u128)
        );
        assert_eq!(
            BinOp::Sub
                .eval(
                    &Value::from_type(10u128),
                    &Value::from_type(5i128),
                    RigidTy::Uint(UintTy::U128)
                )
                .unwrap(),
            Value::from_type(5i128)
        );
        assert_eq!(
            BinOp::Mul
                .eval(
                    &Value::from_type(10u128),
                    &Value::from_type(5i128),
                    RigidTy::Uint(UintTy::U128)
                )
                .unwrap(),
            Value::from_type(50u128)
        );
        assert_eq!(
            BinOp::Div
                .eval(
                    &Value::from_type(10u128),
                    &Value::from_type(5i128),
                    RigidTy::Uint(UintTy::U128)
                )
                .unwrap(),
            Value::from_type(2u128)
        );
    }

    #[test]
    fn test_division_by_zero() {
        assert!(
            BinOp::Div
                .eval(
                    &Value::from_type(10u128),
                    &Value::from_type(0i128),
                    RigidTy::Int(IntTy::I128)
                )
                .is_err()
        );
        assert!(
            BinOp::Div
                .eval(
                    &Value::from_type(10u128),
                    &Value::from_type(0i128),
                    RigidTy::Uint(UintTy::U128)
                )
                .is_err()
        );
    }

    #[test]
    fn test_bool_binary_operations() {
        assert_eq!(
            BinOp::BitAnd
                .eval(
                    &Value::from_bool(true),
                    &Value::from_bool(false),
                    RigidTy::Bool
                )
                .unwrap(),
            Value::from_bool(false)
        );
        assert_eq!(
            BinOp::BitOr
                .eval(
                    &Value::from_bool(true),
                    &Value::from_bool(false),
                    RigidTy::Bool
                )
                .unwrap(),
            Value::from_bool(true)
        );
        assert_eq!(
            BinOp::Eq
                .eval(
                    &Value::from_bool(true),
                    &Value::from_bool(true),
                    RigidTy::Bool
                )
                .unwrap(),
            Value::from_bool(true)
        );
    }

    #[test]
    fn test_unary_operations() {
        assert_eq!(
            UnOp::Neg
                .eval(&Value::from_type(5i128), RigidTy::Int(IntTy::I128))
                .unwrap(),
            Value::from_type(-5i128)
        );
        assert_eq!(
            UnOp::Not
                .eval(&Value::from_bool(true), RigidTy::Bool)
                .unwrap(),
            Value::from_bool(false)
        );
    }

    #[test]
    fn test_different_int_sizes() {
        // i8
        assert_eq!(
            BinOp::Add
                .eval(
                    &Value::from_type(5i8),
                    &Value::from_type(3i8),
                    RigidTy::Int(IntTy::I8)
                )
                .unwrap(),
            Value::from_type(8i8)
        );

        // i16
        assert_eq!(
            BinOp::Mul
                .eval(
                    &Value::from_type(10i16),
                    &Value::from_type(4i16),
                    RigidTy::Int(IntTy::I16)
                )
                .unwrap(),
            Value::from_type(40i16)
        );

        // i32
        assert_eq!(
            BinOp::Sub
                .eval(
                    &Value::from_type(100i32),
                    &Value::from_type(25i32),
                    RigidTy::Int(IntTy::I32)
                )
                .unwrap(),
            Value::from_type(75i32)
        );

        // i64
        assert_eq!(
            BinOp::Div
                .eval(
                    &Value::from_type(64i64),
                    &Value::from_type(8i64),
                    RigidTy::Int(IntTy::I64)
                )
                .unwrap(),
            Value::from_type(8i64)
        );

        // isize
        assert_eq!(
            BinOp::Add
                .eval(
                    &Value::from_type(17isize),
                    &Value::from_type(5isize),
                    RigidTy::Int(IntTy::Isize)
                )
                .unwrap(),
            Value::from_type(22isize)
        );
    }

    #[test]
    fn test_different_uint_sizes() {
        // u8
        assert_eq!(
            BinOp::Add
                .eval(
                    &Value::from_type(200u8),
                    &Value::from_type(50u8),
                    RigidTy::Uint(UintTy::U8)
                )
                .unwrap(),
            Value::from_type(250u8)
        );

        // u16
        assert_eq!(
            BinOp::Mul
                .eval(
                    &Value::from_type(300u16),
                    &Value::from_type(2u16),
                    RigidTy::Uint(UintTy::U16)
                )
                .unwrap(),
            Value::from_type(600u16)
        );

        // u32
        assert_eq!(
            BinOp::Sub
                .eval(
                    &Value::from_type(1000u32),
                    &Value::from_type(250u32),
                    RigidTy::Uint(UintTy::U32)
                )
                .unwrap(),
            Value::from_type(750u32)
        );

        // u64
        assert_eq!(
            BinOp::Div
                .eval(
                    &Value::from_type(1024u64),
                    &Value::from_type(16u64),
                    RigidTy::Uint(UintTy::U64)
                )
                .unwrap(),
            Value::from_type(64u64)
        );

        // usize
        assert_eq!(
            BinOp::Mul
                .eval(
                    &Value::from_type(23usize),
                    &Value::from_type(7usize),
                    RigidTy::Uint(UintTy::Usize)
                )
                .unwrap(),
            Value::from_type(161usize)
        );
    }

    #[test]
    fn test_bitwise_operations() {
        assert_eq!(
            BinOp::BitAnd
                .eval(
                    &Value::from_type(0b1100u8),
                    &Value::from_type(0b1010u8),
                    RigidTy::Uint(UintTy::U8)
                )
                .unwrap(),
            Value::from_type(0b1000u8)
        );
        assert_eq!(
            BinOp::BitOr
                .eval(
                    &Value::from_type(0b1100u8),
                    &Value::from_type(0b1010u8),
                    RigidTy::Uint(UintTy::U8)
                )
                .unwrap(),
            Value::from_type(0b1110u8)
        );
        assert_eq!(
            BinOp::BitXor
                .eval(
                    &Value::from_type(0b1100u8),
                    &Value::from_type(0b1010u8),
                    RigidTy::Uint(UintTy::U8)
                )
                .unwrap(),
            Value::from_type(0b0110u8)
        );
    }

    #[test]
    fn test_comparison_operations() {
        assert_eq!(
            BinOp::Eq
                .eval(
                    &Value::from_type(42i32),
                    &Value::from_type(42i32),
                    RigidTy::Int(IntTy::I32)
                )
                .unwrap(),
            Value::from_bool(true)
        );
        assert_eq!(
            BinOp::Ne
                .eval(
                    &Value::from_type(42i32),
                    &Value::from_type(43i32),
                    RigidTy::Int(IntTy::I32)
                )
                .unwrap(),
            Value::from_bool(true)
        );
        assert_eq!(
            BinOp::Lt
                .eval(
                    &Value::from_type(10i32),
                    &Value::from_type(20i32),
                    RigidTy::Int(IntTy::I32)
                )
                .unwrap(),
            Value::from_bool(true)
        );
        assert_eq!(
            BinOp::Le
                .eval(
                    &Value::from_type(10i32),
                    &Value::from_type(10i32),
                    RigidTy::Int(IntTy::I32)
                )
                .unwrap(),
            Value::from_bool(true)
        );
        assert_eq!(
            BinOp::Gt
                .eval(
                    &Value::from_type(20i32),
                    &Value::from_type(10i32),
                    RigidTy::Int(IntTy::I32)
                )
                .unwrap(),
            Value::from_bool(true)
        );
        assert_eq!(
            BinOp::Ge
                .eval(
                    &Value::from_type(20i32),
                    &Value::from_type(20i32),
                    RigidTy::Int(IntTy::I32)
                )
                .unwrap(),
            Value::from_bool(true)
        );
    }
}
