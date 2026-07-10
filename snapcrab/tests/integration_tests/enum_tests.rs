use crate::common::TestResult;

// --- Basic enum tests (create + return raw value) ---

check_custom_start!(
    #[ignore]
    test_enum_create_north,
    input = "enum/basic.rs",
    start_fn = "create_north",
    result = TestResult::SuccessWithValue(vec![0])
);

check_custom_start!(
    #[ignore]
    test_enum_create_west,
    input = "enum/basic.rs",
    start_fn = "create_west",
    result = TestResult::SuccessWithValue(vec![3])
);

check_custom_start!(
    #[ignore]
    test_enum_match_north,
    input = "enum/basic.rs",
    start_fn = "match_north",
    result = TestResult::SuccessWithValue(vec![0, 0, 0, 0])
);

check_custom_start!(
    #[ignore]
    test_enum_match_west,
    input = "enum/basic.rs",
    start_fn = "match_west",
    result = TestResult::SuccessWithValue(vec![3, 0, 0, 0])
);

#[rustfmt::skip]
check_custom_start!(
    #[ignore]
    test_enum_create_some,
    input = "enum/basic.rs",
    start_fn = "create_some",
    result = TestResult::SuccessWithValue(vec![1, 0, 0, 0, 42, 0, 0, 0])
);

#[rustfmt::skip]
check_custom_start!(
    #[ignore]
    test_enum_create_none,
    input = "enum/basic.rs",
    start_fn = "create_none",
    result = TestResult::SuccessWithValue(vec![0, 0, 0, 0, 0, 0, 0, 0])
);

check_custom_start!(
    #[ignore]
    test_enum_unwrap_some,
    input = "enum/basic.rs",
    start_fn = "unwrap_some",
    result = TestResult::SuccessWithValue(vec![99, 0, 0, 0])
);

check_custom_start!(
    #[ignore]
    test_enum_unwrap_none,
    input = "enum/basic.rs",
    start_fn = "unwrap_none",
    result = TestResult::SuccessWithValue(vec![255, 255, 255, 255])
);

#[rustfmt::skip]
check_custom_start!(
    #[ignore]
    test_enum_create_left,
    input = "enum/basic.rs",
    start_fn = "create_left",
    result = TestResult::SuccessWithValue(vec![0, 0, 0, 0, 10, 0, 0, 0])
);

#[rustfmt::skip]
check_custom_start!(
    #[ignore]
    test_enum_create_right,
    input = "enum/basic.rs",
    start_fn = "create_right",
    result = TestResult::SuccessWithValue(vec![1, 255, 0, 0, 0, 0, 0, 0])
);

check_custom_start!(
    #[ignore]
    test_enum_match_left,
    input = "enum/basic.rs",
    start_fn = "match_left",
    result = TestResult::SuccessWithValue(vec![7, 0, 0, 0])
);

check_custom_start!(
    #[ignore]
    test_enum_match_right,
    input = "enum/basic.rs",
    start_fn = "match_right",
    result = TestResult::SuccessWithValue(vec![200, 0, 0, 0])
);

