#![allow(unused)]

pub struct Inner {
    x: i32,
    y: i32,
}

pub struct Outer {
    inner: Inner,
    z: bool,
}

pub fn create_nested() -> Outer {
    Outer {
        inner: Inner { x: 10, y: 20 },
        z: true,
    }
}
