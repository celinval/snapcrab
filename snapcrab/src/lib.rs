//! SnapCrab Interpreter Library
//!
//! A rustc wrapper that leverages `rustc_public` to interpret Rust code at the MIR level.

#![feature(rustc_private)]

extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_public;

pub mod core;
pub mod heap;
pub mod interpreter;
pub mod stack;

use crate::heap::Heap;
use crate::interpreter::FnInterpreter;
use anyhow::Result;
use rustc_public::mir::mono::Instance;
use rustc_public::{CrateDef, entry_fn};
use std::process::ExitCode;
use tracing::info;

pub fn run_main() -> Result<ExitCode> {
    let entry_fn = entry_fn().ok_or_else(|| anyhow::anyhow!("No entry function found"))?;
    info!("Found entry function: {}", entry_fn.name());

    let instance = Instance::try_from(entry_fn)
        .map_err(|_| anyhow::anyhow!("Failed to create instance from entry function"))?;

    run(instance)
}

pub fn run(instance: Instance) -> Result<ExitCode> {
    let interpreter = FnInterpreter::new(instance)?;
    let mut heap = Heap::new();
    let result = interpreter.run(&mut heap, vec![])?;

    // Convert the result value to an exit code
    match result {
        val if val.as_u128() == Some(0) || val.is_unit() => Ok(ExitCode::SUCCESS),
        _ => Ok(ExitCode::FAILURE),
    }
}
