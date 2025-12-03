#![allow(unused)]

// Read element from slice
pub fn read_slice_element() -> i32 {
    let arr = [10, 20, 30, 40, 50];
    let slice: &[i32] = &arr;
    slice[2]
}

// Get slice length
pub fn get_slice_len() -> usize {
    let arr = [10, 20, 30];
    let slice: &[usize] = &arr;
    slice.len()
}
