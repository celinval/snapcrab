//! Closures exercised through different capture modes and call styles.

// --- Direct calls ---

check_custom_start!(
    test_closure_direct_no_capture,
    input = "closures.rs",
    start_fn = "direct_no_capture",
);

check_custom_start!(
    test_closure_direct_capture_ref,
    input = "closures.rs",
    start_fn = "direct_capture_ref",
);

check_custom_start!(
    test_closure_direct_capture_mut,
    input = "closures.rs",
    start_fn = "direct_capture_mut",
);

check_custom_start!(
    test_closure_direct_capture_move,
    input = "closures.rs",
    start_fn = "direct_capture_move",
);

// --- Calls through generic bounds (routes via a call shim) ---
//
// Owning a closure inside the callee makes it responsible for dropping it,
// which emits a `Drop` terminator handled by the drop glue support.

check_custom_start!(
    test_closure_generic_fn,
    input = "closures.rs",
    start_fn = "generic_fn",
);

check_custom_start!(
    test_closure_generic_fn_capture,
    input = "closures.rs",
    start_fn = "generic_fn_capture",
);

check_custom_start!(
    test_closure_generic_fn_mut,
    input = "closures.rs",
    start_fn = "generic_fn_mut",
);

check_custom_start!(
    test_closure_generic_fn_once,
    input = "closures.rs",
    start_fn = "generic_fn_once",
);

// --- Closures as function pointers and trait objects ---

check_custom_start!(
    // TODO: Requires the `ClosureFnPointer` cast coercion.
    #[ignore]
    test_closure_as_fn_pointer,
    input = "closures.rs",
    start_fn = "as_fn_pointer",
);

check_custom_start!(
    // TODO: Requires closure -> `dyn Fn` unsizing coercion.
    #[ignore]
    test_closure_as_dyn_fn,
    input = "closures.rs",
    start_fn = "as_dyn_fn",
);

// --- Composition ---

check_custom_start!(
    test_closure_nested,
    input = "closures.rs",
    start_fn = "nested_closures",
);

check_custom_start!(
    test_closure_returns_value,
    input = "closures.rs",
    start_fn = "returns_value",
);
