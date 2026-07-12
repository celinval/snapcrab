#![allow(unused)]

use std::ptr;

/// Cast &str to *const str — exercises AddressOf for wide pointers.
pub fn test_str_to_raw_ptr() {
    let s: &str = "hello";
    let ptr: *const str = s;
    let len = unsafe { &*ptr }.len();
    assert!(len == 5);
}

/// Reborrow a &str.
pub fn test_str_reborrow() {
    let s: &str = "hello";
    let r: &str = &*s;
    assert!(r.len() == 5);
}

/// Wide pointer equality via std::ptr::eq.
pub fn test_wide_ptr_eq() {
    let s: &str = "hello";
    let a: &str = s;
    let b: &str = s;
    assert!(ptr::eq(a, b));
}

/// Different wide pointers are not equal.
pub fn test_wide_ptr_ne() {
    let s1: &str = "hello";
    let s2: &str = "world";
    assert!(!ptr::eq(s1, s2));
}

// --- Wrapper struct with unsized field ---

struct Wrapper<T: ?Sized> {
    value: T,
}

/// Create &Wrapper<[u8]> from &Wrapper<[u8; 5]> via unsized coercion,
/// then read the length of the inner slice.
pub fn test_wrapper_slice_len() {
    let w = Wrapper { value: [1u8, 2, 3, 4, 5] };
    let w_ref: &Wrapper<[u8]> = &w;
    assert!(w_ref.value.len() == 5);
}

/// Access elements through the unsized reference.
pub fn test_wrapper_slice_elements() {
    let w = Wrapper { value: [104u8, 101, 108, 108, 111] };
    let w_ref: &Wrapper<[u8]> = &w;
    assert!(w_ref.value[0] == 104);
    assert!(w_ref.value[4] == 111);
}

/// Dereference the unsized field and take a new reference to it.
pub fn test_wrapper_field_ref() {
    let w = Wrapper { value: [10u8, 20, 30] };
    let w_ref: &Wrapper<[u8]> = &w;
    let slice: &[u8] = &w_ref.value;
    assert!(slice.len() == 3);
    assert!(slice[0] == 10);
    assert!(slice[2] == 30);
}

/// Take a raw pointer to the unsized field.
pub fn test_wrapper_field_raw_ptr() {
    let w = Wrapper { value: [5u8, 6, 7, 8] };
    let w_ref: &Wrapper<[u8]> = &w;
    let ptr: *const [u8] = &w_ref.value;
    let len = unsafe { &*ptr }.len();
    assert!(len == 4);
}

/// Create &Wrapper<dyn Debug> from &Wrapper<u32>.
pub fn test_wrapper_dyn_debug() {
    use std::fmt::Debug;
    let w = Wrapper { value: 42u32 };
    let w_ref: &Wrapper<dyn Debug> = &w;
    let _ = &w_ref.value;
}
