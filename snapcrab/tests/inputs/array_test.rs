#![allow(unused)]

#[derive(Copy, Clone)]
pub struct Point {
    x: i32,
    y: i32,
}

// Empty array
pub fn create_empty_array() -> [i32; 0] {
    []
}

// Array with repeat syntax [value; len]
pub fn create_array_repeat() -> [i32; 5] {
    [42; 5]
}

// Array with explicit elements
pub fn create_array_explicit() -> [i32; 4] {
    [10, 20, 30, 40]
}

// Array of structs with repeat syntax
pub fn create_struct_array_repeat() -> [Point; 3] {
    [Point { x: 1, y: 2 }; 3]
}

// Array of structs with explicit elements
pub fn create_struct_array_explicit() -> [Point; 2] {
    [Point { x: 5, y: 10 }, Point { x: 15, y: 20 }]
}

// Single element array
pub fn create_single_element_array() -> [i32; 1] {
    [99]
}

// Array with zero repeat
pub fn create_zero_array() -> [i32; 4] {
    [0; 4]
}
