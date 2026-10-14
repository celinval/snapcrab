//! Unsizing coercions through a smart pointer's own `CoerceUnsized` impl.
//!
//! Unlike `&T` and `Box<T>`, types such as `Rc`, `Arc` and `Pin` are ordinary
//! structs, so the pointer being unsized sits in a field and the whole struct
//! has to be rebuilt around the resulting wide pointer. The zero-sized cases at
//! the end cover coercions where the metadata or the pointee carries no bytes.

#![allow(unused)]

use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;

trait Shape {
    fn area(&self) -> u32;
}

struct Square(u32);

impl Shape for Square {
    fn area(&self) -> u32 {
        self.0 * self.0
    }
}

/// A zero-sized implementor, so a coerced pointer addresses no bytes.
impl Shape for () {
    fn area(&self) -> u32 {
        7
    }
}

/// `Rc<T> -> Rc<dyn Trait>`, then a virtual call through the coerced vtable.
pub fn rc_to_dyn() {
    let shape: Rc<dyn Shape> = Rc::new(Square(3));
    assert!(shape.area() == 9);
}

/// `Rc<[T; N]> -> Rc<[T]>`, where the metadata is the element count rather than
/// a vtable.
pub fn rc_to_slice() {
    let slice: Rc<[u32]> = Rc::new([1u32, 2, 3]);
    assert!(slice.len() == 3);
    assert!(slice[0] == 1 && slice[2] == 3);
}

/// Cloning and dropping a coerced `Rc` keeps the refcount consistent, so the
/// rebuilt pointer still refers to the original allocation.
pub fn rc_dyn_clone_and_drop() {
    let shape: Rc<dyn Shape> = Rc::new(Square(2));
    let clone = Rc::clone(&shape);
    assert!(clone.area() == 4);
    assert!(Rc::strong_count(&shape) == 2);
    drop(clone);
    assert!(Rc::strong_count(&shape) == 1);
}

/// `Pin<&mut T> -> Pin<&mut dyn Trait>`: a single-field wrapper around a
/// reference.
pub fn pin_to_dyn() {
    let mut square = Square(6);
    let pinned = Pin::new(&mut square);
    let shape: Pin<&mut dyn Shape> = pinned;
    assert!(shape.area() == 36);
}

/// A struct with an unsized tail held behind a smart pointer:
/// `Rc<Wrapper<T>> -> Rc<Wrapper<dyn Trait>>` rebuilds the `Rc` *and* derives
/// the metadata from the tail field.
pub fn rc_to_unsized_tail() {
    struct Wrapper<T: ?Sized> {
        header: u32,
        tail: T,
    }

    let wrapper: Rc<Wrapper<dyn Shape>> = Rc::new(Wrapper {
        header: 2,
        tail: Square(5),
    });
    assert!(wrapper.header == 2);
    assert!(wrapper.tail.area() == 25);
}

/// A zero-sized pointee still gets a vtable: `Rc<()> -> Rc<dyn Trait>` has
/// nothing to point at, but dispatch must still find the `()` impl.
pub fn rc_zst_to_dyn() {
    let shape: Rc<dyn Shape> = Rc::new(());
    assert!(shape.area() == 7);
}

/// `&Struct<[u8; 0]> -> &Struct<[u8]>`: the tail coerces from a zero-length
/// array to a slice, so the metadata is a length of zero rather than a count
/// of real elements. The sized header stays readable through the wide pointer.
pub fn ref_empty_slice_tail() {
    struct WithTail<T: ?Sized> {
        header: u32,
        tail: T,
    }

    let sized = WithTail {
        header: 5,
        tail: [0u8; 0],
    };
    let unsized_ref: &WithTail<[u8]> = &sized;
    assert!(unsized_ref.header == 5);
    assert!(unsized_ref.tail.len() == 0);
    assert!(unsized_ref.tail.is_empty());
}

/// `Arc<T> -> Arc<dyn Trait>`. The coercion itself behaves like `Rc`'s, but
/// `Arc` keeps its refcount in an atomic, so this currently stops at the
/// unimplemented atomic intrinsics.
pub fn arc_to_dyn() {
    let shape: Arc<dyn Shape> = Arc::new(Square(5));
    assert!(shape.area() == 25);
}

/// `Arc<[T; N]> -> Arc<[T]>`, blocked on the same atomic intrinsics.
pub fn arc_to_slice() {
    let slice: Arc<[u32]> = Arc::new([7u32, 8]);
    assert!(slice.len() == 2 && slice[1] == 8);
}
