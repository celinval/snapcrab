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
