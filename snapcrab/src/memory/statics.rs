//! Static and global memory management.
//!
//! Materializes compiler-known allocations (string literals, const statics, vtables)
//! into real process memory so that pointers to them are valid addresses.
//!
//! Uses interior mutability (`RefCell`) because constant evaluation needs to
//! materialize allocations lazily while the interpreter holds shared references.

use crate::interpreter::native;
use crate::memory::aligned::AlignedBuf;
use crate::memory::sanitizer::MemorySanitizer;
use crate::memory::{MemoryAccessError, MemorySegment, pointer_width};
use crate::ty::contains_mutable_ptr;
use crate::value::Value;
use rustc_public::mir::Mutability;
use rustc_public::mir::alloc::{AllocId, GlobalAlloc};
use rustc_public::mir::mono::{Instance, StaticDef};
use rustc_public::ty::Allocation;
use rustc_public::{CrateDef, local_crate};
use rustc_public_bridge::IndexedVal;
use std::cell::RefCell;
use std::collections::HashMap;
use std::{ptr, slice};

/// Manages static/global allocations materialized from the compiler.
#[derive(Default)]
pub struct Statics {
    inner: RefCell<StaticsInner>,
    /// Registry of reified functions. Kept in its own cell (and with its own
    /// sanitizer) because a function address is *callable*, not *readable*:
    /// it must not satisfy a data read the way materialized data does.
    code: RefCell<CodeSegment>,
}

#[derive(Default)]
struct StaticsInner {
    /// Backing storage for materialized allocations.
    ///
    /// Each entry is an `AlignedBuf` whose heap pointer remains stable regardless
    /// of how the outer `Vec` grows — pushing new entries may move the structs,
    /// but not the heap buffers they point to. Each buffer honours its
    /// allocation's alignment so pointers into it are correctly aligned.
    allocations: Vec<AlignedBuf>,
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
    pub fn resolve_alloc(&self, alloc_id: AllocId) -> anyhow::Result<usize> {
        let id_idx = alloc_id.to_index();
        {
            let inner = self.inner.borrow();
            if let Some(&alloc_idx) = inner.alloc_map.get(&id_idx) {
                return Ok(inner.allocations[alloc_idx].as_ptr() as usize);
            }
        }

        let global = GlobalAlloc::from(alloc_id);
        match global {
            GlobalAlloc::Memory(alloc) => Ok(self.materialize_alloc(alloc_id, &alloc)),
            GlobalAlloc::Static(def) => {
                let name = CrateDef::name(&def);
                let alloc = def
                    .eval_initializer()
                    .map_err(|e| anyhow::anyhow!("failed to evaluate static `{name}`: {e}"))?;
                let is_mutable = alloc.mutability == Mutability::Mut;
                if is_mutable || contains_mutable_ptr(def.ty()) {
                    check_static_not_duplicated(def)?;
                }
                Ok(self.materialize_alloc(alloc_id, &alloc))
            }
            // A vtable resolves to an allocation holding the drop
            // pointer, the type's size and align, and the method pointers.
            //
            // FIXME: a reified address is synthetic, not a real machine
            // address, so a vtable entry (or `fn` pointer) handed to native
            // code and called there won't work until we JIT re-entry stubs.
            GlobalAlloc::VTable(..) => {
                let vtable = global
                    .vtable_allocation()
                    .ok_or_else(|| anyhow::anyhow!("failed to resolve vtable allocation"))?;
                self.resolve_alloc(vtable)
            }
            // A function pointer reifies to a synthetic callable address; this
            // backs const/static `fn` pointers and vtable method/drop slots.
            GlobalAlloc::Function(instance) => Ok(self.reify_fn(instance)),
            GlobalAlloc::TypeId { .. } => Ok(0),
        }
    }

    /// Reify a function to its stable synthetic address.
    pub fn reify_fn(&self, instance: Instance) -> usize {
        self.code.borrow_mut().reify(instance)
    }

    /// Resolve a reified function address back to its `Instance`.
    pub fn resolve_fn(&self, addr: usize) -> anyhow::Result<Instance> {
        self.code.borrow().resolve(addr)
    }

