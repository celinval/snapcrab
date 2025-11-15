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
        crate::stack::Value::I32(0) => Ok(ExitCode::SUCCESS),
        crate::stack::Value::I32(_) => Ok(ExitCode::FAILURE),
        crate::stack::Value::Bool(true) => Ok(ExitCode::SUCCESS),
        crate::stack::Value::Bool(false) => Ok(ExitCode::FAILURE),
        crate::stack::Value::Unit => Ok(ExitCode::SUCCESS),
    }
}
