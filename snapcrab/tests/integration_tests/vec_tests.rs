check_custom_start!(
    test_vec_push_and_iter,
    input = "vec_ops.rs",
    start_fn = "push_and_iter",
);

check_custom_start!(
    test_vec_grow_reallocates,
    input = "vec_ops.rs",
    start_fn = "grow_reallocates",
);

check_custom_start!(test_vec_macro, input = "vec_ops.rs", start_fn = "vec_macro",);

check_custom_start!(
    test_vec_pop_returns_last,
    input = "vec_ops.rs",
    start_fn = "pop_returns_last",
);

check_custom_start!(
    test_vec_collect_from_iter,
    input = "vec_ops.rs",
    start_fn = "collect_from_iter",
);

check_custom_start!(
    test_vec_of_strings,
    input = "vec_ops.rs",
    start_fn = "vec_of_strings",
);

check_custom_start!(
    test_vec_drop_frees_buffer,
    input = "vec_ops.rs",
    start_fn = "drop_frees_buffer",
);
