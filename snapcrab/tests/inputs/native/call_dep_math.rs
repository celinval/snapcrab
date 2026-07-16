#![allow(unused)]

use dep_math;

pub fn test_add() {
    assert!(dep_math::add(10, 32) == 42);
}

pub fn test_multiply() {
    assert!(dep_math::multiply(6, 7) == 42);
}

pub fn test_negate() {
    assert!(dep_math::negate(42) == -42);
}

pub fn test_generic_add() {
    assert!(dep_math::generic_add(10u32, 20u32) == 30);
}

pub fn test_inline_double() {
    assert!(dep_math::inline_double(21) == 42);
}

pub fn test_sum_slice() {
    let data = [10u32, 20, 30, 40];
    assert!(dep_math::sum_slice(&data) == 100);
}
