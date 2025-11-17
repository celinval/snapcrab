#![feature(rustc_private)]

#[macro_use]
mod common;

use common::TestResult;
use snapcrab::value::Value;

check_interpreter!(
    test_simple_success,
    input = "simple_main.rs",
    result = TestResult::Success
);

check_interpreter!(
    test_function_call,
    input = "function_call.rs",
    result = TestResult::Success
);

check_interpreter!(
    test_arithmetic_error,
    input = "arithmetic.rs",
    result = TestResult::ErrorRegex(r".*Unsupported rvalue.*CheckedBinaryOp.*".to_string())
);

// Custom start function tests
check_custom_start!(
    test_valid_custom_start,
    input = "valid_custom_start.rs",
    start_fn = "my_custom_start",
    result = TestResult::SuccessWithValue(Value::from_i128(123))
);

check_custom_start!(
    test_function_with_args_fails,
    input = "function_with_args.rs", 
    start_fn = "takes_argument",
    result = TestResult::ErrorRegex(r".*takes \d+ arguments.*".to_string())
);

check_custom_start!(
    test_generic_function_fails,
    input = "generic_function.rs",
    start_fn = "generic_function", 
    result = TestResult::ErrorRegex(r".*Failed to create instance.*".to_string())
);

check_custom_start!(
    test_bool_return,
    input = "bool_function.rs",
    start_fn = "returns_true",
    result = TestResult::SuccessWithValue(Value::from_bool(true))
);

check_custom_start!(
    test_nonexistent_function_fails,
    input = "valid_custom_start.rs",
    start_fn = "nonexistent_function",
    result = TestResult::ErrorRegex(r".*Function.*not found.*".to_string())
);
