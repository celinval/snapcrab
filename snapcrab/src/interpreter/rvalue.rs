use rustc_public::mir::{BinOp, UnOp, Rvalue};
use anyhow::{Result, bail};
use crate::stack::Value;

impl super::function::FnInterpreter {
    pub(super) fn evaluate_rvalue(&mut self, rvalue: &Rvalue) -> Result<Value> {
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
            Rvalue::Use(operand) => {
                self.evaluate_operand(operand)
            }
            _ => {
                bail!("Unsupported rvalue: {:?}", rvalue);
            }
        }
    }

    pub(super) fn evaluate_binary_op(&self, op: BinOp, left: Value, right: Value) -> Result<Value> {
        match (left, right) {
            (Value::Int(l), Value::Int(r)) => self.eval_int_binop(op, l, r),
            (Value::Uint(l), Value::Uint(r)) => self.eval_uint_binop(op, l, r),
            (Value::Bool(l), Value::Bool(r)) => self.eval_bool_binop(op, l, r),
            _ => bail!("Type mismatch in binary operation: {:?} on {:?} and {:?}", op, left, right),
        }
    }

    fn eval_int_binop(&self, op: BinOp, left: i128, right: i128) -> Result<Value> {
        match op {
            BinOp::Add => left.checked_add(right)
                .map(Value::Int)
                .ok_or_else(|| anyhow::anyhow!("Integer overflow in addition")),
            BinOp::Sub => left.checked_sub(right)
                .map(Value::Int)
                .ok_or_else(|| anyhow::anyhow!("Integer overflow in subtraction")),
            BinOp::Mul => left.checked_mul(right)
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

    fn eval_uint_binop(&self, op: BinOp, left: u128, right: u128) -> Result<Value> {
        match op {
            BinOp::Add => left.checked_add(right)
                .map(Value::Uint)
                .ok_or_else(|| anyhow::anyhow!("Integer overflow in addition")),
            BinOp::Sub => left.checked_sub(right)
                .map(Value::Uint)
                .ok_or_else(|| anyhow::anyhow!("Integer overflow in subtraction")),
            BinOp::Mul => left.checked_mul(right)
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

    pub(super) fn evaluate_unary_op(&self, op: UnOp, operand: Value) -> Result<Value> {
        match operand {
            Value::Int(val) => self.eval_int_unop(op, val),
            Value::Uint(val) => self.eval_uint_unop(op, val),
            Value::Bool(val) => self.eval_bool_unop(op, val),
            _ => bail!("Unsupported unary operation: {:?} on {:?}", op, operand),
        }
    }

    fn eval_int_unop(&self, op: UnOp, val: i128) -> Result<Value> {
        match op {
            UnOp::Neg => val.checked_neg()
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

    fn eval_bool_unop(&self, op: UnOp, val: bool) -> Result<Value> {
        match op {
            UnOp::Not => Ok(Value::Bool(!val)),
            _ => bail!("Unsupported boolean unary operation: {:?}", op),
        }
    }
}
