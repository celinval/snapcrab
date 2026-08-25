//! Drop semantics exercised across the places a drop can be emitted.
//!
//! Drops are observed through a shared `Cell<u32>` counter that each
//! droppable value increments in its `Drop::drop`. A test succeeds silently
//! when the observed drop count matches expectations and panics otherwise.
//!
//! Coverage spans automatic drops (scope end, reverse declaration order,
//! nested scopes, loop bodies, conditional branches, drop-on-reassign, and
//! drop glue for aggregates and `Box`) as well as manual drops via
//! `MaybeUninit`, `ManuallyDrop`, and `ptr::drop_in_place`.

#![allow(unused)]

use std::cell::Cell;
use std::mem::{ManuallyDrop, MaybeUninit};

/// Increments a shared counter when dropped so tests can observe drops.
struct DropCounter<'a> {
    log: &'a Cell<u32>,
}

impl Drop for DropCounter<'_> {
    fn drop(&mut self) {
        self.log.set(self.log.get() + 1);
    }
}

/// Records the value it carries into a shared order log when dropped.
struct DropOrder<'a> {
    id: u32,
    order: &'a Cell<u32>,
}

impl Drop for DropOrder<'_> {
    fn drop(&mut self) {
        // Shift previous entries left and append this id in the low digit.
        self.order.set(self.order.get() * 10 + self.id);
    }
}

// --- Automatic drops ---

/// A value is dropped when its binding goes out of scope.
pub fn scope_end_drop() {
    let log = Cell::new(0);
    {
        let _guard = DropCounter { log: &log };
        assert!(log.get() == 0);
    }
    assert!(log.get() == 1);
}

/// Locals are dropped in reverse declaration order at scope end.
pub fn reverse_declaration_order() {
    let order = Cell::new(0);
    {
        let _first = DropOrder { id: 1, order: &order };
        let _second = DropOrder { id: 2, order: &order };
        let _third = DropOrder { id: 3, order: &order };
    }
    // Dropped third, second, first -> digits 3, 2, 1.
    assert!(order.get() == 321);
}

/// A value in an inner block is dropped before the outer scope resumes.
pub fn nested_scope_drop() {
    let log = Cell::new(0);
    let _outer = DropCounter { log: &log };
    {
        let _inner = DropCounter { log: &log };
    }
    // Only the inner value has been dropped so far.
    assert!(log.get() == 1);
}

/// Each loop iteration constructs and drops its own value.
pub fn drop_in_loop() {
    let log = Cell::new(0);
    let mut i = 0u32;
    while i < 3 {
        let _tmp = DropCounter { log: &log };
        i += 1;
    }
    assert!(log.get() == 3);
}

/// Only the value on the taken branch is dropped.
pub fn conditional_drop() {
    let log = Cell::new(0);
    let flag = true;
    if flag {
        let _taken = DropCounter { log: &log };
    } else {
        let _skipped = DropCounter { log: &log };
    }
    assert!(log.get() == 1);
}

/// Reassigning an initialized place drops the previous value first.
pub fn drop_on_reassign() {
    let log = Cell::new(0);
    let mut slot = DropCounter { log: &log };
    // Overwriting `slot` drops the original before storing the new value.
    slot = DropCounter { log: &log };
    assert!(log.get() == 1);
    drop(slot);
    assert!(log.get() == 2);
}

/// Drop glue for an aggregate drops each field in declaration order.
pub fn aggregate_field_drop() {
    let order = Cell::new(0);
    struct Pair<'a> {
        a: DropOrder<'a>,
        b: DropOrder<'a>,
    }
    {
        let _pair = Pair {
            a: DropOrder { id: 1, order: &order },
            b: DropOrder { id: 2, order: &order },
        };
    }
    // Fields drop in declaration order: a (1) then b (2).
    assert!(order.get() == 12);
}

