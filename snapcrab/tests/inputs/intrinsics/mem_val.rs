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

/// A struct with a sized `header: H` followed by an unsized `tail: T`, so its
/// size and alignment depend on the pointer metadata. Being generic over both
/// fields lets the tests vary how the header and tail alignments compare.
struct Wrapper<H, T: ?Sized> {
    header: H,
    tail: T,
}

/// The header's alignment dominates the tail's.
///
/// - `u64` header (size 8, align 8) + `[u8; 3]` tail: end 8 + 3 = 11, rounded
///   up to align 8 -> size 16, align 8.
/// - `u128` header (size 16, align 16) + `[u16; 3]` tail: end 16 + 6 = 22,
///   rounded up to align 16 -> size 32, align 16.
pub fn header_align_dominates() {
    let a: Wrapper<u64, [u8; 3]> = Wrapper { header: 0, tail: [1, 2, 3] };
    let a: &Wrapper<u64, [u8]> = &a;
    assert!(std::mem::size_of_val(a) == 16);
    assert!(std::mem::align_of_val(a) == 8);

    let b: Wrapper<u128, [u16; 3]> = Wrapper { header: 0, tail: [1, 2, 3] };
    let b: &Wrapper<u128, [u16]> = &b;
    assert!(std::mem::size_of_val(b) == 32);
    assert!(std::mem::align_of_val(b) == 16);
}

/// The tail's alignment dominates the header's: a `u8` header is padded to the
/// tail's offset. `u8` header + `[u32; 3]` tail -> offset 4, end 4 + 12 = 16,
/// align 4 -> size 16, align 4.
pub fn tail_align_dominates() {
    let w: Wrapper<u8, [u32; 3]> = Wrapper { header: 1, tail: [10, 20, 30] };
    let w: &Wrapper<u8, [u32]> = &w;
    assert!(std::mem::size_of_val(w) == 16);
    assert!(std::mem::align_of_val(w) == 4);
}

/// Header and tail share the same alignment. `u32` header + `[u32; 2]` tail ->
/// offset 4, end 4 + 8 = 12, align 4 -> size 12, align 4.
pub fn equal_align() {
    let w: Wrapper<u32, [u32; 2]> = Wrapper { header: 7, tail: [10, 20] };
    let w: &Wrapper<u32, [u32]> = &w;
    assert!(std::mem::size_of_val(w) == 12);
    assert!(std::mem::align_of_val(w) == 4);
}

/// The tail's end must be rounded up to the alignment. `u32` header +
/// `[u8; 3]` tail -> offset 4, end 4 + 3 = 7, rounded up to align 4 -> size 8.
pub fn size_rounds_up_to_align() {
    let w: Wrapper<u32, [u8; 3]> = Wrapper { header: 7, tail: [1, 2, 3] };
    let w: &Wrapper<u32, [u8]> = &w;
    assert!(std::mem::size_of_val(w) == 8);
    assert!(std::mem::align_of_val(w) == 4);
}

/// An empty tail contributes no bytes; the size is just the header padded to
/// the alignment. `u64` header + empty `[u8]` tail -> size 8, align 8.
pub fn empty_tail() {
    let w: Wrapper<u64, [u8; 0]> = Wrapper { header: 0, tail: [] };
    let w: &Wrapper<u64, [u8]> = &w;
    assert!(std::mem::size_of_val(w) == 8);
    assert!(std::mem::align_of_val(w) == 8);
}

/// `dyn Trait`: size and alignment are read from the vtable, so they match the
/// erased concrete type.
pub fn dyn_size_align() {
    use std::fmt::Debug;

    let small: u32 = 0;
    let small: &dyn Debug = &small;
    assert!(std::mem::size_of_val(small) == 4);
    assert!(std::mem::align_of_val(small) == 4);

    let big: u128 = 0;
    let big: &dyn Debug = &big;
    assert!(std::mem::size_of_val(big) == 16);
    assert!(std::mem::align_of_val(big) == 16);

    #[derive(Debug)]
    #[repr(align(8))]
    struct Aligned {
        a: u64,
        b: u64,
        c: u8,
    }
    let s = Aligned { a: 0, b: 0, c: 0 };
    let s: &dyn Debug = &s;
    assert!(std::mem::size_of_val(s) == 24);
    assert!(std::mem::align_of_val(s) == 8);
}

/// Trait upcast `&dyn T -> &dyn Y` (with `T: Y`). The vtable header carries the
/// erased type's size/align, so both survive the upcast to the principal
/// supertrait.
pub fn upcast_dyn_size_align() {
    trait Y {}
    trait T: Y {}
    impl Y for u64 {}
    impl T for u64 {}

    let x: u64 = 0;
    let t: &dyn T = &x;
    let y: &dyn Y = t;
    assert!(std::mem::size_of_val(y) == 8);
    assert!(std::mem::align_of_val(y) == 8);
}

/// Upcast to a non-principal supertrait (`trait T: Y + Z`, `&dyn T -> &dyn Z`).
/// The upcast reads `Z`'s vtable from the `TraitVPtr` slot, and its header
/// describes the concrete type, so size/align are correct.
pub fn upcast_dyn_secondary_supertrait() {
    trait Y {}
    trait Z {}
    trait T: Y + Z {}
    impl Y for u32 {}
    impl Z for u32 {}
    impl T for u32 {}

    let x: u32 = 0;
    let t: &dyn T = &x;
    let z: &dyn Z = t;
    assert!(std::mem::size_of_val(z) == 4);
    assert!(std::mem::align_of_val(z) == 4);
}

/// A slice whose length exceeds `isize::MAX` bytes must be rejected. The
/// length is chosen so `len * size_of::<u8>()` does not overflow `usize` (so a
/// plain `checked_mul` would accept it) but still breaks the `isize::MAX`
/// object-size requirement.
pub fn slice_size_exceeds_isize_max() -> usize {
    let elem = 0u8;
    let len = (isize::MAX as usize) + 1;
    let raw: *const [u8] = std::ptr::slice_from_raw_parts(&elem, len);
    // SAFETY (for the test): `size_of_val` only reads the pointer metadata, not
    // the elements, so the oversize length is what is under test.
    let slice: &[u8] = unsafe { &*raw };
    std::mem::size_of_val(slice)
}

/// The tail is itself an unsized-tail struct, so the layout is computed by
/// recursing through both ADTs down to the slice.
///
/// Inner `Wrapper<u64, [u8; 3]>`: end 8 + 3 = 11 -> size 16, align 8.
/// Outer `Wrapper<u32, _>`: the align-8 inner sits at offset 8 (past the padded
/// `u32`), end 8 + 16 = 24 -> size 24, align 8.
pub fn nested_unsized_tail() {
    let w: Wrapper<u32, Wrapper<u64, [u8; 3]>> = Wrapper {
        header: 1,
        tail: Wrapper {
            header: 2,
            tail: [1, 2, 3],
        },
    };
    let w: &Wrapper<u32, Wrapper<u64, [u8]>> = &w;
    assert!(std::mem::size_of_val(w) == 24);
    assert!(std::mem::align_of_val(w) == 8);
}
