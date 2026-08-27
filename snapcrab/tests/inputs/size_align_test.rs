pub struct SimpleStruct {
    #[allow(dead_code)]
    a: u8,
    #[allow(dead_code)]
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

/// A type whose 32-byte alignment exceeds the system allocator's incidental
/// guarantee, so a correctly-aligned address must come from explicitly aligned
/// backing storage rather than luck. The `[u8; 32]` payload leaves no padding,
/// keeping the test focused on alignment.
#[repr(align(32))]
pub struct OverAligned([u8; 32]);

/// Read a pointer's address as an integer (`ptr as usize` lowers to an
/// unsupported `PointerExposeAddress` cast, so transmute the bits instead).
fn addr_of<T>(ptr: *const T) -> usize {
    unsafe { std::mem::transmute::<*const T, usize>(ptr) }
}

/// The runtime address of a local must satisfy its type's alignment.
pub fn local_alignment() {
    let value = OverAligned([7u8; 32]);
    let addr = addr_of(std::ptr::addr_of!(value));
    // 32 is a power of two, so alignment holds iff the low 5 bits are clear.
    assert!(addr & 31 == 0, "local address not aligned to 32 bytes");
    assert!(value.0[0] == 7);
}

static OVER_ALIGNED_STATIC: OverAligned = OverAligned([9u8; 32]);

/// The runtime address of a static must satisfy its type's alignment.
pub fn static_alignment() {
    let addr = addr_of(std::ptr::addr_of!(OVER_ALIGNED_STATIC));
    assert!(addr & 31 == 0, "static address not aligned to 32 bytes");
    assert!(OVER_ALIGNED_STATIC.0[0] == 9);
}
