//! Memory allocation sanitizer.
//!
//! Provides a memory tracker that records allocated memory regions and validates
//! memory access bounds. Ensures no overlapping allocations and efficient
//! bounds checking for memory safety.
//!
//! Note: For performance optimization, simple stack read/write operations could
//! potentially bypass the global tracker lock in the future, since stack memory
//! is inherently thread-local and bounds are pre-validated during frame creation.

use anyhow::{Result, bail};
use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

/// Global singleton memory tracker instance
static MEMORY_SANITIZER: OnceLock<Mutex<MemorySanitizer>> = OnceLock::new();

/// Gets the global memory tracker instance
pub fn global_memory_tracker() -> &'static Mutex<MemorySanitizer> {
    MEMORY_SANITIZER.get_or_init(|| Mutex::new(MemorySanitizer::new()))
}

/// Tracks memory allocations and validates memory access bounds.
///
/// Maintains a record of all allocated memory regions with their addresses and sizes.
/// Prevents overlapping allocations and provides efficient bounds checking.
#[derive(Debug, Default)]
pub struct MemorySanitizer {
    /// Map from allocation start address to allocation size
    allocations: BTreeMap<usize, usize>,
}

impl MemorySanitizer {
    /// Creates a new empty memory tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a new memory allocation.
    ///
    /// # Arguments
    /// * `address` - Starting address of the allocation
    /// * `size` - Size of the allocation in bytes
    ///
    /// # Returns
    /// * `Ok(())` - Allocation recorded successfully
    /// * `Err(anyhow::Error)` - If allocation overlaps with existing memory
    pub fn allocate(&mut self, address: usize, size: usize) -> Result<()> {
        if self.has_overlap(address, size) {
            bail!(
                "Allocation at 0x{:x} (size {}) overlaps with existing memory",
                address,
                size
            );
        }
        self.allocations.insert(address, size);
        Ok(())
    }

    /// Removes a memory allocation record.
    ///
    /// # Arguments
    /// * `address` - Starting address of the allocation to remove
    ///
    /// # Returns
    /// * `Ok(())` - Allocation removed successfully
    /// * `Err(anyhow::Error)` - If no allocation exists at the given address
    pub fn deallocate(&mut self, address: usize) -> Result<()> {
        if self.allocations.remove(&address).is_none() {
            bail!("No allocation found at address 0x{:x}", address);
        }
        Ok(())
    }

    /// Checks if a memory range is entirely contained within a single allocation.
    ///
    /// # Arguments
    /// * `address` - Starting address of the range to check
    /// * `size` - Size of the range in bytes
    ///
    /// # Returns
    /// * `true` - If the entire range is within a single allocation
    /// * `false` - If any part of the range is outside allocated memory
    pub fn contains(&self, address: usize, size: usize) -> bool {
        if let Some((&start, &alloc_size)) = self.allocations.range(..=address).next_back() {
            let alloc_end = start + alloc_size;
            let request_end = address + size;
            address >= start && request_end <= alloc_end
        } else {
            false
        }
    }

    /// Checks if a proposed allocation would overlap with existing allocations.
    ///
    /// # Arguments
    /// * `address` - Starting address of the proposed allocation
    /// * `size` - Size of the proposed allocation in bytes
    ///
    /// # Returns
    /// * `true` - If the allocation would overlap with existing memory
    /// * `false` - If the allocation would not overlap
    fn has_overlap(&self, address: usize, size: usize) -> bool {
        let end = address + size;

        // Check all allocations that could potentially overlap
        for (&start, &alloc_size) in &self.allocations {
            let alloc_end = start + alloc_size;

            // Two ranges [a1,a2) and [b1,b2) overlap if: a1 < b2 && b1 < a2
            if address < alloc_end && start < end {
                return true;
            }
        }
        false
    }
}
