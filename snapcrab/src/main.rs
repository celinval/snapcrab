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

use clap::Parser;
use rustc_public::{CompilerError, run};
use std::ops::ControlFlow;
use std::process::ExitCode;
use tracing::info;

#[derive(Parser)]
#[command(name = "snapcrab")]
#[command(about = "A Rust interpreter that executes code at the MIR level")]
#[command(
    long_about = "SnapCrab is an experimental Rust interpreter that executes code directly from MIR (Mid-level Intermediate Representation) without compilation overhead, enabling rapid development iteration."
)]
struct Args {
    /// Alternative start function (default: main)
    #[arg(
        long,
        help = "Specify a custom function to execute instead of main (requires fully qualified name)"
    )]
    start_fn: Option<String>,

    /// Input Rust file to interpret
    #[arg(help = "Path to the Rust source file to interpret")]
    input: String,
}

fn main() -> ExitCode {
    let args = Args::parse();

    let log_level = std::env::var("SNAPCRAB_LOG").unwrap_or_else(|_| "info".to_string());
    tracing_subscriber::fmt().with_env_filter(log_level).init();

    println!("SnapCrab Interpreter v{}", env!("CARGO_PKG_VERSION"));
    println!("Experimental Rust interpreter for fast development iteration");

    // Build rustc args from input file
    let mut rustc_args = vec!["snapcrab".to_string()];

    // Add --crate-type=lib only if using custom start function
    if args.start_fn.is_some() {
        rustc_args.push("--crate-type=lib".to_string());
    }

    rustc_args.push(args.input);

    let result = run!(&rustc_args, || start_interpreter(args.start_fn));

    match result {
        Ok(_) | Err(CompilerError::Skipped | CompilerError::Interrupted(_)) => ExitCode::SUCCESS,
        _ => ExitCode::FAILURE,
    }
}

/// Start the interpreter with optional custom start function.
///
/// This function initializes the interpreter and executes either the main function
/// or a custom function specified by the user. It handles the complete execution
/// flow and reports results.
///
/// # Arguments
/// * `start_fn` - Optional name of custom function to execute instead of main
///
/// # Returns
/// * `ControlFlow::Break(())` - Always breaks to exit the compiler callback
fn start_interpreter(start_fn: Option<String>) -> ControlFlow<()> {
    let crate_name = rustc_public::local_crate().name;
    info!("Interpreting crate: {}", crate_name);

    let result = if let Some(fn_name) = start_fn {
        info!("Using custom start function: {}", fn_name);
        snapcrab::run_function(&fn_name).map(|_| ExitCode::SUCCESS)
    } else {
        snapcrab::run_main()
    };

    match result {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{e}");
        }
    }

    ControlFlow::Break(())
}
