use crate::common::TestResult;

check_custom_start!(
    test_rc_to_dyn,
    input = "smart_ptr.rs",
    start_fn = "rc_to_dyn",
);

check_custom_start!(
    test_rc_to_slice,
    input = "smart_ptr.rs",
    start_fn = "rc_to_slice",
);

check_custom_start!(
    test_rc_dyn_clone_and_drop,
    input = "smart_ptr.rs",
    start_fn = "rc_dyn_clone_and_drop",
);

check_custom_start!(
    test_pin_to_dyn,
    input = "smart_ptr.rs",
    start_fn = "pin_to_dyn",
);

// `Arc` coerces like `Rc`, but its atomic refcount needs the atomic
// intrinsics, so these stop there rather than completing.

check_custom_start!(
    test_arc_to_dyn,
    input = "smart_ptr.rs",
    start_fn = "arc_to_dyn",
    result = TestResult::ErrorRegex(r".*Unimplemented intrinsic `atomic_[a-z]+`.*".to_string()),
);

check_custom_start!(
    test_arc_to_slice,
    input = "smart_ptr.rs",
    start_fn = "arc_to_slice",
    result = TestResult::ErrorRegex(r".*Unimplemented intrinsic `atomic_[a-z]+`.*".to_string()),
);

check_custom_start!(
    test_rc_to_unsized_tail,
    input = "smart_ptr.rs",
    start_fn = "rc_to_unsized_tail",
);

// A user-defined `CoerceUnsized` impl, which unlike `Rc`/`Arc`/`Pin` carries
// sized fields alongside the pointer being unsized.

check_custom_start!(
    test_user_ptr_to_dyn,
    input = "user_smart_ptr.rs",
    start_fn = "user_ptr_to_dyn",
);

check_custom_start!(
    test_user_ptr_to_slice,
    input = "user_smart_ptr.rs",
    start_fn = "user_ptr_to_slice",
);
