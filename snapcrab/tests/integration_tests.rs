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
    result = TestResult::SuccessWithValue(Value::from_type(123))
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
    result = TestResult::SuccessWithValue(Value::from_type(42))
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
    test_mut_reference,
    input = "reference_test.rs",
    start_fn = "test_mut_ref",
    result = TestResult::SuccessWithValue(Value::from_type(100i32))
);

check_custom_start!(
    test_multiple_references,
    input = "reference_test.rs",
    start_fn = "test_multiple_refs",
    result = TestResult::ErrorRegex(r".*Unsupported rvalue.*CheckedBinaryOp.*".to_string())
);

check_custom_start!(
    test_reference_to_reference,
    input = "reference_test.rs",
    start_fn = "test_ref_to_ref",
    result = TestResult::ErrorRegex(r".*Unsupported rvalue.*CopyForDeref.*".to_string())
);

check_custom_start!(
    test_mutable_reference_chain,
    input = "reference_test.rs",
    start_fn = "test_mut_ref_chain",
    result = TestResult::SuccessWithValue(Value::from_type(15i32))
);

check_custom_start!(
    test_basic_reference,
    input = "reference_test.rs",
    start_fn = "test_basic_ref",
    result = TestResult::SuccessWithValue(Value::from_type(42i32))
);

#[cfg(target_endian = "little")]
check_custom_start!(
    test_tuple_field_ref,
    input = "reference_test.rs",
    start_fn = "test_tuple_field_ref",
    result = TestResult::SuccessWithValue(Value::from_bytes(&[52, 10]))
);

check_custom_start!(
    test_double_deref,
    input = "reference_test.rs",
    start_fn = "test_double_deref",
    result = TestResult::ErrorRegex(r".*Unsupported rvalue.*CopyForDeref.*".to_string())
);

check_custom_start!(
    test_tuple_field_sub,
    input = "tuple_operations.rs",
    start_fn = "tuple_field_sub",
    result = TestResult::SuccessWithValue(Value::from_type(42i8))
);

check_custom_start!(
    test_nonexistent_function_fails,
    input = "valid_custom_start.rs",
    start_fn = "nonexistent_function",
    result = TestResult::ErrorRegex(r".*Function.*not found.*".to_string())
);

check_custom_start!(
    test_ptr_compare,
    input = "raw_pointer_test.rs",
    start_fn = "ptr_compare",
    result = TestResult::SuccessWithValue(Value::from_bool(true))
);

check_custom_start!(
    test_use_dangling_ptr,
    input = "raw_pointer_test.rs",
    start_fn = "use_dangling_ptr",
    result = TestResult::ErrorRegex(
        r".*(memory access out of bounds|not found in any memory segment).*".to_string()
    )
);

check_custom_start!(
    test_read_misaligned_ptr,
    input = "raw_pointer_test.rs",
    start_fn = "read_misaligned_ptr",
    result = TestResult::ErrorRegex(r".*Unsupported.*Offset.*".to_string())
);

check_custom_start!(
    test_check_size_of,
    input = "size_align_test.rs",
    start_fn = "check_size_of",
    result = TestResult::SuccessWithValue(Value::from_bool(true))
);

check_custom_start!(
    test_check_align_of,
    input = "size_align_test.rs",
    start_fn = "check_align_of",
    result = TestResult::SuccessWithValue(Value::from_bool(true))
);

check_custom_start!(
    test_error_deep_call,
    input = "diagnostic_test.rs",
    start_fn = "error_deep_call",
    result = TestResult::ErrorRegex(
        r"(?s).*panicked at.*diagnostic_test.rs.*divide by zero.*".to_string()
    )
);

check_custom_start!(
    test_division_by_zero_diagnostic,
    input = "diagnostic_test.rs",
    start_fn = "div_by_zero",
    result = TestResult::ErrorRegex(
        r"(?s).*panicked at.*diagnostic_test.rs.*divide by zero.*".to_string()
    )
);
