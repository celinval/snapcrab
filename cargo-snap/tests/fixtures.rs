//! Integration tests driving `cargo snap` over mock fixture crates.
//!
//! Each fixture is a standalone cargo crate under `tests/fixtures/`. We run
//! `cargo snap check` on it and assert the exit status matches expectations:
//! a passing `main` exits 0, a failing (panicking) `main` exits non-zero.

use std::path::PathBuf;
use std::process::Command;

/// Path to the built `cargo-snap` binary.
fn cargo_snap_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_cargo-snap"))
}

/// Directory containing `cargo-snap` and `snapcrab` (same target profile dir).
fn bin_dir() -> PathBuf {
    cargo_snap_bin().parent().unwrap().to_path_buf()
}

/// Run `cargo snap check` on a fixture, returning whether it succeeded.
///
/// Uses a fresh target dir so the wrapper always re-runs (cargo would
/// otherwise skip an unchanged crate).
fn run_fixture(name: &str) -> bool {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    let manifest = fixture.join("Cargo.toml");
    let target_dir = std::env::temp_dir().join(format!("cargo-snap-test-{name}"));

    // Ensure snapcrab is discoverable next to cargo-snap.
    let snapcrab = bin_dir().join(format!("snapcrab{}", std::env::consts::EXE_SUFFIX));
    assert!(
        snapcrab.exists(),
        "snapcrab binary not found at {snapcrab:?}; run `cargo build` for the workspace"
    );

    let path = format!(
        "{}:{}",
        bin_dir().display(),
        std::env::var("PATH").unwrap_or_default()
    );

    let status = Command::new(cargo_snap_bin())
        .args(["snap", "run", "--manifest-path"])
        .arg(&manifest)
        .arg("--target-dir")
        .arg(&target_dir)
        .env("PATH", path)
        .status()
        .expect("failed to run cargo-snap");

    status.success()
}

#[test]
fn fixture_pass_succeeds() {
    assert!(run_fixture("pass"), "expected `pass` fixture to succeed");
}

#[test]
fn fixture_fail_reports_failure() {
    assert!(
        !run_fixture("fail"),
        "expected `fail` fixture to report a failure"
    );
}
