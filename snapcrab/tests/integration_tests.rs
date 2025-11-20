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
    result = TestResult::SuccessWithValue(Value::from_i32(123))
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
    test_tuple_creation,
    input = "tuple_function.rs",
    start_fn = "simple_tuple",
    result = TestResult::SuccessWithValue(Value::from_bytes(&[1, 42, 0, 0, 232, 3, 0, 0]))
);

check_custom_start!(
    test_simple_tuple_ops,
    input = "tuple_operations.rs", 
    start_fn = "simple_tuple",
    result = TestResult::SuccessWithValue(Value::from_bytes(&[1, 42, 0, 0, 232, 3, 0, 0]))
);

check_custom_start!(
    test_unit_tuple,
    input = "tuple_operations.rs",
    start_fn = "unit_tuple",
    result = TestResult::SuccessWithValue(Value::unit().clone())
);

check_custom_start!(
    test_single_element_tuple,
    input = "tuple_operations.rs", 
    start_fn = "single_element_tuple",
    result = TestResult::SuccessWithValue(Value::from_i32(42))
);

check_custom_start!(
    test_reordered_tuple,
    input = "tuple_operations.rs",
    start_fn = "reordered_tuple",
    result = TestResult::SuccessWithValue(Value::from_bytes(&[232, 3, 0, 0, 42, 1, 0, 0]))
);

check_custom_start!(
    test_another_order,
    input = "tuple_operations.rs", 
    start_fn = "another_order",
    result = TestResult::SuccessWithValue(Value::from_bytes(&[232, 3, 0, 0, 1, 42, 0, 0]))
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
