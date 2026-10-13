//! Slice iteration, which drives the iterator protocol through pointer
//! arithmetic (`BinOp::Offset` and the `ptr_offset_from*` intrinsics).

#![allow(unused)]

/// A `for` loop over a slice reference.
pub fn iter_slice() {
    let array = [1u32, 2, 3, 4];
    let slice: &[u32] = &array;
    let mut total = 0u32;
    for x in slice {
        total += *x;
    }
    assert!(total == 10);
}

/// Iterating a sub-slice starts past the base pointer and stops early.
pub fn iter_subslice() {
    let array = [1u32, 2, 3, 4];
    let mut total = 0u32;
    for x in &array[1..3] {
        total += *x;
    }
    assert!(total == 5);
}

/// `iter_mut` writes back through each element pointer.
pub fn iter_mut_doubles() {
    let mut array = [1u32, 2, 3];
    for x in array.iter_mut() {
        *x *= 2;
    }
    assert!(array[0] == 2 && array[1] == 4 && array[2] == 6);
}

/// `rev` walks the iterator from the end pointer backwards.
pub fn iter_rev() {
    let array = [1u32, 2, 3];
    let mut digits = 0u32;
    for x in array.iter().rev() {
        digits = digits * 10 + *x;
    }
    assert!(digits == 321);
}

/// `enumerate` pairs each element with its index.
pub fn iter_enumerate() {
    let array = [10u32, 20, 30];
    let mut weighted = 0u32;
    for (i, x) in array.iter().enumerate() {
        weighted += (i as u32) * *x;
    }
    assert!(weighted == 80);
}

/// `sum` uses `ExactSizeIterator::len`, which is a pointer distance.
pub fn iter_sum() {
    let array = [1u32, 2, 3, 4];
    let total: u32 = array.iter().sum();
    assert!(total == 10);
}

/// `position` returns the index of the first match, or `None`.
pub fn iter_position() {
    let array = [1u32, 7, 3];
    assert!(array.iter().position(|&x| x == 7) == Some(1));
    assert!(array.iter().position(|&x| x == 9) == None);
}

/// `filter` with `count` drives the iterator through a closure.
pub fn iter_filter_count() {
    let array = [1u32, 2, 3, 4];
    assert!(array.iter().filter(|&&x| x % 2 == 0).count() == 2);
}

/// `zip` advances two slice iterators in lockstep.
pub fn iter_zip() {
    let left = [1u32, 2, 3];
    let right = [10u32, 20, 30];
    let mut dot = 0u32;
    for (x, y) in left.iter().zip(right.iter()) {
        dot += *x * *y;
    }
    assert!(dot == 140);
}

/// `windows` yields overlapping sub-slices.
pub fn iter_windows() {
    let array = [1u32, 2, 3];
    let mut count = 0u32;
    let mut last_sum = 0u32;
    for window in array.windows(2) {
        count += 1;
        last_sum = window[0] + window[1];
    }
    assert!(count == 2 && last_sum == 5);
}

/// `chunks` partitions the slice, with a short final chunk.
pub fn iter_chunks() {
    let array = [1u32, 2, 3, 4, 5];
    let mut chunks = 0u32;
    let mut elements = 0u32;
    for chunk in array.chunks(2) {
        chunks += 1;
        elements += chunk.len() as u32;
    }
    assert!(chunks == 3 && elements == 5);
}

/// Iterating an empty slice yields nothing: the start and end pointers are
/// equal, so the loop body never runs.
pub fn iter_empty() {
    let empty: &[u32] = &[];
    let mut ran = false;
    for _ in empty {
        ran = true;
    }
    assert!(!ran);
}
