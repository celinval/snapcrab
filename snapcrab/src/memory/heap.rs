//! Heap memory model.
//!
//! Services the interpreted program's allocator calls (`__rust_alloc` and
//! friends) with real, correctly-aligned process memory. Each allocation is
//! backed by an [`AlignedBuf`] kept alive in a map keyed by its base address;
//! the [`MemorySanitizer`] validates that accesses fall within a live block,
//! which also surfaces use-after-free and invalid-free as errors.
//!
//! The backing store is zero-initialized. `__rust_alloc` is technically
//! allowed to return uninitialized memory, but zeroing is a sound, more
//! deterministic choice and avoids exposing host garbage to the interpreter.

use crate::memory::aligned::AlignedBuf;
use crate::memory::sanitizer::MemorySanitizer;
use crate::memory::{MemoryAccessError, MemorySegment};
use crate::value::Value;
use anyhow::{Result, bail};
use std::alloc::Layout;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Thread-safe heap shared across a process's threads.
#[derive(Clone, Default)]
pub struct Heap(Arc<RwLock<HeapImpl>>);

/// Heap state: the bounds sanitizer and the live allocations.
#[derive(Default)]
struct HeapImpl {
    sanitizer: MemorySanitizer,
    /// Live allocations keyed by base address. Owning the `AlignedBuf` keeps
    /// the backing memory alive until `deallocate`/`reallocate` removes it.
    allocations: HashMap<usize, AlignedBuf>,
}

impl Heap {
    /// Allocate `size` bytes aligned to `align`, returning the base address.
    pub fn allocate(&self, size: usize, align: usize) -> Result<usize> {
        // A zero-sized layout is UB to pass to the allocator; callers use a
        // dangling pointer for ZSTs instead of reaching `__rust_alloc`.
        if size == 0 {
            bail!("invalid allocation: a zero-sized layout is undefined behavior");
        }
        let buf = AlignedBuf::zeroed(size, align)?;
        let addr = buf.as_ptr() as usize;

        let mut inner = self.0.write().unwrap();
        inner.sanitizer.register_alloc(buf.as_bytes());
        inner.allocations.insert(addr, buf);
        Ok(addr)
    }

    /// Free the block at `address`, which must be a live allocation base.
    pub fn deallocate(&self, address: usize, size: usize, align: usize) -> Result<()> {
        let mut inner = self.0.write().unwrap();
        let Some(buf) = inner.allocations.remove(&address) else {
            bail!("invalid deallocation: no heap allocation at 0x{address:x}");
        };
        inner.sanitizer.deregister_alloc(buf.as_bytes());
        drop(inner);

        match Layout::from_size_align(size, align) {
            Ok(layout) if layout != buf.layout() => {
                bail!(
                    "invalid deallocation: expected layout {:?}, but received {size} bytes aligned to {align}",
                    buf.layout()
                );
            }
            Ok(_) => Ok(()),
            Err(e) => {
                bail!("invalid deallocation: {e}");
            }
        }
    }

    /// Grow or shrink the block at `address`, preserving its contents.
    ///
    /// Allocates a fresh block, copies the overlapping prefix, and frees the
    /// old one, returning the new base address.
    pub fn reallocate(
        &self,
        address: usize,
        old_size: usize,
        align: usize,
        new_size: usize,
    ) -> Result<usize> {
        // `realloc` to a zero-sized layout is UB, same as `alloc`.
        if new_size == 0 {
            bail!("invalid realloc: a zero-sized layout is undefined behavior");
        }

        let new_buf = AlignedBuf::zeroed(new_size, align)?;
        let new_ptr = new_buf.as_mut_ptr();

        let mut inner = self.0.write().unwrap();
        let Some(old_buf) = inner.allocations.remove(&address) else {
            bail!("invalid realloc: no heap allocation at 0x{address:x}");
        };
        inner.sanitizer.deregister_alloc(old_buf.as_bytes());
        if old_size != old_buf.len() {
            bail!(
                "invalid realloc: old size `{old_size}` does not match the allocation size `{}`",
                old_buf.len()
            );
        }
        if align != old_buf.layout().align() {
            bail!(
                "invalid realloc: align `{align}` does not match the allocation align `{}`",
                old_buf.layout().align()
            );
        }

        let copy_len = old_size.min(new_size);
        assert!(copy_len > 0, "new and old sizes must be positive");
        // SAFETY: the old block is still live and `copy_len <= old_size ==
        // old_buf.len()`, so it is within bounds of both distinct buffers.
        unsafe {
            std::ptr::copy_nonoverlapping(old_buf.as_ptr(), new_ptr, copy_len);
        }

        // `old_buf` drops at the end of this function, freeing the old backing memory.
        inner.sanitizer.register_alloc(new_buf.as_bytes());
        inner.allocations.insert(new_ptr as usize, new_buf);
        Ok(new_ptr as usize)
    }
}

// SAFETY: Backing allocations live in `AlignedBuf`s whose addresses are stable
// until freed. The sanitizer tracks their ranges and validates every access.
unsafe impl MemorySegment for Heap {
    fn read_addr(&self, address: usize, size: usize) -> Result<Value, MemoryAccessError> {
        let inner = self.0.read().unwrap();
        inner.sanitizer.check_access(address, size)?;
        // SAFETY: sanitizer confirmed the range is within a live allocation.
        // The bytes are copied into an owned `Value` while the read lock (and
        // thus the allocation) is held, so the result cannot dangle if the
        // block is later freed.
        let slice = unsafe { std::slice::from_raw_parts(address as *const u8, size) };
        Ok(Value::from_bytes(slice))
    }

    fn write_addr(&self, address: usize, data: &[u8]) -> Result<(), MemoryAccessError> {
        let inner = self.0.read().unwrap();
        inner.sanitizer.check_access(address, data.len())?;
        // SAFETY: sanitizer confirmed the range is within a live allocation.
        unsafe { std::ptr::copy(data.as_ptr(), address as *mut u8, data.len()) };
        Ok(())
    }
}
