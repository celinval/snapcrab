//! Virtual dispatch through trait-object vtables.
//!
//! Each function exercises a receiver shape or dispatch path and asserts the
//! result, panicking on mismatch.

#![allow(unused)]

use std::cell::Cell;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;

trait Shape {
    fn area(&self) -> u32;
    /// Provided method, to exercise a default vtable entry and its override.
    fn sides(&self) -> u32 {
        0
    }
}

struct Square(u32);
impl Shape for Square {
    fn area(&self) -> u32 {
        self.0 * self.0
    }
}

struct Rect(u32, u32);
impl Shape for Rect {
    fn area(&self) -> u32 {
        self.0 * self.1
    }
    fn sides(&self) -> u32 {
        4
    }
}

/// Dispatch a method through a shared reference to a trait object.
pub fn call_via_ref() {
    let sq = Square(5);
    let d: &dyn Shape = &sq;
    assert!(d.area() == 25);
}

/// A provided method uses the default unless the impl overrides it.
pub fn provided_method() {
    let sq = Square(5);
    let d: &dyn Shape = &sq;
    assert!(d.sides() == 0);

    let rect = Rect(3, 4);
    let d: &dyn Shape = &rect;
    assert!(d.area() == 12 && d.sides() == 4);
}

fn area_of(shape: &dyn Shape) -> u32 {
    shape.area()
}

/// Dispatch through a trait object passed as a function argument.
pub fn call_via_arg() {
    let sq = Square(5);
    let rect = Rect(3, 4);
    assert!(area_of(&sq) + area_of(&rect) == 37);
}

trait Counter {
    fn get(&self) -> u32;
    fn bump(&mut self);
}

struct Cnt(u32);
impl Counter for Cnt {
    fn get(&self) -> u32 {
        self.0
    }
    fn bump(&mut self) {
        self.0 += 1;
    }
}

/// Dispatch a `&mut self` method through a mutable trait object.
pub fn call_via_mut() {
    let mut c = Cnt(41);
    let d: &mut dyn Counter = &mut c;
    d.bump();
    assert!(d.get() == 42);
}

/// Dispatch a method through a boxed trait object.
pub fn call_via_box() {
    let d: Box<dyn Counter> = Box::new(Cnt(42));
    assert!(d.get() == 42);
}

trait Base {
    fn base(&self) -> u32;
}
trait Derived: Base {
    fn derived(&self) -> u32;
}
struct Impl;
impl Base for Impl {
    fn base(&self) -> u32 {
        1
    }
}
impl Derived for Impl {
    fn derived(&self) -> u32 {
        2
    }
}

/// Upcast `&dyn Derived` to its principal supertrait and dispatch through it.
pub fn upcast_then_call() {
    let value = Impl;
    let derived: &dyn Derived = &value;
    assert!(derived.derived() == 2);
    let base: &dyn Base = derived;
    assert!(base.base() == 1);
}

struct Tracked<'a> {
    log: &'a Cell<u32>,
}
impl Counter for Tracked<'_> {
    fn get(&self) -> u32 {
        self.log.get()
    }
    fn bump(&mut self) {}
}
impl Drop for Tracked<'_> {
    fn drop(&mut self) {
        self.log.set(self.log.get() + 1);
    }
}

/// Dropping a boxed trait object runs the concrete destructor via the vtable.
pub fn box_dyn_drop() {
    let log = Cell::new(0);
    {
        let d: Box<dyn Counter> = Box::new(Tracked { log: &log });
        assert!(d.get() == 0);
    }
    assert!(log.get() == 1);
}

// --- Method receivers other than `&self` / `&mut self` ---

trait BoxRecv {
    fn value(self: Box<Self>) -> u32;
}
struct Boxed(u32);
impl BoxRecv for Boxed {
    fn value(self: Box<Self>) -> u32 {
        self.0
    }
}

