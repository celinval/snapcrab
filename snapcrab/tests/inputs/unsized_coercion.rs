#![allow(unused)]

// Generic struct with unsized last field
pub struct Container<T: ?Sized> {
    count: usize,
    data: T,
}

// Create struct with array and try to coerce to slice
// This tests struct unsizing where the last field changes from [T; N] to [T]
pub fn struct_array_to_slice() -> usize {
    let container = Container {
        count: 3,
        data: [10, 20, 30],
    };
    // Try to create a reference that coerces the struct
    let _wide_ref: &Container<[i32]> = &container;
    container.count
}
