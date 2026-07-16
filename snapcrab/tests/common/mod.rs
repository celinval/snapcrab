//! Common test utilities and macros

extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_public;

use std::path::Path;

#[derive(Debug)]
pub enum TestResult {
    Success,
    SuccessWithValue(Vec<u8>),
    Error(String),
    ErrorRegex(String),
}

impl PartialEq for TestResult {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TestResult::Success, TestResult::Success) => true,
            (TestResult::Success, TestResult::SuccessWithValue(v))
            | (TestResult::SuccessWithValue(v), TestResult::Success) => v.is_empty(),
            (TestResult::SuccessWithValue(a), TestResult::SuccessWithValue(b)) => a == b,
            (TestResult::Error(a), TestResult::Error(b)) => a == b,
            (TestResult::ErrorRegex(pattern), TestResult::Error(msg))
            | (TestResult::Error(msg), TestResult::ErrorRegex(pattern)) => {
                regex::Regex::new(pattern).unwrap().is_match(msg)
            }
            _ => false,
        }
    }
}

pub fn run_interpreter_test(input_file: &Path) -> TestResult {
    // Set up rustc environment to compile the input file
    // Main function tests use bin crate type
    let rustc_args = vec![
        "snapcrab".to_string(),
        "--crate-type=bin".to_string(),
        input_file.to_string_lossy().to_string(),
    ];

    // Use rustc_public to run the interpreter
    let no_libs: &[&Path] = &[];
    let result = rustc_public::run!(&rustc_args, || {
        match snapcrab::run_main(snapcrab::CheckConfig::default(), no_libs) {
            Ok(_) => std::ops::ControlFlow::Continue(()),
            Err(e) => std::ops::ControlFlow::Break(TestResult::Error(e.to_string())),
        }
    });

    match result {
        Ok(_) => TestResult::Success,
        Err(e) => TestResult::Error(format!("Compilation failed: {:?}", e)),
    }
}

pub fn run_custom_start_test(input_file: &Path, start_fn: &str) -> TestResult {
    run_custom_start_test_with_libs(input_file, start_fn, &[] as &[&Path])
}

pub fn run_custom_start_test_with_libs(
    input_file: &Path,
    start_fn: &str,
    native_libs: &[impl AsRef<std::path::Path> + Sync],
) -> TestResult {
    // Set up rustc environment to compile the input file
    // Custom function tests use lib crate type
    let rustc_args = vec![
        "snapcrab".to_string(),
        "--crate-type=lib".to_string(),
        input_file.to_string_lossy().to_string(),
    ];

    // Use rustc_public to run the interpreter
    let result: Result<(), rustc_public::CompilerError<TestResult>> =
        rustc_public::run!(&rustc_args, || {
            match snapcrab::run_function(start_fn, snapcrab::CheckConfig::default(), native_libs) {
                Ok(value) => std::ops::ControlFlow::Break(TestResult::SuccessWithValue(value)),
                Err(e) => std::ops::ControlFlow::Break(TestResult::Error(e.to_string())),
            }
        });

    match result {
        Ok(_) => TestResult::Success, // This shouldn't happen with our new logic
        Err(rustc_public::CompilerError::Interrupted(test_result)) => test_result,
        Err(e) => TestResult::Error(format!("Compilation failed: {:?}", e)),
    }
}

/// Compile a Rust source file to a cdylib shared library.
///
/// Uses `rustc_public::run!()` to invoke the same compiler linked into the
/// test binary, guaranteeing ABI/flag consistency.
/// Each thread compiles into its own subdirectory to avoid races.
pub fn compile_cdylib(source: &Path) -> std::path::PathBuf {
    let thread_id = format!("{:?}", std::thread::current().id());
    let out_dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("test-libs")
        .join(&thread_id);
    std::fs::create_dir_all(&out_dir).expect("Failed to create output directory");

    let stem = source.file_stem().unwrap().to_str().unwrap();
    let lib_path = out_dir.join(format!("lib{stem}.so"));

    let rustc_args = vec![
        "snapcrab".to_string(),
        "--crate-type=cdylib".to_string(),
        "--edition=2021".to_string(),
        "-o".to_string(),
        lib_path.to_str().unwrap().to_string(),
        source.to_str().unwrap().to_string(),
    ];

    let result = rustc_public::run!(&rustc_args, || {
        std::ops::ControlFlow::<()>::Continue(())
    });
    assert!(
        result.is_ok(),
        "Failed to compile cdylib: {source:?}: {result:?}"
    );
    lib_path
}

