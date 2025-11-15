use rustc_public::{entry_fn, CrateItem};
use std::process::ExitCode;

#[derive(Debug)]
pub enum IntError {
    UnsupportedOperation(&'static str),
}

pub fn run_main() -> Result<ExitCode, IntError> {
    let entry_fn = entry_fn().ok_or(IntError::UnsupportedOperation("No entry function found"))?;
    run(entry_fn)
}

pub fn run(_entry_fn: CrateItem) -> Result<ExitCode, IntError> {
    Err(IntError::UnsupportedOperation("All"))
}
