#[no_mangle]
pub extern "C" fn make_array2_u64() -> [u64; 2] {
    [10, 20]
}

#[no_mangle]
pub extern "C" fn sum_array2_u64(arr: [u64; 2]) -> u64 {
    arr[0] + arr[1]
}

#[no_mangle]
pub extern "C" fn make_array2_u32() -> [u32; 2] {
    [5, 7]
}

#[no_mangle]
pub extern "C" fn sum_array2_u32(arr: [u32; 2]) -> u32 {
    arr[0] + arr[1]
}