#[rustfmt::skip]
check_custom_start!(
    #[ignore]
    test_enum_create_empty,
    input = "enum/basic.rs",
    start_fn = "create_empty",
    result = TestResult::SuccessWithValue(vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
);

#[rustfmt::skip]
check_custom_start!(
    #[ignore]
    test_enum_create_single,
    input = "enum/basic.rs",
    start_fn = "create_single",
    result = TestResult::SuccessWithValue(vec![1, 0, 0, 0, 77, 0, 0, 0, 0, 0, 0, 0])
);

#[rustfmt::skip]
check_custom_start!(
    #[ignore]
    test_enum_create_pair,
    input = "enum/basic.rs",
    start_fn = "create_pair",
    result = TestResult::SuccessWithValue(vec![2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0])
);

check_custom_start!(
    #[ignore]
    test_enum_match_empty,
    input = "enum/basic.rs",
    start_fn = "match_empty",
    result = TestResult::SuccessWithValue(vec![0, 0, 0, 0])
);

check_custom_start!(
    #[ignore]
    test_enum_match_single,
    input = "enum/basic.rs",
    start_fn = "match_single",
    result = TestResult::SuccessWithValue(vec![55, 0, 0, 0])
);

check_custom_start!(
    #[ignore]
    test_enum_match_pair,
    input = "enum/basic.rs",
    start_fn = "match_pair",
    result = TestResult::SuccessWithValue(vec![30, 0, 0, 0])
);

// --- Fieldless (C-like) enums ---

check_custom_start!(
    #[ignore]
    test_enum_fieldless_direction_discriminants,
    input = "enum/fieldless.rs",
    start_fn = "test_direction_discriminants",
);

check_custom_start!(
    #[ignore]
    test_enum_fieldless_http_status_discriminants,
    input = "enum/fieldless.rs",
    start_fn = "test_http_status_discriminants",
);

check_custom_start!(
    #[ignore]
    test_enum_fieldless_sparse_discriminants,
    input = "enum/fieldless.rs",
    start_fn = "test_sparse_discriminants",
);

check_custom_start!(
    #[ignore]
    test_enum_fieldless_signed_discriminants,
    input = "enum/fieldless.rs",
    start_fn = "test_signed_discriminants",
);

check_custom_start!(
    #[ignore]
    test_enum_fieldless_single_variant,
    input = "enum/fieldless.rs",
    start_fn = "test_single_variant",
);

check_custom_start!(
    #[ignore]
    test_enum_fieldless_match_direction,
    input = "enum/fieldless.rs",
    start_fn = "test_match_direction",
);

check_custom_start!(
    #[ignore]
    test_enum_fieldless_match_http_status,
    input = "enum/fieldless.rs",
    start_fn = "test_match_http_status",
);

// --- Tuple variants ---

check_custom_start!(
    #[ignore]
    test_enum_tuple_option_some,
    input = "enum/tuple_variants.rs",
    start_fn = "test_option_some",
);

check_custom_start!(
    #[ignore]
    test_enum_tuple_option_none,
    input = "enum/tuple_variants.rs",
    start_fn = "test_option_none",
);

check_custom_start!(
    #[ignore]
    test_enum_tuple_shape_circle,
    input = "enum/tuple_variants.rs",
    start_fn = "test_shape_circle",
);

check_custom_start!(
    #[ignore]
    test_enum_tuple_shape_rectangle,
    input = "enum/tuple_variants.rs",
    start_fn = "test_shape_rectangle",
);

check_custom_start!(
    #[ignore]
    test_enum_tuple_shape_triangle,
    input = "enum/tuple_variants.rs",
    start_fn = "test_shape_triangle",
);

check_custom_start!(
    #[ignore]
    test_enum_tuple_single_variant,
    input = "enum/tuple_variants.rs",
    start_fn = "test_single_tuple_variant",
);

check_custom_start!(
    #[ignore]
    test_enum_tuple_all_one,
    input = "enum/tuple_variants.rs",
    start_fn = "test_all_tuples_one",
);

check_custom_start!(
    #[ignore]
    test_enum_tuple_all_three,
    input = "enum/tuple_variants.rs",
    start_fn = "test_all_tuples_three",
);

// --- Struct variants ---

check_custom_start!(
    #[ignore]
    test_enum_struct_message_quit,
    input = "enum/struct_variants.rs",
    start_fn = "test_message_quit",
);

check_custom_start!(
    #[ignore]
    test_enum_struct_message_move,
    input = "enum/struct_variants.rs",
    start_fn = "test_message_move",
);

check_custom_start!(
    #[ignore]
    test_enum_struct_message_write,
    input = "enum/struct_variants.rs",
    start_fn = "test_message_write",
);

check_custom_start!(
    #[ignore]
    test_enum_struct_message_color,
    input = "enum/struct_variants.rs",
    start_fn = "test_message_color",
);

check_custom_start!(
    #[ignore]
    test_enum_struct_single_variant,
    input = "enum/struct_variants.rs",
    start_fn = "test_single_struct_variant",
);

// --- Mixed variants ---

check_custom_start!(
    #[ignore]
    test_enum_mixed_event_nothing,
    input = "enum/mixed_variants.rs",
    start_fn = "test_event_nothing",
);

check_custom_start!(
    #[ignore]
    test_enum_mixed_event_click,
    input = "enum/mixed_variants.rs",
    start_fn = "test_event_click",
);

check_custom_start!(
    #[ignore]
    test_enum_mixed_event_keypress,
    input = "enum/mixed_variants.rs",
    start_fn = "test_event_keypress",
);

check_custom_start!(
    #[ignore]
    test_enum_mixed_event_resize,
    input = "enum/mixed_variants.rs",
    start_fn = "test_event_resize",
);

check_custom_start!(
    #[ignore]
    test_enum_mixed_nested_outer_empty,
    input = "enum/mixed_variants.rs",
    start_fn = "test_nested_outer_empty",
);

check_custom_start!(
    #[ignore]
    test_enum_mixed_nested_inner_a,
    input = "enum/mixed_variants.rs",
    start_fn = "test_nested_inner_a",
);

check_custom_start!(
    #[ignore]
    test_enum_mixed_nested_inner_b,
    input = "enum/mixed_variants.rs",
    start_fn = "test_nested_inner_b",
);

// --- repr-controlled layout ---

check_custom_start!(
    #[ignore]
    test_enum_repr_u8_size,
    input = "enum/repr.rs",
    start_fn = "test_repr_u8_size",
);

check_custom_start!(
    #[ignore]
    test_enum_repr_u8_discriminants,
    input = "enum/repr.rs",
    start_fn = "test_repr_u8_discriminants",
);

check_custom_start!(
    #[ignore]
    test_enum_repr_u16_size,
    input = "enum/repr.rs",
    start_fn = "test_repr_u16_size",
);

check_custom_start!(
    #[ignore]
    test_enum_repr_u16_discriminants,
    input = "enum/repr.rs",
    start_fn = "test_repr_u16_discriminants",
);

check_custom_start!(
    #[ignore]
    test_enum_repr_i8_discriminants,
    input = "enum/repr.rs",
    start_fn = "test_repr_i8_discriminants",
);

check_custom_start!(
    #[ignore]
    test_enum_repr_c_size,
    input = "enum/repr.rs",
    start_fn = "test_repr_c_size",
);

check_custom_start!(
    #[ignore]
    test_enum_repr_c_discriminants,
    input = "enum/repr.rs",
    start_fn = "test_repr_c_discriminants",
);

check_custom_start!(
    #[ignore]
    test_enum_repr_u8_data_match,
    input = "enum/repr.rs",
    start_fn = "test_repr_u8_data_match",
);

check_custom_start!(
    #[ignore]
    test_enum_repr_c_data_match,
    input = "enum/repr.rs",
    start_fn = "test_repr_c_data_match",
);

check_custom_start!(
    #[ignore]
    test_enum_repr_u8_data_size,
    input = "enum/repr.rs",
    start_fn = "test_repr_u8_data_size",
);

check_custom_start!(
    #[ignore]
    test_enum_repr_c_data_size,
    input = "enum/repr.rs",
    start_fn = "test_repr_c_data_size",
);

// --- Niche optimization ---

check_custom_start!(
    #[ignore]
    test_enum_niche_option_nonzero_u32_size,
    input = "enum/niche.rs",
    start_fn = "test_option_nonzero_u32_size",
);

check_custom_start!(
    #[ignore]
    test_enum_niche_option_nonzero_u8_size,
    input = "enum/niche.rs",
    start_fn = "test_option_nonzero_u8_size",
);

check_custom_start!(
    #[ignore]
    test_enum_niche_option_nonzero_usize_size,
    input = "enum/niche.rs",
    start_fn = "test_option_nonzero_usize_size",
);

check_custom_start!(
    #[ignore]
    test_enum_niche_option_bool_size,
    input = "enum/niche.rs",
    start_fn = "test_option_bool_size",
);

check_custom_start!(
    #[ignore]
    test_enum_niche_option_ref_size,
    input = "enum/niche.rs",
    start_fn = "test_option_ref_size",
);

check_custom_start!(
    #[ignore]
    test_enum_niche_option_option_bool_size,
    input = "enum/niche.rs",
    start_fn = "test_option_option_bool_size",
);

check_custom_start!(
    #[ignore]
    test_enum_niche_option_nonzero_some,
    input = "enum/niche.rs",
    start_fn = "test_option_nonzero_some",
);

check_custom_start!(
    #[ignore]
    test_enum_niche_option_nonzero_none,
    input = "enum/niche.rs",
    start_fn = "test_option_nonzero_none",
);

check_custom_start!(
    #[ignore]
    test_enum_niche_option_bool_values,
    input = "enum/niche.rs",
    start_fn = "test_option_bool_values",
);

check_custom_start!(
    #[ignore]
    test_enum_niche_option_ref_some,
    input = "enum/niche.rs",
    start_fn = "test_option_ref_some",
);

check_custom_start!(
    #[ignore]
    test_enum_niche_option_ref_none,
    input = "enum/niche.rs",
    start_fn = "test_option_ref_none",
);

check_custom_start!(
    #[ignore]
    test_enum_niche_nested_option_nonzero,
    input = "enum/niche.rs",
    start_fn = "test_nested_option_nonzero",
);

// --- Single-variant enums ---

check_custom_start!(
    #[ignore]
    test_enum_single_wrapper_size,
    input = "enum/single_variant.rs",
    start_fn = "test_wrapper_single_size",
);

check_custom_start!(
    #[ignore]
    test_enum_single_unit_size,
    input = "enum/single_variant.rs",
    start_fn = "test_unit_single_size",
);

check_custom_start!(
    #[ignore]
    test_enum_single_struct_size,
    input = "enum/single_variant.rs",
    start_fn = "test_struct_single_size",
);

check_custom_start!(
    #[ignore]
    test_enum_single_wrapper_match,
    input = "enum/single_variant.rs",
    start_fn = "test_wrapper_single_match",
);

check_custom_start!(
    #[ignore]
    test_enum_single_struct_match,
    input = "enum/single_variant.rs",
    start_fn = "test_struct_single_match",
);

check_custom_start!(
    #[ignore]
    test_enum_single_unit_match,
    input = "enum/single_variant.rs",
    start_fn = "test_unit_single_match",
);
