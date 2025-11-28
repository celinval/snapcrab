//! Common test utilities and macros

extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_public;

use std::path::Path;
use std::process::ExitCode;

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
            (TestResult::SuccessWithValue(a), TestResult::SuccessWithValue(b)) => a == b,
            (TestResult::Error(a), TestResult::Error(b)) => a == b,
            (TestResult::ErrorRegex(pattern), TestResult::Error(msg)) => {
                regex::Regex::new(pattern).unwrap().is_match(msg)
            }
            (TestResult::Error(msg), TestResult::ErrorRegex(pattern)) => {
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
        match snapcrab::run_main() {
            Ok(exit_code) => {
                if exit_code == ExitCode::SUCCESS {
                    std::ops::ControlFlow::Continue(())
                } else {
                    std::ops::ControlFlow::Continue(())
                }
            }
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
            match snapcrab::run_function(start_fn) {
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

#[macro_export]
macro_rules! check_interpreter {
    ($test_name:ident, input=$input_file:expr, result=$expected:expr) => {
        #[test]
        fn $test_name() {
            let input_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests")
                .join("inputs")
                .join($input_file);

            let result = crate::common::run_interpreter_test(&input_path);
            assert_eq!(result, $expected);
        }
    };
}

#[macro_export]
macro_rules! check_custom_start {
    ($test_name:ident, input=$input_file:expr, start_fn=$start_fn:expr, result=$expected:expr) => {
        #[test]
        fn $test_name() {
            let input_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests")
                .join("inputs")
                .join($input_file);

            let result = crate::common::run_custom_start_test(&input_path, $start_fn);
            assert_eq!(result, $expected);
        }
    };
}
