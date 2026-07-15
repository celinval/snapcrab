// Native call tests — compile a cdylib, load it, interpret functions that call into it.

// --- Integer operations ---

check_native_call!(
    test_native_add_u32,
    native_lib = "native/lib_basic.rs",
    input = "native/call_basic.rs",
    start_fn = "test_add_u32",
);

check_native_call!(
    test_native_add_u64,
    native_lib = "native/lib_basic.rs",
    input = "native/call_basic.rs",
    start_fn = "test_add_u64",
);

check_native_call!(
    test_native_add_i8,
    native_lib = "native/lib_basic.rs",
    input = "native/call_basic.rs",
    start_fn = "test_add_i8",
);

check_native_call!(
    test_native_negate,
    native_lib = "native/lib_basic.rs",
    input = "native/call_basic.rs",
    start_fn = "test_negate",
);

// --- No args / no return ---

check_native_call!(
    test_native_no_args,
    native_lib = "native/lib_basic.rs",
    input = "native/call_basic.rs",
    start_fn = "test_no_args",
);

check_native_call!(
    #[ignore] // void return from native call triggers spurious memory read
    test_native_no_return,
    native_lib = "native/lib_basic.rs",
    input = "native/call_basic.rs",
    start_fn = "test_no_return",
);

// --- Boolean ---

check_native_call!(
    test_native_bool_true,
    native_lib = "native/lib_basic.rs",
    input = "native/call_basic.rs",
    start_fn = "test_bool_true",
);

check_native_call!(
    test_native_bool_false,
    native_lib = "native/lib_basic.rs",
    input = "native/call_basic.rs",
    start_fn = "test_bool_false",
);

// --- Floating point ---

check_native_call!(
    #[ignore] // float Eq not yet supported in interpreter
    test_native_add_f64,
    native_lib = "native/lib_basic.rs",
    input = "native/call_basic.rs",
    start_fn = "test_add_f64",
);

check_native_call!(
    #[ignore] // float Eq not yet supported in interpreter
    test_native_add_f32,
    native_lib = "native/lib_basic.rs",
    input = "native/call_basic.rs",
    start_fn = "test_add_f32",
);

// --- Many args / mixed types ---

check_native_call!(
    test_native_many_args,
    native_lib = "native/lib_basic.rs",
    input = "native/call_basic.rs",
    start_fn = "test_many_args",
);

check_native_call!(
    test_native_sum_mixed,
    native_lib = "native/lib_basic.rs",
    input = "native/call_basic.rs",
    start_fn = "test_sum_mixed",
);

// --- Pointer passing ---

check_native_call!(
    test_native_pass_ptr,
    native_lib = "native/lib_basic.rs",
    input = "native/call_basic.rs",
    start_fn = "test_pass_ptr",
);

check_native_call!(
    test_native_write_ptr,
    native_lib = "native/lib_basic.rs",
    input = "native/call_basic.rs",
    start_fn = "test_write_ptr",
);

// --- Array passing (via pointer + len) ---

check_native_call!(
    test_native_sum_array_3,
    native_lib = "native/lib_basic.rs",
    input = "native/call_basic.rs",
    start_fn = "test_sum_array_3",
);

check_native_call!(
    test_native_sum_array_u8,
    native_lib = "native/lib_basic.rs",
    input = "native/call_basic.rs",
    start_fn = "test_sum_array_u8",
);

check_native_call!(
    test_native_sum_array_u64,
    native_lib = "native/lib_basic.rs",
    input = "native/call_basic.rs",
    start_fn = "test_sum_array_u64",
);

check_native_call!(
    test_native_fill_array,
    native_lib = "native/lib_basic.rs",
    input = "native/call_basic.rs",
    start_fn = "test_fill_array",
);

// --- By-value array pass/return (PassMode::Cast / Indirect) ---

check_native_call!(
    #[ignore] // PassMode::Cast not yet supported
    test_native_sum_val_array_small,
    native_lib = "native/lib_basic.rs",
    input = "native/call_basic.rs",
    start_fn = "test_sum_val_array_small",
);

check_native_call!(
    #[ignore] // PassMode::Cast not yet supported
    test_native_make_array_small,
    native_lib = "native/lib_basic.rs",
    input = "native/call_basic.rs",
    start_fn = "test_make_array_small",
);

check_native_call!(
    test_native_make_array_large,
    native_lib = "native/lib_basic.rs",
    input = "native/call_basic.rs",
    start_fn = "test_make_array_large",
);

// --- Small struct pass/return (PassMode::Cast — not yet supported) ---

check_native_call!(
    #[ignore] // PassMode::Cast return not yet supported
    test_native_make_pair,
    native_lib = "native/lib_structs.rs",
    input = "native/call_structs.rs",
    start_fn = "test_make_pair",
);

check_native_call!(
    #[ignore] // PassMode::Cast arg not yet supported
    test_native_sum_pair,
    native_lib = "native/lib_structs.rs",
    input = "native/call_structs.rs",
    start_fn = "test_sum_pair",
);

check_native_call!(
    #[ignore] // PassMode::Cast arg + return (mixed field sizes)
    test_native_swap_pair,
    native_lib = "native/lib_structs.rs",
    input = "native/call_structs.rs",
    start_fn = "test_swap_pair",
);

// --- Large struct (indirect return, Cast arg) ---

check_native_call!(
    test_native_make_triple,
    native_lib = "native/lib_structs.rs",
    input = "native/call_structs.rs",
    start_fn = "test_make_triple",
);

check_native_call!(
    #[ignore] // PassMode::Cast for struct arg not yet supported
    test_native_sum_triple,
    native_lib = "native/lib_structs.rs",
    input = "native/call_structs.rs",
    start_fn = "test_sum_triple",
);

// --- Pointer-containing struct ---

check_native_call!(
    #[ignore] // PassMode::Cast for struct arg not yet supported
    test_native_read_with_ptr,
    native_lib = "native/lib_structs.rs",
    input = "native/call_structs.rs",
    start_fn = "test_read_with_ptr",
);
