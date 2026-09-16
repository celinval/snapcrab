use crate::common::TestResult;

// --- Valid transmutes ---

check_custom_start!(
    test_transmute_u32_to_i32,
    input = "intrinsics/transmute_valid.rs",
    start_fn = "test_u32_to_i32",
);

check_custom_start!(
    test_transmute_u8_array_to_u32,
    input = "intrinsics/transmute_valid.rs",
    start_fn = "test_u8_array_to_u32",
);

check_custom_start!(
    test_transmute_u32_to_u8_array,
    input = "intrinsics/transmute_valid.rs",
    start_fn = "test_u32_to_u8_array",
);

check_custom_start!(
    test_transmute_struct_to_struct,
    input = "intrinsics/transmute_valid.rs",
    start_fn = "test_struct_to_struct",
);

check_custom_start!(
    test_transmute_bool_to_u8,
    input = "intrinsics/transmute_valid.rs",
    start_fn = "test_bool_to_u8",
);

check_custom_start!(
    test_transmute_u8_to_bool_valid,
    input = "intrinsics/transmute_valid.rs",
    start_fn = "test_u8_to_bool_valid",
);

check_custom_start!(
    test_transmute_i64_to_u64,
    input = "intrinsics/transmute_valid.rs",
    start_fn = "test_i64_to_u64",
);

check_custom_start!(
    test_transmute_unit_struct,
    input = "intrinsics/transmute_valid.rs",
    start_fn = "test_unit_struct_transmute",
);

// --- Invalid transmutes (should be caught by validity checking) ---

check_custom_start!(
    test_transmute_zero_to_nonzero,
    input = "intrinsics/transmute_invalid.rs",
    start_fn = "test_zero_to_nonzero",
    result = TestResult::ErrorRegex(
        r".*Invalid unsigned integer value 0x0 for type.*valid range.*".to_string()
    )
);

check_custom_start!(
    test_transmute_invalid_bool,
    input = "intrinsics/transmute_invalid.rs",
    start_fn = "test_invalid_bool",
    result = TestResult::ErrorRegex(
        r".*Invalid unsigned integer value 0x2 for type `bool`.*valid range: 0..=1.*".to_string()
    )
);

check_custom_start!(
    test_transmute_invalid_bool_255,
    input = "intrinsics/transmute_invalid.rs",
    start_fn = "test_invalid_bool_255",
    result = TestResult::ErrorRegex(
        r".*Invalid unsigned integer value 0xff for type `bool`.*valid range: 0..=1.*".to_string()
    )
);

check_custom_start!(
    test_transmute_invalid_enum_discriminant,
    input = "intrinsics/transmute_invalid.rs",
    start_fn = "test_invalid_enum_discriminant",
    result = TestResult::ErrorRegex(
        r".*Assertion failed.*construct an enum from an invalid value.*".to_string()
    )
);

// --- Bit-counting intrinsics (ctpop / cttz / ctlz and their nonzero forms) ---

check_custom_start!(
    test_ctpop,
    input = "intrinsics/bit_ops.rs",
    start_fn = "count_ones",
);

check_custom_start!(
    test_cttz,
    input = "intrinsics/bit_ops.rs",
    start_fn = "trailing_zeros",
);

check_custom_start!(
    test_ctlz,
    input = "intrinsics/bit_ops.rs",
    start_fn = "leading_zeros",
);

check_custom_start!(
    test_ctlz_cttz_nonzero,
    input = "intrinsics/bit_ops.rs",
    start_fn = "nonzero_bit_ops",
);

// --- size_of_val / align_of_val ---

check_custom_start!(
    test_size_of_val,
    input = "intrinsics/mem_val.rs",
    start_fn = "size_of_val",
);

check_custom_start!(
    test_align_of_val,
    input = "intrinsics/mem_val.rs",
    start_fn = "align_of_val",
);

check_custom_start!(
    test_size_of_val_unsized_tail,
    input = "intrinsics/mem_val.rs",
    start_fn = "size_of_val_unsized_tail",
    result = TestResult::ErrorRegex(
        r".*`intrinsics::size_of_val` does not yet support the unsized type.*".to_string()
    ),
);

check_custom_start!(
    test_align_of_val_unsized_tail,
    input = "intrinsics/mem_val.rs",
    start_fn = "align_of_val_unsized_tail",
    result = TestResult::ErrorRegex(
        r".*`intrinsics::align_of_val` does not yet support the unsized type.*".to_string()
    ),
);

// --- Unchecked integer arithmetic ---

check_custom_start!(
    test_unchecked_add,
    input = "intrinsics/unchecked_arith.rs",
    start_fn = "unchecked_add",
);

check_custom_start!(
    test_unchecked_sub,
    input = "intrinsics/unchecked_arith.rs",
    start_fn = "unchecked_sub",
);

check_custom_start!(
    test_unchecked_mul,
    input = "intrinsics/unchecked_arith.rs",
    start_fn = "unchecked_mul",
);
