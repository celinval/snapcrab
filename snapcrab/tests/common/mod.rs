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
    Failure,
    Error(String),
    ErrorRegex(String),
}

impl PartialEq for TestResult {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TestResult::Success, TestResult::Success) => true,
            (TestResult::Failure, TestResult::Failure) => true,
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

pub fn run_interpreter_test(input_file: &Path, _flags: &[&str]) -> TestResult {
    // Set up rustc environment to compile the input file
    let rustc_args = vec![
        "snapcrab".to_string(),
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

#[macro_export]
macro_rules! check_interpreter {
    ($test_name:ident, input=$input_file:expr, result=$expected:expr) => {
        #[test]
        fn $test_name() {
            let input_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests")
                .join("inputs")
                .join($input_file);

            let result = crate::common::run_interpreter_test(&input_path, &[]);
            assert_eq!(result, $expected);
        }
    };

    ($test_name:ident, input=$input_file:expr, flags=$flags:expr, result=$expected:expr) => {
        #[test]
        fn $test_name() {
            let input_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests")
                .join("inputs")
                .join($input_file);

            let result = crate::common::run_interpreter_test(&input_path, $flags);
            assert_eq!(result, $expected);
        }
    };
}
