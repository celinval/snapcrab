use crate::heap::Heap;
use crate::stack::{StackFrame, Value};
use anyhow::{Result, bail};
use rustc_public::mir::mono::Instance;
use rustc_public::mir::{BasicBlockIdx, Body, Operand, Place, StatementKind, TerminatorKind};
use rustc_public::ty::{ConstantKind, MirConst, RigidTy, TyKind};
use tracing::{debug, info};

/// Function interpreter that executes MIR (Mid-level Intermediate Representation) code.
///
/// The interpreter maintains a stack frame for local variables and executes basic blocks
/// sequentially, handling statements and terminators to implement control flow.
#[derive(Debug)]
pub struct FnInterpreter {
    /// Stack frame containing local variable values
    frame: StackFrame,
    /// Index of the currently executing basic block
    current_block: BasicBlockIdx,
    /// Function instance being interpreted
    instance: Instance,
    /// MIR body containing the function's basic blocks and metadata
    body: Body,
}

impl FnInterpreter {
    /// Creates a new function interpreter for the given instance.
    ///
    /// # Arguments
    /// * `instance` - The function instance to interpret
    ///
    /// # Returns
    /// * `Ok(FnInterpreter)` - Successfully created interpreter
    /// * `Err(anyhow::Error)` - If the instance has no body
    pub fn new(instance: Instance) -> Result<Self> {
        let body = instance
            .body()
            .ok_or_else(|| anyhow::anyhow!("No body for instance"))?;

        let frame_size = body.locals().len();
        Ok(Self {
            frame: vec![None; frame_size],
            current_block: 0,
            instance,
            body,
        })
    }

