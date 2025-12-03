#![allow(unused)]

// Char to u32
pub fn char_to_u32() -> bool {
    'A' as u32 == 65
}

// u32 to char
pub fn u32_to_char_cast() -> bool {
    65 as char == 'A'
}

// Char to u8 (truncate)
pub fn char_to_u8() -> bool {
    'Z' as u8 == 90
}

// u8 to char
pub fn u8_to_char() -> bool {
    90 as char == 'Z'
}

// Unicode char to u8 (truncate)
pub fn unicode_char_to_u8() -> bool {
    '🦀' as u8 == 128
}

// Unsigned to signed - same size
pub fn u32_to_i32() -> bool {
    0xFFFFFFFF_u32 as i32 == -1
}

// Signed to unsigned - same size
pub fn i32_to_u32() -> bool {
    -1_i32 as u32 == 0xFFFFFFFF
}

// Unsigned upcast
pub fn u8_to_u32() -> bool {
    255_u8 as u32 == 255
}

// Signed upcast (sign extend)
pub fn i8_to_i32() -> bool {
    -1_i8 as i32 == -1
}

// Signed to unsigned upcast
pub fn i8_to_u32() -> bool {
    -1_i8 as u32 == 0xFFFFFFFF
}

// Unsigned downcast (truncate)
pub fn u32_to_u8() -> bool {
    0x12345678_u32 as u8 == 0x78
}

// Signed downcast (truncate)
pub fn i32_to_i8() -> bool {
    0x1234_i32 as i8 == 0x34
}

// Negative signed downcast
pub fn neg_i32_to_i8() -> bool {
    -1000_i32 as i8 == 24  // -1000 & 0xFF = 24
}

// u64 to u16 (truncate)
pub fn u64_to_u16() -> bool {
    0xDEADBEEF_u64 as u16 == 0xBEEF
}

// i16 to i64 (sign extend)
pub fn i16_to_i64() -> bool {
    -1000_i16 as i64 == -1000
}

// i32 1000 to u8 (truncate)
pub fn i32_1000_to_u8() -> bool {
    1000_i32 as u8 == 232
}
