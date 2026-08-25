use std::thread;

use crate::memory::ThreadMemory;
use crate::ty::MonoType;
use crate::value::Value;
use anyhow::{Context, Result, anyhow, bail};
use rustc_public::mir::mono::{Instance, InstanceKind};
use rustc_public::mir::{
    BasicBlockIdx, Body, Mutability, Operand, Place, StatementKind, TerminatorKind,
};
use rustc_public::ty::{
    Abi, ClosureKind, ConstantKind, GenericArgKind, MirConst, RigidTy, Ty, TyKind,
};
use tracing::{debug, info};

use super::rvalue::write_discriminant;

/// Function interpreter that executes MIR (Mid-level Intermediate Representation) code.
///
/// The interpreter maintains a stack frame for local variables and executes basic blocks
/// sequentially, handling statements and terminators to implement control flow.
pub struct FnInterpreter<'a> {
    /// The memory accessible to the interpreter
    pub(super) memory: &'a mut ThreadMemory,
    /// Index of the currently executing basic block
    current_block: BasicBlockIdx,
    /// Function instance being interpreted
    instance: Instance,
    /// MIR body containing the function's basic blocks and metadata
    body: &'a Body,
    /// The number a call is in the stack during the unwinding
    unwinding: &'a mut Option<u16>,
}

/// Run the interpreter for the given instance.
///
/// Uses a three-tier dispatch:
/// 1. If the function has a MIR body, interpret it
/// 2. If it's an intrinsic without a body, shim it
/// 3. Otherwise, call the native compiled version via symbol resolution
pub fn invoke_fn(
    instance: Instance,
    memory: &mut ThreadMemory,
    args: Vec<Value>,
    unwinding: &mut Option<u16>,
) -> Result<Value> {
    // Tier 1: interpret MIR body if available
    if instance.has_body() {
        return memory.with_stack_frame(instance, |body, memory| {
            let interpreter = FnInterpreter {
                memory,
                current_block: 0,
                instance,
                body,
                unwinding,
            };
            interpreter.execute(args)
        });
    }

    // Tier 2: intrinsic shims
    if let Some(intrinsic) = instance.intrinsic_name() {
        return super::intrinsics::eval_intrinsic(
            intrinsic.as_str(),
            &args,
            instance,
            &memory.check_config,
        );
    }

    // Detect implicit arguments (e.g., #[track_caller] passes &Location).
    let fn_abi = instance.fn_abi()?;
    if fn_abi.args.len() > args.len() {
        bail!(
            "Failed to invoke `{}`: function expects {} arguments but got {} \
             (implicit arguments like #[track_caller] are not yet supported)",
            instance.name(),
            fn_abi.args.len(),
            args.len()
        );
    }

    // Tier 4: native call via dlsym
    let config = memory.check_config.clone();
    let jit = &memory.jit;
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        super::native::call_native(instance, &args, &config, jit)
    }))
    // Turn the panic payload into the source error, keeping its message, and
    // layer the call context on top.
    .map_err(|panic| anyhow!(panic_message(panic.as_ref())))
    .with_context(|| format!("Native call to `{}` panicked", instance.name()))?
}

/// Extract a human-readable message from a caught panic payload.
pub(crate) fn panic_message(panic: &dyn std::any::Any) -> String {
    if let Some(s) = panic.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = panic.downcast_ref::<String>() {
        s.clone()
    } else {
        "native function panicked".to_string()
    }
}

/// Resolve a closure call shim (`FnOnce/FnMut/Fn::call*`) to the closure body.
///
/// The shim's first generic argument is the closure type. A closure's body is
/// returned as an interpretable `Item` only when it is resolved at a kind it
/// directly implements: resolving a non-capturing `Fn` closure as `FnOnce`,
/// for instance, yields a forwarding shim, not the body. So try the kinds from
/// most to least capable and take the first that resolves to a real body.
pub(crate) fn resolve_closure_shim(shim: Instance) -> Result<Instance> {
    let first = shim
        .args()
        .0
        .first()
        .cloned()
        .with_context(|| format!("closure call shim `{}` has no generic args", shim.name()))?;
    let GenericArgKind::Type(ty) = first else {
        bail!(
            "closure call shim `{}` has an unexpected first arg: {first:?}",
            shim.name()
        );
    };
    let TyKind::RigidTy(RigidTy::Closure(def, args)) = ty.kind() else {
        bail!("`{}` is not a closure call shim (type {ty:?})", shim.name());
    };

    for kind in [ClosureKind::Fn, ClosureKind::FnMut, ClosureKind::FnOnce] {
        // A kind the closure does not directly implement resolves to a shim or
        // an error; skip those and keep looking for the real body.
        if let Ok(closure) = Instance::resolve_closure(def, &args, kind)
            && closure.kind != InstanceKind::Shim
            && closure.has_body()
        {
            return Ok(closure);
        }
    }
    bail!(
        "could not resolve `{}` to an interpretable closure body",
        shim.name()
    );
}

