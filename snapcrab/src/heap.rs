use crate::value::Value;
use std::alloc::Layout;

/// Memory address type for heap-allocated values.
pub type Address = usize;

/// Entry in the heap containing a value and its memory layout information.
#[derive(Debug, Clone)]
struct HeapEntry {
    /// The stored value
    value: Value,
    /// Memory layout information for the value
    layout: Layout,
}

/// Heap memory manager for dynamic allocation during interpretation.
///
/// The heap provides a simple memory model for allocating and deallocating
/// values during program execution. Currently unused but designed for future
/// support of heap-allocated data structures.
#[derive(Debug, Default)]
pub struct Heap {
    /// Vector of heap entries, indexed by address
    memory: Vec<Option<HeapEntry>>,
    /// Next available address for allocation
    next_address: Address,
}

impl Heap {
    /// Creates a new empty heap.
    ///
    /// # Returns
    /// A new heap instance ready for allocation
    pub fn new() -> Self {
        Self {
            memory: Vec::new(),
            next_address: 0,
        }
    }

    /// Allocates space for a value on the heap.
    ///
    /// # Arguments
    /// * `value` - The value to store
    /// * `layout` - Memory layout information for the value
    ///
    /// # Returns
    /// The address where the value was allocated
    #[allow(dead_code)]
    pub fn allocate(&mut self, value: Value, layout: Layout) -> Address {
        let address = self.next_address;
        if address >= self.memory.len() {
            self.memory.resize(address + 1, None);
        }
        self.memory[address] = Some(HeapEntry { value, layout });
        self.next_address += 1;
        address
    }

    /// Reads a value from the heap at the given address.
    ///
    /// # Arguments
    /// * `address` - The address to read from
    ///
    /// # Returns
    /// * `Some(Value)` - The value at the address if it exists
    /// * `None` - If the address is invalid or deallocated
    #[allow(dead_code)]
    pub fn read(&self, address: Address) -> Option<&Value> {
        self.memory
            .get(address)
            .and_then(|entry| entry.as_ref().map(|e| &e.value))
    }

    #[allow(dead_code)]
    pub fn write(&mut self, address: Address, value: Value) -> bool {
        if let Some(Some(entry)) = self.memory.get_mut(address) {
            entry.value = value;
            true
        } else {
            false
        }
    }

    #[allow(dead_code)]
    pub fn get_layout(&self, address: Address) -> Option<Layout> {
        self.memory
            .get(address)
            .and_then(|entry| entry.as_ref().map(|e| e.layout))
    }
}
