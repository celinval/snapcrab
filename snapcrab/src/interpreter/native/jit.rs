//! Cranelift JIT-based native function calls.
//!
//! Generates trampolines at runtime for calling native functions with the
//! correct ABI. Each trampoline has the signature:
//!
//! ```text
//! extern "C" fn(fn_ptr: *const (), args_buf: *const u8, ret_buf: *mut MaybeUninit<u8>)
//! ```
//!
//! The body loads typed arguments from `args_buf`, calls `fn_ptr` with the
//! correct ABI, and stores the result to `ret_buf`. The caller provides a
//! zeroed `ret_buf` so padding bytes are initialized.
//!
//! NOTE: This module handles native calls only. Function pointer support
//! (callbacks from native into the interpreter) will be added later.

use crate::value::Value;
use anyhow::{Result, bail};
use cranelift::prelude::*;
use cranelift_codegen::Context;
use cranelift_codegen::ir::{self, Function};
use cranelift_codegen::isa::CallConv;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};
use rustc_public::abi::{
    FieldsShape, FloatLength, FnAbi, IntegerLength, LayoutShape, PassMode, Primitive, Scalar,
    ValueAbi,
};
use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::sync::{Arc, Mutex};
use tracing::{debug, trace};

/// Trampoline function type.
type Trampoline = unsafe extern "C" fn(*const (), *const u8, *mut MaybeUninit<u8>);

/// Shared JIT engine for compiling native call trampolines.
///
/// Uses Arc<Mutex<>> so multiple ThreadMemory instances (future threading)
/// share the same compiled trampolines. Currently locks for the entire
/// build+compile phase; the granularity can be refined later since all
/// internals are module-private.
#[derive(Clone)]
pub struct JitEngine(Arc<Mutex<JitEngineInner>>);

struct JitEngineInner {
    module: JITModule,
    ctx: Context,
    func_ctx: FunctionBuilderContext,
    pointer_ty: ir::Type,
    call_conv: CallConv,
    counter: u32,
    trampoline_cache: HashMap<FnAbi, Trampoline>,
    symbol_cache: HashMap<String, GlobalSymbol>,
}

// SAFETY: JitEngineInner is only accessed through a Mutex, so concurrent
// access is impossible. The raw pointers inside (JITModule's code memory,
// cached symbols) are valid for the process lifetime and never mutated
// without the lock held.
unsafe impl Send for JitEngineInner {}
unsafe impl Sync for JitEngineInner {}

/// A symbol pointer obtained from dlsym, valid for the process lifetime.
#[derive(Clone, Copy)]
struct GlobalSymbol(*const ());

impl JitEngine {
    /// Create a new JIT engine for the host platform.
    pub fn new() -> Result<Self> {
        let mut flag_builder = settings::builder();
        flag_builder.set("use_colocated_libcalls", "false")?;
        flag_builder.set("is_pic", "false")?;
        let isa_builder = cranelift::native::builder()
            .map_err(|msg| anyhow::anyhow!("host machine is not supported: {msg}"))?;
        let isa = isa_builder.finish(settings::Flags::new(flag_builder))?;

        let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        let module = JITModule::new(builder);
        let pointer_ty = module.target_config().pointer_type();
        let call_conv = module.target_config().default_call_conv;

        Ok(Self(Arc::new(Mutex::new(JitEngineInner {
            module,
            ctx: Context::new(),
            func_ctx: FunctionBuilderContext::new(),
            pointer_ty,
            call_conv,
            counter: 0,
            trampoline_cache: HashMap::new(),
            symbol_cache: HashMap::new(),
        }))))
    }

    /// Resolve a symbol by name, returning a cached pointer or looking it up via dlsym.
    pub fn resolve_symbol(&self, symbol: &str) -> Result<*const ()> {
        let mut inner = self.0.lock().unwrap();
        if let Some(&GlobalSymbol(ptr)) = inner.symbol_cache.get(symbol) {
            return Ok(ptr);
        }
        let ptr = super::resolve_symbol(symbol)
            .ok_or_else(|| anyhow::anyhow!("symbol `{symbol}` not found in current process"))?;
        inner
            .symbol_cache
            .insert(symbol.to_owned(), GlobalSymbol(ptr));
        Ok(ptr)
    }

