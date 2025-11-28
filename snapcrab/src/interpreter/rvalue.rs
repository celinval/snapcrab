use crate::ty::MonoType;
use crate::value::Value;
use anyhow::{Context, Result, bail};
use num_traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedNeg, CheckedSub, Zero};
use rustc_public::mir::{AggregateKind, BinOp, CastKind, NullOp, Operand, Rvalue, UnOp};
use rustc_public::ty::{IntTy, RigidTy, Ty, UintTy};
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
        match result_type {
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
                let address = self.resolve_place_addr(place)?;
                Ok(Value::from_type(address))
            }
            Rvalue::AddressOf(_, place) => {
                let ty = rvalue.ty(self.locals())?;
                if !ty.is_thin_ptr() {
                    bail!("Wide pointers not supported");
                }
                let address = self.resolve_place_addr(place)?;
                Ok(Value::from_type(address))
            }
            Rvalue::Cast(cast_kind, operand, target_ty) => {
                let value = self.evaluate_operand(operand)?;
                self.perform_cast(cast_kind, value, target_ty)
            }
            Rvalue::Aggregate(kind, operands) => self.eval_aggregate(rvalue, kind, operands),
            Rvalue::NullaryOp(op, ty) => match op {
                NullOp::AlignOf => Ok(Value::from_type(ty.alignment()?)),
                NullOp::SizeOf => Ok(Value::from_type(ty.size()?)),
                _ => bail!("Unsupported nullary op: {:?}", op),
            },
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
                Ok(Value::from_val_with_padding(value, ty.size()?))
            }
            AggregateKind::Adt(def, _, _, _, _) if def.kind().is_enum() => {
                // Need to implement set discriminant
                bail!("Unsupported `enum` aggregation")
            }
            AggregateKind::Tuple | AggregateKind::Adt(..) | AggregateKind::Closure(..) => {
                let mut values = Vec::new();
                for operand in operands {
                    values.push(self.evaluate_operand(operand)?);
                }
                let ty = rvalue.ty(self.locals())?;
                Value::from_tuple_with_layout(&values, ty)
            }
            _ => bail!("Unsupported aggregate kind: {:?}", kind),
        }
    }

    /// Performs a cast operation
    fn perform_cast(
        &self,
        cast_kind: &rustc_public::mir::CastKind,
        value: Value,
        target_ty: &Ty,
    ) -> Result<Value> {
        match cast_kind {
            CastKind::PtrToPtr => {
                if !target_ty.is_thin_ptr() {
                    bail!("Wide pointers not supported");
                }
                Ok(value)
            }
            CastKind::Transmute => Ok(value),
            _ => bail!("Unsupported cast kind: {:?}", cast_kind),
        }
    }
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
