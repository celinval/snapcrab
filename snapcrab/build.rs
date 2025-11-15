use std::env;
use std::path::PathBuf;

pub fn main() {
    // Add rustup to the rpath in order to properly link with the correct rustc version.
    let rustup_home = env::var("RUSTUP_HOME").unwrap();
    let toolchain = env::var("RUSTUP_TOOLCHAIN").unwrap();
    let rustc_lib: PathBuf = [&rustup_home, "toolchains", &toolchain, "lib"]
        .iter()
        .collect();

    let bin_name = env::var("CARGO_PKG_NAME").unwrap();
    println!(
        "cargo:rustc-link-arg-bin={}=-Wl,-rpath,{}",
        bin_name,
        rustc_lib.display()
    );
}
