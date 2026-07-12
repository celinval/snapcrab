#![allow(unused)]

unsafe extern "C" {
    fn abs(i: i32) -> i32;
}

pub fn test_abs() {
    let result = unsafe { abs(-42) };
    assert!(result == 42);
}