impl FnInterpreter<'_> {
    /// Executes the function by interpreting its MIR basic blocks.
    ///
    /// Consumes the interpreter and runs until the function returns or an error occurs.
    ///
    /// # Arguments
    /// * args: The arguments to the function
    ///
    /// # Returns
    /// * `Ok(Value)` - The return value of the function
    /// * `Err(anyhow::Error)` - If execution fails
    pub fn execute(mut self, args: Vec<Value>) -> Result<Value> {
        info!("Starting interpretation of {}", self.instance.name());

        // The caller must supply exactly the arguments the body expects (ABI
        // normalization such as rust-call untupling happens at the call site).
        // A mismatch is an interpreter bug, not a user error, so assert rather
        // than returning a recoverable error.
        assert_eq!(
            args.len(),
            self.body.arg_locals().len(),
            "Argument count mismatch invoking `{}`: expected {}, got {}",
            self.instance.name(),
            self.body.arg_locals().len(),
            args.len()
        );

        // Initialize arguments in locals (skip local 0 which is return value)
        for (i, arg) in args.into_iter().enumerate() {
            self.memory.write_local(i + 1, arg)?;
        }

        loop {
            let current_block_idx = self.current_block;
            let stmt_count = self.body.blocks[current_block_idx].statements.len();
            debug!("Executing block {}", current_block_idx);

            // Execute statements
            for stmt_idx in 0..stmt_count {
                self.execute_statement(current_block_idx, stmt_idx)
                    .map_err(|e| self.statement_error(current_block_idx, stmt_idx, e))?;
            }

            // Execute terminator
            match self
                .execute_terminator(current_block_idx)
                .map_err(|e| self.terminator_error(current_block_idx, e))?
            {
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

    /// Prints the error including the failing thread
    fn generate_error(
        &mut self,
        span: rustc_public::ty::Span,
        error: anyhow::Error,
    ) -> anyhow::Error {
        let include_backtrace = match std::env::var("RUST_BACKTRACE").as_deref() {
            Err(_) | Ok("0") => false,
            Ok(_) => true,
        };

        if !include_backtrace && self.unwinding.is_some() {
            // In compact mode and already unwinding
            return error;
        }

        let msg = if self.unwinding.is_none() {
            let tname = thread::current().name().map_or_else(
                || format!("thread_{:?}", thread::current().id()),
                str::to_string,
            );
            let span_info = span.diagnostic();
            let msg = format!("thread '{tname}' panicked at {span_info}\n{}", error);
            if std::env::var("RUST_BACKTRACE").is_err() {
                format!(
                    "{msg}\n note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace"
                )
            } else {
                msg
            }
        } else {
            error.to_string()
        };

        let current = *self.unwinding.get_or_insert_default();
        *self.unwinding = Some(current + 1);

        if include_backtrace {
            // Append stack trace
            let span_info = span.diagnostic();
            let func = self.instance.name();
            anyhow!("{msg}\n  {current}: at {func}\n      {span_info}")
        } else {
            anyhow::Error::msg(msg)
        }
    }

    /// Add context to statement execution errors
    fn statement_error(
        &mut self,
        bb_idx: BasicBlockIdx,
        stmt_idx: usize,
        error: anyhow::Error,
    ) -> anyhow::Error {
        assert!(self.unwinding.is_none()); // Unwinding should only be for terminator.
        let span = self.body.blocks[bb_idx].statements[stmt_idx].span;
        self.generate_error(span, error)
    }

    /// Add context to terminator execution errors
    fn terminator_error(&mut self, bb_idx: BasicBlockIdx, error: anyhow::Error) -> anyhow::Error {
        let span = self.body.blocks[bb_idx].terminator.span;
        self.generate_error(span, error)
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
            StatementKind::SetDiscriminant {
                place,
                variant_index,
            } => {
                let enum_ty = place.ty(self.locals())?;
                let addr = self.resolve_place_addr(place)?;
                let mut enum_val = self.memory.read_addr(addr, enum_ty)?;
                write_discriminant(enum_val.as_bytes_mut(), enum_ty, *variant_index)?;
                self.memory.write_addr(addr, enum_val.as_bytes(), enum_ty)?;
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

    /// Extract discriminant value from a Value as u128.
    ///
    /// Treats memory as unsigned integer of appropriate size.
    fn discriminant_value(&self, value: &Value) -> u128 {
        value.read_uint()
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
    fn execute_terminator(&mut self, bb_idx: BasicBlockIdx) -> Result<ControlFlow> {
        let terminator = &self.body.blocks[bb_idx].terminator;
        debug!("Executing terminator: {:?}", terminator.kind);

        match terminator.kind.clone() {
            TerminatorKind::Return => {
                // Return the value from local 0 (return value)
                let return_value = self.read_from_place(&Place::from(0))?;
                Ok(ControlFlow::Return(return_value))
            }
            TerminatorKind::Goto { target } => Ok(ControlFlow::Continue(target)),
            TerminatorKind::SwitchInt { discr, targets } => {
                let discr_value = self.evaluate_operand(&discr)?;
                let discr_int = self.discriminant_value(&discr_value);

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
                self.execute_call(&func, &args, &destination)?;

                match target {
                    Some(target_bb) => Ok(ControlFlow::Continue(target_bb)),
                    None => bail!("Diverging calls not yet supported"),
                }
            }
            TerminatorKind::Assert {
                cond,
                expected,
                target,
                msg,
                ..
            } => {
                let cond_value = self.evaluate_operand(&cond)?;
                let cond_bool = cond_value
                    .as_bool()
                    .context("Assert condition must be a boolean")?;

                if cond_bool == expected {
                    Ok(ControlFlow::Continue(target))
                } else {
                    let msg_str = msg
                        .description()
                        .unwrap_or("Failed to get assert description");
                    bail!("Assertion failed: {}", msg_str);
                }
            }
            TerminatorKind::Drop { place, target, .. } => {
                self.execute_drop(&place)?;
                Ok(ControlFlow::Continue(target))
            }
            TerminatorKind::Unreachable => {
                bail!("Entered unreachable code");
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
    ) -> Result<()> {
        // Evaluate arguments
        let arg_values: Result<Vec<Value>> =
            args.iter().map(|arg| self.evaluate_operand(arg)).collect();
        let mut arg_values = arg_values?;

        // Resolve function instance from operand type
        let func_ty = func
            .ty(self.body.locals())
            .with_context(|| format!("failed to resolve function type for `{:?}`", func))?;

        let func_instance = match func_ty.kind() {
            TyKind::RigidTy(RigidTy::FnDef(def_id, args)) => Instance::resolve(def_id, &args)?,
            _ => bail!("Unsupported function type: {:?}", func_ty),
        };

        // A `rust-call` ABI call (`Fn/FnMut/FnOnce::call*`) bundles its
        // arguments into a trailing tuple, but the callee body expects them
        // spread out (`env, arg0, arg1, ...`), so flatten the tuple here.
        // This applies whether the call devirtualized to the closure body
        // directly (`Item`) or stayed a call shim; preparing arguments at the
        // call site keeps `invoke_fn` free of ABI fixups.
        let is_rust_call = matches!(
            func_ty.kind().fn_sig().map(|sig| sig.skip_binder().abi),
            Some(Abi::RustCall)
        );
        if is_rust_call && let Some(tuple) = arg_values.pop() {
            let tuple_ty = args.last().unwrap().ty(self.body.locals())?;
            arg_values.extend(untuple(&tuple, tuple_ty)?);
        }

        // Resolve shim callees to something interpretable. Drop glue (e.g. a
        // direct `drop_in_place` call from `ManuallyDrop::drop`) carries its
        // own monomorphized MIR, so interpret it directly. Closure call shims
        // (`Fn/FnMut/FnOnce::call*`) have no body of their own and must be
        // redirected to the closure body first.
        //
        // TODO: Handle remaining shim kinds (clone, fn-ptr) here too.
        let callee = if func_instance.kind == InstanceKind::Shim {
            if func_instance.has_body() {
                func_instance
            } else {
                resolve_closure_shim(func_instance)?
            }
        } else {
            func_instance
        };

        let result = invoke_fn(callee, self.memory, arg_values, self.unwinding)?;

        // Store result in destination
        self.assign_to_place(destination, result)?;

        Ok(())
    }

    /// Run the drop glue for the value held in `place`.
    ///
    /// A `Drop` terminator lowers to `drop_in_place::<T>(&mut *place)`. We
    /// resolve that glue instance and invoke it with a `*mut T` to the place.
    /// The monomorphized glue body drives the rest: it calls the user
    /// `Drop::drop` impl (if any) and emits nested `Drop` terminators for the
    /// value's fields, which recurse back through this handler.
    fn execute_drop(&mut self, place: &Place) -> Result<()> {
        let place_ty = place
            .ty(self.body.locals())
            .with_context(|| format!("failed to resolve type of drop place `{place:?}`"))?;

        let drop_instance = Instance::resolve_drop_in_place(place_ty);

        // Types that need no cleanup resolve to an empty shim; skipping it
        // avoids an interpreter round-trip and matches codegen behavior.
        if drop_instance.is_empty_shim() {
            return Ok(());
        }

        // Build the `*mut T` argument. `place_to_ptr` carries wide-pointer
        // metadata for unsized drops (e.g. `[T]`, `dyn Trait`).
        let ptr_ty = Ty::new_ptr(place_ty, Mutability::Mut);
        let ptr = self.place_to_ptr(place, ptr_ty)?;

        invoke_fn(drop_instance, self.memory, vec![ptr], self.unwinding)?;
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
            Operand::Copy(place) | Operand::Move(place) => self.read_from_place(place),
            Operand::Constant(const_op) => self.evaluate_constant(&const_op.const_),
            // TODO: Get the value from rustc_public when a function is available
            Operand::RuntimeChecks(_) => Ok(Value::from_bool(true)),
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
                let mut bytes = alloc.raw_bytes()?;
                // Resolve provenance entries (pointers to other allocations).
                let ptr_size = crate::memory::pointer_width();
                for (offset, prov) in &alloc.provenance.ptrs {
                    let addr = self.memory.resolve_alloc(prov.0)?;
                    let addr_bytes = addr.to_le_bytes();
                    bytes[*offset..*offset + ptr_size].copy_from_slice(&addr_bytes[..ptr_size]);
                }
                Ok(Value::from_bytes(&bytes))
            }
            ConstantKind::ZeroSized => Ok(Value::unit().clone()),
            ConstantKind::Ty(ty_const) => {
                bail!("Unexpected type constant: {:?}", ty_const);
            }
            ConstantKind::Param(_) => {
                bail!("Unexpected parameter constants not supported");
            }
            ConstantKind::Unevaluated(_) => {
                bail!("Unexpected unevaluated constants on instance body");
            }
        }
    }
}

/// Split a tuple value into its field values, in declaration order.
///
/// Used to untuple the `rust-call` ABI argument bundle. A unit tuple yields
/// an empty list.
fn untuple(tuple: &Value, tuple_ty: Ty) -> Result<Vec<Value>> {
    use rustc_public::abi::FieldsShape;

    let TyKind::RigidTy(RigidTy::Tuple(fields)) = tuple_ty.kind() else {
        bail!("expected a tuple for rust-call args, got `{tuple_ty}`");
    };
    let FieldsShape::Arbitrary { offsets } = tuple_ty.layout()?.shape().fields else {
        bail!("tuple `{tuple_ty}` has no field layout");
    };
    let bytes = tuple.as_bytes();
    let mut values = Vec::with_capacity(fields.len());
    for (i, field_ty) in fields.iter().enumerate() {
        let offset = offsets
            .get(i)
            .with_context(|| format!("no offset for tuple field {i}"))?
            .bytes();
        let size = field_ty.size()?;
        values.push(Value::from_bytes(&bytes[offset..offset + size]));
    }
    Ok(values)
}

/// Control flow result from executing a terminator instruction.
#[derive(Debug)]
pub enum ControlFlow {
    /// Continue execution at the specified basic block
    Continue(BasicBlockIdx),
    /// Return from the function with the given value
    Return(Value),
}
