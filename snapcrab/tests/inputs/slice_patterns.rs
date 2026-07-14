#![allow(unused)]

/// ConstantIndex: access first and last elements via pattern matching.
pub fn test_constant_index() {
    let arr = [10, 20, 30, 40, 50];
    let [first, _, _, _, last] = arr;
    assert!(first == 10);
    assert!(last == 50);
}

/// ConstantIndex from_end: access elements from the end.
pub fn test_constant_index_from_end() {
    let arr = [1, 2, 3, 4, 5, 6];
    let [_, _, _, _, second_last, last] = arr;
    assert!(second_last == 5);
    assert!(last == 6);
}

/// Subslice: split array into head and tail via pattern.
pub fn test_subslice_array() {
    let arr = [1, 2, 3, 4, 5];
    let [first, rest @ ..] = arr;
    assert!(first == 1);
    assert!(rest.len() == 4);
    assert!(rest[0] == 2);
    assert!(rest[3] == 5);
}

/// Subslice: split into head, middle, and tail.
pub fn test_subslice_middle() {
    let arr = [10, 20, 30, 40, 50];
    let [first, middle @ .., last] = arr;
    assert!(first == 10);
    assert!(last == 50);
    assert!(middle.len() == 3);
    assert!(middle[0] == 20);
    assert!(middle[2] == 40);
}

/// Slice pattern on a slice reference.
pub fn test_slice_pattern_ref() {
    let data = [100, 200, 300, 400];
    let slice: &[i32] = &data;
    if let [a, rest @ ..] = slice {
        assert!(*a == 100);
        assert!(rest.len() == 3);
        assert!(rest[0] == 200);
    } else {
        panic!();
    }
}

/// ConstantIndex from_end on a slice (negative indexing).
pub fn test_slice_from_end() {
    let data = [1, 2, 3, 4, 5];
    let slice: &[i32] = &data;
    if let [.., second_last, last] = slice {
        assert!(*second_last == 4);
        assert!(*last == 5);
    } else {
        panic!();
    }
}
