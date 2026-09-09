//! Bit-counting intrinsics: `ctpop`, `cttz`/`cttz_nonzero`, `ctlz`/`ctlz_nonzero`.
//!
//! Reached through the stable integer methods that lower to them.

#![allow(unused)]

use std::num::NonZeroU32;

/// `count_ones` -> `ctpop`.
pub fn count_ones() {
    assert!(0b1011u8.count_ones() == 3);
    assert!(0u32.count_ones() == 0);
    assert!(u64::MAX.count_ones() == 64);
    assert!(0xFFu32.count_ones() == 8);
}

/// `trailing_zeros` -> `cttz`. Zero yields the full bit width.
pub fn trailing_zeros() {
    assert!(0b1000u8.trailing_zeros() == 3);
    assert!(1u64.trailing_zeros() == 0);
    assert!(0u32.trailing_zeros() == 32);
    assert!(0b0110_0000u8.trailing_zeros() == 5);
}

/// `leading_zeros` -> `ctlz`. Zero yields the full bit width.
pub fn leading_zeros() {
    assert!(1u8.leading_zeros() == 7);
    assert!(0u32.leading_zeros() == 32);
    assert!(u16::MAX.leading_zeros() == 0);
    assert!((1u32 << 20).leading_zeros() == 11);
}

/// `NonZero` methods -> the `_nonzero` variants (`cttz_nonzero`/`ctlz_nonzero`).
pub fn nonzero_bit_ops() {
    let n = NonZeroU32::new(0b1000).unwrap();
    assert!(n.trailing_zeros() == 3);
    assert!(n.leading_zeros() == 28);
}
