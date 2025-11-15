#![feature(rustc_private)]

#[macro_use]
mod common;

use common::TestResult;

check_interpreter!(
    test_simple_success,
    input = "simple_main.rs",
    result = TestResult::Success
);

check_interpreter!(
    test_arithmetic_error, 
    input = "arithmetic.rs",
    result = TestResult::ErrorRegex(r".*Unsupported rvalue.*CheckedBinaryOp.*".to_string())
);
