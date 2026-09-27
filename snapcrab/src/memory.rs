//! This module contains APIs for modeling memory
//!
//! TODO: Expand on how the memory is organized.
//! - One ThreadMemory per thread. Obviously.
//! - Each thread memory contains their own stack
//! - Thread memories share heap and statics
//! - Read and write to all memory segments are validated to avoid access out of
//!   bounds.

mod aligned;
pub mod heap;
mod sanitizer;
mod stack;
mod statics;

use crate::interpreter::check::CheckConfig;
use crate::interpreter::native::jit::JitEngine;
use crate::ty::MonoType;
use crate::value::Value;
use anyhow::Result;
use heap::Heap;
use rustc_public::mir::Body;
use rustc_public::mir::alloc::AllocId;
use rustc_public::mir::mono::Instance;
use rustc_public::target::MachineInfo;
use rustc_public::ty::Ty;
use stack::Stack;
use statics::Statics;
use std::sync::LazyLock;

static MACHINE_INFO: LazyLock<MachineInfo> = LazyLock::new(MachineInfo::target);

/// Returns the pointer width in bytes for the target machine.
pub fn pointer_width() -> usize {
    MACHINE_INFO.pointer_width.bytes()
}

/// Returns the target machine info.
pub fn machine_info() -> &'static MachineInfo {
    &MACHINE_INFO
}

/// Thread-local memory representation.
///
/// This is the main structure exported to the rest of the interpreter.
pub struct ThreadMemory {
    stack: Stack,
    heap: Heap,
    statics: Statics,
    /// Configuration for which UB checks to perform.
    pub check_config: CheckConfig,
    /// JIT engine for native function calls.
    pub jit: JitEngine,
}

impl Default for ThreadMemory {
    fn default() -> Self {
        Self {
            stack: Stack::default(),
            heap: Heap::default(),
            statics: Statics::default(),
            check_config: CheckConfig::default(),
            jit: JitEngine::new().expect("Failed to initialize JIT engine"),
        }
    }
}

impl ThreadMemory {
    /// Create a new ThreadMemory from scratch.
    ///
    /// This should only be called for the interpreter entry point.
    pub fn new() -> Self {
        ThreadMemory::default()
    }

    /// Runs a method with their own stack frame.
    pub fn with_stack_frame<F, R>(&mut self, instance: Instance, func: F) -> R
    where
        F: FnOnce(&Body, &mut Self) -> R,
    {
        Stack::with_stack_frame(instance, self, func)
    }

    /// Resolve a compiler AllocId to a real memory address.
    pub fn resolve_alloc(&self, alloc_id: AllocId) -> Result<usize> {
        self.statics.resolve_alloc(alloc_id)
    }

    /// Resolve a reified function-pointer address back to its `Instance`.
    pub fn resolve_fn(&self, address: usize) -> Result<Instance> {
        self.statics.resolve_fn(address)
    }

    /// Reify a function to its stable synthetic address.
    pub fn reify_fn(&self, instance: Instance) -> usize {
        self.statics.reify_fn(instance)
    }

    /// Service a `__rust_alloc`/`__rust_alloc_zeroed` request, returning the
    /// base address of a fresh, correctly-aligned heap block.
    pub fn heap_allocate(&self, size: usize, align: usize) -> Result<usize> {
        self.heap.allocate(size, align)
    }

    /// Service a `__rust_dealloc` request for a live heap allocation.
    pub fn heap_deallocate(&self, address: usize, size: usize, align: usize) -> Result<()> {
        self.heap.deallocate(address, size, align)
    }

    /// Service a `__rust_realloc` request, returning the new base address.
    pub fn heap_reallocate(
        &self,
        address: usize,
        old_size: usize,
        align: usize,
        new_size: usize,
    ) -> Result<usize> {
        self.heap.reallocate(address, old_size, align, new_size)
    }

    /// Read local variable.
    #[inline]
    #[allow(dead_code)]
    pub fn read_local(&self, local: usize, local_ty: Ty) -> Result<Value> {
        self.stack.read_local(local, local_ty)
    }

    #[inline]
    pub fn write_local(&mut self, local: usize, value: Value) -> Result<()> {
        self.stack.write_local(local, value)
    }

    #[inline]
    pub fn local_address(&self, local: usize) -> Result<usize> {
        self.stack.local_address(local)
    }

