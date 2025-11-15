use rustc_public::{entry_fn, CrateItem, CrateDef};
use std::process::ExitCode;
use anyhow::{Result, bail};
use tracing::info;

pub fn run_main() -> Result<ExitCode> {
    let entry_fn = entry_fn().ok_or_else(|| anyhow::anyhow!("No entry function found"))?;
    info!("Found entry function: {}", entry_fn.name());
    run(entry_fn)
}

pub fn run(_entry_fn: CrateItem) -> Result<ExitCode> {
    bail!("All operations are unsupported")
}
