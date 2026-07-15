/// Native library with basic extern "C" functions for testing the JIT trampoline.

#[no_mangle]
pub extern "C" fn add_u32(a: u32, b: u32) -> u32 {
    a + b
}

#[no_mangle]
pub extern "C" fn add_u64(a: u64, b: u64) -> u64 {
    a + b
}

#[no_mangle]
pub extern "C" fn add_i8(a: i8, b: i8) -> i8 {
    a.wrapping_add(b)
}

#[no_mangle]
pub extern "C" fn negate_i32(x: i32) -> i32 {
    -x
}

#[no_mangle]
pub extern "C" fn no_args() -> u32 {
    42
}

#[no_mangle]
pub extern "C" fn no_return(_x: u32) {
    // Just returns void.
}

#[no_mangle]
pub extern "C" fn identity_bool(b: bool) -> bool {
    b
}

#[no_mangle]
pub extern "C" fn add_f64(a: f64, b: f64) -> f64 {
    a + b
}

#[no_mangle]
pub extern "C" fn add_f32(a: f32, b: f32) -> f32 {
    a + b
}

#[no_mangle]
pub extern "C" fn many_args(a: u32, b: u32, c: u32, d: u32, e: u32, f: u32) -> u32 {
    a + b + c + d + e + f
}

#[no_mangle]
pub extern "C" fn sum_mixed(a: u8, b: u16, c: u32, d: u64) -> u64 {
    a as u64 + b as u64 + c as u64 + d
}

#[no_mangle]
pub extern "C" fn pass_ptr(ptr: *const u32) -> u32 {
    unsafe { *ptr }
}

#[no_mangle]
pub extern "C" fn write_ptr(ptr: *mut u32, val: u32) {
    unsafe { *ptr = val };
}

#[no_mangle]
pub extern "C" fn sum_array_3(arr: *const u32, len: usize) -> u32 {
    let mut sum = 0u32;
    for i in 0..len {
        sum += unsafe { *arr.add(i) };
    }
    sum
}

#[no_mangle]
pub extern "C" fn sum_array_u8(arr: *const u8, len: usize) -> u32 {
    let mut sum = 0u32;
    for i in 0..len {
        sum += unsafe { *arr.add(i) } as u32;
    }
    sum
}

#[no_mangle]
pub extern "C" fn sum_array_u64(arr: *const u64, len: usize) -> u64 {
    let mut sum = 0u64;
    for i in 0..len {
        sum += unsafe { *arr.add(i) };
    }
    sum
}

#[no_mangle]
pub extern "C" fn fill_array(arr: *mut u32, len: usize, val: u32) {
    for i in 0..len {
        unsafe { *arr.add(i) = val };
    }
}

#[no_mangle]
pub extern "C" fn sum_val_array_small(arr: [u32; 3]) -> u32 {
    arr[0] + arr[1] + arr[2]
}

#[no_mangle]
pub extern "C" fn sum_val_array_u8(arr: [u8; 4]) -> u32 {
    arr[0] as u32 + arr[1] as u32 + arr[2] as u32 + arr[3] as u32
}

#[no_mangle]
pub extern "C" fn sum_val_array_large(arr: [u64; 4]) -> u64 {
    arr[0] + arr[1] + arr[2] + arr[3]
}

#[no_mangle]
pub extern "C" fn make_array_small() -> [u32; 3] {
    [10, 20, 30]
}

#[no_mangle]
pub extern "C" fn make_array_large() -> [u64; 4] {
    [100, 200, 300, 400]
}
