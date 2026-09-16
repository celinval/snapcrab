//! `size_of_val` / `align_of_val` intrinsics, for both sized and unsized types.

#![allow(unused)]

/// `size_of_val`: sized values, plus slices and `str` whose size comes from
/// the pointer metadata.
pub fn size_of_val() {
    let scalar: u64 = 0;
    assert!(std::mem::size_of_val(&scalar) == 8);

    let array = [0u32; 4];
    assert!(std::mem::size_of_val(&array) == 16);

    let slice: &[u32] = &[1, 2, 3];
    assert!(std::mem::size_of_val(slice) == 12);

    let text: &str = "hello";
    assert!(std::mem::size_of_val(text) == 5);
}

/// `align_of_val`: alignment for sized values and for slices (element align).
pub fn align_of_val() {
    let scalar: u64 = 0;
    assert!(std::mem::align_of_val(&scalar) == 8);

    let byte: u8 = 0;
    assert!(std::mem::align_of_val(&byte) == 1);

    let slice: &[u32] = &[1, 2, 3];
    assert!(std::mem::align_of_val(slice) == 4);
}

/// A struct whose tail is unsized, so its size depends on the pointer metadata.
struct Tail<T: ?Sized> {
    header: u64,
    tail: T,
}

/// An unsized tail is rejected rather than answered from the sized prefix,
/// which would report 8 instead of 16 here.
pub fn size_of_val_unsized_tail() -> usize {
    let sized: Tail<[u8; 3]> = Tail { header: 0, tail: [1, 2, 3] };
    let unsized_ref: &Tail<[u8]> = &sized;
    std::mem::size_of_val(unsized_ref)
}

/// The static alignment would happen to be right here, but it is rejected for
/// the same reason: a `dyn Trait` tail would take its alignment from the vtable.
pub fn align_of_val_unsized_tail() -> usize {
    let sized: Tail<[u8; 3]> = Tail { header: 0, tail: [1, 2, 3] };
    let unsized_ref: &Tail<[u8]> = &sized;
    std::mem::align_of_val(unsized_ref)
}