    /// Call a native function via a JIT'd trampoline.
    ///
    /// 1. Build a flat, aligned byte buffer containing all arguments.
    /// 2. Compile a trampoline with signature `fn(fn_ptr, args_buf, ret_buf)`.
    ///    The body loads typed args from `args_buf`, calls `fn_ptr` with the
    ///    correct ABI, and stores the result to `ret_buf`.
    /// 3. Call the trampoline, passing fn_ptr, the arg buffer, and a zeroed
    ///    return buffer. Read the return bytes from the buffer.
    ///
    /// # Safety
    ///
    /// Safe as long as the callee upholds:
    /// 1. It does not write values with uninitialized padding bytes into
    ///    memory reachable by the interpreter.
    /// 2. Its ABI matches what `fn_abi` describes.
    pub unsafe fn call_native(
        &self,
        fn_ptr: *const (),
        fn_abi: &FnAbi,
        args: &[Value],
        fn_name: &str,
    ) -> Result<Value> {
        let mut inner = self.0.lock().unwrap();
        let ret_size = fn_abi.ret.ty.layout()?.shape().size.bytes();

        // Build the argument buffer.
        let (args_buf, arg_layout) = inner.build_args_buffer(fn_abi, args)?;
        trace!(
            "  args_buf: {} bytes, {} entries",
            args_buf.len(),
            arg_layout.len()
        );

        // Look up or compile the trampoline.
        let trampoline = if let Some(&cached) = inner.trampoline_cache.get(fn_abi) {
            trace!("  trampoline cache hit");
            cached
        } else {
            let ret_info = inner.build_ret_info(fn_abi, ret_size)?;
            trace!(
                "  ret: size={ret_size}, mode={:?}",
                ret_info.as_ref().map(|r| &r.mode)
            );
            let t = inner.compile_trampoline(&arg_layout, ret_info.as_ref(), fn_name)?;
            inner.trampoline_cache.insert(fn_abi.clone(), t);
            t
        };

        // Release the lock before calling the trampoline (it might re-enter).
        drop(inner);

        let mut ret_buf = vec![MaybeUninit::<u8>::zeroed(); ret_size];
        // SAFETY: This is only sound if the native function does not leave
        // memory accessible to the interpreter in an uninitialized state.
        // For example, if the function receives a pointer and writes a struct
        // with padding bytes into the pointee, those padding bytes become
        // uninitialized from the interpreter's perspective. We currently only
        // sanitize the return buffer (pre-zeroed above).
        unsafe { trampoline(fn_ptr, args_buf.as_ptr(), ret_buf.as_mut_ptr()) };

        if ret_size == 0 {
            Ok(Value::unit().clone())
        } else {
            // SAFETY: The trampoline wrote the return value for each scalar types.
            // The buffer was pre-zeroed so padding bytes are defined.
            let bytes: Vec<u8> = ret_buf.iter().map(|b| unsafe { b.assume_init() }).collect();
            Ok(Value::from_bytes(&bytes))
        }
    }
}

impl JitEngineInner {
    /// Build return info from the function ABI.
    fn build_ret_info(&self, fn_abi: &FnAbi, ret_size: usize) -> Result<Option<RetInfo>> {
        if ret_size == 0 {
            return Ok(None);
        }
        match &fn_abi.ret.mode {
            PassMode::Direct(_) => {
                let ValueAbi::Scalar(scalar) = &fn_abi.ret.layout.shape().abi else {
                    bail!(
                        "Unsupported return ValueAbi for Direct: {:?}",
                        fn_abi.ret.layout.shape().abi
                    );
                };
                let ty = self.scalar_to_ir_type(scalar);
                Ok(Some(RetInfo {
                    mode: RetMode::Direct(ty),
                }))
            }
            PassMode::Pair(_, _) => {
                let ValueAbi::ScalarPair(first, second) = &fn_abi.ret.layout.shape().abi else {
                    bail!("internal error: PassMode::Pair must have ScalarPair ABI");
                };
                let ty1 = self.scalar_to_ir_type(first);
                let ty2 = self.scalar_to_ir_type(second);
                let second_offset = pair_second_offset(&fn_abi.ret.layout.shape())?;
                Ok(Some(RetInfo {
                    mode: RetMode::Pair(ty1, ty2, second_offset),
                }))
            }
            PassMode::Indirect { .. } => Ok(Some(RetInfo {
                mode: RetMode::Indirect,
            })),
            PassMode::Ignore => Ok(None),
            _ => bail!("Unsupported return PassMode"),
        }
    }

