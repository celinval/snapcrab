//! Heap allocation through `Box`, exercising the `__rust_alloc` hook and the
//! heap memory segment.

use crate::common::TestResult;

// --- Valid allocations ---

check_custom_start!(
    test_heap_box_new_deref,
    input = "heap.rs",
    start_fn = "box_new_deref",
);

check_custom_start!(
    test_heap_box_mutate,
    input = "heap.rs",
    start_fn = "box_mutate",
);

check_custom_start!(
    test_heap_box_struct,
    input = "heap.rs",
    start_fn = "box_struct",
);

check_custom_start!(
    test_heap_nested_box,
    input = "heap.rs",
    start_fn = "nested_box",
);

check_custom_start!(
    test_heap_box_moved_into_fn,
    input = "heap.rs",
    start_fn = "box_moved_into_fn",
);

check_custom_start!(
    test_heap_box_reassign,
    input = "heap.rs",
    start_fn = "box_reassign",
);

// --- Allocator misuse (should be reported, not silently accepted) ---

check_custom_start!(
    test_heap_realloc_old_size_too_large,
    input = "heap_invalid.rs",
    start_fn = "realloc_old_size_too_large",
    result = TestResult::ErrorRegex(r".*invalid realloc.*".to_string()),
);

check_custom_start!(
    test_heap_realloc_align_mismatch,
    input = "heap_invalid.rs",
    start_fn = "realloc_align_mismatch",
    result = TestResult::ErrorRegex(r".*invalid realloc.*".to_string()),
);

check_custom_start!(
    test_heap_alloc_zero_sized,
    input = "heap_invalid.rs",
    start_fn = "alloc_zero_sized",
    result = TestResult::ErrorRegex(r".*invalid allocation.*zero.*".to_string()),
);
