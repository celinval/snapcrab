#![feature(portable_simd)]

/// Dependency crate for testing Rust ABI native calls.
/// Non-generic, non-inline functions are native-called via the loaded dylib.

use std::simd::u32x4;

// --- Scalar (Direct) ---

pub fn add_u32(a: u32, b: u32) -> u32 {
    a + b
}

pub fn add_u64(a: u64, b: u64) -> u64 {
    a + b
}

pub fn negate_i32(x: i32) -> i32 {
    -x
}

pub fn identity_bool(b: bool) -> bool {
    b
}

// --- ScalarPair (Pair) ---

pub fn sum_pair(a: u32, b: u32) -> (u32, u32) {
    (a + 1, b + 1)
}

pub fn swap_tuple(t: (u64, u64)) -> (u64, u64) {
    (t.1, t.0)
}

pub fn tuple_arg(t: (u64, u64)) -> u64 {
    t.0 + t.1
}

pub fn slice_sum(data: &[u32]) -> u32 {
    let mut sum = 0;
    let mut i = 0;
    while i < data.len() {
        sum += data[i];
        i += 1;
    }
    sum
}

pub fn str_len(s: &str) -> usize {
    s.len()
}

// --- Scalar (Direct) with many args ---

pub fn sum_three(a: u64, b: u64, c: u64) -> u64 {
    a + b + c
}

pub fn many_args(a: u32, b: u32, c: u32, d: u32, e: u32, f: u32, g: u32, h: u32) -> u32 {
    a + b + c + d + e + f + g + h
}

pub fn pass_slice_ptr(ptr: *const u32, len: usize) -> u32 {
    let mut sum = 0u32;
    for i in 0..len {
        sum += unsafe { *ptr.add(i) };
    }
    sum
}

// --- Struct (Direct for small, Indirect for large) ---

#[repr(C)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

pub fn make_point(x: i32, y: i32) -> Point {
    Point { x, y }
}

pub fn sum_point(p: Point) -> i32 {
    p.x + p.y
}

#[repr(C)]
pub struct LargeStruct {
    pub a: u64,
    pub b: u64,
    pub c: u64,
}

pub fn large_struct_sum(s: LargeStruct) -> u64 {
    s.a + s.b + s.c
}

pub fn make_large_struct(a: u64, b: u64, c: u64) -> LargeStruct {
    LargeStruct { a, b, c }
}

// --- Indirect (large arrays) ---

pub fn large_array_sum(arr: [u64; 4]) -> u64 {
    arr[0] + arr[1] + arr[2] + arr[3]
}

pub fn make_large_array(a: u64, b: u64, c: u64, d: u64) -> [u64; 4] {
    [a, b, c, d]
}

// --- ZST / Ignore ---

pub fn unit_arg(_: ()) -> u32 {
    99
}

pub fn unit_return(_x: u32) {}

// --- Small array (Cast in C ABI, may differ in Rust ABI) ---

pub fn small_array_sum(arr: [u32; 2]) -> u32 {
    arr[0] + arr[1]
}

pub fn make_small_array(a: u32, b: u32) -> [u32; 2] {
    [a, b]
}

// --- SIMD vectors (Direct with ValueAbi::Vector) ---

pub fn simd_sum(v: u32x4) -> u32 {
    v[0] + v[1] + v[2] + v[3]
}

pub fn simd_make(a: u32, b: u32, c: u32, d: u32) -> u32x4 {
    u32x4::from_array([a, b, c, d])
}

pub fn simd_add(a: u32x4, b: u32x4) -> u32x4 {
    a + b
}

// --- Statics ---

pub static MAGIC: u32 = 42;

pub static mut MUTABLE_COUNTER: u32 = 0;

pub fn read_magic() -> u32 {
    MAGIC
}

pub fn increment_counter() -> u32 {
    unsafe {
        MUTABLE_COUNTER += 1;
        MUTABLE_COUNTER
    }
}

/// Wrapper to make a raw pointer Sync for static placement.
pub struct WrapperPtr(pub *mut u32);
unsafe impl Sync for WrapperPtr {}

pub static STATIC_WITH_PTR: WrapperPtr = WrapperPtr(std::ptr::null_mut());

// --- Unsafe cases (should be rejected by check_call_safety) ---

/// Struct with padding between fields.
pub struct Padded {
    pub a: u8,
    pub b: u64,
}

/// Takes a mutable reference to a padded type.
pub fn write_padded(p: &mut Padded) {
    p.a = 1;
    p.b = 2;
}

/// Takes a raw mutable pointer to a padded type.
pub fn write_padded_raw(p: *mut Padded) {
    unsafe {
        (*p).a = 3;
        (*p).b = 4;
    }
}

/// Takes an immutable reference to a struct containing &mut to padded.
pub struct WrapsMutPadded<'a> {
    pub inner: &'a mut Padded,
}

pub fn read_wraps_mut(w: &WrapsMutPadded) -> u8 {
    w.inner.a
}

/// Returns a mutable pointer to a padded type.
pub fn get_mut_padded(p: &mut Padded) -> *mut Padded {
    p as *mut Padded
}
