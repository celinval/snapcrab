use crate::value::{Value, ValueType};
use anyhow::{Result, bail};
use rustc_public::mir::{BinOp, Rvalue, UnOp};
use rustc_public::ty::{RigidTy, Ty, TyKind};

/// Trait for evaluating binary operations on values.
pub trait BinaryEval {
    /// Evaluates a binary operation on two values.
    ///
    /// # Arguments
    /// * `left` - Left operand value
    /// * `right` - Right operand value
    /// * `result_type` - Expected type of the result
    ///
    /// # Returns
    /// * `Ok(Value)` - Result of the operation
    /// * `Err(anyhow::Error)` - If operation fails or is unsupported
    fn eval(&self, left: Value, right: Value, result_type: ValueType) -> Result<Value>;
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
    fn eval(&self, operand: Value, result_type: ValueType) -> Result<Value>;
}

impl BinaryEval for BinOp {
    fn eval(&self, left: Value, right: Value, result_type: ValueType) -> Result<Value> {
        match result_type {
            ValueType::Int => {
                let l = left
                    .as_i128()
                    .ok_or_else(|| anyhow::anyhow!("Expected i128 for left operand"))?;
                let r = right
                    .as_i128()
                    .ok_or_else(|| anyhow::anyhow!("Expected i128 for right operand"))?;
                eval_int_binop(*self, l, r)
            }
            ValueType::Uint => {
                let l = left
                    .as_u128()
                    .ok_or_else(|| anyhow::anyhow!("Expected u128 for left operand"))?;
                let r = right
                    .as_u128()
                    .ok_or_else(|| anyhow::anyhow!("Expected u128 for right operand"))?;
                eval_uint_binop(*self, l, r)
            }
            ValueType::Bool => {
                let l = left
                    .as_bool()
                    .ok_or_else(|| anyhow::anyhow!("Expected bool for left operand"))?;
                let r = right
                    .as_bool()
                    .ok_or_else(|| anyhow::anyhow!("Expected bool for right operand"))?;
                eval_bool_binop(*self, l, r)
            }
            ValueType::Unit => bail!("Cannot perform binary operations on unit type"),
        }
    }
}

impl UnaryEval for UnOp {
    fn eval(&self, operand: Value, result_type: ValueType) -> Result<Value> {
        match result_type {
            ValueType::Int => {
                let val = operand
                    .as_i128()
                    .ok_or_else(|| anyhow::anyhow!("Expected i128 for operand"))?;
                eval_int_unop(*self, val)
            }
            ValueType::Bool => {
                let val = operand
                    .as_bool()
                    .ok_or_else(|| anyhow::anyhow!("Expected bool for operand"))?;
                eval_bool_unop(*self, val)
            }
            ValueType::Uint => bail!("Unary operations on unsigned integers not supported"),
            ValueType::Unit => bail!("Cannot perform unary operations on unit type"),
        }
    }
}

/// Evaluates a binary operation on signed integers.
fn eval_int_binop(op: BinOp, left: i128, right: i128) -> Result<Value> {
    match op {
        BinOp::Add => left
            .checked_add(right)
            .map(Value::from_i128)
            .ok_or_else(|| anyhow::anyhow!("Integer overflow in addition")),
        BinOp::Sub => left
            .checked_sub(right)
            .map(Value::from_i128)
            .ok_or_else(|| anyhow::anyhow!("Integer overflow in subtraction")),
        BinOp::Mul => left
            .checked_mul(right)
            .map(Value::from_i128)
            .ok_or_else(|| anyhow::anyhow!("Integer overflow in multiplication")),
        BinOp::Div => {
            if right == 0 {
                bail!("Division by zero");
            }
            left.checked_div(right)
                .map(Value::from_i128)
                .ok_or_else(|| anyhow::anyhow!("Integer overflow in division"))
        }
        _ => bail!("Unsupported integer binary operation: {:?}", op),
    }
}

/// Evaluates a binary operation on unsigned integers.
fn eval_uint_binop(op: BinOp, left: u128, right: u128) -> Result<Value> {
    match op {
        BinOp::Add => left
            .checked_add(right)
            .map(Value::from_u128)
            .ok_or_else(|| anyhow::anyhow!("Integer overflow in addition")),
        BinOp::Sub => left
            .checked_sub(right)
            .map(Value::from_u128)
            .ok_or_else(|| anyhow::anyhow!("Integer overflow in subtraction")),
        BinOp::Mul => left
            .checked_mul(right)
            .map(Value::from_u128)
            .ok_or_else(|| anyhow::anyhow!("Integer overflow in multiplication")),
        BinOp::Div => {
            if right == 0 {
                bail!("Division by zero");
            }
            left.checked_div(right)
                .map(Value::from_u128)
                .ok_or_else(|| anyhow::anyhow!("Integer overflow in division"))
        }
        _ => bail!("Unsupported unsigned integer binary operation: {:?}", op),
    }
}

