#![allow(unused)]

// Basic char literal
pub fn get_char_a() -> char {
    'a'
}

// Unicode char
pub fn get_unicode_char() -> char {
    '🦀'
}

// Char comparison
pub fn compare_chars() -> bool {
    'x' == 'x'
}

// Char to u32
pub fn char_to_u32() -> u32 {
    'A' as u32
}

// Char from u32
pub fn u32_to_char() -> char {
    char::from_u32(65).unwrap()
}

// Char is alphabetic
pub fn is_alphabetic() -> bool {
    'z'.is_alphabetic()
}

// Char is numeric
pub fn is_numeric() -> bool {
    '5'.is_numeric()
}

// Char is whitespace
pub fn is_whitespace() -> bool {
    ' '.is_whitespace()
}
