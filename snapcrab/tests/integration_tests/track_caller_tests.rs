use crate::common::TestResult;

// `#[track_caller]` reports a panic at the user's call site. The column is not
// pinned: rustc uses the call's `fn_span` while `Body::caller_location` uses
// `source_info.span`, which differ for a method call.

check_custom_start!(
    test_track_caller_unwrap_none,
    input = "track_caller.rs",
    start_fn = "unwrap_none",
    result = TestResult::ErrorRegex(
        r"(?s).*track_caller\.rs:21:.*called `Option::unwrap\(\)` on a `None` value.*".to_string()
    )
);

check_custom_start!(
    test_track_caller_expect_none,
    input = "track_caller.rs",
    start_fn = "expect_none",
    result =
        TestResult::ErrorRegex(r"(?s).*track_caller\.rs:27:.*value must be present.*".to_string())
);

check_custom_start!(
    test_track_caller_propagates,
    input = "track_caller.rs",
    start_fn = "propagates_through_track_caller",
    result =
        TestResult::ErrorRegex(r"(?s).*track_caller\.rs:38:5.*requirement failed.*".to_string())
);

check_custom_start!(
    test_track_caller_explicit_panic,
    input = "track_caller.rs",
    start_fn = "explicit_panic",
    result = TestResult::ErrorRegex(r"(?s).*track_caller\.rs:43:5.*boom.*".to_string())
);

check_custom_start!(
    test_track_caller_index_out_of_bounds,
    input = "track_caller.rs",
    start_fn = "index_out_of_bounds",
    result =
        TestResult::ErrorRegex(r"(?s).*track_caller\.rs:53:.*index out of bounds.*".to_string())
);

check_custom_start!(
    test_track_caller_location_caller_line,
    input = "track_caller.rs",
    start_fn = "location_caller_line",
    result = TestResult::SuccessWithValue(vec![1])
);

check_custom_start!(
    test_track_caller_location_propagates_two_levels,
    input = "track_caller.rs",
    start_fn = "location_propagates_two_levels",
    result = TestResult::SuccessWithValue(vec![1])
);

check_custom_start!(
    test_track_caller_result_unwrap_err,
    input = "track_caller.rs",
    start_fn = "result_unwrap_err",
    result = TestResult::ErrorRegex(
        r"(?s).*track_caller\.rs:90:.*called `Result::unwrap\(\)` on an `Err` value.*".to_string()
    )
);

// A formatted panic keeps its location but not its message.
check_custom_start!(
    test_track_caller_assert_eq_fails,
    input = "track_caller.rs",
    start_fn = "assert_eq_fails",
    result = TestResult::ErrorRegex(r"(?s).*track_caller\.rs:98:5.*assertion failed.*".to_string())
);

check_custom_start!(
    test_track_caller_result_expect_message,
    input = "track_caller.rs",
    start_fn = "result_expect_message",
    result =
        TestResult::ErrorRegex(r"(?s).*track_caller\.rs:105:.*custom expect message.*".to_string())
);

// A panic entry point the interpreter does not recognize must still fail
// cleanly, rather than entering the native panic runtime and aborting.

check_custom_start!(
    test_track_caller_slice_range_out_of_bounds,
    input = "track_caller.rs",
    start_fn = "slice_range_out_of_bounds",
    result = TestResult::ErrorRegex(
        r"(?s).*unsupported native call: `core::slice::index::slice_index_fail` never returns.*"
            .to_string()
    )
);

check_custom_start!(
    test_track_caller_panic_any_payload,
    input = "track_caller.rs",
    start_fn = "panic_any_payload",
    result =
        TestResult::ErrorRegex(r"(?s).*unsupported native call: .*never returns.*".to_string())
);
