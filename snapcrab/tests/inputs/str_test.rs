#![allow(unused)]

// Get str length
pub fn get_str_len() -> usize {
    let s = "world";
    s.len()
}

// Read byte from str - should fail with pointer error
pub fn read_str_byte() -> u8 {
    let s = "hello";
    unsafe { *(s.as_ptr() as *const u8) }
}

