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
                self.evaluate_binary_op(*op, left_val, right_val)
            }
            Rvalue::UnaryOp(op, operand) => {
                let val = self.evaluate_operand(operand)?;
                self.evaluate_unary_op(*op, val)
            }
            Rvalue::Use(operand) => self.evaluate_operand(operand),
            _ => {
                bail!("Unsupported rvalue: {:?}", rvalue);
            }
        }
    }

    /// Evaluates a binary operation on two values.
    ///
    /// # Arguments
    /// * `op` - The binary operation to perform
    /// * `left` - Left operand value
    /// * `right` - Right operand value
    ///
    /// # Returns
    /// * `Ok(Value)` - Result of the operation
    /// * `Err(anyhow::Error)` - If types don't match or operation fails
    pub(super) fn evaluate_binary_op(&self, op: BinOp, left: Value, right: Value) -> Result<Value> {
        match (left, right) {
            (Value::Int(l), Value::Int(r)) => self.eval_int_binop(op, l, r),
            (Value::Uint(l), Value::Uint(r)) => self.eval_uint_binop(op, l, r),
            (Value::Bool(l), Value::Bool(r)) => self.eval_bool_binop(op, l, r),
            _ => bail!(
                "Type mismatch in binary operation: {:?} on {:?} and {:?}",
                op,
                left,
                right
            ),
        }
    }

    /// Evaluates a binary operation on signed integers.
    ///
    /// # Arguments
    /// * `op` - The binary operation to perform
    /// * `left` - Left integer operand
    /// * `right` - Right integer operand
    ///
    /// # Returns
    /// * `Ok(Value::Int)` - Result of the operation
    /// * `Err(anyhow::Error)` - If operation overflows or is unsupported
    fn eval_int_binop(&self, op: BinOp, left: i128, right: i128) -> Result<Value> {
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
            BinOp::AddUnchecked => Ok(Value::Int(left.wrapping_add(right))),
            BinOp::Eq => Ok(Value::Bool(left == right)),
            BinOp::Lt => Ok(Value::Bool(left < right)),
            BinOp::Le => Ok(Value::Bool(left <= right)),
            BinOp::Ne => Ok(Value::Bool(left != right)),
            BinOp::Ge => Ok(Value::Bool(left >= right)),
            BinOp::Gt => Ok(Value::Bool(left > right)),
            _ => bail!("Unsupported integer operation: {:?}", op),
        }
    }

    /// Evaluates a binary operation on unsigned integers.
    ///
    /// # Arguments
    /// * `op` - The binary operation to perform
    /// * `left` - Left unsigned integer operand
    /// * `right` - Right unsigned integer operand
    ///
    /// # Returns
    /// * `Ok(Value::Uint)` - Result of the operation
    /// * `Err(anyhow::Error)` - If operation overflows or is unsupported
    fn eval_uint_binop(&self, op: BinOp, left: u128, right: u128) -> Result<Value> {
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
                Ok(Value::Uint(left / right))
            }
            BinOp::AddUnchecked => Ok(Value::Uint(left.wrapping_add(right))),
            BinOp::Eq => Ok(Value::Bool(left == right)),
            BinOp::Lt => Ok(Value::Bool(left < right)),
            BinOp::Le => Ok(Value::Bool(left <= right)),
            BinOp::Ne => Ok(Value::Bool(left != right)),
            BinOp::Ge => Ok(Value::Bool(left >= right)),
            BinOp::Gt => Ok(Value::Bool(left > right)),
            _ => bail!("Unsupported unsigned integer operation: {:?}", op),
        }
    }

    /// Evaluates a binary operation on boolean values.
    ///
    /// # Arguments
    /// * `op` - The binary operation to perform
    /// * `left` - Left boolean operand
    /// * `right` - Right boolean operand
    ///
    /// # Returns
    /// * `Ok(Value::Bool)` - Result of the operation
    /// * `Err(anyhow::Error)` - If operation is unsupported for booleans
    fn eval_bool_binop(&self, op: BinOp, left: bool, right: bool) -> Result<Value> {
        match op {
            BinOp::BitAnd => Ok(Value::Bool(left & right)),
            BinOp::BitOr => Ok(Value::Bool(left | right)),
            BinOp::BitXor => Ok(Value::Bool(left ^ right)),
            BinOp::Eq => Ok(Value::Bool(left == right)),
            BinOp::Ne => Ok(Value::Bool(left != right)),
            _ => bail!("Unsupported boolean operation: {:?}", op),
        }
    }

    /// Evaluates a unary operation on a value.
    ///
    /// # Arguments
    /// * `op` - The unary operation to perform
    /// * `operand` - The value to operate on
    ///
    /// # Returns
    /// * `Ok(Value)` - Result of the operation
    /// * `Err(anyhow::Error)` - If operation fails or type is unsupported
    pub(super) fn evaluate_unary_op(&self, op: UnOp, operand: Value) -> Result<Value> {
        match operand {
            Value::Int(val) => self.eval_int_unop(op, val),
            Value::Uint(val) => self.eval_uint_unop(op, val),
            Value::Bool(val) => self.eval_bool_unop(op, val),
            _ => bail!("Unsupported unary operation: {:?} on {:?}", op, operand),
        }
    }

    /// Evaluates a unary operation on a signed integer.
    ///
    /// # Arguments
    /// * `op` - The unary operation to perform
    /// * `val` - The integer value to operate on
    ///
    /// # Returns
    /// * `Ok(Value::Int)` - Result of the operation
    /// * `Err(anyhow::Error)` - If operation overflows or is unsupported
    fn eval_int_unop(&self, op: UnOp, val: i128) -> Result<Value> {
        match op {
            UnOp::Neg => val
                .checked_neg()
                .map(Value::Int)
                .ok_or_else(|| anyhow::anyhow!("Integer overflow in negation")),
            UnOp::Not => Ok(Value::Int(!val)),
            _ => bail!("Unsupported integer unary operation: {:?}", op),
        }
    }

    fn eval_uint_unop(&self, op: UnOp, val: u128) -> Result<Value> {
        match op {
            UnOp::Not => Ok(Value::Uint(!val)),
            UnOp::Neg => bail!("Cannot negate unsigned integer"),
            _ => bail!("Unsupported unsigned integer unary operation: {:?}", op),
        }
    }

    /// Evaluates a unary operation on a boolean value.
    ///
    /// # Arguments
    /// * `op` - The unary operation to perform
    /// * `val` - The boolean value to operate on
    ///
    /// # Returns
    /// * `Ok(Value::Bool)` - Result of the operation
    /// * `Err(anyhow::Error)` - If operation is unsupported for booleans
    fn eval_bool_unop(&self, op: UnOp, val: bool) -> Result<Value> {
        match op {
            UnOp::Not => Ok(Value::Bool(!val)),
            _ => bail!("Unsupported boolean unary operation: {:?}", op),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::function::FnInterpreter;
    use rustc_public::mir::BinOp;

    #[test]
    fn test_int_addition() {
        let interpreter = FnInterpreter::new();
        let result = interpreter.eval_int_binop(BinOp::Add, 5, 3).unwrap();
        assert_eq!(result, Value::Int(8));
    }

    #[test]
    fn test_int_overflow() {
        let interpreter = FnInterpreter::new();
        let result = interpreter.eval_int_binop(BinOp::Add, i128::MAX, 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_uint_operations() {
        let interpreter = FnInterpreter::new();
        assert_eq!(
            interpreter.eval_uint_binop(BinOp::Add, 10, 5).unwrap(),
            Value::Uint(15)
        );
        assert_eq!(
            interpreter.eval_uint_binop(BinOp::Sub, 10, 5).unwrap(),
            Value::Uint(5)
        );
        assert_eq!(
            interpreter.eval_uint_binop(BinOp::Mul, 10, 5).unwrap(),
            Value::Uint(50)
        );
        assert_eq!(
            interpreter.eval_uint_binop(BinOp::Div, 10, 5).unwrap(),
            Value::Uint(2)
        );
    }

    #[test]
    fn test_division_by_zero() {
        let interpreter = FnInterpreter::new();
        assert!(interpreter.eval_int_binop(BinOp::Div, 10, 0).is_err());
        assert!(interpreter.eval_uint_binop(BinOp::Div, 10, 0).is_err());
    }

    #[test]
    fn test_bool_operations() {
        let interpreter = FnInterpreter::new();
        assert_eq!(
            interpreter
                .eval_bool_binop(BinOp::BitAnd, true, false)
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            interpreter
                .eval_bool_binop(BinOp::BitOr, true, false)
                .unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            interpreter.eval_bool_binop(BinOp::Eq, true, true).unwrap(),
            Value::Bool(true)
        );
    }
}
