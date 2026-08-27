//! Aligned, zeroed backing buffer for memory segments.
//!
//! Memory segments model the interpreted program's memory using real process
//! allocations, and the interpreter enforces the program's alignment rules on
//! those addresses. A `Box<[u8]>` is only guaranteed 1-byte aligned, so using
//! one as backing storage would leave alignment up to whatever the system
//! allocator happens to return. `AlignedBuf` instead allocates through an
//! explicit `Layout`, so the base address honours the requested alignment by
//! construction.

use std::alloc::{self, Layout};
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

/// A heap buffer allocated with an explicit size and alignment.
///
/// The buffer is zero-initialized and freed on drop. Its address is stable for
/// the lifetime of the value. Zero-sized buffers perform no allocation and use
/// an aligned dangling pointer.
#[derive(Debug)]
pub(super) struct AlignedBuf {
    ptr: NonNull<u8>,
    layout: Layout,
}

impl AlignedBuf {
    /// Allocate a zeroed buffer of `size` bytes aligned to `align`.
    ///
    /// `align` must be a non-zero power of two, as guaranteed by type layouts
    /// and compiler allocations.
    pub(super) fn zeroed(size: usize, align: usize) -> Self {
        let layout = Layout::from_size_align(size, align)
            .unwrap_or_else(|e| panic!("invalid layout (size {size}, align {align}): {e}"));
        let ptr = if size == 0 {
            // No allocation for a zero-sized buffer; an aligned, non-null
            // dangling pointer is valid for zero-length slices.
            NonNull::new(align as *mut u8).expect("alignment is non-zero")
        } else {
            // SAFETY: `size` is non-zero, so the layout is valid to allocate.
            let raw = unsafe { alloc::alloc_zeroed(layout) };
            NonNull::new(raw).unwrap_or_else(|| alloc::handle_alloc_error(layout))
        };
        Self { ptr, layout }
    }

    /// The buffer's base address as a const pointer.
    pub(super) fn as_ptr(&self) -> *const u8 {
        self.ptr.as_ptr()
    }
}

impl Deref for AlignedBuf {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        // SAFETY: `ptr` is non-null, aligned, and valid for `layout.size()`
        // bytes (a dangling but aligned pointer is valid for a zero-length
        // slice).
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.layout.size()) }
    }
}

impl DerefMut for AlignedBuf {
    fn deref_mut(&mut self) -> &mut [u8] {
        // SAFETY: see `deref`; `&mut self` guarantees exclusive access.
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.layout.size()) }
    }
}

impl Drop for AlignedBuf {
    fn drop(&mut self) {
        if self.layout.size() != 0 {
            // SAFETY: allocated with this exact layout via `alloc_zeroed`, and
            // never freed elsewhere.
            unsafe { alloc::dealloc(self.ptr.as_ptr(), self.layout) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AlignedBuf;

    #[test]
    fn honours_requested_alignment() {
        // Includes alignments larger than the system allocator's guarantee, so
        // the buffer's alignment cannot be attributed to incidental over-
        // alignment.
        for align in [1usize, 2, 4, 8, 16, 32, 64, 128, 4096] {
            let buf = AlignedBuf::zeroed(align * 3 + 1, align);
            assert_eq!(
                buf.as_ptr() as usize % align,
                0,
                "address {:p} is not aligned to {align}",
                buf.as_ptr()
            );
        }
    }

    #[test]
    fn is_zero_initialized() {
        let buf = AlignedBuf::zeroed(64, 16);
        assert!(buf.iter().all(|&b| b == 0));
    }

    #[test]
    fn reads_back_written_bytes() {
        let mut buf = AlignedBuf::zeroed(4, 4);
        buf.copy_from_slice(&[1, 2, 3, 4]);
        assert_eq!(&*buf, &[1, 2, 3, 4]);
    }

    #[test]
    fn zero_sized_is_aligned_and_empty() {
        let buf = AlignedBuf::zeroed(0, 16);
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.as_ptr() as usize % 16, 0);
    }
}
