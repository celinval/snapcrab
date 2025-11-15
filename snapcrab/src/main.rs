//! SnapCrab Interpreter
//!
//! A rustc wrapper that leverages `rustc_public` to interpret Rust code at the MIR level.
//! This component executes Rust code without LLVM code generation and linking overhead,
//! enabling rapid development iteration.

#![feature(rustc_private)]

extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_public;

mod runner;
mod interpreter;
mod stack;
mod heap;

use rustc_public::{CompilerError, run};
use std::ops::ControlFlow;
use std::process::ExitCode;
use tracing::{error, info};

fn main() -> ExitCode {
    let log_level = std::env::var("SNAPCRAB_LOG").unwrap_or_else(|_| "info".to_string());

    tracing_subscriber::fmt().with_env_filter(log_level).init();

    println!("SnapCrab Interpreter v{}", env!("CARGO_PKG_VERSION"));
    println!("Experimental Rust interpreter for fast development iteration");

    let rustc_args: Vec<String> = std::env::args().collect();
    let result = run!(&rustc_args, start_interpreter);

    match result {
        Ok(_) | Err(CompilerError::Skipped | CompilerError::Interrupted(_)) => ExitCode::SUCCESS,
        _ => ExitCode::FAILURE,
    }
}

fn start_interpreter() -> ControlFlow<()> {
    let crate_name = rustc_public::local_crate().name;
    info!("Interpreting crate: {}", crate_name);

    match runner::run_main() {
        Ok(exit_code) => {
            info!("Interpretation completed with exit code: {:?}", exit_code);
        }
        Err(e) => {
            error!("Interpretation failed: {}", e);
        }
    }

    ControlFlow::Break(())
}
