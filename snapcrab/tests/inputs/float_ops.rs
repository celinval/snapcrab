//! Floating-point arithmetic and comparison, and three-way comparison
//! (`BinOp::Cmp`) on the types that support it.
//!
//! Operands go through `black_box` so the compiler cannot fold the operation
//! away and the interpreter actually evaluates it.

#![allow(unused)]

use std::cmp::Ordering;
use std::hint::black_box;

/// Equality follows IEEE 754: NaN is unequal to everything, itself included,
/// and the two zeros are equal although their bits differ.
pub fn float_equality() {
    let nan = black_box(f64::NAN);
    assert!(nan != nan);
    assert!(!(nan == nan));
    assert!(black_box(0.0f64) == black_box(-0.0f64));
    assert!(black_box(1.5f32) == 1.5);
    assert!(black_box(1.5f32) != 2.5);
}

/// Every ordered comparison with NaN is false.
pub fn float_ordering() {
    let one = black_box(1.0f64);
    assert!(one < 2.0 && one <= 1.0 && one > 0.5 && one >= 1.0);
    let nan = black_box(f64::NAN);
    assert!(!(nan < one) && !(nan <= one) && !(nan > one) && !(nan >= one));
    assert!(black_box(-1.0f32) < black_box(0.0f32));
}

/// `partial_cmp` reports NaN as unordered.
pub fn float_partial_cmp() {
    let one = black_box(1.0f64);
    assert!(one.partial_cmp(&2.0) == Some(Ordering::Less));
    assert!(one.partial_cmp(&1.0) == Some(Ordering::Equal));
    assert!(one.partial_cmp(&f64::NAN).is_none());
}

/// `total_cmp` orders every value, putting `-0.0` before `0.0`.
pub fn float_total_cmp() {
    assert!(black_box(-0.0f64).total_cmp(&0.0) == Ordering::Less);
    assert!(black_box(1.0f32).total_cmp(&2.0) == Ordering::Less);
    assert!(black_box(f64::NAN).total_cmp(&f64::INFINITY) == Ordering::Greater);
}

/// Arithmetic, including the results that would panic for integers: dividing
/// by zero gives an infinity, and the remainder takes the dividend's sign.
pub fn float_arithmetic() {
    let x = black_box(1.5f64);
    assert!(x + 0.25 == 1.75);
    assert!(x - 2.0 == -0.5);
    assert!(x * 4.0 == 6.0);
    assert!(x / 0.5 == 3.0);
    assert!(black_box(7.5f64) % 2.0 == 1.5);
    assert!(black_box(-7.5f64) % 2.0 == -1.5);
    assert!(black_box(1.0f64) / black_box(0.0f64) == f64::INFINITY);
    assert!((black_box(0.0f64) / black_box(0.0f64)).is_nan());
    assert!(black_box(0.1f64) + black_box(0.2f64) != 0.3);
}

/// Negation flips the sign bit, so negating zero gives negative zero.
pub fn float_negation() {
    assert!(-black_box(1.5f64) == -1.5);
    assert!((-black_box(0.0f64)).is_sign_negative());
    assert!(-black_box(f32::INFINITY) == f32::NEG_INFINITY);
}

/// `max` and `min` ignore a NaN operand.
pub fn float_min_max() {
    assert!(black_box(1.0f64).max(2.0) == 2.0);
    assert!(black_box(1.0f64).min(2.0) == 1.0);
    assert!(black_box(f64::NAN).max(3.0) == 3.0);
}

/// `Ord::cmp` lowers to `BinOp::Cmp` for integers, `char` and `bool`.
pub fn three_way_compare() {
    assert!(black_box(-3i32).cmp(&2) == Ordering::Less);
    assert!(black_box(200u8).cmp(&200) == Ordering::Equal);
    assert!(black_box(u64::MAX).cmp(&0) == Ordering::Greater);
    assert!(black_box('b').cmp(&'a') == Ordering::Greater);
    assert!(black_box(false).cmp(&true) == Ordering::Less);
}
