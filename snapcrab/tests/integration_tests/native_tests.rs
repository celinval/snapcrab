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

// --- Extern crate (dylib+rlib, real Rust mangled symbols) ---

check_extern_crate!(
    test_extern_crate_native_call,
    dep = "native/dep_math.rs",
    input = "native/call_dep_math.rs",
    start_fn = "test_add",
);

check_extern_crate!(
    test_extern_crate_interpreted_generic,
    dep = "native/dep_math.rs",
    input = "native/call_dep_math.rs",
    start_fn = "test_generic_add",
);

check_extern_crate!(
    test_extern_crate_repeated_calls,
    dep = "native/dep_math.rs",
    input = "native/call_dep_math.rs",
    start_fn = "test_repeated_calls",
);

check_extern_crate!(
    test_extern_crate_shared_trampoline,
    dep = "native/dep_math.rs",
    input = "native/call_dep_math.rs",
    start_fn = "test_shared_trampoline",
);

check_extern_crate!(
    test_extern_crate_different_abis,
    dep = "native/dep_math.rs",
    input = "native/call_dep_math.rs",
    start_fn = "test_different_abis",
);

// --- Rust ABI: Scalar (Direct) ---

check_extern_crate!(
    test_rust_abi_add_u32,
    dep = "native/dep_rust_abi.rs",
    input = "native/call_rust_abi.rs",
    start_fn = "test_add_u32",
);

check_extern_crate!(
    test_rust_abi_add_u64,
    dep = "native/dep_rust_abi.rs",
    input = "native/call_rust_abi.rs",
    start_fn = "test_add_u64",
);

check_extern_crate!(
    test_rust_abi_negate,
    dep = "native/dep_rust_abi.rs",
    input = "native/call_rust_abi.rs",
    start_fn = "test_negate",
);

check_extern_crate!(
    test_rust_abi_identity_bool,
    dep = "native/dep_rust_abi.rs",
    input = "native/call_rust_abi.rs",
    start_fn = "test_identity_bool",
);

// --- Rust ABI: ScalarPair (Pair) ---

check_extern_crate!(
    test_rust_abi_sum_pair,
    dep = "native/dep_rust_abi.rs",
    input = "native/call_rust_abi.rs",
    start_fn = "test_sum_pair",
);

check_extern_crate!(
    test_rust_abi_swap_tuple,
    dep = "native/dep_rust_abi.rs",
    input = "native/call_rust_abi.rs",
    start_fn = "test_swap_tuple",
);

check_extern_crate!(
    test_rust_abi_tuple_arg,
    dep = "native/dep_rust_abi.rs",
    input = "native/call_rust_abi.rs",
    start_fn = "test_tuple_arg",
);

check_extern_crate!(
    test_rust_abi_slice_sum,
    dep = "native/dep_rust_abi.rs",
    input = "native/call_rust_abi.rs",
    start_fn = "test_slice_sum",
);

check_extern_crate!(
    test_rust_abi_str_len,
    dep = "native/dep_rust_abi.rs",
    input = "native/call_rust_abi.rs",
    start_fn = "test_str_len",
);

// --- Rust ABI: Many args ---

check_extern_crate!(
    test_rust_abi_sum_three,
    dep = "native/dep_rust_abi.rs",
    input = "native/call_rust_abi.rs",
    start_fn = "test_sum_three",
);

check_extern_crate!(
    test_rust_abi_many_args,
    dep = "native/dep_rust_abi.rs",
    input = "native/call_rust_abi.rs",
    start_fn = "test_many_args",
);

check_extern_crate!(
    test_rust_abi_pass_slice_ptr,
    dep = "native/dep_rust_abi.rs",
    input = "native/call_rust_abi.rs",
    start_fn = "test_pass_slice_ptr",
);

// --- Rust ABI: Struct (Direct for small, Indirect for large) ---

check_extern_crate!(
    test_rust_abi_make_point,
    dep = "native/dep_rust_abi.rs",
    input = "native/call_rust_abi.rs",
    start_fn = "test_make_point",
);

check_extern_crate!(
    test_rust_abi_sum_point,
    dep = "native/dep_rust_abi.rs",
    input = "native/call_rust_abi.rs",
    start_fn = "test_sum_point",
);

check_extern_crate!(
    test_rust_abi_large_struct_sum,
    dep = "native/dep_rust_abi.rs",
    input = "native/call_rust_abi.rs",
    start_fn = "test_large_struct_sum",
);

check_extern_crate!(
    test_rust_abi_make_large_struct,
    dep = "native/dep_rust_abi.rs",
    input = "native/call_rust_abi.rs",
    start_fn = "test_make_large_struct",
);

// --- Rust ABI: Indirect (large arrays) ---

