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

use clap::{Args as ClapArgs, Parser, Subcommand};
use rustc_public::target::{Endian, MachineInfo, MachineSize};
use rustc_public::{CompilerError, run};
use std::ops::ControlFlow;
use std::process::ExitCode;
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
    /// Skip specific UB checks (comma-separated: validity, alignment, bounds)
    #[arg(long = "skip-check", value_delimiter = ',', global = true)]
    skip_checks: Vec<String>,

    /// Native shared libraries to load before interpretation
    #[arg(long = "native-lib", global = true)]
    native_libs: Vec<String>,

    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand, Clone)]
enum Action {
    /// Interpret the crate's entry point (`main`).
    Run(RunArgs),
    /// Discover and interpret the crate's tests.
    Test(TestArgs),
}

/// Arguments for the `run` subcommand.
#[derive(ClapArgs, Clone)]
struct RunArgs {
    /// Alternative start function (fully qualified name) instead of main.
    #[arg(long)]
    start_fn: Option<String>,

    /// Input Rust file to interpret (the crate entry point).
    input: Option<String>,
}

/// Arguments for the `test` subcommand.
#[derive(ClapArgs, Clone)]
struct TestArgs {
    /// Only run tests whose name contains this substring.
    #[arg(long)]
    filter: Option<String>,

    /// Input Rust file to interpret (the crate entry point).
    input: Option<String>,
}

impl Action {
    /// The standalone input file, if provided.
    fn input(&self) -> Option<&String> {
        match self {
            Action::Run(args) => args.input.as_ref(),
            Action::Test(args) => args.input.as_ref(),
        }
    }
}

fn main() -> ExitCode {
    let log_level = std::env::var("SNAPCRAB_LOG").unwrap_or_else(|_| "info".to_string());
    tracing_subscriber::fmt().with_env_filter(log_level).init();

    // Wrapper mode: cargo-snap sets SNAPCRAB_WRAPPER and invokes us as
    // `snapcrab <rustc-path> <rustc-args...>`. The action and options come
    // from SNAPCRAB_ARGS, parsed with the same CLI as standalone mode.
    if std::env::var_os("SNAPCRAB_WRAPPER").is_some() {
        run_as_wrapper()
    } else {
        run_standalone()
    }
}

/// Runs the standalone mode of snapcrab, parsing arguments and running checks.
fn run_standalone() -> ExitCode {
    let args = Args::parse();

    let Some(input) = args.action.input() else {
        eprintln!("error: no input file provided");
        return ExitCode::FAILURE;
    };

    let mut rustc_args = vec!["snapcrab".to_string()];
    // A custom start function requires a lib crate (no main needed).
    if matches!(
        &args.action,
        Action::Run(RunArgs {
            start_fn: Some(_),
            ..
        })
    ) {
        rustc_args.push("--crate-type=lib".to_string());
    }
    rustc_args.push(input.clone());

    run_rustc(&rustc_args, || start_interpreter(&args))
}

/// Run as a `RUSTC_WORKSPACE_WRAPPER`, invoked as `snapcrab <rustc> <args...>`.
///
/// Probe invocations (e.g. `rustc -vV`, `--print`) are forwarded to the real
/// compiler. Actual crate compilations are run through the interpreter, which
/// performs the action (run/test) after the build completes.
fn run_as_wrapper() -> ExitCode {
    // argv: [snapcrab, <rustc-path>, <rustc-args...>]
    let rustc_args: Vec<String> = std::env::args().skip(1).collect();
    if rustc_args.is_empty() {
        eprintln!("error: wrapper mode requires a compiler path");
        return ExitCode::FAILURE;
    }

    // Probe calls (rustc -vV, --print) have no crate to interpret; rustc
    // handles them and stops before the callback fires.
    if is_probe_invocation(&rustc_args) {
        return run_rustc(&rustc_args, || ControlFlow::Continue(()));
    }

    // The action and options are supplied by cargo-snap via SNAPCRAB_ARGS,
    // e.g. "run" or "test --filter foo". Parse them with the standard CLI.
    let snap_args = std::env::var("SNAPCRAB_ARGS").unwrap_or_else(|_| "run".to_string());
    let argv = std::iter::once("snapcrab".to_string())
        .chain(snap_args.split_whitespace().map(String::from));
    let args = match Args::try_parse_from(argv) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: invalid SNAPCRAB_ARGS: {e}");
            return ExitCode::FAILURE;
        }
    };

    run_rustc(&rustc_args, || start_interpreter(&args))
}

/// Whether this rustc invocation is a probe (version/print query) rather than
/// an actual compilation.
fn is_probe_invocation(rustc_args: &[String]) -> bool {
    // rustc_args[0] is the compiler path; inspect the rest.
    rustc_args[1..]
        .iter()
        .any(|a| a == "-vV" || a == "--version" || a.starts_with("--print"))
}

/// Compile `rustc_args`, running `callback` after analysis.
///
/// The callback returns `Continue(())` on success (letting compilation
/// finish) or `Break(())` on failure. Maps outcomes to an exit code:
/// - `Ok(())`         — compilation finished, callback succeeded → success
/// - `Skipped`        — callback never ran (probe) → success
/// - `Interrupted(_)` — callback signalled failure → failure
/// - `Failed`         — compilation error → failure
fn run_rustc(
    rustc_args: &[String],
    callback: impl Fn() -> ControlFlow<()> + Send + Sync,
) -> ExitCode {
    match run!(rustc_args, callback) {
        Ok(()) | Err(CompilerError::Skipped) => ExitCode::SUCCESS,
        Err(CompilerError::Interrupted(()) | CompilerError::Failed) => ExitCode::FAILURE,
    }
}

/// Interpret the crate per `args`.
///
/// Returns `Continue(())` on success (nothing to interpret, or interpretation
/// succeeded) so compilation finishes, or `Break(())` on failure to signal a
/// non-zero exit.
fn start_interpreter(args: &Args) -> ControlFlow<()> {
    let check_config = snapcrab::CheckConfig::with_skipped(&args.skip_checks);
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

    let result = match &args.action {
        Action::Run(RunArgs {
            start_fn: Some(fn_name),
            ..
        }) => {
            info!("Using custom start function: {}", fn_name);
            snapcrab::run_function(fn_name, check_config, &args.native_libs).map(|_| ())
        }
        Action::Run(RunArgs { start_fn: None, .. }) => {
            if rustc_public::entry_fn().is_none() {
                // No entry function (e.g., a library crate being built as a
                // dependency). Nothing to interpret — let compilation finish.
                debug!("No entry function found; skipping interpretation of `{crate_name}`");
                return ControlFlow::Continue(());
            }
            snapcrab::run_main(check_config, &args.native_libs).map(|_| ())
        }
        Action::Test(TestArgs { filter, .. }) => {
            match snapcrab::run_tests(filter.as_deref(), check_config, &args.native_libs) {
                Ok(true) => return ControlFlow::Continue(()),
                Ok(false) => return ControlFlow::Break(()),
                Err(e) => {
                    eprintln!("{e}");
                    return ControlFlow::Break(());
                }
            }
        }
    };

    match result {
        Ok(()) => ControlFlow::Continue(()),
        Err(e) => {
            eprintln!("{e}");
            ControlFlow::Break(())
        }
    }
}
