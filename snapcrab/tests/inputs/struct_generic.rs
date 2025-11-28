#![allow(unused)]

pub struct Triple<A, B, C> {
    first: A,
    second: B,
    third: C,
}

pub fn create_triple_u8_u128_i16() -> Triple<u8, u128, i16> {
    Triple {
        first: 10u8,
        second: 1000u128,
        third: -50i16,
    }
}

pub fn create_triple_i32_unit_bool() -> Triple<i32, (), bool> {
    Triple {
        first: 42i32,
        second: (),
        third: true,
    }
}
