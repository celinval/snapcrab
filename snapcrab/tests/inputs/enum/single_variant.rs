#![allow(unused)]

use core::mem;

/// Single-variant enum with no fields (ZST).
pub enum UnitSingle {
    Only,
}

/// Single-variant enum wrapping data (no discriminant needed).
pub enum WrapperSingle {
    Value(u64),
}

/// Single-variant enum with struct fields.
pub enum StructSingle {
    Point { x: i32, y: i32 },
}

/// Single-variant enum is same size as its payload (no tag needed).
pub fn test_wrapper_single_size() {
    let size = mem::size_of::<WrapperSingle>();
    let expected = mem::size_of::<u64>();
    if size != expected { panic!() }
}

pub fn test_unit_single_size() {
    let size = mem::size_of::<UnitSingle>();
    if size != 0 { panic!() }
}

pub fn test_struct_single_size() {
    let size = mem::size_of::<StructSingle>();
    let expected = mem::size_of::<(i32, i32)>();
    if size != expected { panic!() }
}

pub fn test_wrapper_single_match() {
    let w = WrapperSingle::Value(0xCAFE_BABE);
    let val = match w {
        WrapperSingle::Value(v) => v,
    };
    if val != 0xCAFE_BABE { panic!() }
}

pub fn test_struct_single_match() {
    let p = StructSingle::Point { x: -5, y: 10 };
    let sum = match p {
        StructSingle::Point { x, y } => x + y,
    };
    if sum != 5 { panic!() }
}

pub fn test_unit_single_match() {
    let u = UnitSingle::Only;
    match u {
        UnitSingle::Only => {}
    }
}
