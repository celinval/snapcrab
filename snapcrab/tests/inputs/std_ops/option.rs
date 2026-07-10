#![allow(unused)]

pub fn test_option_unwrap() {
    let x: Option<u32> = Some(42);
    assert!(x.unwrap() == 42);
}

pub fn test_option_unwrap_or() {
    let x: Option<u32> = None;
    assert!(x.unwrap_or(10) == 10);
}

pub fn test_option_map() {
    let x: Option<u32> = Some(5);
    let y = x.map(|v| v * 2);
    assert!(y == Some(10));
}

pub fn test_option_and_then() {
    let x: Option<u32> = Some(10);
    let y = x.and_then(|v| if v > 5 { Some(v + 1) } else { None });
    assert!(y == Some(11));
}

pub fn test_option_and_then_none() {
    let x: Option<u32> = Some(3);
    let y = x.and_then(|v| if v > 5 { Some(v + 1) } else { None });
    assert!(y.is_none());
}

pub fn test_option_is_some_and() {
    let x: Option<u32> = Some(42);
    assert!(x.is_some_and(|v| v > 10));
}
