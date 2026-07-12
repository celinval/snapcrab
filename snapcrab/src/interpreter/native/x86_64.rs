//! x86-64 System V ABI support for native function calls.
//!
//! Arguments are passed in registers in declaration order:
//! - Integer/pointer args go in rdi, rsi, rdx, rcx, r8, r9 (up to 6).
//! - Float args go in xmm0–xmm7 (up to 8).
//! - Integer and float registers are assigned independently.
//! - If a register class is exhausted, remaining args spill to the stack.
//!
//! Return values:
//! - Single integer/pointer: rax.
//! - Integer pair (e.g., u128, (u64, u64)): rax + rdx.
//! - Single float: xmm0.
//! - Float pair: xmm0 + xmm1.
//! - Large values that don't fit in registers: caller passes a hidden pointer
//!   as the first integer argument (indirect return), callee writes there.
//!
//! # TODO
//!
//! Implement stack spill for functions exceeding the register limits.

use crate::value::Value;
use anyhow::{Result, bail};
use rustc_public::abi::{FnAbi, PassMode, ValueAbi};
use std::arch::asm;

use super::{is_float_abi, is_float_scalar};

/// Prepare arguments and invoke the function using the System V AMD64 ABI.
pub fn call(fn_ptr: *const (), fn_abi: &FnAbi, args: &[Value], fn_name: &str) -> Result<Value> {
    let ret_size = fn_abi.ret.ty.layout()?.shape().size.bytes();
    let ret_indirect = matches!(fn_abi.ret.mode, PassMode::Indirect { .. });
    let mut ret_buf = vec![0u8; ret_size];

    // Build register buffers. Process indirect return first so it gets
    // the first integer register slot.
    let mut call_args = CallArgs::default();
    if ret_indirect {
        let ptr = ret_buf.as_mut_ptr() as u64;
        call_args.append_int(&ptr.to_le_bytes(), fn_name)?;
    }
    prepare_args(&mut call_args, fn_abi, args, fn_name)?;

    // Determine return shape and call via trampoline.
    let ret_is_float =
        !ret_indirect && ret_size > 0 && is_float_abi(&fn_abi.ret.layout.shape().abi);
    let ret_is_pair = matches!(fn_abi.ret.mode, PassMode::Pair(_, _));

    if ret_indirect || ret_size == 0 {
        trampoline_void(fn_ptr, &call_args.int_args, &call_args.float_args);
        if ret_indirect {
            Ok(Value::from_bytes(&ret_buf))
        } else {
            Ok(Value::unit().clone())
        }
    } else if ret_is_pair {
        let (lo, hi) = trampoline_pair(fn_ptr, &call_args.int_args, &call_args.float_args);
        let mut buf = [0u8; 16];
        buf[..8].copy_from_slice(&lo.to_le_bytes());
        buf[8..16].copy_from_slice(&hi.to_le_bytes());
        Ok(Value::from_bytes(&buf[..ret_size]))
    } else if ret_is_float {
        let ret = trampoline_float(fn_ptr, &call_args.int_args, &call_args.float_args);
        Ok(Value::from_bytes(&ret.to_le_bytes()[..ret_size]))
    } else {
        let ret = trampoline_int(fn_ptr, &call_args.int_args, &call_args.float_args);
        Ok(Value::from_bytes(&ret.to_le_bytes()[..ret_size]))
    }
}

/// Arguments split into integer and float register buffers.
#[derive(Default)]
struct CallArgs {
    int_args: [u64; 6],
    float_args: [f64; 8],
    int_count: usize,
    float_count: usize,
}

impl CallArgs {
    fn append_int(&mut self, bytes: &[u8], fn_name: &str) -> Result<()> {
        if self.int_count >= 6 {
            bail!(
                "Failed to invoke `{fn_name}`: \
                 more than 6 integer register arguments (currently unsupported)"
            );
        }
        let mut buf = [0u8; 8];
        buf[..bytes.len()].copy_from_slice(bytes);
        self.int_args[self.int_count] = u64::from_le_bytes(buf);
        self.int_count += 1;
        Ok(())
    }

    fn append_float(&mut self, bytes: &[u8], fn_name: &str) -> Result<()> {
        if self.float_count >= 8 {
            bail!(
                "Failed to invoke `{fn_name}`: \
                 more than 8 float register arguments (currently unsupported)"
            );
        }
        let mut buf = [0u8; 8];
        buf[..bytes.len()].copy_from_slice(bytes);
        self.float_args[self.float_count] = f64::from_le_bytes(buf);
        self.float_count += 1;
        Ok(())
    }
}

