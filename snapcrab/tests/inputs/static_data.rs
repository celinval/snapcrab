#![allow(unused)]

/// Single string literal length.
pub fn test_str_len() {
    let s = "hello";
    assert!(s.len() == 5);
}

/// Array of string literals — two levels of indirection.
/// The constant is: [&str; 2] = [fat_ptr_0, fat_ptr_1]
/// Each fat_ptr points to a separate byte allocation.
pub fn test_str_array_lens() {
    let arr: [&str; 2] = ["foo", "quux"];
    assert!(arr[0].len() == 3);
    assert!(arr[1].len() == 4);
}

/// Nested reference to a static array of &str.
pub fn test_static_str_slice_len() {
    static WORDS: &[&str] = &["alpha", "beta", "gamma"];
    assert!(WORDS.len() == 3);
    assert!(WORDS[0].len() == 5);
    assert!(WORDS[1].len() == 4);
    assert!(WORDS[2].len() == 5);
}

/// Reference to a byte array constant.
pub fn test_byte_array_ref() {
    let bytes: &[u8; 4] = b"rust";
    assert!(bytes[0] == b'r');
    assert!(bytes[1] == b'u');
    assert!(bytes[2] == b's');
    assert!(bytes[3] == b't');
}
