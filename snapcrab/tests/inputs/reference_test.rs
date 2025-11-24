pub fn test_basic_ref() -> i32 {
    let x = 42;
    let r = &x;
    *r
}

pub fn test_mut_ref() -> i32 {
    let mut x = 42;
    let r = &mut x;
    *r = 100;
    x
}

pub fn test_multiple_refs() -> i32 {
    let x = 10;
    let r1 = &x;
    let r2 = &x;
    *r1 + *r2
}

pub fn test_ref_to_ref() -> i32 {
    let x = 25;
    let r1 = &x;
    let r2 = &r1;
    **r2
}

pub fn test_mut_ref_chain() -> i32 {
    let mut x = 5;
    let r = &mut x;
    *r = 15;
    let r2 = &x;
    *r2
}
