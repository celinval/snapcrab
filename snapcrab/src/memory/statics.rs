//! Static and global memory management.
//!
//! Materializes compiler-known allocations (string literals, const statics, vtables)
//! into real process memory so that pointers to them are valid addresses.
//!
//! Uses interior mutability (`RefCell`) because constant evaluation needs to
//! materialize allocations lazily while the interpreter holds shared references.

use crate::memory::sanitizer::MemorySanitizer;
use crate::memory::{MemoryAccessError, MemorySegment};
use rustc_public::mir::alloc::{AllocId, GlobalAlloc};
use rustc_public_bridge::IndexedVal;
use std::cell::RefCell;
use std::collections::HashMap;

/// Manages static/global allocations materialized from the compiler.
#[derive(Default)]
pub struct Statics {
    inner: RefCell<StaticsInner>,
}

#[derive(Default)]
struct StaticsInner {
    /// Backing storage for materialized allocations.
    ///
    /// Each entry is a `Box<[u8]>` whose heap pointer remains stable regardless of
    /// how the outer `Vec` grows — pushing new entries may move the `Box` structs,
    /// but not the heap buffers they point to.
    allocations: Vec<Box<[u8]>>,
    /// Maps AllocId index to the index in `allocations`.
    alloc_map: HashMap<usize, usize>,
    /// Tracks which addresses belong to us for bounds checking.
    sanitizer: MemorySanitizer,
}

impl Statics {
    /// Resolve an AllocId to a real memory address.
    ///
    /// Materializes the allocation on first access, recursively resolving
    /// nested provenance (e.g., a `&str` constant pointing to string bytes).
    pub fn resolve_alloc(&self, alloc_id: AllocId) -> usize {
        let id_idx = alloc_id.to_index();
        {
            let inner = self.inner.borrow();
            if let Some(&alloc_idx) = inner.alloc_map.get(&id_idx) {
                return inner.allocations[alloc_idx].as_ptr() as usize;
            }
        }

        let global = GlobalAlloc::from(alloc_id);
        match global {
            GlobalAlloc::Memory(alloc) => self.materialize_alloc(alloc_id, &alloc),
            GlobalAlloc::Static(def) => match def.eval_initializer() {
                Ok(alloc) => self.materialize_alloc(alloc_id, &alloc),
                Err(_) => 0,
            },
            GlobalAlloc::Function(_) | GlobalAlloc::VTable(..) | GlobalAlloc::TypeId { .. } => 0,
        }
    }

    /// Materialize an allocation into real memory, resolving nested provenance.
    fn materialize_alloc(&self, alloc_id: AllocId, alloc: &rustc_public::ty::Allocation) -> usize {
        let id_idx = alloc_id.to_index();

        let bytes = match alloc.raw_bytes() {
            Ok(b) => b,
            Err(_) => vec![0; alloc.bytes.len()],
        };

        let mut buf = bytes;

        // Resolve provenance: patch pointer-sized segments with real addresses.
        let ptr_size = crate::memory::pointer_width();
        for (offset, prov) in &alloc.provenance.ptrs {
            let target_addr = self.resolve_alloc(prov.0);
            let addr_bytes = target_addr.to_le_bytes();
            buf[*offset..*offset + ptr_size].copy_from_slice(&addr_bytes[..ptr_size]);
        }

        let boxed: Box<[u8]> = buf.into_boxed_slice();
        let addr = boxed.as_ptr() as usize;
        let len = boxed.len();

        let mut inner = self.inner.borrow_mut();
        let alloc_idx = inner.allocations.len();
        inner.allocations.push(boxed);
        // SAFETY: Box heap pointer remains stable after push (only the Box struct moves).
        let slice = unsafe { std::slice::from_raw_parts(addr as *const u8, len) };
        inner.sanitizer.register_alloc(slice);
        inner.alloc_map.insert(id_idx, alloc_idx);

        addr
    }
}

// SAFETY: Allocations are stored in Vec<u8> that are never moved or reallocated
// after creation (we only push to the Vec of allocations, never remove or resize
// individual entries). The sanitizer tracks their addresses for bounds checking.
unsafe impl MemorySegment for Statics {
    fn read_addr(&self, address: usize, size: usize) -> Result<&[u8], MemoryAccessError> {
        let inner = self.inner.borrow();
        if !inner.sanitizer.contains(address, size) {
            return Err(MemoryAccessError::NotFound);
        }
        let ptr = address as *const u8;
        // SAFETY: sanitizer confirmed the range is within a live allocation.
        Ok(unsafe { std::slice::from_raw_parts(ptr, size) })
    }

    fn write_addr(&self, _address: usize, _data: &[u8]) -> Result<(), MemoryAccessError> {
        Err(MemoryAccessError::OutOfBounds)
    }
}
