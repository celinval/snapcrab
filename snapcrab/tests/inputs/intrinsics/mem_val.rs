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