/// A `self: Box<Self>` receiver dispatched through a `Box<dyn _>`.
pub fn recv_box_self() {
    let d: Box<dyn BoxRecv> = Box::new(Boxed(42));
    assert!(d.value() == 42);
}

trait RcRecv {
    fn value(self: Rc<Self>) -> u32;
}
struct RcHeld(u32);
impl RcRecv for RcHeld {
    fn value(self: Rc<Self>) -> u32 {
        self.0
    }
}

/// A `self: Rc<Self>` receiver. Coercing `Rc<T> -> Rc<dyn _>` goes through
/// `Rc`'s `CoerceUnsized`, which is not yet supported.
pub fn recv_rc_self() {
    let d: Rc<dyn RcRecv> = Rc::new(RcHeld(42));
    assert!(d.value() == 42);
}

trait ArcRecv {
    fn value(self: Arc<Self>) -> u32;
}
struct ArcHeld(u32);
impl ArcRecv for ArcHeld {
    fn value(self: Arc<Self>) -> u32 {
        self.0
    }
}

/// A `self: Arc<Self>` receiver; `Arc<T> -> Arc<dyn _>` coercion, unsupported.
pub fn recv_arc_self() {
    let d: Arc<dyn ArcRecv> = Arc::new(ArcHeld(42));
    assert!(d.value() == 42);
}

trait PinRecv {
    fn value(self: Pin<&mut Self>) -> u32;
}
struct Pinned(u32);
impl PinRecv for Pinned {
    fn value(self: Pin<&mut Self>) -> u32 {
        self.0
    }
}

/// A `self: Pin<&mut Self>` receiver; `Pin<&mut T> -> Pin<&mut dyn _>`
/// coercion, unsupported.
pub fn recv_pin_self() {
    let mut value = Pinned(42);
    // SAFETY: `value` is not moved after being pinned.
    let pinned: Pin<&mut Pinned> = unsafe { Pin::new_unchecked(&mut value) };
    let d: Pin<&mut dyn PinRecv> = pinned;
    assert!(d.value() == 42);
}

trait Tail {
    fn v(&self) -> u32;
}
struct TailImpl<'a> {
    log: &'a Cell<u32>,
    v: u32,
}
impl Tail for TailImpl<'_> {
    fn v(&self) -> u32 {
        self.v
    }
}
impl Drop for TailImpl<'_> {
    fn drop(&mut self) {
        self.log.set(self.log.get() + 1);
    }
}
struct WithTail<X: ?Sized> {
    header: u64,
    tail: X,
}

/// Dropping an unsized ADT whose tail is a trait object runs the tail's
/// destructor: the ADT's drop glue drops the tail field, which dispatches
/// through the vtable.
pub fn drop_adt_dyn_tail() {
    let log = Cell::new(0);
    {
        let sized: Box<WithTail<TailImpl>> = Box::new(WithTail {
            header: 7,
            tail: TailImpl { log: &log, v: 42 },
        });
        let d: Box<WithTail<dyn Tail>> = sized;
        assert!(d.tail.v() == 42);
    }
    assert!(log.get() == 1);
}

trait Left {
    fn left(&self) -> u32;
}
trait Right {
    fn right(&self) -> u32;
}
trait Both: Left + Right {
    fn both(&self) -> u32;
}
struct Multi;
impl Left for Multi {
    fn left(&self) -> u32 {
        1
    }
}
impl Right for Multi {
    fn right(&self) -> u32 {
        2
    }
}
impl Both for Multi {
    fn both(&self) -> u32 {
        3
    }
}

/// Upcast to a non-principal supertrait (reached via a `TraitVPtr` in the
/// vtable) and dispatch a method through it.
pub fn upcast_secondary_dispatch() {
    let m = Multi;
    let both: &dyn Both = &m;
    let right: &dyn Right = both;
    assert!(right.right() == 2);
    // The principal supertrait reuses the same vtable.
    let left: &dyn Left = both;
    assert!(left.left() == 1);
}
