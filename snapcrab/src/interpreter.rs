use rustc_public::mir::{BasicBlockIdx, StatementKind, TerminatorKind, Operand, Place, BinOp, UnOp, Rvalue};
use rustc_public::mir::mono::Instance;
use rustc_public::ty::MirConst;
use anyhow::{Result, bail};
use tracing::{debug, info};
use crate::stack::{StackFrame, Value};
use crate::heap::Heap;

pub struct FnInterpreter {
    frame: StackFrame,
    current_block: BasicBlockIdx,
}

impl FnInterpreter {
    pub fn new() -> Self {
        Self {
            frame: Vec::new(),
            current_block: 0,
        }
    }

    pub fn run(&mut self, instance: Instance, _heap: &mut Heap) -> Result<Value> {
        let body = instance.body().ok_or_else(|| anyhow::anyhow!("No body for instance"))?;
        info!("Starting interpretation of {}", instance.name());

        // Initialize frame with space for all locals
        let frame_size = body.locals().len();
        self.frame = vec![None; frame_size];

        loop {
            let block = &body.blocks[self.current_block];
            debug!("Executing block {}", self.current_block);

            // Execute statements
            for statement in &block.statements {
                self.execute_statement(statement)?;
            }

            // Execute terminator
            match self.execute_terminator(&block.terminator)? {
                ControlFlow::Continue(next_block) => {
                    self.current_block = next_block;
                }
                ControlFlow::Return(value) => {
                    info!("Function returned with value: {:?}", value);
                    return Ok(value);
                }
            }
        }
    }

    fn execute_statement(&mut self, statement: &rustc_public::mir::Statement) -> Result<()> {
        debug!("Executing statement: {:?}", statement.kind);
        
        match &statement.kind {
            StatementKind::Assign(place, rvalue) => {
                let value = self.evaluate_rvalue(rvalue)?;
                self.assign_to_place(place, value)?;
            }
            StatementKind::StorageLive(_) | StatementKind::StorageDead(_) => {
                // Ignore storage annotations for now
            }
            StatementKind::Nop => {
                // Do nothing
            }
            _ => {
                bail!("Unsupported statement: {:?}", statement.kind);
            }
        }
        Ok(())
    }

    fn execute_terminator(&mut self, terminator: &rustc_public::mir::Terminator) -> Result<ControlFlow> {
        debug!("Executing terminator: {:?}", terminator.kind);
        
        match &terminator.kind {
            TerminatorKind::Return => {
                // Return the value from local 0 (return value)
                let return_value = self.read_from_place(&Place::from(0))?;
                Ok(ControlFlow::Return(return_value))
            }
            TerminatorKind::Goto { target } => {
                Ok(ControlFlow::Continue(*target))
            }
            TerminatorKind::SwitchInt { discr, targets } => {
                let discr_value = self.evaluate_operand(discr)?;
                let discr_int = match discr_value {
                    Value::Int(i) => i as u128,
                    Value::Uint(u) => u,
                    Value::Bool(b) => if b { 1 } else { 0 },
                    _ => bail!("Cannot switch on non-integer value: {:?}", discr_value),
                };
                
                // Find the target for this value
                let target = targets.branches()
                    .find(|(value, _)| *value == discr_int)
                    .map(|(_, target)| target)
                    .unwrap_or_else(|| targets.otherwise());
                    
                Ok(ControlFlow::Continue(target))
            }
            _ => {
                bail!("Unsupported terminator: {:?}", terminator.kind);
            }
        }
    }

    fn evaluate_rvalue(&mut self, rvalue: &Rvalue) -> Result<Value> {
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

    fn evaluate_operand(&self, operand: &Operand) -> Result<Value> {
        match operand {
            Operand::Copy(place) | Operand::Move(place) => {
                self.read_from_place(place)
            }
            Operand::Constant(const_op) => {
                self.evaluate_constant(&const_op.const_)
            }
        }
    }

    fn evaluate_binary_op(&self, op: BinOp, left: Value, right: Value) -> Result<Value> {
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

    fn evaluate_unary_op(&self, op: UnOp, operand: Value) -> Result<Value> {
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

    fn evaluate_constant(&self, const_: &MirConst) -> Result<Value> {
        match const_.kind() {
            rustc_public::ty::ConstantKind::Allocated(alloc) => {
                let bytes = alloc.raw_bytes()?;
                // Use the MIR type info to determine signed vs unsigned
                match const_.ty().kind() {
                    rustc_public::ty::TyKind::RigidTy(rustc_public::ty::RigidTy::Int(_)) => {
                        let val = match bytes.len() {
                            1 => i8::from_le_bytes([bytes[0]]) as i128,
                            2 => i16::from_le_bytes([bytes[0], bytes[1]]) as i128,
                            4 => i32::from_le_bytes(bytes.try_into().unwrap()) as i128,
                            8 => i64::from_le_bytes(bytes.try_into().unwrap()) as i128,
                            16 => i128::from_le_bytes(bytes.try_into().unwrap()),
                            _ => bail!("Unsupported int size: {}", bytes.len()),
                        };
                        Ok(Value::Int(val))
                    }
                    rustc_public::ty::TyKind::RigidTy(rustc_public::ty::RigidTy::Uint(_)) => {
                        let val = match bytes.len() {
                            1 => bytes[0] as u128,
                            2 => u16::from_le_bytes([bytes[0], bytes[1]]) as u128,
                            4 => u32::from_le_bytes(bytes.try_into().unwrap()) as u128,
                            8 => u64::from_le_bytes(bytes.try_into().unwrap()) as u128,
                            16 => u128::from_le_bytes(bytes.try_into().unwrap()),
                            _ => bail!("Unsupported uint size: {}", bytes.len()),
                        };
                        Ok(Value::Uint(val))
                    }
                    rustc_public::ty::TyKind::RigidTy(rustc_public::ty::RigidTy::Bool) => {
                        Ok(Value::Bool(bytes[0] != 0))
                    }
                    _ => bail!("Unsupported constant type: {:?}", const_.ty()),
                }
            }
            rustc_public::ty::ConstantKind::ZeroSized => {
                Ok(Value::Unit)
            }
            rustc_public::ty::ConstantKind::Ty(ty_const) => {
                bail!("Unsupported type constant: {:?}", ty_const);
            }
            rustc_public::ty::ConstantKind::Param(_) => {
                bail!("Parameter constants not supported");
            }
            rustc_public::ty::ConstantKind::Unevaluated(_) => {
                bail!("Unevaluated constants not supported");
            }
        }
    }

    fn assign_to_place(&mut self, place: &Place, value: Value) -> Result<()> {
        if !place.projection.is_empty() {
            bail!("Place projections not yet supported");
        }
        
        debug!("Assigning {:?} to local {}", value, place.local);
        
        if place.local >= self.frame.len() {
            bail!("Local index {} out of bounds", place.local);
        }
        
        self.frame[place.local] = Some(value);
        Ok(())
    }

    fn read_from_place(&self, place: &Place) -> Result<Value> {
        if !place.projection.is_empty() {
            bail!("Place projections not yet supported");
        }
        
        if place.local >= self.frame.len() {
            bail!("Local index {} out of bounds", place.local);
        }
        
        self.frame[place.local]
            .ok_or_else(|| anyhow::anyhow!("Uninitialized local: {}", place.local))
    }
}

#[derive(Debug)]
enum ControlFlow {
    Continue(BasicBlockIdx),
    Return(Value),
}
