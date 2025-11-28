use std::env;
use std::path::PathBuf;
use std::process::{Command, exit};

fn main() {
    println!("SnapCrab Installer");
    println!("==================\n");

    // Parse arguments
    let args: Vec<String> = env::args().collect();
    let path = match args.get(1).map(String::as_str) {
        None => None,
        Some("--help") => {
            println!("Usage: snapcrab-installer [--path <folder>]");
            println!("\nOptions:");
            println!("  --path <folder>  Install from .crate file in folder");
            println!("  --help           Show this help message");
            return;
        }
        Some("--path") => {
            if args.len() < 3 {
                eprintln!("Error: --path requires a value");
                eprintln!("Usage: snapcrab-installer [--path <folder>]");
                exit(1);
            }
            Some(PathBuf::from(&args[2]))
        }
        Some(arg) => {
            eprintln!("Error: Unknown argument '{}'", arg);
            eprintln!("Usage: snapcrab-installer [--path <folder>]");
            exit(1);
        }
    };

    // Check rustup is available
    if Command::new("rustup").arg("--version").output().is_err() {
        eprintln!("Error: rustup not found. Please install from https://rustup.rs");
        exit(1);
    }
    println!("✓ rustup found\n");

    // Install snapcrab
    println!("Building and installing snapcrab...");
    println!("(This may take a few minutes on first run)\n");

    let version = env!("CARGO_PKG_VERSION");
    let mut cmd = Command::new("cargo");
    cmd.arg("install");

    if let Some(folder) = path {
        // Construct expected .crate filename
        let crate_file = folder.join(format!("snapcrab-{}.crate", version));

        if !crate_file.is_file() {
            eprintln!("Error: {} not found", crate_file.display());
            exit(1);
        }

        // Extract .crate file to temp directory
        let temp_dir = std::env::temp_dir().join("snapcrab-install");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap_or_else(|_| {
            eprintln!("Error: Cannot create temp directory");
            exit(1);
        });

        let extract_status = Command::new("tar")
            .args([
                "-xzf",
                crate_file.to_str().unwrap(),
                "-C",
                temp_dir.to_str().unwrap(),
            ])
            .status();

        if !matches!(extract_status, Ok(s) if s.success()) {
            eprintln!("Error: Failed to extract .crate file");
            exit(1);
        }

        // Find extracted directory (snapcrab-VERSION)
        let extracted = std::fs::read_dir(&temp_dir)
            .unwrap()
            .flatten()
            .find(|e| e.path().is_dir())
            .unwrap()
            .path();

        cmd.arg("--path").arg(extracted);
    } else {
        cmd.args(["snapcrab", "--version", version]);
    }

    // Get rustc sysroot and set rpath
    let sysroot = Command::new("rustc")
        .arg("--print")
        .arg("sysroot")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string());

    if let Some(sysroot) = sysroot {
        let rpath = format!("-C link-arg=-Wl,-rpath,{}/lib", sysroot);
        cmd.env("RUSTFLAGS", rpath);
    }

    cmd.env("RUSTC_BOOTSTRAP", "1");

    match cmd.status() {
        Ok(s) if s.success() => {
            println!("\n✓ SnapCrab installed successfully!");
            println!("Run 'snapcrab --help' to get started");
        }
        _ => {
            eprintln!("\nError: Failed to install snapcrab");
            exit(1);
        }
    }
}
