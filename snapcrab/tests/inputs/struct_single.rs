#![allow(unused)]

pub struct Single {
    value: i32,
}

pub fn create_single() -> Single {
    Single { value: 42 }
}

pub fn read_field() -> i32 {
    let s = Single { value: 100 };
    s.value
}

pub fn write_field() -> Single {
    let mut s = Single { value: 10 };
    s.value = 50;
    s
}
