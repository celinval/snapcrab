#![allow(unused)]

pub fn test_result_ok() {
    let x: Result<u32, u32> = Ok(10);
    assert!(x.is_ok());
    assert!(!x.is_err());
}

pub fn test_result_err() {
    let x: Result<u32, u32> = Err(99);
    assert!(x.is_err());
    assert!(!x.is_ok());
}

pub fn test_result_unwrap() {
    let x: Result<u32, u32> = Ok(42);
    assert!(x.unwrap() == 42);
}

pub fn test_result_unwrap_or() {
    let x: Result<u32, u32> = Err(5);
    assert!(x.unwrap_or(0) == 0);
}

pub fn test_result_map() {
    let x: Result<u32, u32> = Ok(3);
    let y = x.map(|v| v + 10);
    assert!(y == Ok(13));
}

pub fn test_result_and_then() {
    let x: Result<u32, u32> = Ok(2);
    let y = x.and_then(|v| if v > 1 { Ok(v * 10) } else { Err(0) });
    assert!(y == Ok(20));
}
