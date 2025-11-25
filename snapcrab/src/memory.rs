//! This module contains APIs for modeling memory

mod sanitizer;
pub mod stack;

use crate::ty::MonoType;
use crate::value::Value;
use anyhow::Result;
use rustc_public::ty::Ty;
use sanitizer::global_memory_tracker;
use std::{ptr, slice};

/// Reads data from a memory address with bounds checking
pub fn read_addr(address: usize, ty: Ty) -> Result<Value> {
    let size = ty.size()?;
    let tracker = global_memory_tracker().lock().unwrap();
    if !tracker.contains(address, size) {
        anyhow::bail!("Invalid memory read at 0x{:x}", address);
    }

    // SAFETY: Memory tracker verified this address range is valid
    // and memory tracker is locked preventing competing operations
    unsafe {
        let slice = slice::from_raw_parts(address as *const u8, size);
        Ok(Value::from_bytes(slice))
    }
}

/// Writes data to a memory address with bounds checking
pub fn write_addr(address: usize, data: &[u8], ty: Ty) -> Result<()> {
    let size = ty.size()?;
    if data.len() != size {
        anyhow::bail!("Data size mismatch: expected {}, got {}", size, data.len());
    }

    let tracker = global_memory_tracker().lock().unwrap();
    if !tracker.contains(address, size) {
        anyhow::bail!("Invalid memory write at 0x{:x}", address);
    }

    // SAFETY: Memory tracker verified this address range is valid
    // and memory tracker is locked preventing competing operations
    unsafe {
        ptr::copy(data.as_ptr(), address as *mut u8, size);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sanitizer::MemorySanitizer;

    #[test]
    fn test_allocate_and_contains() {
        let mut tracker = MemorySanitizer::new();

        // Allocate memory at address 100, size 50
        tracker.allocate(100, 50).unwrap();

        // Check various ranges
        assert!(tracker.contains(100, 1)); // Start of allocation
        assert!(tracker.contains(125, 10)); // Middle of allocation
        assert!(tracker.contains(149, 1)); // End of allocation
        assert!(!tracker.contains(99, 1)); // Before allocation
        assert!(!tracker.contains(150, 1)); // After allocation
        assert!(!tracker.contains(100, 51)); // Extends beyond allocation
    }

    #[test]
    fn test_overlapping_allocations() {
        let mut tracker = MemorySanitizer::new();

        tracker.allocate(100, 50).unwrap();

        // These should fail due to overlap
        assert!(tracker.allocate(90, 20).is_err()); // Overlaps at start
        assert!(tracker.allocate(140, 20).is_err()); // Overlaps at end
        assert!(tracker.allocate(110, 10).is_err()); // Contained within
        assert!(tracker.allocate(80, 100).is_err()); // Contains existing

        // This should succeed (no overlap)
        assert!(tracker.allocate(200, 50).is_ok());
    }

    #[test]
    fn test_deallocate() {
        let mut tracker = MemorySanitizer::new();

        tracker.allocate(100, 50).unwrap();
        assert!(tracker.contains(125, 10));

        tracker.deallocate(100).unwrap();
        assert!(!tracker.contains(125, 10));

        // Deallocating again should fail
        assert!(tracker.deallocate(100).is_err());
    }
}
