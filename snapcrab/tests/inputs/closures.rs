//! Closures exercised through different capture modes and call styles.
//!
//! Direct calls (`c()`) devirtualize straight to the closure body, while
//! passing a closure to a generic bound (`F: Fn`) or a trait object routes
//! through a `call*` shim that the interpreter must resolve to the body.

#![allow(unused)]

// --- Direct calls ---

/// Non-capturing closure called directly.
pub fn direct_no_capture() {
    let add = |a: u32, b: u32| a + b;
    assert!(add(2, 3) == 5);
}

/// Closure capturing a local by reference, called directly.
pub fn direct_capture_ref() {
    let base = 10u32;
    let add_base = |x: u32| x + base;
    assert!(add_base(5) == 15);
}

/// Closure mutating captured state (`FnMut`), called directly.
pub fn direct_capture_mut() {
    let mut count = 0u32;
    let mut bump = || count += 2;
    bump();
    bump();
    assert!(count == 4);
}

/// Closure taking ownership of a captured value (`FnOnce`), called directly.
pub fn direct_capture_move() {
    let owned = 42u32;
    let consume = move || owned;
    assert!(consume() == 42);
}

// --- Calls through generic bounds (routes via a call shim) ---

fn apply_fn<F: Fn(u32) -> u32>(f: F, x: u32) -> u32 {
    f(x)
}

fn apply_fn_mut<F: FnMut()>(mut f: F) {
    f();
    f();
}

fn apply_fn_once<F: FnOnce() -> u32>(f: F) -> u32 {
    f()
}

/// Non-capturing closure passed to a `Fn` bound.
pub fn generic_fn() {
    let double = |x: u32| x * 2;
    assert!(apply_fn(double, 21) == 42);
}

/// Closure capturing by reference passed to a `Fn` bound.
pub fn generic_fn_capture() {
    let offset = 7u32;
    let shift = |x: u32| x + offset;
    assert!(apply_fn(shift, 35) == 42);
}

/// `FnMut` closure passed to a generic bound; mutation persists across calls.
pub fn generic_fn_mut() {
    let mut total = 0u32;
    apply_fn_mut(|| total += 3);
    assert!(total == 6);
}

/// `FnOnce` closure moving a captured value into a generic bound.
pub fn generic_fn_once() {
    let owned = 100u32;
    assert!(apply_fn_once(move || owned) == 100);
}

// --- Closures as function pointers and trait objects ---

/// Non-capturing closure coerced to a `fn` pointer.
pub fn as_fn_pointer() {
    let f: fn(u32) -> u32 = |x| x + 1;
    assert!(f(41) == 42);
}

/// Closure invoked through a `&dyn Fn` trait object.
pub fn as_dyn_fn() {
    let scale = 3u32;
    let f: &dyn Fn(u32) -> u32 = &|x| x * scale;
    assert!(f(14) == 42);
}

// --- Composition ---

/// A closure that captures and calls another closure.
pub fn nested_closures() {
    let inc = |x: u32| x + 1;
    let twice = |x: u32| inc(inc(x));
    assert!(twice(40) == 42);
}

/// Closure returning a value with no arguments.
pub fn returns_value() {
    let answer = || 42u32;
    assert!(answer() == 42);
}
