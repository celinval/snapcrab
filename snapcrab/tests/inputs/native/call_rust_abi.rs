#![allow(unused)]
#![feature(portable_simd)]

use dep_rust_abi::{self, LargeStruct, Point};
use std::simd::u32x4;

// --- Scalar (Direct) ---

pub fn test_add_u32() {
    assert!(dep_rust_abi::add_u32(10, 32) == 42);
}

pub fn test_add_u64() {
    assert!(dep_rust_abi::add_u64(100, 200) == 300);
}

pub fn test_negate() {
    assert!(dep_rust_abi::negate_i32(42) == -42);
}

pub fn test_identity_bool() {
    assert!(dep_rust_abi::identity_bool(true));
    assert!(!dep_rust_abi::identity_bool(false));
}

// --- ScalarPair (Pair) ---

pub fn test_sum_pair() {
    let (a, b) = dep_rust_abi::sum_pair(10, 20);
    assert!(a == 11);
    assert!(b == 21);
}

pub fn test_swap_tuple() {
    let (a, b) = dep_rust_abi::swap_tuple((100, 200));
    assert!(a == 200);
    assert!(b == 100);
}

pub fn test_tuple_arg() {
    assert!(dep_rust_abi::tuple_arg((100, 200)) == 300);
}

pub fn test_slice_sum() {
    let data = [10u32, 20, 30, 40];
    assert!(dep_rust_abi::slice_sum(&data) == 100);
}

pub fn test_str_len() {
    assert!(dep_rust_abi::str_len("hello") == 5);
}

// --- Many args ---

pub fn test_sum_three() {
    assert!(dep_rust_abi::sum_three(10, 20, 30) == 60);
}

pub fn test_many_args() {
    assert!(dep_rust_abi::many_args(1, 2, 3, 4, 5, 6, 7, 8) == 36);
}

pub fn test_pass_slice_ptr() {
    let arr: [u32; 4] = [10, 20, 30, 40];
    assert!(dep_rust_abi::pass_slice_ptr(arr.as_ptr(), 4) == 100);
}

// --- Struct ---

pub fn test_make_point() {
    let p = dep_rust_abi::make_point(3, 7);
    assert!(p.x == 3);
    assert!(p.y == 7);
}

pub fn test_sum_point() {
    let p = Point { x: 15, y: 25 };
    assert!(dep_rust_abi::sum_point(p) == 40);
}

pub fn test_large_struct_sum() {
    let s = LargeStruct { a: 100, b: 200, c: 300 };
    assert!(dep_rust_abi::large_struct_sum(s) == 600);
}

pub fn test_make_large_struct() {
    let s = dep_rust_abi::make_large_struct(5, 10, 15);
    assert!(s.a == 5);
    assert!(s.b == 10);
    assert!(s.c == 15);
}

// --- Indirect (large arrays) ---

pub fn test_large_array_sum() {
    let arr: [u64; 4] = [100, 200, 300, 400];
    assert!(dep_rust_abi::large_array_sum(arr) == 1000);
}

pub fn test_make_large_array() {
    let arr = dep_rust_abi::make_large_array(10, 20, 30, 40);
    assert!(arr[0] == 10);
    assert!(arr[1] == 20);
    assert!(arr[2] == 30);
    assert!(arr[3] == 40);
}

// --- ZST / Ignore ---

pub fn test_unit_arg() {
    assert!(dep_rust_abi::unit_arg(()) == 99);
}

pub fn test_unit_return() {
    dep_rust_abi::unit_return(42);
}

// --- Small arrays ---

pub fn test_small_array_sum() {
    let arr: [u32; 2] = [30, 12];
    assert!(dep_rust_abi::small_array_sum(arr) == 42);
}

pub fn test_make_small_array() {
    let arr = dep_rust_abi::make_small_array(7, 8);
    assert!(arr[0] == 7);
    assert!(arr[1] == 8);
}

// --- SIMD vectors (Direct with ValueAbi::Vector) ---

pub fn test_simd_sum() {
    let v = u32x4::from_array([10, 20, 30, 40]);
    assert!(dep_rust_abi::simd_sum(v) == 100);
}

pub fn test_simd_make() {
    let v = dep_rust_abi::simd_make(1, 2, 3, 4);
    assert!(v[0] == 1);
    assert!(v[1] == 2);
    assert!(v[2] == 3);
    assert!(v[3] == 4);
}

pub fn test_simd_add() {
    let a = u32x4::from_array([1, 2, 3, 4]);
    let b = u32x4::from_array([10, 20, 30, 40]);
    let c = dep_rust_abi::simd_add(a, b);
    assert!(c[0] == 11);
    assert!(c[1] == 22);
    assert!(c[2] == 33);
    assert!(c[3] == 44);
}

// --- Unsafe cases (should be rejected) ---

pub fn test_write_padded() {
    let mut p = dep_rust_abi::Padded { a: 0, b: 0 };
    dep_rust_abi::write_padded(&mut p);
}

pub fn test_write_padded_raw() {
    let mut p = dep_rust_abi::Padded { a: 0, b: 0 };
    dep_rust_abi::write_padded_raw(&mut p as *mut dep_rust_abi::Padded);
}

pub fn test_read_wraps_mut() {
    let mut p = dep_rust_abi::Padded { a: 0, b: 0 };
    let w = dep_rust_abi::WrapsMutPadded { inner: &mut p };
    dep_rust_abi::read_wraps_mut(&w);
}

pub fn test_return_mut_padded() {
    let mut p = dep_rust_abi::Padded { a: 0, b: 0 };
    let _ = dep_rust_abi::get_mut_padded(&mut p);
}