/// Evaluates a binary operation on boolean values.
fn eval_bool_binop(op: BinOp, left: bool, right: bool) -> Result<Value> {
    match op {
        BinOp::BitAnd => Ok(Value::from_bool(left & right)),
        BinOp::BitOr => Ok(Value::from_bool(left | right)),
        BinOp::Eq => Ok(Value::from_bool(left == right)),
        BinOp::Ne => Ok(Value::from_bool(left != right)),
        _ => bail!("Unsupported boolean binary operation: {:?}", op),
    }
}

/// Evaluates a unary operation on a signed integer.
fn eval_int_unop(op: UnOp, val: i128) -> Result<Value> {
    match op {
        UnOp::Neg => val
            .checked_neg()
            .map(Value::from_i128)
            .ok_or_else(|| anyhow::anyhow!("Integer overflow in negation")),
        _ => bail!("Unsupported integer unary operation: {:?}", op),
    }
}

/// Evaluates a unary operation on a boolean value.
fn eval_bool_unop(op: UnOp, val: bool) -> Result<Value> {
    match op {
        UnOp::Not => Ok(Value::from_bool(!val)),
        _ => bail!("Unsupported boolean unary operation: {:?}", op),
    }
}

impl super::function::FnInterpreter {
    /// Convert a MIR type to ValueType for operations
    fn ty_to_value_type(&self, ty: Ty) -> Result<ValueType> {
        match ty.kind() {
            TyKind::RigidTy(RigidTy::Bool) => Ok(ValueType::Bool),
            TyKind::RigidTy(RigidTy::Int(_)) => Ok(ValueType::Int),
            TyKind::RigidTy(RigidTy::Uint(_)) => Ok(ValueType::Uint),
            TyKind::RigidTy(RigidTy::Tuple(fields)) if fields.is_empty() => Ok(ValueType::Unit),
            _ => bail!("Unsupported type for operation: {:?}", ty),
        }
    }

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
                let result_type = self.ty_to_value_type(rvalue.ty(self.locals())?)?;
                op.eval(left_val, right_val, result_type)
            }
            Rvalue::UnaryOp(op, operand) => {
                let val = self.evaluate_operand(operand)?;
                let result_type = self.ty_to_value_type(rvalue.ty(self.locals())?)?;
                op.eval(val, result_type)
            }
            Rvalue::Use(operand) => self.evaluate_operand(operand),
            _ => {
                bail!("Unsupported rvalue: {:?}", rvalue);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_binary_operations() {
        let result = BinOp::Add
            .eval(Value::from_i128(5), Value::from_i128(3), ValueType::Int)
            .unwrap();
        assert_eq!(result, Value::from_i128(8));
    }

    #[test]
    fn test_int_overflow() {
        let result = BinOp::Add.eval(
            Value::from_i128(i128::MAX),
            Value::from_i128(1),
            ValueType::Int,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_uint_binary_operations() {
        assert_eq!(
            BinOp::Add
                .eval(Value::from_u128(10), Value::from_u128(5), ValueType::Uint)
                .unwrap(),
            Value::from_u128(15)
        );
        assert_eq!(
            BinOp::Sub
                .eval(Value::from_u128(10), Value::from_u128(5), ValueType::Uint)
                .unwrap(),
            Value::from_u128(5)
        );
        assert_eq!(
            BinOp::Mul
                .eval(Value::from_u128(10), Value::from_u128(5), ValueType::Uint)
                .unwrap(),
            Value::from_u128(50)
        );
        assert_eq!(
            BinOp::Div
                .eval(Value::from_u128(10), Value::from_u128(5), ValueType::Uint)
                .unwrap(),
            Value::from_u128(2)
        );
    }

    #[test]
    fn test_division_by_zero() {
        assert!(
            BinOp::Div
                .eval(Value::from_i128(10), Value::from_i128(0), ValueType::Int)
                .is_err()
        );
        assert!(
            BinOp::Div
                .eval(Value::from_u128(10), Value::from_u128(0), ValueType::Uint)
                .is_err()
        );
    }

    #[test]
    fn test_bool_binary_operations() {
        assert_eq!(
            BinOp::BitAnd
                .eval(
                    Value::from_bool(true),
                    Value::from_bool(false),
                    ValueType::Bool
                )
                .unwrap(),
            Value::from_bool(false)
        );
        assert_eq!(
            BinOp::BitOr
                .eval(
                    Value::from_bool(true),
                    Value::from_bool(false),
                    ValueType::Bool
                )
                .unwrap(),
            Value::from_bool(true)
        );
        assert_eq!(
            BinOp::Eq
                .eval(
                    Value::from_bool(true),
                    Value::from_bool(true),
                    ValueType::Bool
                )
                .unwrap(),
            Value::from_bool(true)
        );
    }

    #[test]
    fn test_unary_operations() {
        assert_eq!(
            UnOp::Neg.eval(Value::from_i128(5), ValueType::Int).unwrap(),
            Value::from_i128(-5)
        );
        assert_eq!(
            UnOp::Not
                .eval(Value::from_bool(true), ValueType::Bool)
                .unwrap(),
            Value::from_bool(false)
        );
    }
}
