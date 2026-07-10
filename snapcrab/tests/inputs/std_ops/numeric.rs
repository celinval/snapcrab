#![allow(unused)]

pub fn test_u32_min_max() {
    assert!(u32::MIN == 0);
    assert!(u32::MAX == 4294967295);
}

pub fn test_i32_min_max() {
    assert!(i32::MIN == -2147483648);
    assert!(i32::MAX == 2147483647);
}

pub fn test_wrapping_add() {
    let x: u8 = 250;
    assert!(x.wrapping_add(10) == 4);
}

pub fn test_wrapping_sub() {
    let x: u8 = 5;
    assert!(x.wrapping_sub(10) == 251);
}

pub fn test_saturating_add() {
    let x: u8 = 250;
    assert!(x.saturating_add(10) == 255);
}

pub fn test_saturating_sub() {
    let x: u8 = 5;
    assert!(x.saturating_sub(10) == 0);
}

pub fn test_checked_add_some() {
    let x: u8 = 100;
    assert!(x.checked_add(50) == Some(150));
}

pub fn test_checked_add_overflow() {
    let x: u8 = 250;
    assert!(x.checked_add(10).is_none());
}

pub fn test_pow() {
    assert!(2u32.pow(10) == 1024);
}

pub fn test_leading_zeros() {
    assert!(1u32.leading_zeros() == 31);
    assert!(0u32.leading_zeros() == 32);
}

pub fn test_count_ones() {
    assert!(0b1010_1010u8.count_ones() == 4);
}

pub fn test_swap_bytes() {
    assert!(0x1234u16.swap_bytes() == 0x3412);
}

pub fn test_rotate_left() {
    assert!(0x12u8.rotate_left(4) == 0x21);
}
