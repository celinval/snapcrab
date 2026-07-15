#![allow(unused, improper_ctypes)]

unsafe extern "C" {
    fn make_array2_u64() -> [u64; 2];
    fn sum_array2_u64(arr: [u64; 2]) -> u64;
    fn make_array2_u32() -> [u32; 2];
    fn sum_array2_u32(arr: [u32; 2]) -> u32;
}

pub fn test_make_array2_u64() {
    let arr = unsafe { make_array2_u64() };
    assert!(arr[0] == 10);
    assert!(arr[1] == 20);
}

pub fn test_sum_array2_u64() {
    let arr: [u64; 2] = [100, 200];
    assert!(unsafe { sum_array2_u64(arr) } == 300);
}

pub fn test_make_array2_u32() {
    let arr = unsafe { make_array2_u32() };
    assert!(arr[0] == 5);
    assert!(arr[1] == 7);
}

pub fn test_sum_array2_u32() {
    let arr: [u32; 2] = [30, 40];
    assert!(unsafe { sum_array2_u32(arr) } == 70);
}