/// Dropping a `Box` runs the inner value's drop glue and frees the heap.
pub fn box_drop() {
    let log = Cell::new(0);
    {
        let _boxed = Box::new(DropCounter { log: &log });
    }
    assert!(log.get() == 1);
}

// --- Moves suppress the source drop ---

fn consume(value: DropCounter<'_>) {
    // `value` is dropped here, at the end of the callee.
}

/// Moving a value into a callee transfers drop responsibility; the source is
/// not dropped again.
pub fn move_suppresses_source_drop() {
    let log = Cell::new(0);
    let guard = DropCounter { log: &log };
    consume(guard);
    // Dropped exactly once, inside `consume`.
    assert!(log.get() == 1);
}

// --- Explicit early drop ---

/// `drop(x)` runs the destructor early rather than at scope end.
pub fn explicit_drop() {
    let log = Cell::new(0);
    let guard = DropCounter { log: &log };
    assert!(log.get() == 0);
    drop(guard);
    assert!(log.get() == 1);
}

/// `mem::needs_drop` reports drop glue by type, not a blanket answer.
pub fn needs_drop_by_type() {
    assert!(std::mem::needs_drop::<DropCounter>());
    assert!(std::mem::needs_drop::<(u32, DropCounter)>());
    assert!(!std::mem::needs_drop::<u32>());
    assert!(!std::mem::needs_drop::<[u8; 4]>());
}

// --- Manual drops ---

/// A `MaybeUninit` value is not dropped automatically; `assume_init_drop`
/// runs the destructor exactly once.
pub fn maybe_uninit_manual_drop() {
    let log = Cell::new(0);
    let mut slot: MaybeUninit<DropCounter> = MaybeUninit::uninit();
    slot.write(DropCounter { log: &log });
    // The wrapper leaks the value unless we drop it by hand.
    assert!(log.get() == 0);
    unsafe {
        slot.assume_init_drop();
    }
    assert!(log.get() == 1);
}

/// `ptr::drop_in_place` on a `MaybeUninit`'s backing storage drops the value.
pub fn maybe_uninit_drop_in_place() {
    let log = Cell::new(0);
    let mut slot: MaybeUninit<DropCounter> = MaybeUninit::uninit();
    slot.write(DropCounter { log: &log });
    unsafe {
        std::ptr::drop_in_place(slot.as_mut_ptr());
    }
    assert!(log.get() == 1);
}

/// A `MaybeUninit` that is never initialized must not run any destructor.
pub fn maybe_uninit_never_dropped() {
    let log = Cell::new(0);
    {
        let _slot: MaybeUninit<DropCounter> = MaybeUninit::uninit();
    }
    assert!(log.get() == 0);
}

/// `ManuallyDrop` suppresses the automatic drop until `ManuallyDrop::drop`.
pub fn manually_drop_manual() {
    let log = Cell::new(0);
    let mut wrapped = ManuallyDrop::new(DropCounter { log: &log });
    assert!(log.get() == 0);
    unsafe {
        ManuallyDrop::drop(&mut wrapped);
    }
    assert!(log.get() == 1);
}

/// Reading an uninhabited value out of `MaybeUninit` must abort.
///
/// `assume_init` calls `assert_inhabited`, which the interpreter checks
/// directly (codegen would emit an abort). `HasVoid` is uninhabited via its
/// field, not just an empty enum.
pub fn assume_init_uninhabited() {
    enum Void {}
    struct HasVoid(Void, u32);
    let slot: MaybeUninit<HasVoid> = MaybeUninit::uninit();
    let _value = unsafe { slot.assume_init() };
    unreachable!("assume_init on an uninhabited type must not return");
}

/// A `ManuallyDrop` left untouched never runs its inner destructor.
pub fn manually_drop_leaks() {
    let log = Cell::new(0);
    {
        let _wrapped = ManuallyDrop::new(DropCounter { log: &log });
    }
    // No automatic drop and no manual call -> the value leaks.
    assert!(log.get() == 0);
}
