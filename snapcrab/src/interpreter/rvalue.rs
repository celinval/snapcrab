use crate::stack::Value;
use anyhow::{Result, bail};
use rustc_public::mir::{BinOp, Rvalue, UnOp};

impl super::function::FnInterpreter {
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
                op.eval(left_val, right_val)
            }
            Rvalue::UnaryOp(op, operand) => {
                let val = self.evaluate_operand(operand)?;
                op.eval(val)
            }
            Rvalue::Use(operand) => self.evaluate_operand(operand),
            _ => {
                bail!("Unsupported rvalue: {:?}", rvalue);
            }
        }
    }
}

/// Trait for evaluating binary operations on values.
pub trait BinaryEval {
    /// Evaluates a binary operation on two values.
    ///
    /// # Arguments
    /// * `left` - Left operand value
    /// * `right` - Right operand value
    ///
    /// # Returns
    /// * `Ok(Value)` - Result of the operation
    /// * `Err(anyhow::Error)` - If operation fails or is unsupported
    fn eval(&self, left: Value, right: Value) -> Result<Value>;
}

/// Trait for evaluating unary operations on values.
pub trait UnaryEval {
    /// Evaluates a unary operation on a value.
    ///
    /// # Arguments
    /// * `operand` - The value to operate on
    ///
    /// # Returns
    /// * `Ok(Value)` - Result of the operation
    /// * `Err(anyhow::Error)` - If operation fails or is unsupported
    fn eval(&self, operand: Value) -> Result<Value>;
}

impl BinaryEval for BinOp {
    fn eval(&self, left: Value, right: Value) -> Result<Value> {
        match (left, right) {
            (Value::Int(l), Value::Int(r)) => eval_int_binop(*self, l, r),
            (Value::Uint(l), Value::Uint(r)) => eval_uint_binop(*self, l, r),
            (Value::Bool(l), Value::Bool(r)) => eval_bool_binop(*self, l, r),
            _ => bail!(
                "Type mismatch in binary operation: {:?} on {:?} and {:?}",
                self,
                left,
                right
            ),
        }
    }
}

impl UnaryEval for UnOp {
    fn eval(&self, operand: Value) -> Result<Value> {
        match operand {
            Value::Int(val) => eval_int_unop(*self, val),
            Value::Bool(val) => eval_bool_unop(*self, val),
            _ => bail!("Unsupported unary operation: {:?} on {:?}", self, operand),
        }
    }
}

/// Evaluates a binary operation on signed integers.
fn eval_int_binop(op: BinOp, left: i128, right: i128) -> Result<Value> {
    match op {
        BinOp::Add => left
            .checked_add(right)
            .map(Value::Int)
            .ok_or_else(|| anyhow::anyhow!("Integer overflow in addition")),
        BinOp::Sub => left
            .checked_sub(right)
            .map(Value::Int)
            .ok_or_else(|| anyhow::anyhow!("Integer overflow in subtraction")),
        BinOp::Mul => left
            .checked_mul(right)
            .map(Value::Int)
            .ok_or_else(|| anyhow::anyhow!("Integer overflow in multiplication")),
        BinOp::Div => {
            if right == 0 {
                bail!("Division by zero");
            }
            left.checked_div(right)
                .map(Value::Int)
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
            .map(Value::Uint)
            .ok_or_else(|| anyhow::anyhow!("Integer overflow in addition")),
        BinOp::Sub => left
            .checked_sub(right)
            .map(Value::Uint)
            .ok_or_else(|| anyhow::anyhow!("Integer overflow in subtraction")),
        BinOp::Mul => left
            .checked_mul(right)
            .map(Value::Uint)
            .ok_or_else(|| anyhow::anyhow!("Integer overflow in multiplication")),
        BinOp::Div => {
            if right == 0 {
                bail!("Division by zero");
            }
            left.checked_div(right)
                .map(Value::Uint)
                .ok_or_else(|| anyhow::anyhow!("Integer overflow in division"))
        }
        _ => bail!("Unsupported unsigned integer binary operation: {:?}", op),
    }
}

/// Evaluates a binary operation on boolean values.
fn eval_bool_binop(op: BinOp, left: bool, right: bool) -> Result<Value> {
    match op {
        BinOp::BitAnd => Ok(Value::Bool(left & right)),
        BinOp::BitOr => Ok(Value::Bool(left | right)),
        BinOp::Eq => Ok(Value::Bool(left == right)),
        BinOp::Ne => Ok(Value::Bool(left != right)),
        _ => bail!("Unsupported boolean binary operation: {:?}", op),
    }
}

/// Evaluates a unary operation on a signed integer.
fn eval_int_unop(op: UnOp, val: i128) -> Result<Value> {
    match op {
        UnOp::Neg => val
            .checked_neg()
            .map(Value::Int)
            .ok_or_else(|| anyhow::anyhow!("Integer overflow in negation")),
        _ => bail!("Unsupported integer unary operation: {:?}", op),
    }
}

/// Evaluates a unary operation on a boolean value.
fn eval_bool_unop(op: UnOp, val: bool) -> Result<Value> {
    match op {
        UnOp::Not => Ok(Value::Bool(!val)),
        _ => bail!("Unsupported boolean unary operation: {:?}", op),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_binary_operations() {
        let result = BinOp::Add.eval(Value::Int(5), Value::Int(3)).unwrap();
        assert_eq!(result, Value::Int(8));
    }

    #[test]
    fn test_int_overflow() {
        let result = BinOp::Add.eval(Value::Int(i128::MAX), Value::Int(1));
        assert!(result.is_err());
    }

    #[test]
    fn test_uint_binary_operations() {
        assert_eq!(
            BinOp::Add.eval(Value::Uint(10), Value::Uint(5)).unwrap(),
            Value::Uint(15)
        );
        assert_eq!(
            BinOp::Sub.eval(Value::Uint(10), Value::Uint(5)).unwrap(),
            Value::Uint(5)
        );
        assert_eq!(
            BinOp::Mul.eval(Value::Uint(10), Value::Uint(5)).unwrap(),
            Value::Uint(50)
        );
        assert_eq!(
            BinOp::Div.eval(Value::Uint(10), Value::Uint(5)).unwrap(),
            Value::Uint(2)
        );
    }

    #[test]
    fn test_division_by_zero() {
        assert!(BinOp::Div.eval(Value::Int(10), Value::Int(0)).is_err());
        assert!(BinOp::Div.eval(Value::Uint(10), Value::Uint(0)).is_err());
    }

    #[test]
    fn test_bool_binary_operations() {
        assert_eq!(
            BinOp::BitAnd
                .eval(Value::Bool(true), Value::Bool(false))
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            BinOp::BitOr
                .eval(Value::Bool(true), Value::Bool(false))
                .unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            BinOp::Eq
                .eval(Value::Bool(true), Value::Bool(true))
                .unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn test_unary_operations() {
        assert_eq!(UnOp::Neg.eval(Value::Int(5)).unwrap(), Value::Int(-5));
        assert_eq!(
            UnOp::Not.eval(Value::Bool(true)).unwrap(),
            Value::Bool(false)
        );
    }
}
