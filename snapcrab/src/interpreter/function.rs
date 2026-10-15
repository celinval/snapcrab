use std::any::Any;
use std::panic::{self, AssertUnwindSafe};
use std::thread;

use crate::interpreter::{intrinsics, native, rvalue};
use crate::memory::{ThreadMemory, pointer_width};
use crate::ty::{self, MonoType, pointee_ty};
use crate::value::Value;
use anyhow::{Context, Result, anyhow, bail};
use rustc_public::CrateDef;
use rustc_public::abi::FieldsShape;
use rustc_public::mir::mono::{Instance, InstanceKind};
use rustc_public::mir::{
    BasicBlockIdx, Body, LocalDecl, Mutability, NonDivergingIntrinsic, Operand, Place,
    StatementKind, TerminatorKind,
};
use rustc_public::ty::{
    Abi, ClosureKind, ConstantKind, GenericArgKind, MirConst, RigidTy, Ty, TyKind, UintTy,
};
use tracing::{debug, info};

const ALLOC_SHIMS: &[&str] = &[
    "__rust_alloc",
    "__rust_alloc_zeroed",
    "__rust_dealloc",
    "__rust_realloc",
    "__rust_no_alloc_shim_is_unstable_v2",
];

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
    /// The location of a function callsite for functions with `#[track_caller]`.
    ///
    /// The MIR body has no local for this implicit argument, so it is held here.
    caller_location: Option<MirConst>,
}

/// Run the interpreter for the given instance.
///
/// Uses a four-tier dispatch:
/// 1. If the function has a MIR body, interpret it
/// 2. If it's an intrinsic without a body, shim it
/// 3. If it's a Rust allocator shim, service it from our `Heap`
/// 4. Otherwise, call the native compiled version via symbol resolution
pub fn invoke_fn(
    instance: Instance,
    memory: &mut ThreadMemory,
    args: Vec<Value>,
    unwinding: &mut Option<u16>,
    caller_location: Option<MirConst>,
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
                caller_location,
            };
            interpreter.execute(args)
        });
    }

    // Tier 2: intrinsic shims
    if let Some(intrinsic) = instance.intrinsic_name() {
        return intrinsics::eval_intrinsic(
            intrinsic.as_str(),
            &args,
            instance,
            memory,
            caller_location,
        );
    }

    // Tier 3: Rust allocator shims -- Use our Heap abstraction.
    let name = instance.name();
    let base = name
        .rsplit_once("::")
        .map(|(_, fn_name)| fn_name)
        .unwrap_or(name.as_str());
    if ALLOC_SHIMS.contains(&base) {
        return eval_alloc_shim(base, &args, memory);
    }

    // Tier 4: native call via dlsym. `call_native` rejects the calls it cannot
    // support, such as one that never returns.
    let mut native_args = args;
    if let Some(location) = caller_location {
        native_args.push(evaluate_constant(&location, memory)?)
    }

    let config = memory.check_config.clone();
    let jit = &memory.jit;
    panic::catch_unwind(AssertUnwindSafe(|| {
        native::call_native(instance, &native_args, &config, jit)
    }))
    // Turn the panic payload into the source error, keeping its message, and
    // layer the call context on top.
    .map_err(|panic| anyhow!(panic_message(panic.as_ref())))
    .with_context(|| format!("Native call to `{}` panicked", instance.name()))?
}

/// Resolve a trait-object (`Virtual`) call to the concrete method instance.
///
/// The receiver (`args[0]`) is a wide `&dyn` pointer whose metadata is the
/// vtable address. Slot `idx` holds the method's reified function pointer,
/// which resolves back to its `Instance`. The receiver is then narrowed to the
/// thin data pointer the concrete method expects as `&self`.
fn resolve_virtual(idx: usize, args: &mut [Value], memory: &ThreadMemory) -> Result<Instance> {
    let vtable = args
        .first()
        .context("virtual call has no receiver")?
        .ptr_metadata()?
        .read_uint() as usize;
    let slot = vtable + idx * pointer_width();
    let fn_addr = memory.read_addr(slot, Ty::usize_ty())?.read_uint() as usize;
    let method = memory.resolve_fn(fn_addr)?;
    args[0] = args[0].clone().to_data_addr()?;
    Ok(method)
}