    /// Executes the function by interpreting its MIR basic blocks.
    ///
    /// Consumes the interpreter and runs until the function returns or an error occurs.
    ///
    /// # Arguments
    /// * `_heap` - Heap for dynamic memory allocation (currently unused)
    ///
    /// # Returns
    /// * `Ok(Value)` - The return value of the function
    /// * `Err(anyhow::Error)` - If execution fails
    pub fn run(mut self, heap: &mut Heap, args: Vec<Value>) -> Result<Value> {
        info!("Starting interpretation of {}", self.instance.name());

        // Ensure argument count matches expected
        debug_assert_eq!(
            args.len(),
            self.body.arg_locals().len(),
            "Argument count mismatch: expected {}, got {}",
            self.body.arg_locals().len(),
            args.len()
        );

        // Initialize arguments in locals (skip local 0 which is return value)
        for (i, arg) in args.into_iter().enumerate() {
            self.frame[i + 1] = Some(arg);
        }

        loop {
            let current_block_idx = self.current_block;
            let stmt_count = self.body.blocks[current_block_idx].statements.len();
            debug!("Executing block {}", current_block_idx);

            // Execute statements
            for stmt_idx in 0..stmt_count {
                self.execute_statement(current_block_idx, stmt_idx)?;
            }

            // Execute terminator
            match self.execute_terminator(current_block_idx, heap)? {
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

    /// Get the local declarations for type checking
    pub(super) fn locals(&self) -> &[rustc_public::mir::LocalDecl] {
        self.body.locals()
    }

    /// Executes a single statement within a basic block.
    ///
    /// # Arguments
    /// * `bb_idx` - Index of the basic block containing the statement
    /// * `stmt_idx` - Index of the statement within the basic block
    ///
    /// # Returns
    /// * `Ok(())` - Statement executed successfully
    /// * `Err(anyhow::Error)` - If statement execution fails
    fn execute_statement(&mut self, bb_idx: BasicBlockIdx, stmt_idx: usize) -> Result<()> {
        let statement_kind = self.body.blocks[bb_idx].statements[stmt_idx].kind.clone();
        debug!("Executing statement: {:?}", statement_kind);

        match &statement_kind {
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
                bail!("Unsupported statement: {:?}", statement_kind);
            }
        }
        Ok(())
    }

    /// Executes a terminator instruction that ends a basic block.
    ///
    /// # Arguments
    /// * `bb_idx` - Index of the basic block containing the terminator
    ///
    /// # Returns
    /// * `Ok(ControlFlow::Continue(target))` - Continue to target basic block
    /// * `Ok(ControlFlow::Return(value))` - Function returns with value
    /// * `Err(anyhow::Error)` - If terminator execution fails
    fn execute_terminator(
        &mut self,
        bb_idx: BasicBlockIdx,
        heap: &mut Heap,
    ) -> Result<ControlFlow> {
        let terminator = &self.body.blocks[bb_idx].terminator;
        debug!("Executing terminator: {:?}", terminator.kind);

        match terminator.kind.clone() {
            TerminatorKind::Return => {
                // Return the value from local 0 (return value)
                let return_value = self.read_from_place(&Place::from(0))?.clone();
                Ok(ControlFlow::Return(return_value))
            }
            TerminatorKind::Goto { target } => Ok(ControlFlow::Continue(target)),
            TerminatorKind::SwitchInt { discr, targets } => {
                let discr_value = self.evaluate_operand(&discr)?;
                let discr_int = match discr_value {
                    val if val.as_i128().is_some() => val.as_i128().unwrap() as u128,
                    val if val.as_u128().is_some() => val.as_u128().unwrap(),
                    val if val.as_bool().is_some() => {
                        if val.as_bool().unwrap() {
                            1
                        } else {
                            0
                        }
                    }
                    _ => bail!("Cannot switch on non-integer value: {:?}", discr_value),
                };

                // Find the target for this value
                let target = targets
                    .branches()
                    .find(|(value, _)| *value == discr_int)
                    .map(|(_, target)| target)
                    .unwrap_or_else(|| targets.otherwise());

                Ok(ControlFlow::Continue(target))
            }
            TerminatorKind::Call {
                func,
                args,
                destination,
                target,
                ..
            } => {
                self.execute_call(&func, &args, &destination, heap)?;

                match target {
                    Some(target_bb) => Ok(ControlFlow::Continue(target_bb)),
                    None => bail!("Diverging calls not yet supported"),
                }
            }
            _ => {
                bail!("Unsupported terminator: {:?}", terminator.kind);
            }
        }
    }

    /// Execute a function call
    fn execute_call(
        &mut self,
        func: &Operand,
        args: &[Operand],
        destination: &Place,
        heap: &mut Heap,
    ) -> Result<()> {
        // Evaluate arguments
        let arg_values: Result<Vec<Value>> =
            args.iter().map(|arg| self.evaluate_operand(arg)).collect();
        let arg_values = arg_values?;

        // Resolve function instance
        let func_instance = match func {
            Operand::Constant(const_op) => {
                // Extract instance from constant type
                let func_ty = const_op.ty();
                match func_ty.kind() {
                    TyKind::RigidTy(RigidTy::FnDef(def_id, args)) => {
                        Instance::resolve(def_id, &args)?
                    }
                    _ => bail!("Unsupported function type: {:?}", func_ty),
                }
            }
            _ => bail!("Only constant function operands supported"),
        };

        // Create new interpreter and call function
        let callee_interpreter = FnInterpreter::new(func_instance)?;
        let result = callee_interpreter.run(heap, arg_values)?;

        // Store result in destination
        self.assign_to_place(destination, result)?;

        Ok(())
    }

    /// Evaluates an operand to produce a value.
    ///
    /// # Arguments
    /// * `operand` - The operand to evaluate (copy, move, or constant)
    ///
    /// # Returns
    /// * `Ok(Value)` - The evaluated value
    /// * `Err(anyhow::Error)` - If evaluation fails
    pub(super) fn evaluate_operand(&self, operand: &Operand) -> Result<Value> {
        match operand {
            Operand::Copy(place) | Operand::Move(place) => Ok(self.read_from_place(place)?.clone()),
            Operand::Constant(const_op) => self.evaluate_constant(&const_op.const_),
        }
    }

    /// Evaluates a MIR constant to produce a runtime value.
    ///
    /// # Arguments
    /// * `const_` - The MIR constant to evaluate
    ///
    /// # Returns
    /// * `Ok(Value)` - The constant value
    /// * `Err(anyhow::Error)` - If constant evaluation fails or type is unsupported
    fn evaluate_constant(&self, const_: &MirConst) -> Result<Value> {
        match const_.kind() {
            ConstantKind::Allocated(alloc) => {
                let bytes = alloc.raw_bytes()?;
                // Use the MIR type info to determine signed vs unsigned
                match const_.ty().kind() {
                    TyKind::RigidTy(RigidTy::Int(_)) => {
                        let val = match bytes.len() {
                            1 => i8::from_le_bytes([bytes[0]]) as i128,
                            2 => i16::from_le_bytes([bytes[0], bytes[1]]) as i128,
                            4 => i32::from_le_bytes(bytes.try_into().unwrap()) as i128,
                            8 => i64::from_le_bytes(bytes.try_into().unwrap()) as i128,
                            16 => i128::from_le_bytes(bytes.try_into().unwrap()),
                            _ => bail!("Unsupported int size: {}", bytes.len()),
                        };
                        Ok(Value::from_i128(val))
                    }
                    TyKind::RigidTy(RigidTy::Uint(_)) => {
                        let val = match bytes.len() {
                            1 => bytes[0] as u128,
                            2 => u16::from_le_bytes([bytes[0], bytes[1]]) as u128,
                            4 => u32::from_le_bytes(bytes.try_into().unwrap()) as u128,
                            8 => u64::from_le_bytes(bytes.try_into().unwrap()) as u128,
                            16 => u128::from_le_bytes(bytes.try_into().unwrap()),
                            _ => bail!("Unsupported uint size: {}", bytes.len()),
                        };
                        Ok(Value::from_u128(val))
                    }
                    TyKind::RigidTy(RigidTy::Bool) => Ok(Value::from_bool(bytes[0] != 0)),
                    _ => bail!("Unsupported constant type: {:?}", const_.ty()),
                }
            }
            ConstantKind::ZeroSized => Ok(Value::unit().clone()),
            ConstantKind::Ty(ty_const) => {
                bail!("Unsupported type constant: {:?}", ty_const);
            }
            ConstantKind::Param(_) => {
                bail!("Parameter constants not supported");
            }
            ConstantKind::Unevaluated(_) => {
                bail!("Unexpected unevaluated constants on instance body");
            }
        }
    }

    /// Assigns a value to a place (local variable or memory location).
    ///
    /// # Arguments
    /// * `place` - The place to assign to
    /// * `value` - The value to assign
    ///
    /// # Returns
    /// * `Ok(())` - Assignment successful
    /// * `Err(anyhow::Error)` - If assignment fails (e.g., out of bounds)
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

    /// Reads a value from a place (local variable or memory location).
    ///
    /// For zero-sized types (like unit type `()`), returns `Value::unit()` even if
    /// the local is uninitialized, since zero-sized values don't need storage.
    ///
    /// # Arguments
    /// * `place` - The place to read from
    ///
    /// # Returns
    /// * `Ok(&Value)` - Reference to the value at the place
    /// * `Err(anyhow::Error)` - If place is uninitialized or out of bounds
    fn read_from_place(&self, place: &Place) -> Result<&Value> {
        if !place.projection.is_empty() {
            bail!("Place projections not yet supported");
        }

        if place.local >= self.frame.len() {
            bail!("Local index {} out of bounds", place.local);
        }

        // Check if this is a zero-sized type
        let local_ty = self.body.locals()[place.local].ty;
        if matches!(local_ty.kind(), TyKind::RigidTy(RigidTy::Tuple(fields)) if fields.is_empty()) {
            // For zero-sized types, return a reference to the unit value
            return Ok(Value::unit());
        }

        self.frame[place.local]
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Uninitialized local: {}", place.local))
    }
}

/// Control flow result from executing a terminator instruction.
#[derive(Debug)]
pub enum ControlFlow {
    /// Continue execution at the specified basic block
    Continue(BasicBlockIdx),
    /// Return from the function with the given value
    Return(Value),
}
