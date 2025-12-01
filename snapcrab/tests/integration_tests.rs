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
    result = TestResult::SuccessWithValue(vec![123, 0, 0, 0])
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

#[rustfmt::skip]
check_custom_start!(
    test_tuple_creation,
    input = "tuple_function.rs",
    start_fn = "simple_tuple",
    result = TestResult::SuccessWithValue(vec![
        1,
        42, 0, 0,
        232, 3, 0, 0
    ])
);

#[rustfmt::skip]
check_custom_start!(
    test_simple_tuple_ops,
    input = "tuple_operations.rs",
    start_fn = "simple_tuple",
    result = TestResult::SuccessWithValue(vec![
        1,
        42, 0, 0,
        232, 3, 0, 0
    ])
);

check_custom_start!(
    test_unit_tuple,
    input = "tuple_operations.rs",
    start_fn = "unit_tuple",
    result = TestResult::SuccessWithValue(vec![])
);

check_custom_start!(
    test_single_element_tuple,
    input = "tuple_operations.rs",
    start_fn = "single_element_tuple",
    result = TestResult::SuccessWithValue(vec![42, 0, 0, 0])
);

#[rustfmt::skip]
check_custom_start!(
    test_reordered_tuple,
    input = "tuple_operations.rs",
    start_fn = "reordered_tuple",
    result = TestResult::SuccessWithValue(vec![
        232, 3, 0, 0,
        42, 1, 0, 0
    ])
);

#[rustfmt::skip]
check_custom_start!(
    test_another_order,
    input = "tuple_operations.rs",
    start_fn = "another_order",
    result = TestResult::SuccessWithValue(vec![
        232, 3, 0, 0,
        1,
        42, 0, 0
    ])
);

check_custom_start!(
    test_bool_return,
    input = "bool_function.rs",
    start_fn = "returns_true",
    result = TestResult::SuccessWithValue(vec![1])
);

check_custom_start!(
    test_mut_reference,
    input = "reference_test.rs",
    start_fn = "test_mut_ref",
    result = TestResult::SuccessWithValue(vec![100, 0, 0, 0])
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
    result = TestResult::SuccessWithValue(vec![15, 0, 0, 0])
);

check_custom_start!(
    test_basic_reference,
    input = "reference_test.rs",
    start_fn = "test_basic_ref",
    result = TestResult::SuccessWithValue(vec![42, 0, 0, 0])
);

#[cfg(target_endian = "little")]
check_custom_start!(
    test_tuple_field_ref,
    input = "reference_test.rs",
    start_fn = "test_tuple_field_ref",
    result = TestResult::SuccessWithValue(vec![52, 10])
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
    result = TestResult::SuccessWithValue(vec![42])
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
    result = TestResult::SuccessWithValue(vec![1])
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
    result = TestResult::SuccessWithValue(vec![1])
);

