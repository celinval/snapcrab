//! Allocator misuse that the heap model is expected to reject.
//!
//! These call the raw `std::alloc` API directly so the bad `size`/`align`
//! values reach the `__rust_alloc`/`__rust_realloc` shims unmodified. Each
//! function violates the `GlobalAlloc` contract, so the interpreter should
//! report an error rather than run to completion.

#![allow(unused)]

use std::alloc::{Layout, alloc, dealloc, realloc};

/// `realloc` that overstates the old block's size.
///
/// The shrink makes the copy length `min(old_size, new_size) == 4`, which fits
/// inside the real 8-byte block, so the bogus `old_size` is invisible unless
/// `realloc` validates it against the recorded layout.
pub fn realloc_old_size_too_large() {
    unsafe {
        let ptr = alloc(Layout::from_size_align(8, 8).unwrap());
        let new_ptr = realloc(ptr, Layout::from_size_align_unchecked(4096, 8), 4);
        dealloc(new_ptr, Layout::from_size_align(4, 8).unwrap());
    }
}

/// `realloc` that passes an alignment the block was not allocated with.
///
/// Rust requires the old layout's alignment to be reused, so the new block
/// silently changing alignment must be reported.
pub fn realloc_align_mismatch() {
    unsafe {
        let ptr = alloc(Layout::from_size_align(8, 8).unwrap());
        let new_ptr = realloc(ptr, Layout::from_size_align_unchecked(8, 16), 16);
        dealloc(new_ptr, Layout::from_size_align(16, 16).unwrap());
    }
}

/// Two zero-sized allocations with the same alignment.
///
/// `GlobalAlloc::alloc` requires a non-zero size, so the first call is already
/// invalid and should be reported there. Backing zero-sized requests with a
/// dangling `align`-valued address instead makes both calls return the same
/// pointer, and the error only surfaces later as a bogus double free.
pub fn alloc_zero_sized() {
    unsafe {
        let layout = Layout::from_size_align(0, 8).unwrap();
        let first = alloc(layout);
        let second = alloc(layout);
        dealloc(first, layout);
        dealloc(second, layout);
    }
}
