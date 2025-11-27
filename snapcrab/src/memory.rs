//! This module contains APIs for modeling memory
//!
//! TODO: Expand on how the memory is organized.
//! - One ThreadMemory per thread. Obviously.
//! - Each thread memory contains their own stack
//! - Thread memories share heap and statics
//! - Read and write to all memory segments are validated to avoid access out of
//!   bounds.

pub mod heap;
mod sanitizer;
mod stack;
mod statics;

use crate::ty::MonoType;
use crate::value::Value;
use anyhow::Result;
use heap::Heap;
use rustc_public::mir::Body;
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

/// Thread-local memory representation.
///
/// This is the main structure exported to the rest of the interpreter.
#[derive(Default)]
pub struct ThreadMemory {
    stack: Stack,
    #[allow(unused)]
    heap: Heap,
    #[allow(unused)]
    statics: Statics,
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

    /// Read local variable.
    /// TODO: Use rustc_public Local here and in subsequent calls
    #[inline]
    pub fn read_local(&self, local: usize) -> Result<Value> {
        self.stack.read_local(local)
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
            Ok(data) => return Ok(Value::from_bytes(data)),
            Err(MemoryAccessError::OutOfBounds) => {
                anyhow::bail!(
                    "Stack memory access out of bounds at address 0x{:x}",
                    address
                )
            }
            Err(MemoryAccessError::NotFound) => {} // Continue to next segment
        }

        // Try heap
        match self.heap.read_addr(address, size) {
            Ok(data) => return Ok(Value::from_bytes(data)),
            Err(MemoryAccessError::OutOfBounds) => {
                anyhow::bail!(
                    "Heap memory access out of bounds at address 0x{:x}",
                    address
                )
            }
            Err(MemoryAccessError::NotFound) => {} // Continue to next segment
        }

        // Try statics
        match self.statics.read_addr(address, size) {
            Ok(data) => Ok(Value::from_bytes(data)),
            Err(MemoryAccessError::OutOfBounds) => {
                anyhow::bail!(
                    "Static memory access out of bounds at address 0x{:x}",
                    address
                )
            }
            Err(MemoryAccessError::NotFound) => {
                anyhow::bail!("Address 0x{:x} not found in any memory segment", address)
            }
        }
    }

    pub fn write_addr(&mut self, address: usize, data: &[u8], ty: Ty) -> Result<()> {
        let size = ty.size()?;
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
        }

        // Try statics
        match self.statics.write_addr(address, data) {
            Ok(()) => Ok(()),
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
}

/// The type of errors that can be encountered during a memory access.
#[derive(Debug)]
enum MemoryAccessError {
    /// The base address is in the memory segment, but request is out-of-bounds.
    OutOfBounds,
    /// The base address is not in this memory segment.
    NotFound,
}

impl std::fmt::Display for MemoryAccessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryAccessError::OutOfBounds => write!(f, "Memory access out of bounds"),
            MemoryAccessError::NotFound => write!(f, "Address not found in memory segment"),
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
    /// Reads data from a memory address.
    ///
    /// # Arguments
    /// * `address` - The memory address to read from
    /// * `size` - Number of bytes to read
    ///
    /// # Returns
    /// * `Ok(&[u8])` - Reference to the memory data if the read is valid
    /// * `Err` - Error found when trying to satisfy the request
    fn read_addr(&self, address: usize, size: usize) -> Result<&[u8], MemoryAccessError>;

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