check_custom_start!(
    test_check_align_of,
    input = "size_align_test.rs",
    start_fn = "check_align_of",
    result = TestResult::SuccessWithValue(vec![1])
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

// Struct tests
check_custom_start!(
    test_empty_struct,
    input = "struct_empty.rs",
    start_fn = "create_empty",
    result = TestResult::SuccessWithValue(vec![])
);

check_custom_start!(
    test_single_element_struct,
    input = "struct_single.rs",
    start_fn = "create_single",
    result = TestResult::SuccessWithValue(vec![42, 0, 0, 0])
);

check_custom_start!(
    test_read_struct_field,
    input = "struct_single.rs",
    start_fn = "read_field",
    result = TestResult::SuccessWithValue(vec![100, 0, 0, 0])
);

check_custom_start!(
    test_write_struct_field,
    input = "struct_single.rs",
    start_fn = "write_field",
    result = TestResult::SuccessWithValue(vec![50, 0, 0, 0])
);

#[rustfmt::skip]
#[cfg(target_endian = "little")]
check_custom_start!(
    test_generic_struct_u8_u128_i16,
    input = "struct_generic.rs",
    start_fn = "create_triple_u8_u128_i16",
    result = TestResult::SuccessWithValue(vec![
        232, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        206, 255,
        10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
    ])
);

#[rustfmt::skip]
check_custom_start!(
    test_generic_struct_i32_unit_bool,
    input = "struct_generic.rs",
    start_fn = "create_triple_i32_unit_bool",
    result = TestResult::SuccessWithValue(vec![
        42, 0, 0, 0,
        1, 0, 0, 0
    ])
);

check_custom_start!(
    test_read_generic_field,
    input = "struct_generic.rs",
    start_fn = "read_generic_field",
    result = TestResult::SuccessWithValue(vec![99, 0, 0, 0])
);

#[rustfmt::skip]
check_custom_start!(
    test_write_generic_field,
    input = "struct_generic.rs",
    start_fn = "write_generic_field",
    result = TestResult::SuccessWithValue(vec![
        1, 0, 0, 0,
        1, 0, 0, 0
    ])
);

#[rustfmt::skip]
check_custom_start!(
    test_nested_struct,
    input = "struct_nested.rs",
    start_fn = "create_nested",
    result = TestResult::SuccessWithValue(vec![
        10, 0, 0, 0,
        20, 0, 0, 0,
        1, 0, 0, 0
    ])
);

check_custom_start!(
    test_read_nested_field,
    input = "struct_nested.rs",
    start_fn = "read_nested_field",
    result = TestResult::SuccessWithValue(vec![5, 0, 0, 0])
);

#[rustfmt::skip]
check_custom_start!(
    test_write_nested_field,
    input = "struct_nested.rs",
    start_fn = "write_nested_field",
    result = TestResult::SuccessWithValue(vec![
        1, 0, 0, 0,
        99, 0, 0, 0,
        1, 0, 0, 0
    ])
);

#[rustfmt::skip]
check_custom_start!(
    test_struct_to_tuple,
    input = "struct_nested.rs",
    start_fn = "struct_to_tuple",
    result = TestResult::SuccessWithValue(vec![
        7, 0, 0, 0,
        8, 0, 0, 0,
        1, 0, 0, 0
    ])
);

// Union tests
check_custom_start!(
    test_union_int,
    input = "union_test.rs",
    start_fn = "create_union_int",
    result = TestResult::SuccessWithValue(vec![42, 0, 0, 0])
);

check_custom_start!(
    test_union_float,
    input = "union_test.rs",
    start_fn = "create_union_float",
    result = TestResult::SuccessWithValue(vec![195, 245, 72, 64])
);

check_custom_start!(
    test_union_empty_variant,
    input = "union_test.rs",
    start_fn = "create_union_empty",
    result = TestResult::SuccessWithValue(vec![0, 0, 0, 0])
);

check_custom_start!(
    test_union_value_variant,
    input = "union_test.rs",
    start_fn = "create_union_value",
    result = TestResult::SuccessWithValue(vec![100, 0, 0, 0])
);

check_custom_start!(
    test_union_assign_all_fields,
    input = "union_test.rs",
    start_fn = "assign_all_fields",
    result = TestResult::SuccessWithValue(vec![120, 86, 52, 18])
);

check_custom_start!(
    test_read_union_field,
    input = "union_test.rs",
    start_fn = "read_union_field",
    result = TestResult::SuccessWithValue(vec![42, 0, 0, 0])
);

check_custom_start!(
    test_write_union_field,
    input = "union_test.rs",
    start_fn = "write_union_field",
    result = TestResult::SuccessWithValue(vec![77, 0, 0, 0])
);

// Array tests
check_custom_start!(
    test_empty_array,
    input = "array_test.rs",
    start_fn = "create_empty_array",
    result = TestResult::SuccessWithValue(vec![])
);

#[rustfmt::skip]
check_custom_start!(
    test_array_repeat,
    input = "array_test.rs",
    start_fn = "create_array_repeat",
    result = TestResult::SuccessWithValue(vec![
        42, 0, 0, 0,
        42, 0, 0, 0,
        42, 0, 0, 0,
        42, 0, 0, 0,
        42, 0, 0, 0
    ])
);

#[rustfmt::skip]
check_custom_start!(
    test_array_explicit,
    input = "array_test.rs",
    start_fn = "create_array_explicit",
    result = TestResult::SuccessWithValue(vec![
        10, 0, 0, 0,
        20, 0, 0, 0,
        30, 0, 0, 0,
        40, 0, 0, 0
    ])
);

#[rustfmt::skip]
check_custom_start!(
    test_struct_array_repeat,
    input = "array_test.rs",
    start_fn = "create_struct_array_repeat",
    result = TestResult::SuccessWithValue(vec![
        1, 0, 0, 0, 2, 0, 0, 0,
        1, 0, 0, 0, 2, 0, 0, 0,
        1, 0, 0, 0, 2, 0, 0, 0
    ])
);

#[rustfmt::skip]
check_custom_start!(
    test_struct_array_explicit,
    input = "array_test.rs",
    start_fn = "create_struct_array_explicit",
    result = TestResult::SuccessWithValue(vec![
        5, 0, 0, 0, 10, 0, 0, 0,
        15, 0, 0, 0, 20, 0, 0, 0
    ])
);

check_custom_start!(
    test_single_element_array,
    input = "array_test.rs",
    start_fn = "create_single_element_array",
    result = TestResult::SuccessWithValue(vec![99, 0, 0, 0])
);

#[rustfmt::skip]
check_custom_start!(
    test_zero_array,
    input = "array_test.rs",
    start_fn = "create_zero_array",
    result = TestResult::SuccessWithValue(vec![
        0, 0, 0, 0,
        0, 0, 0, 0,
        0, 0, 0, 0,
        0, 0, 0, 0
    ])
);

check_custom_start!(
    test_read_array_element,
    input = "array_test.rs",
    start_fn = "read_array_element",
    result = TestResult::SuccessWithValue(vec![30, 0, 0, 0])
);

#[rustfmt::skip]
check_custom_start!(
    test_write_array_element,
    input = "array_test.rs",
    start_fn = "write_array_element",
    result = TestResult::SuccessWithValue(vec![
        1, 0, 0, 0,
        99, 0, 0, 0,
        3, 0, 0, 0,
        4, 0, 0, 0
    ])
);

check_custom_start!(
    test_read_struct_array_element,
    input = "array_test.rs",
    start_fn = "read_struct_array_element",
    result = TestResult::SuccessWithValue(vec![3, 0, 0, 0, 4, 0, 0, 0])
);

#[rustfmt::skip]
check_custom_start!(
    test_write_struct_array_element,
    input = "array_test.rs",
    start_fn = "write_struct_array_element",
    result = TestResult::SuccessWithValue(vec![
        0, 0, 0, 0, 0, 0, 0, 0,
        5, 0, 0, 0, 10, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0
    ])
);

#[rustfmt::skip]
check_custom_start!(
    test_write_via_mut_ref,
    input = "array_test.rs",
    start_fn = "write_via_mut_ref",
    result = TestResult::SuccessWithValue(vec![
        10, 0, 0, 0,
        20, 0, 0, 0,
        99, 0, 0, 0,
        40, 0, 0, 0,
        50, 0, 0, 0
    ])
);
