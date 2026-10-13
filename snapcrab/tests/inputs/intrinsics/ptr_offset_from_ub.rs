//! Undefined behavior in `ptr_offset_from*` that the safe wrappers reject
//! before the intrinsic runs.
//!
//! `offset_from` and `offset_from_unsigned` assert on a zero-sized `T` and
//! debug-assert the pointer order, so these cases are only reachable by calling
//! the intrinsics directly. Both are confirmed UB by Miri; native `rustc`
//! instead crashes with `SIGFPE` on the first and silently returns a wrapped
//! distance on the second.

#![allow(internal_features)]
#![feature(core_intrinsics)]
#![allow(unused)]

/// A zero-sized element type makes the element distance undefined: the division
/// by the element size has a zero divisor.
pub fn offset_from_zst() -> isize {
    let units = [(), ()];
    let p = units.as_ptr();
    unsafe { std::intrinsics::ptr_offset_from(p, p) }
}

/// `ptr_offset_from_unsigned` requires `ptr >= base`, so a pointer before the
/// base is undefined behavior rather than a negative result.
pub fn offset_from_unsigned_backwards() -> usize {
    let array = [1u32, 2, 3];
    let p = array.as_ptr();
    let q = unsafe { p.add(2) };
    unsafe { std::intrinsics::ptr_offset_from_unsigned(p, q) }
}
