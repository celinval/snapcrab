//! SnapCrab Interpreter Library
//!
//! A rustc wrapper that leverages `rustc_public` to interpret Rust code at the MIR level.

#![feature(rustc_private)]

extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_public;

pub mod interpreter;
pub mod stack;
pub mod heap;
pub mod core;

use rustc_public::{entry_fn, CrateDef};
use rustc_public::mir::mono::Instance;
use std::process::ExitCode;
use anyhow::Result;
use tracing::info;
use crate::interpreter::FnInterpreter;
use crate::heap::Heap;

pub fn run_main() -> Result<ExitCode> {
    let entry_fn = entry_fn().ok_or_else(|| anyhow::anyhow!("No entry function found"))?;
    info!("Found entry function: {}", entry_fn.name());
    
    let instance = Instance::try_from(entry_fn)
        .map_err(|_| anyhow::anyhow!("Failed to create instance from entry function"))?;
    
    run(instance)
}

pub fn run(instance: Instance) -> Result<ExitCode> {
    let mut interpreter = FnInterpreter::new();
    let mut heap = Heap::new();
    let result = interpreter.run(instance, &mut heap)?;
    
    // Convert the result value to an exit code
    match result {
        crate::stack::Value::Int(0) => Ok(ExitCode::SUCCESS),
        crate::stack::Value::Int(_) => Ok(ExitCode::FAILURE),
        crate::stack::Value::Uint(0) => Ok(ExitCode::SUCCESS),
        crate::stack::Value::Uint(_) => Ok(ExitCode::FAILURE),
        crate::stack::Value::Bool(true) => Ok(ExitCode::SUCCESS),
        crate::stack::Value::Bool(false) => Ok(ExitCode::FAILURE),
        crate::stack::Value::Unit => Ok(ExitCode::SUCCESS),
    }
}
