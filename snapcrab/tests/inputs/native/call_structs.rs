#![allow(unused)]

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

unsafe extern "C" {
    fn make_pair(a: u32, b: u32) -> Pair;
    fn swap_pair(p: Pair) -> Pair;
    fn sum_pair(p: Pair) -> u32;
    fn make_pair_mixed(x: u64, y: u8) -> PairMixed;
    fn read_pair_mixed(p: PairMixed) -> u64;
    fn make_triple(a: u64, b: u64, c: u64) -> Triple;
    fn sum_triple(t: Triple) -> u64;
    fn make_quad(a: u32, b: u32, c: u32, d: u32, e: u32) -> Quad;
    fn sum_quad(q: Quad) -> u32;
    fn read_with_ptr(w: WithPtr) -> u32;
    fn make_with_ptr(ptr: *const u32, len: usize) -> WithPtr;
}

// --- Small struct return (PassMode::Cast) ---

pub fn test_make_pair() {
    let p = unsafe { make_pair(10, 20) };
    assert!(p.a == 10);
    assert!(p.b == 20);
}

pub fn test_swap_pair() {
    let p = Pair { a: 1, b: 2 };
    let s = unsafe { swap_pair(p) };
    assert!(s.a == 2);
    assert!(s.b == 1);
}

pub fn test_sum_pair() {
    let p = Pair { a: 100, b: 200 };
    assert!(unsafe { sum_pair(p) } == 300);
}

pub fn test_make_pair_mixed() {
    let p = unsafe { make_pair_mixed(1000, 42) };
    assert!(p.x == 1000);
    assert!(p.y == 42);
}

pub fn test_read_pair_mixed() {
    let p = PairMixed { x: 500, y: 5 };
    assert!(unsafe { read_pair_mixed(p) } == 505);
}

// --- Large struct (indirect pass/return) ---

pub fn test_make_triple() {
    let t = unsafe { make_triple(10, 20, 30) };
    assert!(t.a == 10);
    assert!(t.b == 20);
    assert!(t.c == 30);
}

pub fn test_sum_triple() {
    let t = Triple { a: 100, b: 200, c: 300 };
    assert!(unsafe { sum_triple(t) } == 600);
}

pub fn test_make_quad() {
    let q = unsafe { make_quad(1, 2, 3, 4, 5) };
    assert!(q.a == 1);
    assert!(q.b == 2);
    assert!(q.c == 3);
    assert!(q.d == 4);
    assert!(q.e == 5);
}

pub fn test_sum_quad() {
    let q = Quad { a: 10, b: 20, c: 30, d: 40, e: 50 };
    assert!(unsafe { sum_quad(q) } == 150);
}

// --- Pointer-in-struct ---

pub fn test_read_with_ptr() {
    let val: u32 = 777;
    let w = WithPtr { ptr: &val as *const u32, len: 1 };
    assert!(unsafe { read_with_ptr(w) } == 777);
}

pub fn test_make_with_ptr() {
    let val: u32 = 999;
    let w = unsafe { make_with_ptr(&val as *const u32, 42) };
    assert!(w.len == 42);
}
