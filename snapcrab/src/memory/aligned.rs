//! Aligned, zeroed backing buffer for memory segments.
//!
//! Memory segments model the interpreted program's memory using real process
//! allocations, and the interpreter enforces the program's alignment rules on
//! those addresses. A `Box<[u8]>` is only guaranteed 1-byte aligned, so using
//! one as backing storage would leave alignment up to whatever the system
//! allocator happens to return. `AlignedBuf` instead allocates through an
//! explicit `Layout`, so the base address honours the requested alignment by
//! construction.

use anyhow::{Result, anyhow};
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
    /// Errors if `size`/`align` do not form a valid layout (`align` not a
    /// non-zero power of two, or the size rounded up to `align` overflowing
    /// `isize`). These values may originate from the interpreted program (heap
    /// allocation), so an invalid request is an error rather than a bug.
    pub(super) fn zeroed(size: usize, align: usize) -> Result<Self> {
        let layout = Layout::from_size_align(size, align)
            .map_err(|e| anyhow!("invalid layout (size {size}, align {align}): {e}"))?;
        let ptr = if size == 0 {
            // No allocation for a zero-sized buffer; an aligned, non-null
            // dangling pointer is valid for zero-length slices.
            NonNull::new(align as *mut u8).expect("alignment is non-zero")
        } else {
            // SAFETY: `size` is non-zero, so the layout is valid to allocate.
            let raw = unsafe { alloc::alloc_zeroed(layout) };
            NonNull::new(raw).unwrap_or_else(|| alloc::handle_alloc_error(layout))
        };
        Ok(Self { ptr, layout })
    }

    /// The buffer's base address as a const pointer.
    pub(super) fn as_ptr(&self) -> *const u8 {
        self.ptr.as_ptr()
    }

    /// The buffer's base address as a mutable pointer.
    ///
    /// Takes `&self` because the interpreter writes to buffers through raw
    /// addresses rather than through this handle; the sanitizer, not the borrow
    /// checker, is what keeps those writes in bounds.
    pub(super) fn as_mut_ptr(&self) -> *mut u8 {
        self.ptr.as_ptr()
    }

    /// The buffer's contents as a byte slice spanning its full layout.
    pub(super) fn as_bytes(&self) -> &[u8] {
        // SAFETY: `ptr` is non-null, initialized and valid for `layout.size()` bytes.
        // Alignment of `[u8]` is 1.
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.layout.size()) }
    }

    /// Retrieve the memory layout of the buffer.
    pub(super) fn layout(&self) -> Layout {
        self.layout
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
            let buf = AlignedBuf::zeroed(align * 3 + 1, align).unwrap();
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
        let buf = AlignedBuf::zeroed(64, 16).unwrap();
        assert!(buf.iter().all(|&b| b == 0));
    }

    #[test]
    fn reads_back_written_bytes() {
        let mut buf = AlignedBuf::zeroed(4, 4).unwrap();
        buf.copy_from_slice(&[1, 2, 3, 4]);
        assert_eq!(&*buf, &[1, 2, 3, 4]);
    }

    #[test]
    fn invalid_layout_errors() {
        // Non-power-of-two alignment and an overflowing size are rejected
        // rather than panicking.
        assert!(AlignedBuf::zeroed(8, 3).is_err());
        assert!(AlignedBuf::zeroed(isize::MAX as usize + 1, 1).is_err());
    }

    #[test]
    fn zero_sized_is_aligned_and_empty() {
        let buf = AlignedBuf::zeroed(0, 16).unwrap();
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.as_ptr() as usize % 16, 0);
    }
}