    /// Build a flat, zero-initialized, aligned byte buffer containing all arguments.
    fn build_args_buffer(
        &self,
        fn_abi: &FnAbi,
        args: &[Value],
    ) -> Result<(Vec<u8>, Vec<ArgEntry>)> {
        let mut ab = ArgsBuffer::new();

        for (arg_abi, arg_val) in fn_abi.args.iter().zip(args.iter()) {
            match &arg_abi.mode {
                PassMode::Ignore => {}
                PassMode::Direct(_) => match &arg_abi.layout.shape().abi {
                    ValueAbi::Scalar(scalar) => {
                        let ty = self.scalar_to_ir_type(scalar);
                        ab.push(ty, arg_val.as_bytes());
                    }
                    ValueAbi::Vector { element, count } => {
                        let ty = self.scalar_to_ir_type(element);
                        let num_bytes = ty.bytes() as usize;
                        for i in 0..(*count as usize) {
                            let offset = i * num_bytes;
                            let bytes = arg_val.as_bytes();
                            ab.push(ty, &bytes[offset..offset + num_bytes]);
                        }
                    }
                    val_abi => {
                        bail!("internal error: Unexpected ValueAbi: {val_abi:?}")
                    }
                },
                PassMode::Pair(_, _) => {
                    if let ValueAbi::ScalarPair(first, second) = &arg_abi.layout.shape().abi {
                        let ty1 = self.scalar_to_ir_type(first);
                        let ty2 = self.scalar_to_ir_type(second);
                        let second_offset = pair_second_offset(&arg_abi.layout.shape())?;
                        let bytes = arg_val.as_bytes();
                        ab.push(ty1, &bytes[..ty1.bytes() as usize]);
                        ab.push(
                            ty2,
                            &bytes[second_offset..second_offset + ty2.bytes() as usize],
                        );
                    }
                }
                PassMode::Indirect { .. } => {
                    let ptr = arg_val.as_bytes().as_ptr() as u64;
                    ab.push(self.pointer_ty, &ptr.to_le_bytes());
                }
                PassMode::Cast { .. } => {
                    bail!("PassMode::Cast is not supported");
                }
            }
        }

        Ok((ab.buf, ab.layout))
    }

    /// Compile a trampoline for the given argument layout and return info.
    fn compile_trampoline(
        &mut self,
        arg_layout: &[ArgEntry],
        ret_info: Option<&RetInfo>,
        fn_name: &str,
    ) -> Result<Trampoline> {
        debug!("Compiling trampoline for `{fn_name}`");

        // Trampoline signature: fn(fn_ptr: ptr, args_buf: ptr, ret_buf: ptr)
        let mut trampoline_sig = Signature::new(self.call_conv);
        trampoline_sig.params.push(AbiParam::new(self.pointer_ty));
        trampoline_sig.params.push(AbiParam::new(self.pointer_ty));
        trampoline_sig.params.push(AbiParam::new(self.pointer_ty));

        // Target function signature.
        let mut target_sig = Signature::new(self.call_conv);
        target_sig
            .params
            .extend(arg_layout.iter().map(|entry| AbiParam::new(entry.ty)));

        // Configure return type on target signature.
        let ret_indirect = matches!(
            ret_info,
            Some(RetInfo {
                mode: RetMode::Indirect
            })
        );
        if let Some(info) = ret_info {
            match &info.mode {
                RetMode::Direct(ty) => {
                    target_sig.returns.push(AbiParam::new(*ty));
                }
                RetMode::Pair(ty1, ty2, _) => {
                    target_sig.returns.push(AbiParam::new(*ty1));
                    target_sig.returns.push(AbiParam::new(*ty2));
                }
                RetMode::Indirect => {
                    target_sig.params.insert(0, AbiParam::new(self.pointer_ty));
                }
            }
        }

        // Declare the trampoline.
        let name = format!("__snapcrab_{fn_name}_{}", self.counter);
        self.counter += 1;
        let func_id = self
            .module
            .declare_function(&name, Linkage::Local, &trampoline_sig)?;

        // Build the function body.
        let mut func = Function::with_name_signature(
            ir::UserFuncName::user(0, func_id.as_u32()),
            trampoline_sig,
        );

        {
            let mut builder = FunctionBuilder::new(&mut func, &mut self.func_ctx);
            let block = builder.create_block();
            builder.append_block_params_for_function_params(block);
            builder.switch_to_block(block);

            let fn_ptr_param = builder.block_params(block)[0];
            let args_buf_param = builder.block_params(block)[1];
            let ret_buf_param = builder.block_params(block)[2];

            // Load each argument from the buffer.
            let mut call_args = Vec::new();
            if ret_indirect {
                call_args.push(ret_buf_param);
            }
            for entry in arg_layout {
                let addr = builder.ins().iadd_imm(args_buf_param, entry.offset as i64);
                let val = builder
                    .ins()
                    .load(entry.ty, MemFlagsData::trusted(), addr, 0);
                call_args.push(val);
            }

            // Call fn_ptr with the target signature.
            let sig_ref = builder.import_signature(target_sig);
            let call = builder
                .ins()
                .call_indirect(sig_ref, fn_ptr_param, &call_args);

            // Store result to ret_buf.
            if let Some(info) = ret_info {
                match &info.mode {
                    RetMode::Direct(_) => {
                        let result = builder.inst_results(call)[0];
                        builder
                            .ins()
                            .store(MemFlagsData::trusted(), result, ret_buf_param, 0);
                    }
                    RetMode::Pair(_, _, second_offset) => {
                        let results = builder.inst_results(call).to_vec();
                        builder
                            .ins()
                            .store(MemFlagsData::trusted(), results[0], ret_buf_param, 0);
                        builder.ins().store(
                            MemFlagsData::trusted(),
                            results[1],
                            ret_buf_param,
                            *second_offset as i32,
                        );
                    }
                    RetMode::Indirect => {}
                }
            }

            builder.ins().return_(&[]);
            builder.seal_all_blocks();
            builder.finalize();
        }

        // Compile.
        self.ctx.func = func;
        self.module.define_function(func_id, &mut self.ctx)?;
        self.module.clear_context(&mut self.ctx);
        self.module.finalize_definitions()?;

        let code_ptr = self.module.get_finalized_function(func_id);
        trace!("Trampoline compiled at {code_ptr:?}");
        // SAFETY: code_ptr points to JIT'd code matching the Trampoline signature.
        Ok(unsafe { std::mem::transmute::<*const u8, Trampoline>(code_ptr) })
    }

