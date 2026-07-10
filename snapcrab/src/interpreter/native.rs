//! Native function call support via dynamic symbol resolution.
//!
//! When a function has no MIR body and is not an intrinsic we can shim,
//! we fall back to calling the native compiled version directly. This works
//! because:
//! - The interpreter's memory uses real process addresses
//! - The std library linked into the compiler process uses the same ABI
//! - The same rustc produced both the MIR and the native code

use crate::interpreter::check::{CheckConfig, validate_value};
use crate::value::Value;
use anyhow::{Result, bail};
use rustc_public::abi::{PassMode, ValueAbi};
use rustc_public::mir::mono::Instance;
use std::ffi::CString;
use tracing::debug;

/// Call a native function by resolving its mangled symbol name.
///
/// Validates all argument values before the call to prevent UB from invalid
/// values crossing the native boundary.
pub fn call_native(instance: Instance, args: &[Value], config: &CheckConfig) -> Result<Value> {
    let mangled = instance.mangled_name();
    debug!("Native call: {} ({})", instance.name(), mangled);

    let fn_abi = instance.fn_abi()?;

    // Validate arguments before passing to native code
    for (arg_abi, arg_val) in fn_abi.args.iter().zip(args.iter()) {
        validate_value(arg_val, arg_abi.ty, config)?;
    }

    let ret_size = fn_abi.ret.ty.layout()?.shape().size.bytes();

    // Resolve symbol from the current process via dlsym(RTLD_DEFAULT, ...).
    let symbol_name = mangled.as_str();
    // SAFETY: dlsym with RTLD_DEFAULT searches the current process's loaded symbols.
    // The returned pointer is valid for the process lifetime (std is always loaded).
    let fn_ptr = unsafe {
        let c_name = CString::new(symbol_name).expect("Symbol name should not contain null bytes");
        let ptr = libc::dlsym(libc::RTLD_DEFAULT, c_name.as_ptr());
        if ptr.is_null() {
            bail!("Symbol `{symbol_name}` not found in current process");
        }
        ptr
    };

    call_with_abi(fn_ptr.cast(), &fn_abi, args, ret_size)
}

/// Perform the actual native call using the function's ABI information.
fn call_with_abi(
    fn_ptr: *const (),
    fn_abi: &rustc_public::abi::FnAbi,
    args: &[Value],
    ret_size: usize,
) -> Result<Value> {
    // For now, support scalar/scalar-pair arguments and return values only.
    // This covers the vast majority of std functions we'd call natively.
    let mut raw_args: Vec<u64> = Vec::new();

    for (arg_abi, arg_val) in fn_abi.args.iter().zip(args.iter()) {
        match &arg_abi.mode {
            PassMode::Ignore => {}
            PassMode::Direct(_) => {
                // Scalar: pass as u64 (zero-extended)
                let mut buf = [0u8; 8];
                let bytes = arg_val.as_bytes();
                buf[..bytes.len()].copy_from_slice(bytes);
                raw_args.push(u64::from_le_bytes(buf));
            }
            PassMode::Pair(_, _) => {
                // ScalarPair: pass as two u64s
                let bytes = arg_val.as_bytes();
                let half = bytes.len() / 2;
                let mut buf1 = [0u8; 8];
                let mut buf2 = [0u8; 8];
                buf1[..half].copy_from_slice(&bytes[..half]);
                buf2[..bytes.len() - half].copy_from_slice(&bytes[half..]);
                raw_args.push(u64::from_le_bytes(buf1));
                raw_args.push(u64::from_le_bytes(buf2));
            }
            PassMode::Indirect { .. } => {
                // Pass pointer to the value's memory (which is a real address)
                raw_args.push(arg_val.as_bytes().as_ptr() as u64);
            }
            PassMode::Cast { .. } => {
                bail!(
                    "Unsupported PassMode::Cast for native call to `{}`",
                    fn_abi.args.len()
                );
            }
        }
    }

    // Check if the return is indirect (large struct returned via pointer)
    let ret_indirect = matches!(fn_abi.ret.mode, PassMode::Indirect { .. });

    // SAFETY: fn_ptr was resolved via dlsym and arguments were validated above.
    // The ABI matching is guaranteed by using the same toolchain for both
    // compilation and interpretation.
    let result = if ret_indirect {
        let mut ret_buf = vec![0u8; ret_size];
        let ret_ptr = ret_buf.as_mut_ptr() as u64;
        raw_args.insert(0, ret_ptr);
        unsafe { call_raw(fn_ptr, &raw_args, 0) };
        Value::from_bytes(&ret_buf)
    } else if ret_size == 0 {
        unsafe { call_raw(fn_ptr, &raw_args, 0) };
        Value::unit().clone()
    } else {
        let raw_ret = unsafe { call_raw(fn_ptr, &raw_args, ret_size) };
        let ret_bytes = raw_ret.to_le_bytes();
        // For scalar-pair returns, we need both halves
        match &fn_abi.ret.layout.shape().abi {
            ValueAbi::ScalarPair(_, _) => {
                // Two-register return: use the full 16 bytes
                Value::from_bytes(&ret_bytes[..ret_size])
            }
            _ => Value::from_bytes(&ret_bytes[..ret_size]),
        }
    };

    Ok(result)
}

