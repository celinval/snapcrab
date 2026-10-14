//! Unsizing coercion through a *user-defined* `CoerceUnsized` impl.
//!
//! `Rc`, `Arc` and `Pin` only carry zero-sized extras alongside their pointer,
//! so they never exercise copying a field that keeps its type. `Holder` does:
//! `lo` and `hi` must survive the coercion, and because the pointer grows from
//! thin to wide they sit at *different offsets* in the two layouts (`lo` moves
//! from 12 to 20 and `hi` from 8 to 16), so each side's offsets have to come
//! from its own layout.

#![feature(coerce_unsized, unsize)]
#![allow(unused)]

use std::marker::Unsize;
use std::ops::CoerceUnsized;

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

/// A minimal smart pointer: the pointer being unsized, plus two sized fields
/// that the coercion must preserve.
struct Holder<T: ?Sized> {
    lo: u8,
    ptr: *const T,
    hi: u32,
}

impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<Holder<U>> for Holder<T> {}

/// `Holder<T> -> Holder<dyn Trait>` attaches a vtable and keeps the other
/// fields intact.
pub fn user_ptr_to_dyn() {
    let square = Square(7);
    let sized = Holder {
        lo: 0xAB,
        ptr: &square,
        hi: 0xDEAD_BEEF,
    };
    let coerced: Holder<dyn Shape> = sized;
    assert!(coerced.lo == 0xAB);
    assert!(coerced.hi == 0xDEAD_BEEF);
    let shape: &dyn Shape = unsafe { &*coerced.ptr };
    assert!(shape.area() == 49);
}

/// `Holder<()> -> Holder<dyn Trait>`: the pointee is zero-sized, so the data
/// pointer addresses nothing, but a vtable is still attached and dispatched
/// through.
///
/// A coercion that changes *only* a zero-sized field's type cannot be written:
/// rustc rejects such a `CoerceUnsized` impl with `E0374`, since no field would
/// be coerced. A zero-sized field that changes type alongside the pointer is
/// simply carried over, which `Rc`'s `PhantomData` already exercises.
pub fn user_ptr_zst_to_dyn() {
    let unit = ();
    let sized = Holder {
        lo: 3,
        ptr: &unit,
        hi: 4,
    };
    let coerced: Holder<dyn Shape> = sized;
    assert!(coerced.lo == 3 && coerced.hi == 4);
    let shape: &dyn Shape = unsafe { &*coerced.ptr };
    assert!(shape.area() == 7);
}

/// `Holder<[T; N]> -> Holder<[T]>` attaches the element count instead.
pub fn user_ptr_to_slice() {
    let array = [1u32, 2, 3];
    let sized = Holder {
        lo: 1,
        ptr: &array,
        hi: 2,
    };
    let coerced: Holder<[u32]> = sized;
    assert!(coerced.lo == 1 && coerced.hi == 2);
    let slice: &[u32] = unsafe { &*coerced.ptr };
    assert!(slice.len() == 3 && slice[2] == 3);
}
