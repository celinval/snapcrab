//! Writing to static memory: allowed for read-write statics (`static mut` and
//! statics with interior mutability), rejected for read-only statics.

#![allow(unused)]

use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicU32, Ordering};

static mut COUNTER: u32 = 0;

static ATOMIC: AtomicU32 = AtomicU32::new(0);

/// A `Sync` wrapper around `UnsafeCell` so it can back a `static`.
struct SyncCell(UnsafeCell<u32>);
unsafe impl Sync for SyncCell {}

static CELL: SyncCell = SyncCell(UnsafeCell::new(0));

static READ_ONLY: u32 = 7;

/// Writing to a `static mut` (read-write memory) updates it.
pub fn write_static_mut() {
    unsafe {
        COUNTER = 42;
        assert!(COUNTER == 42);
    }
}

/// A `static` with interior mutability (`AtomicU32`) is read-write memory, so
/// storing through it is allowed.
pub fn write_interior_mutable() {
    ATOMIC.store(5, Ordering::Relaxed);
    assert!(ATOMIC.load(Ordering::Relaxed) == 5);
}

/// A `static` whose type has interior mutability (`UnsafeCell`) is read-write
/// memory, so writing through the cell's pointer is allowed — no atomics.
pub fn write_interior_unsafecell() {
    unsafe {
        let p = CELL.0.get();
        *p = 13;
        assert!(*p == 13);
    }
}

/// Writing through a raw pointer to an immutable `static` targets read-only
/// memory, which is undefined behavior and must be rejected.
pub fn write_read_only_static() {
    let p = &READ_ONLY as *const u32 as *mut u32;
    // `black_box` hides the pointer's origin from the compiler's
    // `invalid_reference_casting` lint so the write compiles; the point is to
    // exercise the interpreter's runtime rejection of the read-only write.
    unsafe {
        *std::hint::black_box(p) = 99;
    }
}
