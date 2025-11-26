//! Static memory management
//!
//! This module handles static variables and global data.

use crate::memory::{MemoryAccessError, MemorySegment};

/// Static memory manager
#[derive(Debug, Default)]
pub struct Statics {
    // TODO: Implement static memory management
}

unsafe impl MemorySegment for Statics {
    fn read_addr(&self, _address: usize, _size: usize) -> Result<&[u8], MemoryAccessError> {
        tracing::error!("Static memory access not yet supported");
        Err(MemoryAccessError::NotFound)
    }

    fn write_addr(&self, _address: usize, _data: &[u8]) -> Result<(), MemoryAccessError> {
        tracing::error!("Static memory access not yet supported");
        Err(MemoryAccessError::NotFound)
    }
}
