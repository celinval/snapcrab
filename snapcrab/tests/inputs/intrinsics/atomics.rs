//! Atomic intrinsics, exercised through the stable `std::sync::atomic` API.

#![allow(unused)]

use std::sync::Arc;
use std::sync::atomic::{
    AtomicBool, AtomicI8, AtomicI32, AtomicPtr, AtomicU8, AtomicU32, AtomicU64, AtomicUsize,
    Ordering, fence,
};

/// `load` and `store` round-trip a value.
pub fn load_store() {
    let value = AtomicU32::new(7);
    assert!(value.load(Ordering::SeqCst) == 7);
    value.store(42, Ordering::Release);
    assert!(value.load(Ordering::Acquire) == 42);
}

/// `swap` stores the new value and returns the old one.
pub fn swap() {
    let value = AtomicU64::new(1);
    assert!(value.swap(2, Ordering::AcqRel) == 1);
    assert!(value.load(Ordering::Relaxed) == 2);
}

/// `compare_exchange` stores only when the current value matches, and reports
/// the value it saw either way.
pub fn compare_exchange() {
    let value = AtomicU32::new(5);
    assert!(value.compare_exchange(5, 6, Ordering::SeqCst, Ordering::SeqCst) == Ok(5));
    assert!(value.load(Ordering::Relaxed) == 6);
    assert!(value.compare_exchange(5, 9, Ordering::SeqCst, Ordering::Relaxed) == Err(6));
    assert!(value.load(Ordering::Relaxed) == 6);
}

/// `compare_exchange_weak` succeeds on a match. It may fail spuriously, so the
/// loop is the idiomatic way to use it.
pub fn compare_exchange_weak() {
    let value = AtomicUsize::new(10);
    let mut current = value.load(Ordering::Relaxed);
    while let Err(seen) =
        value.compare_exchange_weak(current, current * 2, Ordering::SeqCst, Ordering::Relaxed)
    {
        current = seen;
    }
    assert!(value.load(Ordering::Relaxed) == 20);
}

/// The arithmetic and bitwise read-modify-writes return the previous value.
pub fn fetch_ops() {
    let value = AtomicU32::new(0b1100);
    assert!(value.fetch_add(1, Ordering::SeqCst) == 0b1100);
    assert!(value.fetch_sub(1, Ordering::SeqCst) == 0b1101);
    assert!(value.fetch_and(0b1010, Ordering::SeqCst) == 0b1100);
    assert!(value.fetch_or(0b0001, Ordering::SeqCst) == 0b1000);
    assert!(value.fetch_xor(0b1111, Ordering::SeqCst) == 0b1001);
    assert!(value.load(Ordering::SeqCst) == 0b0110);
    assert!(value.fetch_nand(0b0100, Ordering::SeqCst) == 0b0110);
    assert!(value.load(Ordering::SeqCst) == !0b0100);
}

/// Atomic arithmetic wraps instead of overflowing, at the value's own width.
pub fn fetch_add_wraps() {
    let value = AtomicU8::new(250);
    assert!(value.fetch_add(10, Ordering::SeqCst) == 250);
    assert!(value.load(Ordering::SeqCst) == 4);
    assert!(value.fetch_sub(5, Ordering::SeqCst) == 4);
    assert!(value.load(Ordering::SeqCst) == 255);
}

/// `fetch_max` and `fetch_min` compare signed values as signed: `-1` is less
/// than `1`, although its bit pattern is larger.
pub fn fetch_max_min_signed() {
    let value = AtomicI32::new(-1);
    assert!(value.fetch_max(1, Ordering::SeqCst) == -1);
    assert!(value.load(Ordering::SeqCst) == 1);
    assert!(value.fetch_min(-5, Ordering::SeqCst) == 1);
    assert!(value.load(Ordering::SeqCst) == -5);

    let narrow = AtomicI8::new(i8::MIN);
    assert!(narrow.fetch_max(-1, Ordering::SeqCst) == i8::MIN);
    assert!(narrow.load(Ordering::SeqCst) == -1);
}

/// For unsigned values `fetch_max`/`fetch_min` compare as unsigned, so the
/// largest bit pattern wins.
pub fn fetch_max_min_unsigned() {
    let value = AtomicU32::new(u32::MAX);
    assert!(value.fetch_min(3, Ordering::SeqCst) == u32::MAX);
    assert!(value.load(Ordering::SeqCst) == 3);
    assert!(value.fetch_max(u32::MAX, Ordering::SeqCst) == 3);
    assert!(value.load(Ordering::SeqCst) == u32::MAX);
}

/// `AtomicBool` is a byte, and its logical operations go through the same
/// intrinsics.
pub fn atomic_bool() {
    let flag = AtomicBool::new(false);
    assert!(!flag.swap(true, Ordering::SeqCst));
    assert!(flag.fetch_and(false, Ordering::SeqCst));
    assert!(!flag.load(Ordering::SeqCst));
    assert!(!flag.fetch_or(true, Ordering::SeqCst));
    assert!(flag.load(Ordering::SeqCst));
}

/// `AtomicPtr` stores and swaps addresses, and its byte arithmetic offsets the
/// pointer by a byte count.
pub fn atomic_ptr() {
    let mut values = [10u32, 20, 30];
    let first: *mut u32 = &mut values[0];
    let pointer = AtomicPtr::new(first);
    let third: *mut u32 = &mut values[2];
    assert!(pointer.swap(third, Ordering::SeqCst) == first);
    assert!(unsafe { *pointer.load(Ordering::SeqCst) } == 30);
    pointer.fetch_byte_sub(4, Ordering::SeqCst);
    assert!(unsafe { *pointer.load(Ordering::SeqCst) } == 20);
}

/// Fences have nothing to order in a single thread, but must be accepted.
pub fn fences() {
    fence(Ordering::SeqCst);
    std::sync::atomic::compiler_fence(Ordering::SeqCst);
}

/// `Arc` keeps its reference counts in atomics, so cloning and dropping one
/// exercises them end to end.
pub fn arc_refcount() {
    let shared = Arc::new(5u32);
    let clone = Arc::clone(&shared);
    assert!(Arc::strong_count(&shared) == 2);
    drop(clone);
    assert!(Arc::strong_count(&shared) == 1);
    assert!(*shared == 5);
}
