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

// Unsized-tail ADTs: size/align combine the sized prefix with the tail's
// runtime layout. Cases vary how the header and tail alignments compare.

check_custom_start!(
    test_unsized_tail_header_align_dominates,
    input = "intrinsics/mem_val.rs",
    start_fn = "header_align_dominates",
);

check_custom_start!(
    test_unsized_tail_align_dominates,
    input = "intrinsics/mem_val.rs",
    start_fn = "tail_align_dominates",
);

check_custom_start!(
    test_unsized_tail_equal_align,
    input = "intrinsics/mem_val.rs",
    start_fn = "equal_align",
);

check_custom_start!(
    test_unsized_tail_size_rounds_up,
    input = "intrinsics/mem_val.rs",
    start_fn = "size_rounds_up_to_align",
);

check_custom_start!(
    test_unsized_tail_empty,
    input = "intrinsics/mem_val.rs",
    start_fn = "empty_tail",
);

check_custom_start!(
    test_unsized_tail_nested,
    input = "intrinsics/mem_val.rs",
    start_fn = "nested_unsized_tail",
);

check_custom_start!(
    test_slice_size_exceeds_isize_max,
    input = "intrinsics/mem_val.rs",
    start_fn = "slice_size_exceeds_isize_max",
    result = TestResult::ErrorRegex(r".*exceeds the maximum object size.*".to_string()),
);

check_custom_start!(
    test_dyn_size_align,
    input = "intrinsics/mem_val.rs",
    start_fn = "dyn_size_align",
);

check_custom_start!(
    test_upcast_dyn_size_align,
    input = "intrinsics/mem_val.rs",
    start_fn = "upcast_dyn_size_align",
);

check_custom_start!(
    test_upcast_dyn_secondary_supertrait,
    input = "intrinsics/mem_val.rs",
    start_fn = "upcast_dyn_secondary_supertrait",
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

// --- `copy_nonoverlapping` / `copy` ---

check_custom_start!(
    test_copy_nonoverlapping_basic,
    input = "intrinsics/mem_copy.rs",
    start_fn = "copy_nonoverlapping_basic",
);

check_custom_start!(
    test_copy_nonoverlapping_structs,
    input = "intrinsics/mem_copy.rs",
    start_fn = "copy_nonoverlapping_structs",
);

check_custom_start!(
    test_copy_overlapping,
    input = "intrinsics/mem_copy.rs",
    start_fn = "copy_overlapping",
);

// --- Raw pointer arithmetic ---

check_custom_start!(
    test_ptr_add,
    input = "intrinsics/ptr_arith.rs",
    start_fn = "ptr_add",
);

check_custom_start!(
    test_ptr_add_scales_by_size,
    input = "intrinsics/ptr_arith.rs",
    start_fn = "ptr_add_scales_by_size",
);

check_custom_start!(
    test_ptr_negative_offset,
    input = "intrinsics/ptr_arith.rs",
    start_fn = "ptr_negative_offset",
);

check_custom_start!(
    test_ptr_sub,
    input = "intrinsics/ptr_arith.rs",
    start_fn = "ptr_sub",
);

check_custom_start!(
    test_ptr_one_past_end,
    input = "intrinsics/ptr_arith.rs",
    start_fn = "ptr_one_past_end",
);

check_custom_start!(
    test_ptr_offset_from,
    input = "intrinsics/ptr_arith.rs",
    start_fn = "ptr_offset_from",
);

check_custom_start!(
    test_ptr_offset_from_unsigned,
    input = "intrinsics/ptr_arith.rs",
    start_fn = "ptr_offset_from_unsigned",
);

check_custom_start!(
    test_ptr_addr_roundtrip,
    input = "intrinsics/ptr_arith.rs",
    start_fn = "addr_roundtrip",
);

check_custom_start!(
    test_ptr_addr_truncates_to_narrower_int,
    input = "intrinsics/ptr_arith.rs",
    start_fn = "addr_truncates_to_narrower_int",
);

check_custom_start!(
    test_ptr_offset_from_not_multiple,
    input = "intrinsics/ptr_arith.rs",
    start_fn = "offset_from_not_multiple",
    result = TestResult::ErrorRegex(r".*not a multiple of the size of `u32`.*".to_string())
);

check_custom_start!(
    test_ptr_offset_overflows_isize,
    input = "intrinsics/ptr_arith.rs",
    start_fn = "offset_overflows_isize",
    result = TestResult::ErrorRegex(r".*overflows an `isize`.*".to_string())
);

check_custom_start!(
    test_ptr_offset_from_zst,
    input = "intrinsics/ptr_offset_from_ub.rs",
    start_fn = "offset_from_zst",
    result = TestResult::ErrorRegex(r".*not supported on the zero-sized type `\(\)`.*".to_string())
);

check_custom_start!(
    test_ptr_offset_from_unsigned_backwards,
    input = "intrinsics/ptr_offset_from_ub.rs",
    start_fn = "offset_from_unsigned_backwards",
    result = TestResult::ErrorRegex(r".*before `base`.*negative distance.*".to_string())
);

// --- Atomics ---

check_custom_start!(
    test_atomic_load_store,
    input = "intrinsics/atomics.rs",
    start_fn = "load_store",
);

check_custom_start!(
    test_atomic_swap,
    input = "intrinsics/atomics.rs",
    start_fn = "swap",
);

check_custom_start!(
    test_atomic_compare_exchange,
    input = "intrinsics/atomics.rs",
    start_fn = "compare_exchange",
);

check_custom_start!(
    test_atomic_compare_exchange_weak,
    input = "intrinsics/atomics.rs",
    start_fn = "compare_exchange_weak",
);

check_custom_start!(
    test_atomic_fetch_ops,
    input = "intrinsics/atomics.rs",
    start_fn = "fetch_ops",
);

check_custom_start!(
    test_atomic_fetch_add_wraps,
    input = "intrinsics/atomics.rs",
    start_fn = "fetch_add_wraps",
);

check_custom_start!(
    test_atomic_fetch_max_min_signed,
    input = "intrinsics/atomics.rs",
    start_fn = "fetch_max_min_signed",
);

check_custom_start!(
    test_atomic_fetch_max_min_unsigned,
    input = "intrinsics/atomics.rs",
    start_fn = "fetch_max_min_unsigned",
);

check_custom_start!(
    test_atomic_bool,
    input = "intrinsics/atomics.rs",
    start_fn = "atomic_bool",
);

check_custom_start!(
    test_atomic_ptr,
    input = "intrinsics/atomics.rs",
    start_fn = "atomic_ptr",
);

check_custom_start!(
    test_atomic_fences,
    input = "intrinsics/atomics.rs",
    start_fn = "fences",
);

check_custom_start!(
    test_atomic_arc_refcount,
    input = "intrinsics/atomics.rs",
    start_fn = "arc_refcount",
);
