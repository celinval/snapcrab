// --- Option operations ---

check_custom_start!(
    test_std_option_unwrap,
    input = "std_ops/option.rs",
    start_fn = "test_option_unwrap",
);

check_custom_start!(
    test_std_option_unwrap_or,
    input = "std_ops/option.rs",
    start_fn = "test_option_unwrap_or",
);

check_custom_start!(
    #[ignore]
    test_std_option_map,
    input = "std_ops/option.rs",
    start_fn = "test_option_map",
);

check_custom_start!(
    #[ignore]
    test_std_option_and_then,
    input = "std_ops/option.rs",
    start_fn = "test_option_and_then",
);

check_custom_start!(
    test_std_option_and_then_none,
    input = "std_ops/option.rs",
    start_fn = "test_option_and_then_none",
);

check_custom_start!(
    test_std_option_is_some_and,
    input = "std_ops/option.rs",
    start_fn = "test_option_is_some_and",
);

// --- Result operations ---

check_custom_start!(
    test_std_result_ok,
    input = "std_ops/result.rs",
    start_fn = "test_result_ok",
);

check_custom_start!(
    test_std_result_err,
    input = "std_ops/result.rs",
    start_fn = "test_result_err",
);

check_custom_start!(
    test_std_result_unwrap,
    input = "std_ops/result.rs",
    start_fn = "test_result_unwrap",
);

check_custom_start!(
    #[ignore]
    test_std_result_unwrap_or,
    input = "std_ops/result.rs",
    start_fn = "test_result_unwrap_or",
);

check_custom_start!(
    #[ignore]
    test_std_result_map,
    input = "std_ops/result.rs",
    start_fn = "test_result_map",
);

check_custom_start!(
    #[ignore]
    test_std_result_and_then,
    input = "std_ops/result.rs",
    start_fn = "test_result_and_then",
);

// --- Numeric operations ---

check_custom_start!(
    test_std_u32_min_max,
    input = "std_ops/numeric.rs",
    start_fn = "test_u32_min_max",
);

check_custom_start!(
    test_std_i32_min_max,
    input = "std_ops/numeric.rs",
    start_fn = "test_i32_min_max",
);

check_custom_start!(
    #[ignore]
    test_std_wrapping_add,
    input = "std_ops/numeric.rs",
    start_fn = "test_wrapping_add",
);

check_custom_start!(
    #[ignore]
    test_std_wrapping_sub,
    input = "std_ops/numeric.rs",
    start_fn = "test_wrapping_sub",
);

check_custom_start!(
    #[ignore]
    test_std_saturating_add,
    input = "std_ops/numeric.rs",
    start_fn = "test_saturating_add",
);

check_custom_start!(
    #[ignore]
    test_std_saturating_sub,
    input = "std_ops/numeric.rs",
    start_fn = "test_saturating_sub",
);

check_custom_start!(
    #[ignore]
    test_std_checked_add_some,
    input = "std_ops/numeric.rs",
    start_fn = "test_checked_add_some",
);

check_custom_start!(
    #[ignore]
    test_std_checked_add_overflow,
    input = "std_ops/numeric.rs",
    start_fn = "test_checked_add_overflow",
);

check_custom_start!(
    test_std_pow,
    input = "std_ops/numeric.rs",
    start_fn = "test_pow",
);

check_custom_start!(
    #[ignore]
    test_std_leading_zeros,
    input = "std_ops/numeric.rs",
    start_fn = "test_leading_zeros",
);

check_custom_start!(
    #[ignore]
    test_std_count_ones,
    input = "std_ops/numeric.rs",
    start_fn = "test_count_ones",
);

check_custom_start!(
    #[ignore]
    test_std_swap_bytes,
    input = "std_ops/numeric.rs",
    start_fn = "test_swap_bytes",
);

check_custom_start!(
    #[ignore]
    test_std_rotate_left,
    input = "std_ops/numeric.rs",
    start_fn = "test_rotate_left",
);

// --- Comparison operations ---

check_custom_start!(
    #[ignore]
    test_std_min,
    input = "std_ops/cmp.rs",
    start_fn = "test_min",
);

check_custom_start!(
    #[ignore]
    test_std_max,
    input = "std_ops/cmp.rs",
    start_fn = "test_max",
);

check_custom_start!(
    test_std_clamp,
    input = "std_ops/cmp.rs",
    start_fn = "test_clamp",
);

check_custom_start!(
    #[ignore]
    test_std_ord_methods,
    input = "std_ops/cmp.rs",
    start_fn = "test_ord_methods",
);
