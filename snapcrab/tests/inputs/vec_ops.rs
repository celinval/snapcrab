//! `Vec` operations, which combine heap allocation with the slice iteration
//! and pointer arithmetic the iterator protocol relies on.

#![allow(unused)]

/// Pushing onto an empty `Vec` allocates, and iterating reads the elements
/// back.
pub fn push_and_iter() {
    let mut v = Vec::new();
    v.push(1u32);
    v.push(2);
    v.push(3);
    let mut total = 0u32;
    for x in &v {
        total += *x;
    }
    assert!(total == 6);
}

/// Growing past the initial capacity reallocates, preserving the elements.
pub fn grow_reallocates() {
    let mut v = Vec::new();
    for i in 0..20u32 {
        v.push(i);
    }
    assert!(v.len() == 20 && v[0] == 0 && v[19] == 19);
}

/// The `vec!` macro builds from a constant slice.
pub fn vec_macro() {
    let v = vec![1u32, 2, 3];
    assert!(v.len() == 3 && v[1] == 2);
}

/// `pop` shrinks the length and returns the last element.
pub fn pop_returns_last() {
    let mut v = vec![1u32, 2];
    assert!(v.pop() == Some(2));
    assert!(v.len() == 1);
    assert!(v.pop() == Some(1));
    assert!(v.pop() == None);
}

/// `collect` builds a `Vec` from an iterator.
pub fn collect_from_iter() {
    let v: Vec<u32> = (0..5u32).map(|x| x * 2).collect();
    assert!(v.len() == 5 && v[4] == 8);
}

/// A `Vec` of heap-allocating elements drops each element with the buffer.
pub fn vec_of_strings() {
    let mut v = Vec::new();
    v.push(String::from("hi"));
    v.push(String::from("there"));
    assert!(v[0].len() == 2 && v[1].len() == 5);
}

/// Dropping a `Vec` frees its buffer.
pub fn drop_frees_buffer() {
    let v = vec![1u32, 2, 3];
    drop(v);
}
