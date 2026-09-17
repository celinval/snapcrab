//! Function-pointer reification, indirect calls, storage, and identity.

check_custom_start!(
    test_fn_ptr_reify_fn_item,
    input = "fn_pointer.rs",
    start_fn = "reify_fn_item",
);

check_custom_start!(
    test_fn_ptr_as_arg,
    input = "fn_pointer.rs",
    start_fn = "fn_ptr_as_arg",
);

check_custom_start!(
    test_fn_ptr_closure_to_fn_ptr,
    input = "fn_pointer.rs",
    start_fn = "closure_to_fn_ptr",
);

check_custom_start!(
    test_fn_ptr_identity,
    input = "fn_pointer.rs",
    start_fn = "fn_ptr_identity",
);

check_custom_start!(
    test_fn_ptr_in_struct,
    input = "fn_pointer.rs",
    start_fn = "fn_ptr_in_struct",
);
