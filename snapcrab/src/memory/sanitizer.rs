//! Memory allocation sanitizer.
//!
//! Provides a memory tracker that records allocated memory regions and validates
//! memory access bounds. Ensures no overlapping allocations and efficient
//! bounds checking for memory safety.
use std::collections::BTreeMap;

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
    /// Records a new memory allocation.
    ///
    /// # Arguments
    /// * `buf` - The buffer to be registered
    pub fn register_alloc(&mut self, buf: &[u8]) {
        let size = buf.len();
        if size > 0 {
            let address = buf.as_ptr() as usize;
            // This shouldn't happen unless there is a bug which compromises
            // safety. So, kaboom!
            assert!(
                !self.has_overlap(address, size),
                "Allocation at 0x{:x} (size {}) overlaps with existing memory",
                address,
                size
            );
            self.allocations.insert(address, size);
        }
    }

    /// Removes a memory allocation record.
    ///
    /// # Arguments
    /// * `buf`: The buffer to deregister
    pub fn deregister_alloc(&mut self, buf: &[u8]) {
        if !buf.is_empty() {
            let address = buf.as_ptr() as usize;
            if self.allocations.remove(&address).is_none() {
                // This shouldn't happen unless there is a bug which compromises
                // safety. So, kaboom!
                panic!("No allocation found at address 0x{:x}", address);
            }
        }
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
        let mut tracker = MemorySanitizer::default();

        // Create a real buffer and use a slice of it
        let buffer = vec![0u8; 1000];
        let slice = &buffer[100..150]; // 50 bytes starting at offset 100
        tracker.register_alloc(slice);

        let base_addr = slice.as_ptr() as usize;

        // Check various ranges
        assert!(tracker.contains(base_addr, 1)); // Start of allocation
        assert!(tracker.contains(base_addr + 25, 10)); // Middle of allocation
        assert!(tracker.contains(base_addr + 49, 1)); // End of allocation
        assert!(!tracker.contains(base_addr - 1, 1)); // Before allocation
        assert!(!tracker.contains(base_addr + 50, 1)); // After allocation
        assert!(!tracker.contains(base_addr, 51)); // Extends beyond allocation
    }

    #[test]
    #[should_panic(expected = "overlaps with existing memory")]
    fn test_overlapping_allocations() {
        let mut tracker = MemorySanitizer::default();

        let buffer = vec![0u8; 1000];
        let slice1 = &buffer[100..150]; // 50 bytes
        tracker.register_alloc(slice1);

        // This should panic due to overlap
        let slice2 = &buffer[90..110]; // 20 bytes, overlaps at start
        tracker.register_alloc(slice2);
    }

    #[test]
    fn test_deallocate() {
        let mut tracker = MemorySanitizer::default();

        let buffer = vec![0u8; 1000];
        let slice = &buffer[100..150]; // 50 bytes
        tracker.register_alloc(slice);

        let base_addr = slice.as_ptr() as usize;
        assert!(tracker.contains(base_addr + 25, 10));

        tracker.deregister_alloc(slice);
        assert!(!tracker.contains(base_addr + 25, 10));
    }

    #[test]
    #[should_panic(expected = "No allocation found at address")]
    fn test_deallocate_untracked() {
        let mut tracker = MemorySanitizer::default();

        let buffer = vec![0u8; 1000];
        let slice = &buffer[100..150]; // 50 bytes

        // Try to deregister a buffer that was never registered
        tracker.deregister_alloc(slice);
    }
}
