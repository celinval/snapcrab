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

/// Calls the same function twice (exercises symbol cache).
pub fn test_repeated_calls() {
    assert!(dep_math::add(1, 2) == 3);
    assert!(dep_math::add(100, 200) == 300);
}

/// Calls two functions with identical FnAbi (exercises trampoline cache sharing).
pub fn test_shared_trampoline() {
    assert!(dep_math::add(10, 5) == 15);
    assert!(dep_math::subtract(10, 5) == 5);
    assert!(dep_math::add(100, 200) == 300);
}

/// Calls functions with different FnAbi shapes (cache miss after hit).
pub fn test_different_abis() {
    assert!(dep_math::add(1, 2) == 3);
    assert!(dep_math::multiply(3, 4) == 12);
    assert!(dep_math::negate(5) == -5);
    assert!(dep_math::add(10, 20) == 30);
}