/// Split arguments into register buffers per the System V AMD64 calling convention.
fn prepare_args(
    call_args: &mut CallArgs,
    fn_abi: &FnAbi,
    args: &[Value],
    fn_name: &str,
) -> Result<()> {
    for (arg_abi, arg_val) in fn_abi.args.iter().zip(args.iter()) {
        match &arg_abi.mode {
            PassMode::Ignore => {}
            PassMode::Direct(_) => {
                if is_float_abi(&arg_abi.layout.shape().abi) {
                    call_args.append_float(arg_val.as_bytes(), fn_name)?;
                } else {
                    call_args.append_int(arg_val.as_bytes(), fn_name)?;
                }
            }
            PassMode::Pair(_, _) => {
                let (first_float, second_float) = match &arg_abi.layout.shape().abi {
                    ValueAbi::ScalarPair(first, second) => {
                        (is_float_scalar(first), is_float_scalar(second))
                    }
                    _ => (false, false),
                };

                let bytes = arg_val.as_bytes();
                let half = std::mem::size_of::<usize>();

                if first_float {
                    call_args.append_float(&bytes[..half], fn_name)?;
                } else {
                    call_args.append_int(&bytes[..half], fn_name)?;
                }

                if second_float {
                    call_args.append_float(&bytes[half..], fn_name)?;
                } else {
                    call_args.append_int(&bytes[half..], fn_name)?;
                }
            }
            PassMode::Indirect { .. } => {
                let ptr = arg_val.as_bytes().as_ptr() as u64;
                call_args.append_int(&ptr.to_le_bytes(), fn_name)?;
            }
            PassMode::Cast { .. } => {
                bail!("Failed to invoke `{fn_name}`: PassMode::Cast is not supported");
            }
        }
    }
    Ok(())
}

/// Loads all argument registers and calls fn_ptr. Discards the return value.
///
/// # TODO
///
/// Support stack spill for functions with >6 integer or >8 float arguments.
#[inline(never)]
fn trampoline_void(fn_ptr: *const (), int_args: &[u64; 6], float_args: &[f64; 8]) {
    // SAFETY: fn_ptr was resolved via dlsym, args populated from validated Values.
    unsafe {
        asm!(
            "call {fn_ptr}",
            fn_ptr = in(reg) fn_ptr,
            in("rdi") int_args[0],
            in("rsi") int_args[1],
            in("rdx") int_args[2],
            in("rcx") int_args[3],
            in("r8") int_args[4],
            in("r9") int_args[5],
            in("xmm0") float_args[0],
            in("xmm1") float_args[1],
            in("xmm2") float_args[2],
            in("xmm3") float_args[3],
            in("xmm4") float_args[4],
            in("xmm5") float_args[5],
            in("xmm6") float_args[6],
            in("xmm7") float_args[7],
            lateout("rax") _,
            lateout("rdx") _,
            lateout("r10") _,
            lateout("r11") _,
        );
    }
}

/// Loads all argument registers, calls fn_ptr, returns rax.
#[inline(never)]
fn trampoline_int(fn_ptr: *const (), int_args: &[u64; 6], float_args: &[f64; 8]) -> u64 {
    let ret: u64;
    // SAFETY: fn_ptr was resolved via dlsym, args populated from validated Values.
    unsafe {
        asm!(
            "call {fn_ptr}",
            fn_ptr = in(reg) fn_ptr,
            in("rdi") int_args[0],
            in("rsi") int_args[1],
            in("rdx") int_args[2],
            in("rcx") int_args[3],
            in("r8") int_args[4],
            in("r9") int_args[5],
            in("xmm0") float_args[0],
            in("xmm1") float_args[1],
            in("xmm2") float_args[2],
            in("xmm3") float_args[3],
            in("xmm4") float_args[4],
            in("xmm5") float_args[5],
            in("xmm6") float_args[6],
            in("xmm7") float_args[7],
            lateout("rax") ret,
            lateout("rdx") _,
            lateout("r10") _,
            lateout("r11") _,
        );
    }
    ret
}

/// Loads all argument registers, calls fn_ptr, returns xmm0.
#[inline(never)]
fn trampoline_float(fn_ptr: *const (), int_args: &[u64; 6], float_args: &[f64; 8]) -> f64 {
    let ret: f64;
    // SAFETY: fn_ptr was resolved via dlsym, args populated from validated Values.
    unsafe {
        asm!(
            "call {fn_ptr}",
            fn_ptr = in(reg) fn_ptr,
            in("rdi") int_args[0],
            in("rsi") int_args[1],
            in("rdx") int_args[2],
            in("rcx") int_args[3],
            in("r8") int_args[4],
            in("r9") int_args[5],
            in("xmm0") float_args[0],
            in("xmm1") float_args[1],
            in("xmm2") float_args[2],
            in("xmm3") float_args[3],
            in("xmm4") float_args[4],
            in("xmm5") float_args[5],
            in("xmm6") float_args[6],
            in("xmm7") float_args[7],
            lateout("xmm0") ret,
            lateout("rax") _,
            lateout("rdx") _,
            lateout("r10") _,
            lateout("r11") _,
        );
    }
    ret
}

/// Loads all argument registers, calls fn_ptr, returns (rax, rdx).
#[inline(never)]
fn trampoline_pair(fn_ptr: *const (), int_args: &[u64; 6], float_args: &[f64; 8]) -> (u64, u64) {
    let lo: u64;
    let hi: u64;
    // SAFETY: fn_ptr was resolved via dlsym, args populated from validated Values.
    unsafe {
        asm!(
            "call {fn_ptr}",
            fn_ptr = in(reg) fn_ptr,
            in("rdi") int_args[0],
            in("rsi") int_args[1],
            in("rdx") int_args[2],
            in("rcx") int_args[3],
            in("r8") int_args[4],
            in("r9") int_args[5],
            in("xmm0") float_args[0],
            in("xmm1") float_args[1],
            in("xmm2") float_args[2],
            in("xmm3") float_args[3],
            in("xmm4") float_args[4],
            in("xmm5") float_args[5],
            in("xmm6") float_args[6],
            in("xmm7") float_args[7],
            lateout("rax") lo,
            lateout("rdx") hi,
            lateout("r10") _,
            lateout("r11") _,
        );
    }
    (lo, hi)
}