/// Run an interpreter test with native libraries loaded before interpretation.
pub fn run_native_call_test(
    input_file: &Path,
    start_fn: &str,
    native_libs: &[&Path],
) -> TestResult {
    run_custom_start_test_with_libs(input_file, start_fn, native_libs)
}

/// Compile a Rust source file as both `dylib` and `rlib`.
///
/// Returns `(dylib_path, rlib_path)`. Both are produced in a single
/// `rustc` invocation so the crate hash matches exactly.
pub fn compile_dylib_and_rlib(source: &Path) -> (std::path::PathBuf, std::path::PathBuf) {
    let thread_id = format!("{:?}", std::thread::current().id());
    let out_dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("test-libs")
        .join(&thread_id);
    std::fs::create_dir_all(&out_dir).expect("Failed to create output directory");

    let stem = source.file_stem().unwrap().to_str().unwrap();

    let rustc_args = vec![
        "snapcrab".to_string(),
        "--crate-type=dylib".to_string(),
        "--crate-type=rlib".to_string(),
        "--edition=2021".to_string(),
        format!("--out-dir={}", out_dir.to_str().unwrap()),
        source.to_str().unwrap().to_string(),
    ];

    let result = rustc_public::run!(&rustc_args, || {
        std::ops::ControlFlow::<()>::Continue(())
    });
    assert!(
        result.is_ok(),
        "Failed to compile dylib+rlib: {source:?}: {result:?}"
    );

    // Find the generated files (names include a crate hash).
    let dylib = find_file_with_prefix(&out_dir, &format!("lib{stem}"), ".so");
    let rlib = find_file_with_prefix(&out_dir, &format!("lib{stem}"), ".rlib");
    (dylib, rlib)
}

/// Run a test where the caller is compiled against an rlib (via `--extern`),
/// with the corresponding dylib loaded for native symbol resolution.
pub fn run_extern_crate_test(dep_source: &Path, input_file: &Path, start_fn: &str) -> TestResult {
    let (dylib_path, rlib_path) = compile_dylib_and_rlib(dep_source);
    let dep_name = dep_source.file_stem().unwrap().to_str().unwrap();

    // Compile the caller against the rlib, then interpret it.
    let rustc_args = vec![
        "snapcrab".to_string(),
        "--crate-type=lib".to_string(),
        "--edition=2021".to_string(),
        format!("--extern={}={}", dep_name, rlib_path.to_str().unwrap()),
        input_file.to_string_lossy().to_string(),
    ];

    let native_libs: &[&Path] = &[dylib_path.as_path()];
    let result: Result<(), rustc_public::CompilerError<TestResult>> =
        rustc_public::run!(&rustc_args, || {
            match snapcrab::run_function(start_fn, snapcrab::CheckConfig::default(), native_libs) {
                Ok(value) => std::ops::ControlFlow::Break(TestResult::SuccessWithValue(value)),
                Err(e) => std::ops::ControlFlow::Break(TestResult::Error(e.to_string())),
            }
        });

    match result {
        Ok(_) => TestResult::Success,
        Err(rustc_public::CompilerError::Interrupted(test_result)) => test_result,
        Err(e) => TestResult::Error(format!("Compilation failed: {:?}", e)),
    }
}

/// Find a file in `dir` starting with `prefix` and ending with `suffix`.
fn find_file_with_prefix(dir: &Path, prefix: &str, suffix: &str) -> std::path::PathBuf {
    let entries: Vec<_> = std::fs::read_dir(dir)
        .expect("Failed to read output directory")
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.starts_with(prefix) && name.ends_with(suffix)
        })
        .collect();
    assert!(
        entries.len() == 1,
        "Expected exactly one {prefix}*{suffix} in {dir:?}, found: {entries:?}"
    );
    entries[0].path()
}

