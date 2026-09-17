//! Function pointers: reification (`fn` items and non-capturing closures),
//! indirect calls, storage, and identity.

#![allow(unused)]

fn add(a: u32, b: u32) -> u32 {
    a + b
}
fn double(x: u32) -> u32 {
    x * 2
}

/// A `fn` item coerced to a `fn` pointer and called indirectly.
pub fn reify_fn_item() {
    let f: fn(u32, u32) -> u32 = add;
    assert!(f(40, 2) == 42);
}

fn apply(g: fn(u32) -> u32, x: u32) -> u32 {
    g(x)
}

/// A `fn` pointer passed as an argument and called there.
pub fn fn_ptr_as_arg() {
    let f: fn(u32) -> u32 = double;
    assert!(apply(f, 21) == 42);
}

/// A non-capturing closure coerced to a `fn` pointer.
pub fn closure_to_fn_ptr() {
    let f: fn(u32) -> u32 = |x| x + 1;
    assert!(f(41) == 42);
}

/// Reifying the same function twice yields the same address, so `fn` pointers
/// to it compare equal.
pub fn fn_ptr_identity() {
    let f: fn(u32) -> u32 = double;
    let g: fn(u32) -> u32 = double;
    assert!(f == g);
    let h: fn(u32) -> u32 = |x| x + 1;
    assert!(f != h);
}

/// A `fn` pointer stored in a struct field, then loaded and called.
pub fn fn_ptr_in_struct() {
    struct Ops {
        op: fn(u32) -> u32,
    }
    let ops = Ops { op: double };
    assert!((ops.op)(21) == 42);
}