    pub fn read_addr(&self, address: usize, ty: Ty) -> Result<Value> {
        let size = ty.size()?;
        if size == 0 {
            return Ok(Value::unit().clone());
        }
        let alignment = ty.alignment()?;

        // Check alignment
        if !address.is_multiple_of(alignment) {
            anyhow::bail!(
                "Misaligned memory access: address 0x{:x} is not aligned to {} bytes",
                address,
                alignment
            );
        }

        // Try stack first
        match self.stack.read_addr(address, size) {
            Ok(value) => return Ok(value),
            Err(MemoryAccessError::OutOfBounds) => {
                anyhow::bail!(
                    "Stack memory access out of bounds at address 0x{:x}",
                    address
                )
            }
            Err(MemoryAccessError::NotFound) => {} // Continue to next segment
            Err(MemoryAccessError::ReadOnly) => unreachable!("reads never hit read-only memory"),
        }

        // Try heap
        match self.heap.read_addr(address, size) {
            Ok(value) => return Ok(value),
            Err(MemoryAccessError::OutOfBounds) => {
                anyhow::bail!(
                    "Heap memory access out of bounds at address 0x{:x}",
                    address
                )
            }
            Err(MemoryAccessError::NotFound) => {} // Continue to next segment
            Err(MemoryAccessError::ReadOnly) => unreachable!("reads never hit read-only memory"),
        }

        // Try statics
        match self.statics.read_addr(address, size) {
            Ok(value) => Ok(value),
            Err(MemoryAccessError::OutOfBounds) => {
                anyhow::bail!(
                    "Static memory access out of bounds at address 0x{:x}",
                    address
                )
            }
            Err(MemoryAccessError::NotFound) => {
                anyhow::bail!("Address 0x{:x} not found in any memory segment", address)
            }
            Err(MemoryAccessError::ReadOnly) => unreachable!("reads never hit read-only memory"),
        }
    }

    pub fn write_addr(&mut self, address: usize, data: &[u8], ty: Ty) -> Result<()> {
        let size = ty.size()?;
        if size == 0 {
            return Ok(());
        }
        let alignment = ty.alignment()?;

        // Check alignment
        if !address.is_multiple_of(alignment) {
            anyhow::bail!(
                "Misaligned memory access: address 0x{:x} is not aligned to {} bytes",
                address,
                alignment
            );
        }

        // Check data size matches type size
        if data.len() != size {
            anyhow::bail!(
                "Data size mismatch: expected {} bytes, got {}",
                size,
                data.len()
            );
        }

        // Try stack first
        match self.stack.write_addr(address, data) {
            Ok(()) => return Ok(()),
            Err(MemoryAccessError::OutOfBounds) => {
                anyhow::bail!(
                    "Stack memory access out of bounds at address 0x{:x}",
                    address
                )
            }
            Err(MemoryAccessError::NotFound) => {} // Continue to next segment
            Err(MemoryAccessError::ReadOnly) => unreachable!("the stack is always writable"),
        }

        // Try heap
        match self.heap.write_addr(address, data) {
            Ok(()) => return Ok(()),
            Err(MemoryAccessError::OutOfBounds) => {
                anyhow::bail!(
                    "Heap memory access out of bounds at address 0x{:x}",
                    address
                )
            }
            Err(MemoryAccessError::NotFound) => {} // Continue to next segment
            Err(MemoryAccessError::ReadOnly) => unreachable!("the heap is always writable"),
        }

        // Try statics
        match self.statics.write_addr(address, data) {
            Ok(()) => Ok(()),
            Err(MemoryAccessError::ReadOnly) => {
                anyhow::bail!(
                    "Cannot write to read-only memory at address 0x{:x}",
                    address
                )
            }
            Err(MemoryAccessError::OutOfBounds) => {
                anyhow::bail!(
                    "Static memory access out of bounds at address 0x{:x}",
                    address
                )
            }
            Err(MemoryAccessError::NotFound) => {
                // No more segments to try
                anyhow::bail!("Address 0x{:x} not found in any memory segment", address)
            }
        }
    }

    /// Copy `size` bytes from `src` to `dst`, allowing the ranges to overlap.
    ///
    /// Has `ptr::copy` semantics; both ranges are bounds-checked first.
    pub fn copy(&mut self, src: usize, dst: usize, size: usize) -> Result<()> {
        self.check_readable(src, size)?;
        self.check_writable(dst, size)?;
        // SAFETY: both ranges lie within live allocations (checked above), and
        // the interpreter's memory is backed by real, correctly-sized buffers.
        unsafe { std::ptr::copy(src as *const u8, dst as *mut u8, size) };
        Ok(())
    }

    /// Copy `size` bytes from `src` to `dst`
    ///
    /// The `src` and `dst` must not overlap (overlap is UB).
    pub fn copy_nonoverlapping(&mut self, src: usize, dst: usize, size: usize) -> Result<()> {
        self.check_readable(src, size)?;
        self.check_writable(dst, size)?;
        if src < dst.saturating_add(size) && dst < src.saturating_add(size) {
            anyhow::bail!("`copy_nonoverlapping` called with overlapping ranges, which is UB");
        }
        // SAFETY: both ranges lie within live, non-overlapping allocations.
        unsafe { std::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, size) };
        Ok(())
    }

    /// Bail unless `[address, address + size)` is a readable allocation.
    fn check_readable(&self, address: usize, size: usize) -> Result<()> {
        if self.stack.contains(address, size)
            || self.heap.contains(address, size)
            || self.statics.contains(address, size)
        {
            Ok(())
        } else {
            anyhow::bail!("copy source [0x{address:x}, +{size}) is not a readable allocation")
        }
    }

    /// Bail unless `[address, address + size)` is a writable allocation.
    ///
    /// Read-only statics (`const`/immutable `static`) are excluded; only
    /// `static mut` and interior-mutable statics are writable.
    fn check_writable(&self, address: usize, size: usize) -> Result<()> {
        if self.stack.contains(address, size)
            || self.heap.contains(address, size)
            || self.statics.contains_writable(address, size)
        {
            Ok(())
        } else {
            anyhow::bail!("copy destination [0x{address:x}, +{size}) is not a writable allocation")
        }
    }
}

