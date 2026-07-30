//! SnapCrab Interpreter
//!
//! A rustc wrapper that leverages `rustc_public` to interpret Rust code at the MIR level.
//! This component executes Rust code without LLVM code generation and linking overhead,
//! enabling rapid development iteration.

#![feature(rustc_private)]

#[cfg(not(target_endian = "little"))]
compile_error!("snapcrab only supports little-endian host machines");

extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_public;

use clap::Parser;
use rustc_public::target::{Endian, MachineInfo, MachineSize};
use rustc_public::{CompilerError, run};
use std::ops::ControlFlow;
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{debug, info};

#[derive(Parser)]
#[command(name = "snapcrab")]
#[command(about = "A Rust interpreter that executes code at the MIR level")]
#[command(long_about = "\
SnapCrab is an experimental Rust interpreter that executes code directly \
from MIR (Mid-level Intermediate Representation) without compilation \
overhead, enabling rapid development iteration.\n\n\
This interface currently targets single-crate interpretation. \
Multi-crate support will be provided by cargo-snap.")]
struct Args {
    /// Alternative start function (default: main)
    #[arg(
        long,
        help = "Specify a custom function to execute instead of main (requires fully qualified name)"
    )]
    start_fn: Option<String>,

    /// Skip specific UB checks (comma-separated: validity, alignment, bounds)
    #[arg(long = "skip-check", value_delimiter = ',')]
    skip_checks: Vec<String>,

    /// Native shared libraries to load before interpretation
    #[arg(long = "native-lib")]
    native_libs: Vec<String>,

    /// Input Rust file to interpret (standalone single-file mode)
    #[arg(help = "Path to the Rust source file to interpret")]
    input: Option<String>,
}

fn main() -> ExitCode {
    let log_level = std::env::var("SNAPCRAB_LOG").unwrap_or_else(|_| "info".to_string());
    tracing_subscriber::fmt().with_env_filter(log_level).init();

    // Wrapper mode: cargo-snap sets SNAPCRAB_WRAPPER and invokes us as
    // `snapcrab <rustc-path> <rustc-args...>`.
    if std::env::var_os("SNAPCRAB_WRAPPER").is_some() {
        return run_as_wrapper();
    }

    let args = Args::parse();
    let Some(input) = args.input else {
        eprintln!("error: no input file provided");
        return ExitCode::FAILURE;
    };

    let check_config = snapcrab::CheckConfig::with_skipped(&args.skip_checks);
    let native_libs = args.native_libs;

    let mut rustc_args = vec!["snapcrab".to_string()];
    // Add --crate-type=lib only if using custom start function.
    if args.start_fn.is_some() {
        rustc_args.push("--crate-type=lib".to_string());
    }
    rustc_args.push(input);

    run_interpreter(&rustc_args, args.start_fn, check_config, &native_libs)
}

/// Run as a `RUSTC_WORKSPACE_WRAPPER`, invoked as `snapcrab <rustc> <args...>`.
///
/// Probe invocations (e.g. `rustc -vV`, `--print`) are forwarded to the real
/// compiler. Actual crate compilations are run through the interpreter, which
/// executes the entry function (if any) after the build completes.
fn run_as_wrapper() -> ExitCode {
    // argv: [snapcrab, <rustc-path>, <rustc-args...>]
    let rustc_args: Vec<String> = std::env::args().skip(1).collect();
    if rustc_args.is_empty() {
        eprintln!("error: wrapper mode requires a compiler path");
        return ExitCode::FAILURE;
    }

    // Probe calls (no crate to build) just run the real rustc.
    if is_probe_invocation(&rustc_args) {
        return exec_real_rustc(&rustc_args);
    }

    let skip_checks = std::env::var("SNAPCRAB_SKIP_CHECKS").unwrap_or_default();
    let skip: Vec<String> = skip_checks.split(',').map(String::from).collect();
    let check_config = snapcrab::CheckConfig::with_skipped(&skip);

    run_interpreter(&rustc_args, None, check_config, &[])
}

/// Whether this rustc invocation is a probe (version/print query) rather than
/// an actual compilation.
fn is_probe_invocation(rustc_args: &[String]) -> bool {
    // rustc_args[0] is the compiler path; inspect the rest.
    rustc_args[1..]
        .iter()
        .any(|a| a == "-vV" || a == "--version" || a.starts_with("--print"))
}

/// Forward the invocation to the real rustc unchanged.
fn exec_real_rustc(rustc_args: &[String]) -> ExitCode {
    let status = std::process::Command::new(&rustc_args[0])
        .args(&rustc_args[1..])
        .status();
    match status {
        Ok(s) if s.success() => ExitCode::SUCCESS,
        Ok(_) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("error: failed to run rustc: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Drive the compiler + interpreter over `rustc_args`.
fn run_interpreter(
    rustc_args: &[String],
    start_fn: Option<String>,
    check_config: snapcrab::CheckConfig,
    native_libs: &[String],
) -> ExitCode {
    // The interpreter runs inside the compiler callback but returns
    // `Continue` so compilation finishes. Record interpretation failures
    // here so we can surface them as a non-zero exit code.
    let failed = AtomicBool::new(false);
    let result = run!(rustc_args, || start_interpreter(
        start_fn.clone(),
        check_config.clone(),
        native_libs,
        &failed,
    ));

    let compile_ok = matches!(
        result,
        Ok(_) | Err(CompilerError::Skipped | CompilerError::Interrupted(_))
    );
    if compile_ok && !failed.load(Ordering::Relaxed) {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
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
fn start_interpreter(
    start_fn: Option<String>,
    check_config: snapcrab::CheckConfig,
    native_libs: &[String],
    failed: &AtomicBool,
) -> ControlFlow<()> {
    let target = MachineInfo::target();
    let host = MachineInfo {
        endian: Endian::Little,
        pointer_width: MachineSize::from_bits(usize::BITS as usize),
    };
    if target != host {
        eprintln!(
            "error: snapcrab does not support interpreting code for a different target than the host machine"
        );
        return ControlFlow::Break(());
    }

    let crate_name = rustc_public::local_crate().name;
    info!("Interpreting crate: {}", crate_name);

    let result = if let Some(fn_name) = start_fn {
        info!("Using custom start function: {}", fn_name);
        snapcrab::run_function(&fn_name, check_config, native_libs).map(|_| ExitCode::SUCCESS)
    } else if rustc_public::entry_fn().is_some() {
        snapcrab::run_main(check_config, native_libs)
    } else {
        // No entry function (e.g., a library crate being built as a
        // dependency). Nothing to interpret — let the compilation finish.
        debug!("No entry function found; skipping interpretation of `{crate_name}`");
        return ControlFlow::Continue(());
    };

    if let Err(e) = result {
        eprintln!("{e}");
        failed.store(true, Ordering::Relaxed);
    }

    ControlFlow::Continue(())
}
