#![allow(unused)]

use std::mem;

/// repr(u8) fieldless enum.
#[repr(u8)]
pub enum ReprU8 {
    A,
    B,
    C,
}

/// repr(u16) with explicit discriminants.
#[repr(u16)]
pub enum ReprU16 {
    X = 1000,
    Y = 2000,
    Z = 3000,
}

/// repr(i8) with negative discriminants.
#[repr(i8)]
pub enum ReprI8 {
    Neg = -128,
    Zero = 0,
    Pos = 127,
}

/// repr(C) fieldless enum (same as repr(i32) for fieldless).
#[repr(C)]
pub enum ReprC {
    First,
    Second,
    Third,
}

/// repr(u8) enum with tuple variant (tag + payload).
#[repr(u8)]
pub enum ReprU8Data {
    Empty,
    Val(u32),
    Pair(u16, u16),
}

/// repr(C) enum with data variants.
#[repr(C)]
pub enum ReprCData {
    None,
    Some(u64),
}

pub fn test_repr_u8_size() {
    let size = mem::size_of::<ReprU8>();
    if size != 1 { panic!() }
}

pub fn test_repr_u8_discriminants() {
    let a = ReprU8::A as u8;
    let b = ReprU8::B as u8;
    let c = ReprU8::C as u8;
    if a != 0 { panic!() }
    if b != 1 { panic!() }
    if c != 2 { panic!() }
}

pub fn test_repr_u16_size() {
    let size = mem::size_of::<ReprU16>();
    if size != 2 { panic!() }
}

pub fn test_repr_u16_discriminants() {
    let x = ReprU16::X as u16;
    let y = ReprU16::Y as u16;
    let z = ReprU16::Z as u16;
    if x != 1000 { panic!() }
    if y != 2000 { panic!() }
    if z != 3000 { panic!() }
}

pub fn test_repr_i8_discriminants() {
    let neg = ReprI8::Neg as i8;
    let zero = ReprI8::Zero as i8;
    let pos = ReprI8::Pos as i8;
    if neg != -128 { panic!() }
    if zero != 0 { panic!() }
    if pos != 127 { panic!() }
}

pub fn test_repr_c_size() {
    // repr(C) fieldless enum is at least i32 sized
    let size = mem::size_of::<ReprC>();
    if size != 4 { panic!() }
}

pub fn test_repr_c_discriminants() {
    let f = ReprC::First as i32;
    let s = ReprC::Second as i32;
    let t = ReprC::Third as i32;
    if f != 0 { panic!() }
    if s != 1 { panic!() }
    if t != 2 { panic!() }
}

pub fn test_repr_u8_data_match() {
    let empty = ReprU8Data::Empty;
    let val = ReprU8Data::Val(1000);
    let pair = ReprU8Data::Pair(10, 20);

    let r1 = match empty {
        ReprU8Data::Empty => 0u32,
        ReprU8Data::Val(v) => v,
        ReprU8Data::Pair(a, b) => (a + b) as u32,
    };
    if r1 != 0 { panic!() }

    let r2 = match val {
        ReprU8Data::Empty => 0u32,
        ReprU8Data::Val(v) => v,
        ReprU8Data::Pair(a, b) => (a + b) as u32,
    };
    if r2 != 1000 { panic!() }

    let r3 = match pair {
        ReprU8Data::Empty => 0u32,
        ReprU8Data::Val(v) => v,
        ReprU8Data::Pair(a, b) => (a + b) as u32,
    };
    if r3 != 30 { panic!() }
}

pub fn test_repr_c_data_match() {
    let none = ReprCData::None;
    let some = ReprCData::Some(0xDEAD_BEEF);

    let r1 = match none {
        ReprCData::None => 0u64,
        ReprCData::Some(v) => v,
    };
    if r1 != 0 { panic!() }

    let r2 = match some {
        ReprCData::None => 0u64,
        ReprCData::Some(v) => v,
    };
    if r2 != 0xDEAD_BEEF { panic!() }
}

pub fn test_repr_u8_data_size() {
    // tag is u8, payload is max(u32, (u16,u16)) = 4 bytes, align 4
    // total: 4 (align) + 4 (payload) = 8
    let size = mem::size_of::<ReprU8Data>();
    if size != 8 { panic!() }
}

pub fn test_repr_c_data_size() {
    // repr(C): tag is i32 (4 bytes), padding to align 8, payload u64 (8 bytes) = 16
    let size = mem::size_of::<ReprCData>();
    if size != 16 { panic!() }
}
