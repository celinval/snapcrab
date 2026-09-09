//! Unchecked integer arithmetic intrinsics (`add`/`sub`/`mul` unchecked).
//!
//! The interpreter evaluates these as their checked forms, so results match
//! when no overflow occurs (overflow would be undefined behavior).

#![allow(unused)]

/// `unchecked_add` -> `AddUnchecked`.
pub fn unchecked_add() {
    unsafe {
        assert!(10u32.unchecked_add(32) == 42);
        assert!((-5i32).unchecked_add(47) == 42);
        assert!(0u8.unchecked_add(255) == 255);
    }
}

/// `unchecked_sub` -> `SubUnchecked`.
pub fn unchecked_sub() {
    unsafe {
        assert!(50u32.unchecked_sub(8) == 42);
        assert!(0i32.unchecked_sub(42) == -42);
    }
}

/// `unchecked_mul` -> `MulUnchecked`.
pub fn unchecked_mul() {
    unsafe {
        assert!(6u32.unchecked_mul(7) == 42);
        assert!((-6i32).unchecked_mul(7) == -42);
    }
}
