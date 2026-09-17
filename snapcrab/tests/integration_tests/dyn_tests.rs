//! Virtual dispatch and drop through trait-object vtables.

use crate::common::TestResult;

check_custom_start!(
    test_dyn_call_via_ref,
    input = "dyn_dispatch.rs",
    start_fn = "call_via_ref",
);

check_custom_start!(
    test_dyn_provided_method,
    input = "dyn_dispatch.rs",
    start_fn = "provided_method",
);

check_custom_start!(
    test_dyn_call_via_arg,
    input = "dyn_dispatch.rs",
    start_fn = "call_via_arg",
);

check_custom_start!(
    test_dyn_call_via_mut,
    input = "dyn_dispatch.rs",
    start_fn = "call_via_mut",
);

check_custom_start!(
    test_dyn_call_via_box,
    input = "dyn_dispatch.rs",
    start_fn = "call_via_box",
);

check_custom_start!(
    test_dyn_upcast_then_call,
    input = "dyn_dispatch.rs",
    start_fn = "upcast_then_call",
);

check_custom_start!(
    test_dyn_box_drop,
    input = "dyn_dispatch.rs",
    start_fn = "box_dyn_drop",
);

// --- Receiver types beyond `&self`/`&mut self` ---

check_custom_start!(
    test_dyn_recv_box_self,
    input = "dyn_dispatch.rs",
    start_fn = "recv_box_self",
);

// Coercing a smart pointer to a trait object (`Rc<T>`/`Arc<T>`/`Pin<&mut T>`
// -> `dyn`) goes through the pointer's own `CoerceUnsized` impl, which is not
// yet supported. These document the current limitation via a clean error.

check_custom_start!(
    test_dyn_recv_rc_self,
    input = "dyn_dispatch.rs",
    start_fn = "recv_rc_self",
    result =
        TestResult::ErrorRegex(r".*unsized coercion of .*Rc.* is not yet supported.*".to_string()),
);

check_custom_start!(
    test_dyn_recv_arc_self,
    input = "dyn_dispatch.rs",
    start_fn = "recv_arc_self",
    result =
        TestResult::ErrorRegex(r".*unsized coercion of .*Arc.* is not yet supported.*".to_string()),
);

check_custom_start!(
    test_dyn_recv_pin_self,
    input = "dyn_dispatch.rs",
    start_fn = "recv_pin_self",
    result =
        TestResult::ErrorRegex(r".*unsized coercion of .*Pin.* is not yet supported.*".to_string()),
);

check_custom_start!(
    test_dyn_drop_adt_dyn_tail,
    input = "dyn_dispatch.rs",
    start_fn = "drop_adt_dyn_tail",
);

check_custom_start!(
    test_dyn_upcast_secondary_dispatch,
    input = "dyn_dispatch.rs",
    start_fn = "upcast_secondary_dispatch",
);
