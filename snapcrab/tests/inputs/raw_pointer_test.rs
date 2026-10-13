pub fn ptr_compare() -> bool {
    let x = 42;
    let ptr1 = &x as *const i32;
    let ptr2 = &x as *const i32;
    ptr1 == ptr2
}

#[allow(dangling_pointers_from_locals)]
pub fn use_dangling_ptr() -> i32 {
    fn dangling_ptr() -> *const i32 {
        let x = 99;
        &x as *const i32
    }

    let ptr = dangling_ptr();
    unsafe { *ptr }
}

/// Reading a `u32` one *byte* past an aligned base is misaligned, which is UB.
/// The offset has to be applied as bytes: offsetting a `*const u32` by whole
/// elements would keep the address 4-byte aligned.
pub fn read_misaligned_ptr() -> u32 {
    let x: i128 = 0;
    let bytes = &x as *const i128 as *const u8;
    let misaligned = unsafe { bytes.add(1) } as *const u32;
    unsafe { *misaligned }
}
