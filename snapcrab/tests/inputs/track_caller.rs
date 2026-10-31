//! `#[track_caller]`, which passes the caller's `Location` as an implicit
//! argument that the MIR body has no local for.
//!
//! The panic machinery relies on it, so a panic should be reported at the
//! user's call site rather than inside `core`.
//!
//! NOTE: `track_caller_tests.rs` asserts the line numbers below, and two
//! functions assert their own call line, so inserting or removing lines in this
//! file means updating both.
//!
//! The reported column can differ from rustc's for a method call: rustc builds
//! the location from the call terminator's `fn_span`, which points at the
//! method segment, while `Body::caller_location` uses `source_info.span`, which
//! starts at the receiver. The tests therefore pin the line, not the column.

#![allow(unused)]

/// `Option::unwrap` on `None` reports the `unwrap` call site.
pub fn unwrap_none() -> u32 {
    let empty: Option<u32> = None;
    empty.unwrap()
}

/// `Option::expect` reports the user's message and the `expect` call site.
pub fn expect_none() -> u32 {
    let empty: Option<u32> = None;
    empty.expect("value must be present")
}

/// A user `#[track_caller]` function propagates its own caller's location, so
/// the panic points at the `require` call rather than inside `require`.
#[track_caller]
fn require(condition: bool) {
    assert!(condition, "requirement failed");
}

pub fn propagates_through_track_caller() {
    require(false);
}

/// An explicit `panic!` reports the invocation site.
pub fn explicit_panic() {
    panic!("boom");
}

/// Indexing a `Vec` out of bounds reports the indexing site. The bounds check
/// is an `Assert` inside core's `#[track_caller]` `Index` impl, so reporting it
/// at the user's line relies on the inherited location. The index is hidden
/// from the compiler so it cannot reject the program outright.
pub fn index_out_of_bounds() -> u32 {
    let values = vec![1u32, 2];
    let index = std::hint::black_box(5);
    values[index]
}

/// `Location::caller()` inside a `#[track_caller]` function returns the
/// caller's location, which the `caller_location` intrinsic provides.
#[track_caller]
fn caller_line() -> u32 {
    std::panic::Location::caller().line()
}

/// The call below sits on line 65, so that is the line reported.
pub fn location_caller_line() -> bool {
    caller_line() == 65
}

/// A `#[track_caller]` function called from another one reports the outermost
/// user call site, not the intermediate hop.
#[track_caller]
fn inner_line() -> u32 {
    std::panic::Location::caller().line()
}

#[track_caller]
fn outer_line() -> u32 {
    inner_line()
}

/// `outer_line` is `#[track_caller]`, so `inner_line` observes the location
/// that `outer_line` inherited: the call on line 83.
pub fn location_propagates_two_levels() -> bool {
    outer_line() == 83
}

/// `Result::unwrap` on `Err` reports the call site. The `Err` value's `Debug`
/// output is omitted, since rendering it needs the formatting machinery.
pub fn result_unwrap_err() -> u32 {
    let failed: Result<u32, &str> = Err("oh no");
    failed.unwrap()
}

/// A formatted panic is reported at the call site without its message, rather
/// than reaching the native panic runtime, which cannot unwind back across the
/// JIT trampoline and would abort.
pub fn assert_eq_fails() {
    let value = std::hint::black_box(1u32);
    assert_eq!(value, 2);
}

/// `Result::expect` reports the user's message. The `Err` value's `Debug`
/// output still needs the formatting machinery.
pub fn result_expect_message() -> u32 {
    let failed: Result<u32, &str> = Err("oh no");
    failed.expect("custom expect message")
}

/// Slicing out of range panics through `slice_index_fail`, which is bodiless
/// and diverging: it must fail cleanly rather than entering the native panic
/// runtime, which cannot unwind back across the JIT trampoline.
pub fn slice_range_out_of_bounds() -> u32 {
    let values = [1u32, 2, 3];
    let end = std::hint::black_box(9);
    let slice = &values[1..end];
    slice[0]
}

/// `panic_any` carries a non-`&str` payload, so it reaches a panic entry the
/// interpreter does not recognize and must still fail cleanly.
pub fn panic_any_payload() {
    std::panic::panic_any(42u32);
}

/// A `panic!` with format parameters is reported at its location, but its
/// message is not rendered yet, since that needs the formatting machinery.
pub fn formatted_panic() {
    let value = std::hint::black_box(7u32);
    panic!("value was {value}");
}
