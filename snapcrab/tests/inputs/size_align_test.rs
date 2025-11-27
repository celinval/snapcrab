struct SimpleStruct {
    a: u8,
    b: u32,
}

pub fn check_size_of() -> bool {
    let s1 = std::mem::size_of::<i32>() == 4;
    let s2 = std::mem::size_of::<u64>() == 8;
    let s3 = std::mem::size_of::<bool>() == 1;
    let s4 = std::mem::size_of::<(u8, u32)>() == 8;
    let s5 = std::mem::size_of::<SimpleStruct>() == 8;
    let s6 = std::mem::size_of::<[i32; 5]>() == 20;
    let s7 = std::mem::size_of::<[u8; 10]>() == 10;
    s1 && s2 && s3 && s4 && s5 && s6 && s7
}

pub fn check_align_of() -> bool {
    let a1 = std::mem::align_of::<i32>() == 4;
    let a2 = std::mem::align_of::<u64>() == 8;
    let a3 = std::mem::align_of::<bool>() == 1;
    let a4 = std::mem::align_of::<(u8, u32)>() == 4;
    let a5 = std::mem::align_of::<SimpleStruct>() == 4;
    let a6 = std::mem::align_of::<[i32; 5]>() == 4;
    let a7 = std::mem::align_of::<[u8; 10]>() == 1;
    a1 && a2 && a3 && a4 && a5 && a6 && a7
}
