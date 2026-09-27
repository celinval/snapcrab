//! `copy_nonoverlapping` (the `CopyNonOverlapping` statement) and `copy` (the
//! overlapping `core::intrinsics::copy`).
//!
//! Comparisons use indexing rather than `==` on arrays/slices, which would pull
//! in `ptr::offset` (not yet supported).

#![allow(unused)]

/// `copy_nonoverlapping`: copy every element from one array into another.
pub fn copy_nonoverlapping_basic() {
    let src = [1u32, 2, 3, 4];
    let mut dst = [0u32; 4];
    unsafe {
        std::ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr(), src.len());
    }
    assert!(dst[0] == 1 && dst[1] == 2 && dst[2] == 3 && dst[3] == 4);
}

/// `copy_nonoverlapping` with a padded element type exercises copying the
/// element's full byte width (including padding).
pub fn copy_nonoverlapping_structs() {
    #[derive(Clone, Copy, PartialEq)]
    struct P {
        x: u32,
        y: u64,
    }
    let src = [P { x: 1, y: 2 }, P { x: 3, y: 4 }];
    let mut dst = [P { x: 0, y: 0 }; 2];
    unsafe {
        std::ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr(), 2);
    }
    assert!(dst[0] == src[0] && dst[1] == src[1]);
}

/// Overlapping `copy` where `dst > src` reads the source before writing.
///
/// A correct implementation must read all source bytes before writing,
/// otherwise the writes clobber not-yet-read source elements. The two pointers
/// are formed from constant indices (place projections) to avoid `ptr::offset`.
pub fn copy_overlapping() {
    let mut buf = [1u8, 2, 3, 4, 5];
    let src = &raw const buf[0];
    let dst = &raw mut buf[1];
    unsafe {
        std::ptr::copy(src, dst, 4);
    }
    assert!(buf[0] == 1 && buf[1] == 1 && buf[2] == 2 && buf[3] == 3 && buf[4] == 4);
}
