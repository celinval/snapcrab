//! Cargo SnapCrab Driver
//!
//! A cargo subcommand that builds a crate (and its dependencies) with MIR
//! encoded into rmeta, using `snapcrab` as a `RUSTC_WORKSPACE_WRAPPER`. Each
//! workspace crate is compiled normally by rustc and then interpreted by
//! snapcrab.
//!
//! Usage via `cargo snap` (running `cargo-snap` should also work):
//!   cargo snap run  [cargo args...]              interpret `main`
//!   cargo snap test [--filter PAT] [cargo args]  interpret tests

use clap::{Args, Parser, Subcommand};
use std::env;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

/// Cargo invokes this as `cargo-snap snap <action> ...`, so the top-level
/// command is `snap` wrapping the real actions.
#[derive(Parser)]
#[command(name = "cargo-snap", bin_name = "cargo")]
enum Cargo {
    Snap {
        #[command(subcommand)]
        action: Action,
    },
    Run(CargoArgs),
    Test(TestArgs),
}

/// Discover and interpret the crate's tests.
#[derive(Args)]
struct TestArgs {
    /// Only run tests whose name contains this substring.
    #[arg(long)]
    filter: Option<String>,

    #[command(flatten)]
    cargo: CargoArgs,
}

#[derive(Subcommand)]
enum Action {
    /// Interpret the crate's `main`.
    Run(CargoArgs),
    Test(TestArgs),
}

/// Arguments forwarded verbatim to the underlying `cargo` invocation.
#[derive(Args)]
struct CargoArgs {
    /// Arguments passed through to `cargo check` (e.g. --manifest-path).
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    cargo_args: Vec<String>,
}

fn main() -> ExitCode {
    let action = match Cargo::parse() {
        Cargo::Snap { action } => action,
        Cargo::Run(cargo_args) => Action::Run(cargo_args),
        Cargo::Test(test_args) => Action::Test(test_args),
    };

    let snapcrab = snapcrab_path().unwrap_or_else(|e| {
        eprintln!("warning: failed to find snapcrab, defaulting to PATH lookup. {e}");
        // Fall back to a PATH lookup by returning the bare name.
        PathBuf::from("snapcrab")
    });

    let (snap_args, is_test, cargo_args) = match &action {
        Action::Run(c) => ("run".to_string(), false, &c.cargo_args),
        Action::Test(TestArgs { filter, cargo }) => {
            let args = match filter {
                Some(f) => format!("test --filter {f}"),
                None => "test".to_string(),
            };
            (args, true, &cargo.cargo_args)
        }
    };

    let mut cmd = Command::new("cargo");
    cmd.arg("check");
    // `test` needs the test harness cfg so `#[test]` functions are present.
    if is_test {
        cmd.arg("--tests");
    }
    cmd.args(cargo_args);

    // Encode MIR bodies (even for private fns) into rmeta so the interpreter
    // can see dependency code. Requires RUSTC_BOOTSTRAP to allow the -Z flag.
    cmd.env("RUSTC_WORKSPACE_WRAPPER", &snapcrab);
    cmd.env("SNAPCRAB_WRAPPER", "1");
    cmd.env("SNAPCRAB_ARGS", snap_args);
    cmd.env("RUSTC_BOOTSTRAP", "1");
    append_rustflags(&mut cmd, "-Zalways-encode-mir=yes");

    match cmd.status() {
        Ok(status) if status.success() => ExitCode::SUCCESS,
        Ok(_) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("error: failed to run cargo: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Locate the `snapcrab` binary, preferring one next to `cargo-snap`.
fn snapcrab_path() -> Result<PathBuf, String> {
    let exe = env::current_exe().map_err(|e| format!("cannot find current executable: {e}"))?;
    let dir = exe
        .parent()
        .ok_or("cannot determine executable directory")?;
    let candidate = dir.join(format!("snapcrab{}", env::consts::EXE_SUFFIX));
    if candidate.exists() {
        Ok(candidate)
    } else {
        Err("snapcrab binary not found".to_string())
    }
}

/// Append a flag to the existing RUSTFLAGS (preserving any user value).
fn append_rustflags(cmd: &mut Command, flag: &str) {
    let existing = env::var("RUSTFLAGS").unwrap_or_default();
    let combined = if existing.is_empty() {
        flag.to_string()
    } else {
        format!("{existing} {flag}")
    };
    cmd.env("RUSTFLAGS", combined);
}