#[macro_export]
macro_rules! check_interpreter {
    ($(#[$attr:meta])* $test_name:ident, input=$input_file:expr, result=$expected:expr) => {
        $(#[$attr])*
        #[test]
        fn $test_name() {
            let input_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests")
                .join("inputs")
                .join($input_file);

            let result = $crate::common::run_interpreter_test(&input_path);
            assert_eq!(result, $expected);
        }
    };
}

#[macro_export]
macro_rules! check_custom_start {
    ($(#[$attr:meta])* $test_name:ident, input=$input_file:expr, start_fn=$start_fn:expr $(,)?) => {
        check_custom_start!(
            $(#[$attr])*
            $test_name,
            input = $input_file,
            start_fn = $start_fn,
            result = $crate::common::TestResult::Success
        );
    };
    ($(#[$attr:meta])* $test_name:ident, input=$input_file:expr, start_fn=$start_fn:expr, result=$expected:expr $(,)?) => {
        $(#[$attr])*
        #[test]
        fn $test_name() {
            let input_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests")
                .join("inputs")
                .join($input_file);

            let result = $crate::common::run_custom_start_test(&input_path, $start_fn);
            assert_eq!(result, $expected);
        }
    };
}

/// Declare a native call test that compiles a cdylib, loads it, then interprets a function.
///
/// The `native_lib` source is compiled to a `.so` and loaded before interpretation.
/// The `input` file declares `extern "C"` functions matching the library's exports.
/// Omit `result` for tests that assert internally (preferred style).
#[macro_export]
macro_rules! check_native_call {
    ($(#[$attr:meta])* $test_name:ident, native_lib=$lib_file:expr, input=$input_file:expr, start_fn=$start_fn:expr $(,)?) => {
        check_native_call!(
            $(#[$attr])*
            $test_name,
            native_lib = $lib_file,
            input = $input_file,
            start_fn = $start_fn,
            result = $crate::common::TestResult::Success
        );
    };
    ($(#[$attr:meta])* $test_name:ident, native_lib=$lib_file:expr, input=$input_file:expr, start_fn=$start_fn:expr, result=$expected:expr $(,)?) => {
        $(#[$attr])*
        #[test]
        fn $test_name() {
            let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests")
                .join("inputs");

            let lib_source = base.join($lib_file);
            let lib_path = $crate::common::compile_cdylib(&lib_source);

            let input_path = base.join($input_file);
            let result = $crate::common::run_native_call_test(
                &input_path,
                $start_fn,
                &[lib_path.as_path()],
            );
            assert_eq!(result, $expected);
        }
    };
}

/// Declare a test that compiles a dependency as dylib+rlib and interprets a caller.
///
/// The `dep` source is compiled to both `.so` and `.rlib`. The `input` file
/// imports the dep with `use` — non-generic functions without MIR bodies
/// are resolved via dlsym from the loaded dylib.
/// Omit `result` for tests that assert internally (preferred style).
#[macro_export]
macro_rules! check_extern_crate {
    ($(#[$attr:meta])* $test_name:ident, dep=$dep_file:expr, input=$input_file:expr, start_fn=$start_fn:expr $(,)?) => {
        check_extern_crate!(
            $(#[$attr])*
            $test_name,
            dep = $dep_file,
            input = $input_file,
            start_fn = $start_fn,
            result = $crate::common::TestResult::Success
        );
    };
    ($(#[$attr:meta])* $test_name:ident, dep=$dep_file:expr, input=$input_file:expr, start_fn=$start_fn:expr, result=$expected:expr $(,)?) => {
        $(#[$attr])*
        #[test]
        fn $test_name() {
            let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests")
                .join("inputs");

            let dep_source = base.join($dep_file);
            let input_path = base.join($input_file);
            let result = $crate::common::run_extern_crate_test(
                &dep_source,
                &input_path,
                $start_fn,
            );
            assert_eq!(result, $expected);
        }
    };
}
