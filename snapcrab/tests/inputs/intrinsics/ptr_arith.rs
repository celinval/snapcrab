//! Raw pointer arithmetic: `BinOp::Offset` (`ptr::offset`/`add`/`sub`) and the
//! `ptr_offset_from*` intrinsics that invert it.

#![allow(unused)]

/// `add` moves forward by whole elements, scaling by the pointee size.
pub fn ptr_add() {
    let array = [1u8, 2, 3];
    let p = array.as_ptr();
    unsafe {
        assert!(*p.add(0) == 1);
        assert!(*p.add(2) == 3);
    }
}

/// A larger pointee scales the byte offset accordingly.
pub fn ptr_add_scales_by_size() {
    let array = [1u64, 2, 3];
    let p = array.as_ptr();
    let base = p as usize;
    unsafe {
        assert!(p.add(1) as usize - base == 8);
        assert!(*p.add(1) == 2);
    }
}

/// A negative `offset` moves backwards.
pub fn ptr_negative_offset() {
    let array = [1u8, 2, 3];
    let p = unsafe { array.as_ptr().add(2) };
    unsafe {
        assert!(*p.offset(-2) == 1);
    }
}

/// `sub` moves backwards by whole elements.
pub fn ptr_sub() {
    let array = [1u32, 2, 3];
    let p = unsafe { array.as_ptr().add(2) };
    unsafe {
        assert!(*p.sub(1) == 2);
    }
}

/// Offsetting to one past the end is allowed, as long as it is not read.
pub fn ptr_one_past_end() {
    let array = [1u32, 2];
    let p = array.as_ptr();
    let end = unsafe { p.add(2) };
    assert!(end as usize - p as usize == 8);
}

/// `offset_from` returns the signed element distance, and is the inverse of
/// `add`.
pub fn ptr_offset_from() {
    let array = [1u32, 2, 3];
    let p = array.as_ptr();
    let q = unsafe { p.add(2) };
    unsafe {
        assert!(q.offset_from(p) == 2);
        assert!(p.offset_from(q) == -2);
    }
}

/// `offset_from_unsigned` returns the distance when `ptr >= base`.
pub fn ptr_offset_from_unsigned() {
    let array = [1u64, 2, 3, 4];
    let p = array.as_ptr();
    let q = unsafe { p.add(3) };
    unsafe {
        assert!(q.offset_from_unsigned(p) == 3);
        assert!(p.offset_from_unsigned(p) == 0);
    }
}

/// Casting an address back to a pointer round-trips, since a pointer value is
/// its address.
pub fn addr_roundtrip() {
    let x = 42u32;
    let addr = &x as *const u32 as usize;
    unsafe {
        assert!(*(addr as *const u32) == 42);
    }
}

/// `ptr as` a narrower integer truncates the address, like any integer cast.
pub fn addr_truncates_to_narrower_int() {
    let x = 42u32;
    let addr = &x as *const u32 as usize;
    assert!(&x as *const u32 as u32 == addr as u32);
}

/// A byte offset that is not a whole number of elements makes `offset_from`
/// undefined behavior.
pub fn offset_from_not_multiple() -> isize {
    let array = [0u32; 4];
    let p = array.as_ptr();
    let unaligned = unsafe { (p as *const u8).add(1) } as *const u32;
    unsafe { unaligned.offset_from(p) }
}

/// A byte offset that exceeds `isize::MAX` is undefined behavior.
pub fn offset_overflows_isize() {
    let array = [0u64; 2];
    let p = array.as_ptr();
    let _past = unsafe { p.add(usize::MAX / 2) };
}