/// Raw function call with up to 6 u64 arguments (System V AMD64 ABI registers).
///
/// # Safety
///
/// Caller must ensure fn_ptr is valid and arguments match the expected ABI.
unsafe fn call_raw(fn_ptr: *const (), args: &[u64], ret_size: usize) -> u128 {
    unsafe {
        // Transmute to various function signatures based on argument count.
        // This is the simplest approach that works for the System V AMD64 ABI
        // where the first 6 integer/pointer args go in registers.
        match args.len() {
            0 => {
                if ret_size <= 8 {
                    let f: unsafe extern "C" fn() -> u64 = std::mem::transmute(fn_ptr);
                    f() as u128
                } else {
                    let f: unsafe extern "C" fn() -> u128 = std::mem::transmute(fn_ptr);
                    f()
                }
            }
            1 => {
                if ret_size <= 8 {
                    let f: unsafe extern "C" fn(u64) -> u64 = std::mem::transmute(fn_ptr);
                    f(args[0]) as u128
                } else {
                    let f: unsafe extern "C" fn(u64) -> u128 = std::mem::transmute(fn_ptr);
                    f(args[0])
                }
            }
            2 => {
                if ret_size <= 8 {
                    let f: unsafe extern "C" fn(u64, u64) -> u64 = std::mem::transmute(fn_ptr);
                    f(args[0], args[1]) as u128
                } else {
                    let f: unsafe extern "C" fn(u64, u64) -> u128 = std::mem::transmute(fn_ptr);
                    f(args[0], args[1])
                }
            }
            3 => {
                if ret_size <= 8 {
                    let f: unsafe extern "C" fn(u64, u64, u64) -> u64 = std::mem::transmute(fn_ptr);
                    f(args[0], args[1], args[2]) as u128
                } else {
                    let f: unsafe extern "C" fn(u64, u64, u64) -> u128 =
                        std::mem::transmute(fn_ptr);
                    f(args[0], args[1], args[2])
                }
            }
            4 => {
                if ret_size <= 8 {
                    let f: unsafe extern "C" fn(u64, u64, u64, u64) -> u64 =
                        std::mem::transmute(fn_ptr);
                    f(args[0], args[1], args[2], args[3]) as u128
                } else {
                    let f: unsafe extern "C" fn(u64, u64, u64, u64) -> u128 =
                        std::mem::transmute(fn_ptr);
                    f(args[0], args[1], args[2], args[3])
                }
            }
            5 => {
                if ret_size <= 8 {
                    let f: unsafe extern "C" fn(u64, u64, u64, u64, u64) -> u64 =
                        std::mem::transmute(fn_ptr);
                    f(args[0], args[1], args[2], args[3], args[4]) as u128
                } else {
                    let f: unsafe extern "C" fn(u64, u64, u64, u64, u64) -> u128 =
                        std::mem::transmute(fn_ptr);
                    f(args[0], args[1], args[2], args[3], args[4])
                }
            }
            6 => {
                if ret_size <= 8 {
                    let f: unsafe extern "C" fn(u64, u64, u64, u64, u64, u64) -> u64 =
                        std::mem::transmute(fn_ptr);
                    f(args[0], args[1], args[2], args[3], args[4], args[5]) as u128
                } else {
                    let f: unsafe extern "C" fn(u64, u64, u64, u64, u64, u64) -> u128 =
                        std::mem::transmute(fn_ptr);
                    f(args[0], args[1], args[2], args[3], args[4], args[5])
                }
            }
            n => {
                panic!("Native call with {n} register args not supported (max 6)");
            }
        }
    }
}
