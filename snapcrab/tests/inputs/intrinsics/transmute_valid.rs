#![allow(unused, unnecessary_transmutes)]

use std::mem;

/// Same-size integer reinterpretation.
pub fn test_u32_to_i32() {
    let x: u32 = 0xFFFF_FFFF;
    let y: i32 = unsafe { mem::transmute(x) };
    assert!(y == -1);
}

/// Transmute between same-size arrays.
pub fn test_u8_array_to_u32() {
    let bytes: [u8; 4] = [0x78, 0x56, 0x34, 0x12];
    let val: u32 = unsafe { mem::transmute(bytes) };
    assert!(val == 0x12345678);
}

/// Transmute u32 to [u8; 4].
pub fn test_u32_to_u8_array() {
    let val: u32 = 0xDEAD_BEEF;
    let bytes: [u8; 4] = unsafe { mem::transmute(val) };
    assert!(bytes[0] == 0xEF);
    assert!(bytes[1] == 0xBE);
    assert!(bytes[2] == 0xAD);
    assert!(bytes[3] == 0xDE);
}

/// Transmute between same-layout structs.
pub fn test_struct_to_struct() {
    #[repr(C)]
    struct A {
        x: u32,
        y: u32,
    }

    #[repr(C)]
    struct B {
        a: u32,
        b: u32,
    }

    let a = A { x: 10, y: 20 };
    let b: B = unsafe { mem::transmute(a) };
    assert!(b.a == 10);
    assert!(b.b == 20);
}

/// Transmute bool to u8.
pub fn test_bool_to_u8() {
    let t: bool = true;
    let v: u8 = unsafe { mem::transmute(t) };
    assert!(v == 1);

    let f: bool = false;
    let v: u8 = unsafe { mem::transmute(f) };
    assert!(v == 0);
}

/// Transmute u8 to bool (valid values only).
pub fn test_u8_to_bool_valid() {
    let one: u8 = 1;
    let t: bool = unsafe { mem::transmute(one) };
    assert!(t);

    let zero: u8 = 0;
    let f: bool = unsafe { mem::transmute(zero) };
    assert!(!f);
}

/// Transmute between signed/unsigned of same size.
pub fn test_i64_to_u64() {
    let x: i64 = -1;
    let y: u64 = unsafe { mem::transmute(x) };
    assert!(y == u64::MAX);
}

/// Transmute unit tuple types.
pub fn test_unit_struct_transmute() {
    #[repr(C)]
    struct Marker;

    #[repr(C)]
    struct OtherMarker;

    let _m: OtherMarker = unsafe { mem::transmute(Marker) };
}
