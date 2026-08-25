//! Drop glue exercised across the places a `Drop` terminator can appear.
//!
//! Each input asserts the observed drop count internally and panics on
//! mismatch, so a passing interpreter returns `()`.

use crate::common::TestResult;

// --- Automatic drops ---

check_custom_start!(
    test_drop_scope_end,
    input = "drop_semantics.rs",
    start_fn = "scope_end_drop",
);

check_custom_start!(
    test_drop_reverse_declaration_order,
    input = "drop_semantics.rs",
    start_fn = "reverse_declaration_order",
);

check_custom_start!(
    test_drop_nested_scope,
    input = "drop_semantics.rs",
    start_fn = "nested_scope_drop",
);

check_custom_start!(
    test_drop_in_loop,
    input = "drop_semantics.rs",
    start_fn = "drop_in_loop",
);

check_custom_start!(
    test_drop_conditional,
    input = "drop_semantics.rs",
    start_fn = "conditional_drop",
);

check_custom_start!(
    test_drop_on_reassign,
    input = "drop_semantics.rs",
    start_fn = "drop_on_reassign",
);

check_custom_start!(
    test_drop_aggregate_fields,
    input = "drop_semantics.rs",
    start_fn = "aggregate_field_drop",
);

check_custom_start!(
    // TODO: Requires heap allocation support (`Box::new` is unsupported).
    #[ignore]
    test_drop_box,
    input = "drop_semantics.rs",
    start_fn = "box_drop",
);

// --- Moves suppress the source drop ---

check_custom_start!(
    test_drop_move_suppresses_source,
    input = "drop_semantics.rs",
    start_fn = "move_suppresses_source_drop",
);

// --- Explicit early drop ---

check_custom_start!(
    test_drop_explicit,
    input = "drop_semantics.rs",
    start_fn = "explicit_drop",
);

check_custom_start!(
    test_drop_needs_drop_by_type,
    input = "drop_semantics.rs",
    start_fn = "needs_drop_by_type",
);

// --- Manual drops ---

check_custom_start!(
    test_drop_maybe_uninit_manual,
    input = "drop_semantics.rs",
    start_fn = "maybe_uninit_manual_drop",
);

check_custom_start!(
    test_drop_maybe_uninit_in_place,
    input = "drop_semantics.rs",
    start_fn = "maybe_uninit_drop_in_place",
);

check_custom_start!(
    test_drop_maybe_uninit_never,
    input = "drop_semantics.rs",
    start_fn = "maybe_uninit_never_dropped",
);

check_custom_start!(
    test_drop_manually_drop_manual,
    input = "drop_semantics.rs",
    start_fn = "manually_drop_manual",
);

check_custom_start!(
    test_drop_manually_drop_leaks,
    input = "drop_semantics.rs",
    start_fn = "manually_drop_leaks",
);

check_custom_start!(
    test_drop_assume_init_uninhabited,
    input = "drop_semantics.rs",
    start_fn = "assume_init_uninhabited",
    result = TestResult::ErrorRegex(r".*attempted to instantiate uninhabited type.*".to_string())
);