/// Extract a human-readable message from a caught panic payload.
pub(crate) fn panic_message(panic: &dyn Any) -> String {
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
    pub(super) fn locals(&self) -> &[LocalDecl] {
        self.body.locals()
    }

    /// Where to report an error raised in this frame.
    ///
    /// A `#[track_caller]` function reports its caller's location, so a panic
    /// raised inside one (such as the bounds-check `Assert` in `Index`) points
    /// at the user's code rather than the library's.
    fn error_location(&self, span_location: &str) -> String {
        self.caller_location
            .as_ref()
            .and_then(|location| caller_location_str(location, self.memory).ok())
            .unwrap_or_else(|| span_location.to_string())
    }

    /// Prints the error including the failing thread
    fn generate_error(&mut self, location: &str, error: anyhow::Error) -> anyhow::Error {
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
            let msg = format!("thread '{tname}' panicked at {location}\n{}", error);
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
            let func = self.instance.name();
            anyhow!("{msg}\n  {current}: at {func}\n      {location}")
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
        let span = self.body.blocks[bb_idx].statements[stmt_idx]
            .source_info
            .span;
        let location = self.error_location(&span.diagnostic());
        self.generate_error(&location, error)
    }

    /// Add context to terminator execution errors
    fn terminator_error(&mut self, bb_idx: BasicBlockIdx, error: anyhow::Error) -> anyhow::Error {
        let span = self.body.blocks[bb_idx].terminator.source_info.span;
        let location = self.error_location(&span.diagnostic());
        self.generate_error(&location, error)
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
                rvalue::write_discriminant(enum_val.as_bytes_mut(), enum_ty, *variant_index)?;
                self.memory.write_addr(addr, enum_val.as_bytes(), enum_ty)?;
            }
            StatementKind::StorageLive(_) | StatementKind::StorageDead(_) => {
                // Ignore storage annotations for now
            }
            StatementKind::Nop => {
                // Do nothing
            }
            // `assume(cond)` is an optimizer hint; assuming a false condition is
            // UB, so we validate it like the `assume` intrinsic.
            StatementKind::Intrinsic(NonDivergingIntrinsic::Assume(op)) => {
                let cond = self.evaluate_operand(op)?;
                if !cond
                    .as_bool()
                    .context("`assume` operand is not a boolean")?
                {
                    bail!("`assume` invoked with a false condition, which is UB");
                }
            }
            // `copy_nonoverlapping(src, dst, count)`: copy `count` elements,
            // whose type is the source pointer's pointee.
            StatementKind::Intrinsic(NonDivergingIntrinsic::CopyNonOverlapping(copy)) => {
                let src = self.evaluate_operand(&copy.src)?.read_uint() as usize;
                let dst = self.evaluate_operand(&copy.dst)?.read_uint() as usize;
                let count = self.evaluate_operand(&copy.count)?.read_uint() as usize;
                let elem_size = pointee_ty(copy.src.ty(self.locals())?)?.size()?;
                self.memory
                    .copy_nonoverlapping(src, dst, count * elem_size)?;
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
            // Indirect call through a `fn` pointer value: resolve the reified
            // address back to its function.
            TyKind::RigidTy(RigidTy::FnPtr(_)) => {
                let addr = self.evaluate_operand(func)?.read_uint() as usize;
                let instance = self.memory.resolve_fn(addr)?;
                // A non-capturing closure reified to a `fn` pointer carries no
                // environment, but its body still takes the (ZST) closure as
                // its first argument, so supply it.
                if matches!(instance.ty().kind(), TyKind::RigidTy(RigidTy::Closure(..))) {
                    arg_values.insert(0, Value::unit().clone());
                }
                instance
            }
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

        // Resolve the callee to something interpretable:
        // - `Virtual`: a trait-object call; read the method from the receiver's
        //   vtable and narrow the wide `&dyn` receiver to the concrete `&self`.
        // - Drop glue and other body-carrying shims interpret directly.
        // - Closure call shims (`Fn/FnMut/FnOnce::call*`) have no body of their
        //   own and must be redirected to the closure body first.
        //
        // TODO: Handle remaining shim kinds (clone, fn-ptr) here too.
        let callee = match func_instance.kind {
            InstanceKind::Virtual { idx } => resolve_virtual(idx, &mut arg_values, self.memory)?,
            InstanceKind::Shim if !func_instance.has_body() => resolve_closure_shim(func_instance)?,
            _ => func_instance,
        };

        let callee_location = self.callee_location(&callee)?;

        // A panic entry point diverges into the native panic runtime, which
        // cannot unwind back across our JIT trampoline and aborts the
        // process. Report the panic here instead, at the location the
        // callee would have reported.
        if let Some(location) = callee_location.as_ref()
            && let Some(message) =
                panic_entry_message(&callee, &callee.name(), &arg_values, self.memory)?
        {
            let where_ = caller_location_str(location, self.memory)
                .unwrap_or_else(|_| "unknown location".to_string());
            return Err(self.generate_error(&where_, anyhow!("{message}")));
        }

        let result = invoke_fn(
            callee,
            self.memory,
            arg_values,
            self.unwinding,
            callee_location,
        )?;

        // Store result in destination
        self.assign_to_place(destination, result)?;

        Ok(())
    }

    /// Computes the callee location for the current block's terminator.
    fn callee_location(&self, callee: &Instance) -> Result<Option<MirConst>> {
        if !callee.requires_caller_location()
            && callee.intrinsic_name().as_deref() != Some("caller_location")
        {
            return Ok(None);
        }

        let inherited = self.caller_location.clone();
        let terminator = &self.body.blocks[self.current_block].terminator;
        let location = self.body.caller_location(terminator, inherited);

        Ok(Some(location))
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

        // Dropping a trait object dispatches through its vtable to the concrete
        // `drop_in_place`. Going through `drop_in_place::<dyn Trait>` instead
        // would re-resolve to the same `dyn` drop glue and recurse forever.
        if place_ty.kind().is_trait() {
            return self.drop_dyn(place, place_ty);
        }

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

        invoke_fn(drop_instance, self.memory, vec![ptr], self.unwinding, None)?;
        Ok(())
    }

    /// Drop a trait-object value by dispatching through its vtable.
    ///
    /// The vtable's first slot (`MetadataDropInPlace`) holds the concrete
    /// type's `drop_in_place`; invoke it with the thin data pointer.
    fn drop_dyn(&mut self, place: &Place, place_ty: Ty) -> Result<()> {
        let ptr_ty = Ty::new_ptr(place_ty, Mutability::Mut);
        let fat = self.place_to_ptr(place, ptr_ty)?;
        let vtable = fat.ptr_metadata()?.read_uint() as usize;

        // A null drop slot means the concrete type needs no cleanup.
        let fn_addr = self.memory.read_addr(vtable, Ty::usize_ty())?.read_uint() as usize;
        if fn_addr == 0 {
            return Ok(());
        }
        let drop_instance = self.memory.resolve_fn(fn_addr)?;

        let data = fat.to_data_addr()?;
        invoke_fn(drop_instance, self.memory, vec![data], self.unwinding, None)?;
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
    fn evaluate_constant(&self, const_: &MirConst) -> Result<Value> {
        evaluate_constant(const_, self.memory)
    }
}

/// Evaluate a MIR constant to a runtime value.
///
/// Pointers inside the constant are resolved to the addresses the referenced
/// allocations were materialized at.
pub(crate) fn evaluate_constant(const_: &MirConst, memory: &ThreadMemory) -> Result<Value> {
    match const_.kind() {
        ConstantKind::Allocated(alloc) => {
            // Copy the initialized bytes and leave uninitialized ones zeroed.
            let mut bytes: Vec<u8> = alloc.bytes.iter().map(|b| b.unwrap_or(0)).collect();
            // Resolve provenance entries (pointers to other allocations).
            let ptr_size = pointer_width();
            for (offset, prov) in &alloc.provenance.ptrs {
                let addr = memory.resolve_alloc(prov.0)?;
                let addr_bytes = addr.to_le_bytes();
                bytes[*offset..*offset + ptr_size].copy_from_slice(&addr_bytes[..ptr_size]);
            }
            Ok(Value::from_bytes(&bytes))
        }
        ConstantKind::ZeroSized => Ok(Value::unit().clone()),
        ConstantKind::Ty(ty_const) => bail!("Unexpected type constant: {:?}", ty_const),
        ConstantKind::Param(_) => bail!("Unexpected parameter constants not supported"),
        ConstantKind::Unevaluated(_) => {
            bail!("Unexpected unevaluated constants on instance body")
        }
    }
}

/// The panic message a panic entry point would produce, or `None` if `name` is
/// not one.
///
/// These functions diverge into the native panic runtime, so the interpreter
/// reports the panic itself. The message comes from the entry's `&str` argument
/// where it has one. `Result`'s variants omit the `Err` value's `Debug`, which
/// would need the formatting machinery, and `panic_fmt`-style formatted panics
/// are not recognized at all.
fn panic_entry_message(
    callee: &Instance,
    name: &str,
    args: &[Value],
    memory: &ThreadMemory,
) -> Result<Option<String>> {
    // Only the standard library's entry points are recognized, so a user
    // function that happens to share a path suffix is still executed.
    if !is_std_crate(callee) {
        return Ok(None);
    }
    let message = if name.ends_with("::panicking::panic")
        || name.ends_with("::panicking::panic_str_2015")
        || name.ends_with("::option::expect_failed")
        || name.contains("::begin_panic::<&str>")
    {
        // A `&str` message argument.
        read_str(&args[0], memory)?
    } else if name.ends_with("::panicking::panic_bounds_check") {
        let index = args[0].read_uint();
        let len = args[1].read_uint();
        format!("index out of bounds: the len is {len} but the index is {index}")
    } else if name.ends_with("::option::unwrap_failed") {
        "called `Option::unwrap()` on a `None` value".to_string()
    } else if name.ends_with("::result::unwrap_failed") {
        // `unwrap_failed(msg: &str, error: &dyn Debug)` is shared by `unwrap`,
        // `expect`, `unwrap_err` and `expect_err`, so the message comes from
        // the argument. The `Err` value needs formatting, so it is left out.
        format!(
            "{}: (the `Err` value is not yet rendered)",
            read_str(&args[0], memory)?
        )
    } else if name.ends_with("::panicking::panic_fmt")
        || name.ends_with("::rt::panic_fmt")
        || name.ends_with("::panicking::panic_nounwind_fmt")
    {
        // The message is a `core::fmt::Arguments`, so rendering it needs the
        // formatting machinery. Report the panic without its message rather
        // than letting it reach the native runtime, which would abort.
        "panicked with a formatted message, which is not yet rendered".to_string()
    } else if name.ends_with("::panicking::assert_failed_inner") {
        // `assert_eq!` and friends. The operands are `&dyn Debug`, so showing
        // them needs the formatting machinery.
        "assertion failed, with operands that are not yet rendered".to_string()
    } else {
        return Ok(None);
    };
    Ok(Some(message))
}

/// Render a resolved caller `Location` constant as `file:line:col`.
fn caller_location_str(location: &MirConst, memory: &ThreadMemory) -> Result<String> {
    let loc_ty = ty::pointee_ty(location.ty())?;
    let addr = evaluate_constant(location, memory)?.read_uint() as usize;
    let fields = ty::struct_fields(loc_ty)?;
    if fields.len() < 3 {
        bail!(
            "`{loc_ty}` has {} fields, expected at least 3",
            fields.len()
        );
    }

    // `Location` holds `filename: NonNull<str>`, then `line` and `col`.
    let ptr_width = pointer_width();
    let file_off = fields[0].offset;
    let read_usize = |offset: usize| -> Result<usize> {
        Ok(memory.read_addr(addr + offset, Ty::usize_ty())?.read_uint() as usize)
    };
    let file_ptr = read_usize(file_off)?;
    let file_len = read_usize(file_off + ptr_width)?;
    let file = String::from_utf8_lossy(&read_bytes(file_ptr, file_len, memory)?).into_owned();

    let u32_ty = Ty::unsigned_ty(UintTy::U32);
    let line = memory
        .read_addr(addr + fields[1].offset, u32_ty)?
        .read_uint();
    let col = memory
        .read_addr(addr + fields[2].offset, u32_ty)?
        .read_uint();
    Ok(format!("{file}:{line}:{col}"))
}

/// Whether an instance is defined in the standard library.
fn is_std_crate(instance: &Instance) -> bool {
    matches!(
        CrateDef::krate(&instance.def).name.as_str(),
        "core" | "std" | "alloc"
    )
}

/// Read a `&str` argument into a `String`.
fn read_str(val: &Value, memory: &ThreadMemory) -> Result<String> {
    let addr = val.clone().to_data_addr()?.read_uint() as usize;
    let len = val.ptr_metadata()?.read_uint() as usize;
    Ok(String::from_utf8_lossy(&read_bytes(addr, len, memory)?).into_owned())
}

/// Read `len` bytes at `addr` through the memory sanitizer.
fn read_bytes(addr: usize, len: usize, memory: &ThreadMemory) -> Result<Vec<u8>> {
    let u8_ty = Ty::unsigned_ty(UintTy::U8);
    (0..len)
        .map(|i| Ok(memory.read_addr(addr + i, u8_ty)?.read_uint() as u8))
        .collect()
}

/// Interpret Rust allocator shims (`__rust_[*]alloc` functions).
///
/// These shims are meant to glue allocation functions with the global allocator.
/// We intercept them here and modeled after our `Heap` abstraction.
fn eval_alloc_shim(name: &str, args: &[Value], memory: &ThreadMemory) -> Result<Value> {
    let usize_arg = |i: usize| -> Result<usize> {
        Ok(args
            .get(i)
            .with_context(|| {
                format!(
                    "Internal Error: Missing arg `{i}` for `{name}`. Only found {} args.",
                    args.len()
                )
            })?
            .read_uint() as usize)
    };
    match name {
        // `fn(size, align) -> *mut u8`. Both allocators zero their memory (see
        // `heap`), so they share one path.
        "__rust_alloc" | "__rust_alloc_zeroed" => {
            let size = usize_arg(0)?;
            let align = usize_arg(1)?;
            let addr = memory.heap_allocate(size, align)?;
            Ok(Value::from_type(addr))
        }
        // `fn(ptr, size, align)`.
        "__rust_dealloc" => {
            let ptr = usize_arg(0)?;
            let size = usize_arg(1)?;
            let align = usize_arg(2)?;
            memory
                .heap_deallocate(ptr, size, align)
                .map(|()| Value::unit().clone())
        }
        // `fn(ptr, old_size, align, new_size) -> *mut u8`.
        "__rust_realloc" => {
            let ptr = usize_arg(0)?;
            let old_size = usize_arg(1)?;
            let align = usize_arg(2)?;
            let new_size = usize_arg(3)?;
            let result = memory.heap_reallocate(ptr, old_size, align, new_size);
            result.map(Value::from_type)
        }
        // A link-time marker with no runtime effect.
        "__rust_no_alloc_shim_is_unstable_v2" => Ok(Value::unit().clone()),
        _ => bail!("Internal Error: unknown allocator shim `{name}`"),
    }
}

/// Split a tuple value into its field values, in declaration order.
///
/// Used to untuple the `rust-call` ABI argument bundle. A unit tuple yields
/// an empty list.
fn untuple(tuple: &Value, tuple_ty: Ty) -> Result<Vec<Value>> {
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
