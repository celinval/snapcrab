use crate::memory::sanitizer::MemorySanitizer;
use crate::memory::{MemoryAccessError, MemorySegment};
use crate::value::Value;
use std::sync::{Arc, RwLock};

/// Thread safe heap modeling
#[allow(unused)] // TODO: Remove-me once we actually use heap
#[derive(Clone, Default)]
pub struct Heap(Arc<RwLock<HeapImpl>>);

/// Heap memory manager containing sanitizer and values
#[allow(unused)]
#[derive(Default)]
struct HeapImpl {
    sanitizer: MemorySanitizer,
    values: Vec<Value>,
}

impl Heap {
    pub fn new() -> Self {
        Self::default()
    }
}

unsafe impl MemorySegment for Heap {
    fn read_addr(&self, _address: usize, _size: usize) -> Result<&[u8], MemoryAccessError> {
        tracing::error!("Heap memory access not yet supported");
        Err(MemoryAccessError::NotFound)
    }

    fn write_addr(&self, _address: usize, _data: &[u8]) -> Result<(), MemoryAccessError> {
        tracing::error!("Heap memory access not yet supported");
        Err(MemoryAccessError::NotFound)
    }
}
