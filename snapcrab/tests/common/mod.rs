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
    let result = rustc_public::run!(&rustc_args, || {
        match snapcrab::run_main(snapcrab::CheckConfig::default()) {
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
            match snapcrab::run_function(start_fn, snapcrab::CheckConfig::default()) {
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

/// Load a shared library into the current process.
///
/// Uses `RTLD_GLOBAL` so symbols are visible to `dlsym(RTLD_DEFAULT, ...)`,
/// which is how the interpreter resolves native function addresses.
/// `RTLD_LOCAL` (the default) would hide symbols from `RTLD_DEFAULT` lookups.
pub fn load_native_lib(path: &Path) {
    let c_path = std::ffi::CString::new(path.to_str().unwrap()).unwrap();
    let handle = unsafe { libc::dlopen(c_path.as_ptr(), libc::RTLD_NOW | libc::RTLD_GLOBAL) };
    assert!(!handle.is_null(), "Failed to dlopen {path:?}: {}", unsafe {
        std::ffi::CStr::from_ptr(libc::dlerror())
            .to_string_lossy()
            .to_string()
    });
}

/// Run an interpreter test with native libraries pre-loaded.
pub fn run_native_call_test(
    input_file: &Path,
    start_fn: &str,
    native_libs: &[&Path],
) -> TestResult {
    for lib in native_libs {
        load_native_lib(lib);
    }

    run_custom_start_test(input_file, start_fn)
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
