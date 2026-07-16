#![allow(unused, improper_ctypes)]

unsafe extern "C" {
    fn add_u32(a: u32, b: u32) -> u32;
    fn add_u64(a: u64, b: u64) -> u64;
    fn add_i8(a: i8, b: i8) -> i8;
    fn negate_i32(x: i32) -> i32;
    fn no_args() -> u32;
    fn no_return(x: u32);
    fn identity_bool(b: bool) -> bool;
    fn add_f64(a: f64, b: f64) -> f64;
    fn add_f32(a: f32, b: f32) -> f32;
    fn many_args(a: u32, b: u32, c: u32, d: u32, e: u32, f: u32) -> u32;
    fn sum_mixed(a: u8, b: u16, c: u32, d: u64) -> u64;
    fn pass_ptr(ptr: *const u32) -> u32;
    fn write_ptr(ptr: *mut u32, val: u32);
    fn sum_array_3(arr: *const u32, len: usize) -> u32;
    fn sum_array_u8(arr: *const u8, len: usize) -> u32;
    fn sum_array_u64(arr: *const u64, len: usize) -> u64;
    fn fill_array(arr: *mut u32, len: usize, val: u32);
    fn sum_val_array_small(arr: [u32; 3]) -> u32;
    fn sum_val_array_u8(arr: [u8; 4]) -> u32;
    fn sum_val_array_large(arr: [u64; 4]) -> u64;
    fn make_array_small() -> [u32; 3];
    fn make_array_large() -> [u64; 4];
}

pub fn test_add_u32() {
    assert!(unsafe { add_u32(10, 32) } == 42);
}

pub fn test_add_u64() {
    assert!(unsafe { add_u64(100, 200) } == 300);
}

pub fn test_add_i8() {
    assert!(unsafe { add_i8(50, 70) } == 120);
}

pub fn test_negate() {
    assert!(unsafe { negate_i32(42) } == -42);
}

pub fn test_no_args() {
    assert!(unsafe { no_args() } == 42);
}

pub fn test_no_return() {
    unsafe { no_return(99) }
}

pub fn test_bool_true() {
    assert!(unsafe { identity_bool(true) });
}

pub fn test_bool_false() {
    assert!(!unsafe { identity_bool(false) });
}

pub fn test_add_f64() {
    let result = unsafe { add_f64(1.5, 2.5) };
    assert!(result == 4.0);
}

pub fn test_add_f32() {
    let result = unsafe { add_f32(1.0, 2.0) };
    assert!(result == 3.0);
}

pub fn test_many_args() {
    assert!(unsafe { many_args(1, 2, 3, 4, 5, 6) } == 21);
}

pub fn test_sum_mixed() {
    assert!(unsafe { sum_mixed(1, 2, 3, 4) } == 10);
}

pub fn test_pass_ptr() {
    let val: u32 = 123;
    assert!(unsafe { pass_ptr(&val as *const u32) } == 123);
}

pub fn test_write_ptr() {
    let mut val: u32 = 0;
    unsafe { write_ptr(&mut val as *mut u32, 456) };
    assert!(val == 456);
}

pub fn test_sum_array_3() {
    let arr: [u32; 3] = [10, 20, 30];
    let result = unsafe { sum_array_3(arr.as_ptr(), 3) };
    assert!(result == 60);
}

pub fn test_sum_array_u8() {
    let arr: [u8; 5] = [1, 2, 3, 4, 5];
    let result = unsafe { sum_array_u8(arr.as_ptr(), 5) };
    assert!(result == 15);
}

pub fn test_sum_array_u64() {
    let arr: [u64; 4] = [100, 200, 300, 400];
    let result = unsafe { sum_array_u64(arr.as_ptr(), 4) };
    assert!(result == 1000);
}

pub fn test_fill_array() {
    let mut arr: [u32; 4] = [0; 4];
    unsafe { fill_array(arr.as_mut_ptr(), 4, 42) };
    assert!(arr[0] == 42);
    assert!(arr[1] == 42);
    assert!(arr[2] == 42);
    assert!(arr[3] == 42);
}

pub fn test_sum_val_array_small() {
    let arr: [u32; 3] = [10, 20, 30];
    assert!(unsafe { sum_val_array_small(arr) } == 60);
}

pub fn test_sum_val_array_u8() {
    let arr: [u8; 4] = [1, 2, 3, 4];
    assert!(unsafe { sum_val_array_u8(arr) } == 10);
}

pub fn test_sum_val_array_large() {
    let arr: [u64; 4] = [100, 200, 300, 400];
    assert!(unsafe { sum_val_array_large(arr) } == 1000);
}

pub fn test_make_array_small() {
    let arr = unsafe { make_array_small() };
    assert!(arr[0] == 10);
    assert!(arr[1] == 20);
    assert!(arr[2] == 30);
}

pub fn test_make_array_large() {
    let arr = unsafe { make_array_large() };
    assert!(arr[0] == 100);
    assert!(arr[1] == 200);
    assert!(arr[2] == 300);
    assert!(arr[3] == 400);
}