    /// Materialize an allocation into real memory, resolving nested provenance.
    fn materialize_alloc(&self, alloc_id: AllocId, alloc: &Allocation) -> usize {
        let id_idx = alloc_id.to_index();

        // Materialize into an aligned buffer so pointers into the allocation
        // honour its declared alignment. Copy each initialized byte and leave
        // uninitialized bytes (e.g. struct padding) zeroed; `raw_bytes` is
        // all-or-nothing and would otherwise force the whole value to zero
        // whenever any padding byte is uninitialized.
        let mut buf = AlignedBuf::zeroed(alloc.bytes.len(), alloc.align as usize)
            .expect("static allocation layout from verified compiler data");
        for (i, byte) in alloc.bytes.iter().enumerate() {
            if let Some(b) = byte {
                buf[i] = *b;
            }
        }

        // Resolve provenance: patch pointer-sized segments with real addresses.
        let ptr_size = pointer_width();
        for (offset, prov) in &alloc.provenance.ptrs {
            // Nested provenance (e.g., &str pointing to string bytes) cannot
            // be a duplicated mutable static, so unwrap is safe here.
            let target_addr = self
                .resolve_alloc(prov.0)
                .expect("nested provenance resolution");
            let addr_bytes = target_addr.to_le_bytes();
            buf[*offset..*offset + ptr_size].copy_from_slice(&addr_bytes[..ptr_size]);
        }

        let addr = buf.as_ptr() as usize;
        let len = buf.len();

        let mut inner = self.inner.borrow_mut();
        let alloc_idx = inner.allocations.len();
        inner.allocations.push(buf);
        // SAFETY: the buffer's heap pointer remains stable after push (only the
        // `AlignedBuf` struct moves, not the allocation it owns).
        let slice = unsafe { slice::from_raw_parts(addr as *const u8, len) };
        inner.sanitizer.register_alloc(slice);
        inner.alloc_map.insert(id_idx, alloc_idx);

        addr
    }
}

/// Bail if an external static with interior mutability also exists as a
/// native symbol.
///
/// If both the interpreter and native code have their own copy, mutations
/// from one side are invisible to the other.
fn check_static_not_duplicated(def: StaticDef) -> anyhow::Result<()> {
    if def.krate() == local_crate() {
        return Ok(());
    }
    let instance = Instance::from(def);
    let mangled = instance.mangled_name();
    if native::resolve_symbol(mangled.as_str()).is_some() {
        anyhow::bail!(
            "unsupported: static `{}` allows mutation and is duplicated in \
             native code — mutations from one side would be invisible to \
             the other",
            instance.name()
        );
    }
    Ok(())
}

// SAFETY: Allocations are stored in Box<[u8]> that are never moved or reallocated
// after creation. The sanitizer tracks their addresses for bounds checking.
unsafe impl MemorySegment for Statics {
    fn read_addr(&self, address: usize, size: usize) -> Result<Value, MemoryAccessError> {
        let inner = self.inner.borrow();
        if !inner.sanitizer.contains(address, size) {
            return Err(MemoryAccessError::NotFound);
        }
        let ptr = address as *const u8;
        // SAFETY: sanitizer confirmed the range is within a live allocation;
        // the bytes are copied into an owned `Value` before returning.
        let slice = unsafe { slice::from_raw_parts(ptr, size) };
        Ok(Value::from_bytes(slice))
    }

    fn write_addr(&self, _address: usize, _data: &[u8]) -> Result<(), MemoryAccessError> {
        Err(MemoryAccessError::OutOfBounds)
    }
}

/// A registered function whose stable heap address is its `fn`-pointer value.
///
/// Rust guarantees no particular alignment for the code a `fn` pointer points
/// at, so we impose none; the address is wherever its `Box` lands.
/// [`CodeSegment::resolve`] still accepts only an exact base, because it checks
/// the full `FunctionInfo` size.
struct FunctionInfo {
    instance: Instance,
}

/// Maps interpreted functions to synthetic, unique, callable addresses.
///
/// Reifying an `Instance` returns the stable address of its boxed
/// `FunctionInfo` (deduplicated, so the same function always reifies to the
/// same address); resolving an address recovers the `Instance`. The addresses
/// are real heap allocations, so they never collide with data allocations, and
/// the sanitizer validates that an address is a genuine function.
#[derive(Default)]
struct CodeSegment {
    sanitizer: MemorySanitizer,
    functions: HashMap<Instance, Box<FunctionInfo>>,
}

impl CodeSegment {
    /// Return the stable address for `instance`, allocating it on first use.
    fn reify(&mut self, instance: Instance) -> usize {
        if let Some(info) = self.functions.get(&instance) {
            return ptr::from_ref(info.as_ref()) as usize;
        }
        let info = Box::new(FunctionInfo { instance });
        let addr = ptr::from_ref(info.as_ref()) as usize;
        // SAFETY: `info` lives in the map for the program's lifetime, so the
        // registered range stays valid; functions are never freed.
        let slice = unsafe { slice::from_raw_parts(addr as *const u8, size_of::<FunctionInfo>()) };
        self.sanitizer.register_alloc(slice);
        self.functions.insert(instance, info);
        addr
    }

    /// Recover the `Instance` a reified address points to.
    fn resolve(&self, addr: usize) -> anyhow::Result<Instance> {
        // Checking the exact `FunctionInfo` size accepts only a true base: an
        // interior or misaligned address exceeds the one-function allocation.
        self.sanitizer
            .check_access(addr, size_of::<FunctionInfo>())
            .map_err(|_| anyhow::anyhow!("invalid function pointer at 0x{addr:x}"))?;
        // SAFETY: the address is a live `FunctionInfo` base (checked above).
        Ok(unsafe { &*(addr as *const FunctionInfo) }.instance)
    }
}
