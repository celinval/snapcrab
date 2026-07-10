#![allow(unused)]

use std::mem;
use std::num::NonZeroU32;

/// Transmute zero to NonZeroU32 — violates validity invariant.
pub fn test_zero_to_nonzero() -> NonZeroU32 {
    let zero: u32 = 0;
    unsafe { mem::transmute(zero) }
}

/// Transmute arbitrary byte to bool — value 2 is invalid for bool.
pub fn test_invalid_bool() -> bool {
    let two: u8 = 2;
    unsafe { mem::transmute(two) }
}

/// Transmute 255 to bool — any value other than 0 or 1 is invalid.
pub fn test_invalid_bool_255() -> bool {
    let val: u8 = 255;
    unsafe { mem::transmute(val) }
}

/// Transmute an invalid discriminant value into an enum.
pub fn test_invalid_enum_discriminant() -> MyEnum {
    let bad: u8 = 5;
    unsafe { mem::transmute(bad) }
}

#[repr(u8)]
pub enum MyEnum {
    A = 0,
    B = 1,
    C = 2,
}
