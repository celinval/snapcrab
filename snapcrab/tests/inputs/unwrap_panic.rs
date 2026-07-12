#![allow(unused)]
pub fn unwrap_none() -> u32 {
    let x: Option<u32> = None;
    x.unwrap()
}