check_extern_crate!(
    test_rust_abi_large_array_sum,
    dep = "native/dep_rust_abi.rs",
    input = "native/call_rust_abi.rs",
    start_fn = "test_large_array_sum",
);

check_extern_crate!(
    test_rust_abi_make_large_array,
    dep = "native/dep_rust_abi.rs",
    input = "native/call_rust_abi.rs",
    start_fn = "test_make_large_array",
);

// --- Rust ABI: ZST / Ignore ---

check_extern_crate!(
    test_rust_abi_unit_arg,
    dep = "native/dep_rust_abi.rs",
    input = "native/call_rust_abi.rs",
    start_fn = "test_unit_arg",
);

check_extern_crate!(
    test_rust_abi_unit_return,
    dep = "native/dep_rust_abi.rs",
    input = "native/call_rust_abi.rs",
    start_fn = "test_unit_return",
);

// --- Rust ABI: Small arrays (Cast even in Rust ABI) ---

check_extern_crate!(
    #[ignore] // PassMode::Cast — small arrays use Cast even in Rust ABI
    test_rust_abi_small_array_sum,
    dep = "native/dep_rust_abi.rs",
    input = "native/call_rust_abi.rs",
    start_fn = "test_small_array_sum",
);

check_extern_crate!(
    #[ignore] // PassMode::Cast — small arrays use Cast even in Rust ABI
    test_rust_abi_make_small_array,
    dep = "native/dep_rust_abi.rs",
    input = "native/call_rust_abi.rs",
    start_fn = "test_make_small_array",
);

// --- Rust ABI: SIMD vectors (Direct with ValueAbi::Vector) ---

check_extern_crate!(
    #[ignore] // ctpop intrinsic not yet shimmed (interpreter limitation)
    test_rust_abi_simd_sum,
    dep = "native/dep_rust_abi.rs",
    input = "native/call_rust_abi.rs",
    start_fn = "test_simd_sum",
);

check_extern_crate!(
    test_rust_abi_simd_make,
    dep = "native/dep_rust_abi.rs",
    input = "native/call_rust_abi.rs",
    start_fn = "test_simd_make",
);

check_extern_crate!(
    #[ignore] // ctpop intrinsic not yet shimmed (interpreter limitation)
    test_rust_abi_simd_add,
    dep = "native/dep_rust_abi.rs",
    input = "native/call_rust_abi.rs",
    start_fn = "test_simd_add",
);

// --- Safety check: calls that should be rejected ---

use crate::common::TestResult;

check_extern_crate!(
    test_reject_mut_ref_to_padded,
    dep = "native/dep_rust_abi.rs",
    input = "native/call_rust_abi.rs",
    start_fn = "test_write_padded",
    result = TestResult::ErrorRegex(r".*mutable pointer.*padding.*".to_string()),
);

check_extern_crate!(
    test_reject_raw_mut_ptr_to_padded,
    dep = "native/dep_rust_abi.rs",
    input = "native/call_rust_abi.rs",
    start_fn = "test_write_padded_raw",
    result = TestResult::ErrorRegex(r".*mutable pointer.*padding.*".to_string()),
);

check_extern_crate!(
    test_reject_nested_mut_ref_to_padded,
    dep = "native/dep_rust_abi.rs",
    input = "native/call_rust_abi.rs",
    start_fn = "test_read_wraps_mut",
    result = TestResult::ErrorRegex(r".*mutable pointer.*padding.*".to_string()),
);

check_extern_crate!(
    test_reject_return_mut_ptr_to_padded,
    dep = "native/dep_rust_abi.rs",
    input = "native/call_rust_abi.rs",
    start_fn = "test_return_mut_padded",
    result = TestResult::ErrorRegex(r".*mutable pointer.*padding.*".to_string()),
);

// --- Statics ---

check_extern_crate!(
    test_local_static,
    dep = "native/dep_rust_abi.rs",
    input = "native/call_rust_abi.rs",
    start_fn = "test_local_static",
);

check_extern_crate!(
    test_external_static,
    dep = "native/dep_rust_abi.rs",
    input = "native/call_rust_abi.rs",
    start_fn = "test_external_static",
);

check_extern_crate!(
    test_external_mutable_static,
    dep = "native/dep_rust_abi.rs",
    input = "native/call_rust_abi.rs",
    start_fn = "test_external_mutable_static",
    result = TestResult::ErrorRegex(r".*unsupported.*duplicated in native code.*".to_string()),
);

check_extern_crate!(
    test_static_with_mut_ptr,
    dep = "native/dep_rust_abi.rs",
    input = "native/call_rust_abi.rs",
    start_fn = "test_static_with_mut_ptr",
    result = TestResult::ErrorRegex(r".*unsupported.*duplicated in native code.*".to_string()),
);