/// The type of errors that can be encountered during a memory access.
#[derive(Debug)]
enum MemoryAccessError {
    /// The base address is in the memory segment, but request is out-of-bounds.
    OutOfBounds,
    /// The base address is not in this memory segment.
    NotFound,
    /// The range is in read-only memory (a `const`/immutable `static`), so it
    /// cannot be written.
    ReadOnly,
}

impl std::fmt::Display for MemoryAccessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryAccessError::OutOfBounds => write!(f, "Memory access out of bounds"),
            MemoryAccessError::NotFound => write!(f, "Address not found in memory segment"),
            MemoryAccessError::ReadOnly => write!(f, "Write to read-only memory"),
        }
    }
}

impl std::error::Error for MemoryAccessError {}

/// Unsafe trait for memory segments that can perform safe memory operations.
///
/// # Safety
///
/// Implementors must guarantee:
/// - The safety of all memory operations performed
/// - Ownership or valid access rights to all memory accessed
/// - That returned memory references remain valid for their lifetime
/// - That concurrent access is properly synchronized if needed
unsafe trait MemorySegment {
    /// Reads data from a memory address into an owned value.
    ///
    /// Returns an owned `Value` (not a borrow) so the bytes are copied out
    /// while the segment's backing allocation is guaranteed live: a borrowed
    /// slice could dangle once the allocation is freed (e.g. heap dealloc).
    ///
    /// # Arguments
    /// * `address` - The memory address to read from
    /// * `size` - Number of bytes to read
    ///
    /// # Returns
    /// * `Ok(Value)` - Copy of the memory data if the read is valid
    /// * `Err` - Error found when trying to satisfy the request
    fn read_addr(&self, address: usize, size: usize) -> Result<Value, MemoryAccessError>;

    /// Writes data to a memory address.
    ///
    /// # Arguments
    /// * `address` - The memory address to write to
    /// * `data` - The data to write
    ///
    /// # Returns
    /// * `Ok(())` - Write was successful
    /// * `Err` - Error found when trying to satisfy the request
    fn write_addr(&self, address: usize, data: &[u8]) -> Result<(), MemoryAccessError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Return a new `ThreadMemory` with two distinct heap allocations
    ///
    /// Return their base addresses.
    fn two_allocs(src_len: usize, dst_len: usize) -> (ThreadMemory, usize, usize) {
        let mem = ThreadMemory::new();
        let src = mem.heap_allocate(src_len, 1).unwrap();
        let dst = mem.heap_allocate(dst_len, 1).unwrap();
        (mem, src, dst)
    }

    /// A copy whose source range is out of bounds is rejected.
    ///
    /// `size` both exceeds the source allocation and would overflow the source
    /// pointer (`src + size` wraps `usize`), so the read-range check must reject
    /// it cleanly rather than panic or wrap.
    #[test]
    fn copy_source_range_out_of_bounds_is_rejected() {
        let (mut mem, src, dst) = two_allocs(8, 64);
        let size = usize::MAX - src + 1;
        assert!(mem.copy(src, dst, size).is_err());
        assert!(mem.copy_nonoverlapping(src, dst, size).is_err());
    }

    /// A copy whose destination range is out of bounds is rejected.
    ///
    /// The source is large enough that the read check passes, so the too-small
    /// destination must be caught by the write-range check specifically.
    #[test]
    fn copy_destination_range_out_of_bounds_is_rejected() {
        let (mut mem, src, dst) = two_allocs(64, 8);
        assert!(mem.copy(src, dst, 16).is_err());
        assert!(mem.copy_nonoverlapping(src, dst, 16).is_err());
    }

    #[test]
    fn copy_nonoverlapping_rejects_overlap_but_copy_allows_it() {
        let mut mem = ThreadMemory::new();
        let base = mem.heap_allocate(16, 1).unwrap();
        assert!(mem.copy_nonoverlapping(base, base + 4, 8).is_err());
        assert!(mem.copy(base, base + 4, 8).is_ok());
    }

    #[test]
    fn copy_within_bounds_succeeds() {
        let (mut mem, src, dst) = two_allocs(16, 16);
        assert!(mem.copy(src, dst, 16).is_ok());
        assert!(mem.copy_nonoverlapping(src, dst, 16).is_ok());
    }
}
