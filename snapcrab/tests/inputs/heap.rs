//! Heap allocation via `Box`, backed by the interpreter's heap segment.
//!
//! Each function allocates on the heap, uses the value, and lets it drop
//! (freeing the block). Success is silent; a wrong value panics.

#![allow(unused)]

/// Allocate, dereference, and drop a boxed scalar.
pub fn box_new_deref() {
    let b = Box::new(42u32);
    assert!(*b == 42);
}

/// Mutate through a `Box`.
pub fn box_mutate() {
    let mut b = Box::new(1u32);
    *b += 41;
    assert!(*b == 42);
}

/// Box a struct and read its fields back.
pub fn box_struct() {
    struct Point {
        x: u32,
        y: u32,
    }
    let b = Box::new(Point { x: 3, y: 4 });
    assert!(b.x + b.y == 7);
}

/// A `Box` of a `Box`: two independent heap allocations.
pub fn nested_box() {
    let b = Box::new(Box::new(7u64));
    assert!(**b == 7);
}

/// Moving a `Box` into a callee transfers ownership; it is freed there.
pub fn box_moved_into_fn() {
    fn consume(b: Box<u32>) -> u32 {
        *b
    }
    let b = Box::new(21u32);
    assert!(consume(b) * 2 == 42);
}

/// Overwriting a `Box` frees the old allocation before storing the new one.
pub fn box_reassign() {
    let mut b = Box::new(1u32);
    b = Box::new(2u32);
    assert!(*b == 2);
}