    /// Map a rustc Scalar to a cranelift IR type.
    fn scalar_to_ir_type(&self, scalar: &Scalar) -> ir::Type {
        let prim = match scalar {
            Scalar::Initialized { value, .. } | Scalar::Union { value } => *value,
        };
        match prim {
            Primitive::Int { length, .. } => match length {
                IntegerLength::I8 => types::I8,
                IntegerLength::I16 => types::I16,
                IntegerLength::I32 => types::I32,
                IntegerLength::I64 => types::I64,
                IntegerLength::I128 => types::I128,
            },
            Primitive::Float { length } => match length {
                FloatLength::F16 => types::F16,
                FloatLength::F32 => types::F32,
                FloatLength::F64 => types::F64,
                FloatLength::F128 => types::F128,
            },
            Primitive::Pointer(_) => self.pointer_ty,
        }
    }
}

/// Argument layout entry: offset into the buffer and cranelift type.
struct ArgEntry {
    offset: usize,
    ty: ir::Type,
}

/// Return value info for the trampoline.
#[derive(Debug)]
struct RetInfo {
    mode: RetMode,
}

#[derive(Debug)]
enum RetMode {
    /// Return value in a single register, store to ret_buf.
    Direct(ir::Type),
    /// Return value in two registers, store both to ret_buf.
    /// Fields: (first_type, second_type, second_offset).
    Pair(ir::Type, ir::Type, usize),
    /// Caller passes ret_buf as hidden first argument.
    Indirect,
}

/// Argument buffer builder.
struct ArgsBuffer {
    buf: Vec<u8>,
    layout: Vec<ArgEntry>,
}

impl ArgsBuffer {
    fn new() -> Self {
        Self {
            buf: Vec::new(),
            layout: Vec::new(),
        }
    }

    /// Append a typed value to the buffer. Aligns, copies bytes, pads to type size.
    fn push(&mut self, ty: ir::Type, bytes: &[u8]) {
        let size = ty.bytes() as usize;
        while !self.buf.len().is_multiple_of(size) {
            self.buf.push(0);
        }
        let offset = self.buf.len();
        self.buf.extend_from_slice(bytes);
        while self.buf.len() < offset + size {
            self.buf.push(0);
        }
        self.layout.push(ArgEntry { offset, ty });
    }
}

/// Get the offset of the second field in a ScalarPair layout.
///
/// TODO: replace this by getting pair offset in newer rustc_public version.
fn pair_second_offset(shape: &LayoutShape) -> Result<usize> {
    match &shape.fields {
        FieldsShape::Arbitrary { offsets } if offsets.len() >= 2 => Ok(offsets[1].bytes()),
        _ => bail!("Expected Arbitrary fields with at least 2 offsets for ScalarPair"),
    }
}
