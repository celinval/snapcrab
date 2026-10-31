check_custom_start!(
    test_float_equality,
    input = "float_ops.rs",
    start_fn = "float_equality",
);

check_custom_start!(
    test_float_ordering,
    input = "float_ops.rs",
    start_fn = "float_ordering",
);

check_custom_start!(
    test_float_partial_cmp,
    input = "float_ops.rs",
    start_fn = "float_partial_cmp",
);

check_custom_start!(
    test_float_total_cmp,
    input = "float_ops.rs",
    start_fn = "float_total_cmp",
);

check_custom_start!(
    test_float_arithmetic,
    input = "float_ops.rs",
    start_fn = "float_arithmetic",
);

check_custom_start!(
    test_float_negation,
    input = "float_ops.rs",
    start_fn = "float_negation",
);

check_custom_start!(
    test_float_min_max,
    input = "float_ops.rs",
    start_fn = "float_min_max",
);

check_custom_start!(
    test_three_way_compare,
    input = "float_ops.rs",
    start_fn = "three_way_compare",
);
