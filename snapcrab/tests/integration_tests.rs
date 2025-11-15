#![feature(rustc_private)]

#[macro_use]
mod common;

use common::TestResult;

check_interpreter!(
    test_simple_error,
    input = "simple_main.rs",
    result = TestResult::ErrorRegex(r".*Uninitialized local.*".to_string())
);

check_interpreter!(
    test_arithmetic_error, 
    input = "arithmetic.rs",
    result = TestResult::ErrorRegex(r".*Unsupported rvalue.*CheckedBinaryOp.*".to_string())
);
