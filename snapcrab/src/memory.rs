//! Memory allocation tracking.
//!
//! Provides a memory tracker that records allocated memory regions and validates
//! memory access bounds. Ensures no overlapping allocations and efficient
//! bounds checking for memory safety.

use anyhow::{Result, bail};
use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

/// Global singleton memory tracker instance
static MEMORY_TRACKER: OnceLock<Mutex<MemoryTracker>> = OnceLock::new();

/// Gets the global memory tracker instance
pub fn global_memory_tracker() -> &'static Mutex<MemoryTracker> {
    MEMORY_TRACKER.get_or_init(|| Mutex::new(MemoryTracker::new()))
}

/// Tracks memory allocations and validates memory access bounds.
///
/// Maintains a record of all allocated memory regions with their addresses and sizes.
/// Prevents overlapping allocations and provides efficient bounds checking.
#[derive(Debug, Default)]
pub struct MemoryTracker {
    /// Map from allocation start address to allocation size
    allocations: BTreeMap<usize, usize>,
}

impl MemoryTracker {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocate_and_contains() {
        let mut tracker = MemoryTracker::new();

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
        let mut tracker = MemoryTracker::new();

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
        let mut tracker = MemoryTracker::new();

        tracker.allocate(100, 50).unwrap();
        assert!(tracker.contains(125, 10));

        tracker.deallocate(100).unwrap();
        assert!(!tracker.contains(125, 10));

        // Deallocating again should fail
        assert!(tracker.deallocate(100).is_err());
    }
}
