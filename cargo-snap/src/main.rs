//! Cargo SnapCrab Driver
//!
//! A cargo subcommand that builds a crate (and its dependencies) with MIR
//! encoded into rmeta, using `snapcrab` as a `RUSTC_WORKSPACE_WRAPPER`. Each
//! workspace crate is compiled normally by rustc and then interpreted by
//! snapcrab (which runs `main`/the test harness when present).
//!
//! Usage: `cargo snap [check|test] [cargo args...]`

use std::env;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    // Invoked as `cargo snap <args...>`; drop the leading `snap`.
    let mut args = env::args().skip(1);
    if args.next().as_deref() != Some("snap") {
        eprintln!("error: cargo-snap must be invoked as `cargo snap`");
        return ExitCode::FAILURE;
    }
    let cargo_args: Vec<String> = args.collect();

    let snapcrab = match snapcrab_path() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };

    // Default to `check` — it compiles without codegen/link, and snapcrab's
    // interpretation runs during the compiler callback.
    let (subcommand, rest) = split_subcommand(&cargo_args);

    let mut cmd = Command::new("cargo");
    cmd.arg(&subcommand);
    cmd.args(rest);

    // Encode MIR bodies (even for private fns) into rmeta so the interpreter
    // can see dependency code. Requires RUSTC_BOOTSTRAP to allow the -Z flag.
    cmd.env("RUSTC_WORKSPACE_WRAPPER", &snapcrab);
    cmd.env("SNAPCRAB_WRAPPER", "1");
    // Tell snapcrab which action to perform. Only `run` is wired up for now.
    cmd.env("SNAPCRAB_ARGS", "run");
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

/// Determine the cargo subcommand (default `check`) and the remaining args.
fn split_subcommand(args: &[String]) -> (String, &[String]) {
    match args.first() {
        Some(first) if first == "check" || first == "test" => (first.clone(), &args[1..]),
        _ => ("check".to_string(), args),
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
        return Ok(candidate);
    }
    // Fall back to a PATH lookup by returning the bare name.
    Ok(PathBuf::from("snapcrab"))
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
