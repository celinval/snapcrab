/// Native library testing struct pass/return modes.
///
/// C ABI on x86-64:
/// - Structs ≤ 16 bytes with integer fields → passed/returned in registers
/// - Structs > 16 bytes → passed/returned indirectly (hidden pointer)

#[repr(C)]
pub struct Pair {
    pub a: u32,
    pub b: u32,
}

#[repr(C)]
pub struct PairMixed {
    pub x: u64,
    pub y: u8,
}

#[repr(C)]
pub struct Triple {
    pub a: u64,
    pub b: u64,
    pub c: u64,
}

#[repr(C)]
pub struct Quad {
    pub a: u32,
    pub b: u32,
    pub c: u32,
    pub d: u32,
    pub e: u32,
}

#[repr(C)]
pub struct WithPtr {
    pub ptr: *const u32,
    pub len: usize,
}

// --- Small structs (register-passed) ---

#[no_mangle]
pub extern "C" fn make_pair(a: u32, b: u32) -> Pair {
    Pair { a, b }
}

#[no_mangle]
pub extern "C" fn swap_pair(p: Pair) -> Pair {
    Pair { a: p.b, b: p.a }
}

#[no_mangle]
pub extern "C" fn sum_pair(p: Pair) -> u32 {
    p.a + p.b
}

#[no_mangle]
pub extern "C" fn make_pair_mixed(x: u64, y: u8) -> PairMixed {
    PairMixed { x, y }
}

#[no_mangle]
pub extern "C" fn read_pair_mixed(p: PairMixed) -> u64 {
    p.x + p.y as u64
}

// --- Large structs (indirect pass/return) ---

#[no_mangle]
pub extern "C" fn make_triple(a: u64, b: u64, c: u64) -> Triple {
    Triple { a, b, c }
}

#[no_mangle]
pub extern "C" fn sum_triple(t: Triple) -> u64 {
    t.a + t.b + t.c
}

#[no_mangle]
pub extern "C" fn make_quad(a: u32, b: u32, c: u32, d: u32, e: u32) -> Quad {
    Quad { a, b, c, d, e }
}

#[no_mangle]
pub extern "C" fn sum_quad(q: Quad) -> u32 {
    q.a + q.b + q.c + q.d + q.e
}

// --- Pointer-containing struct ---

#[no_mangle]
pub extern "C" fn read_with_ptr(w: WithPtr) -> u32 {
    if w.ptr.is_null() || w.len == 0 {
        return 0;
    }
    unsafe { *w.ptr }
}

#[no_mangle]
pub extern "C" fn make_with_ptr(ptr: *const u32, len: usize) -> WithPtr {
    WithPtr { ptr, len }
}
