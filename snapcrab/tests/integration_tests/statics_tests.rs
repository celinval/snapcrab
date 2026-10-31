use crate::common::TestResult;

check_custom_start!(
    test_write_static_mut,
    input = "write_statics.rs",
    start_fn = "write_static_mut",
);

check_custom_start!(
    test_write_interior_unsafecell,
    input = "write_statics.rs",
    start_fn = "write_interior_unsafecell",
);

check_custom_start!(
    test_write_interior_mutable,
    input = "write_statics.rs",
    start_fn = "write_interior_mutable",
);

check_custom_start!(
    test_write_read_only_static,
    input = "write_statics.rs",
    start_fn = "write_read_only_static",
    result =
        TestResult::ErrorRegex(r"(?s).*Cannot write to read-only memory at address.*".to_string())
);
