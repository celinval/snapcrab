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

pub fn read_nested_field() -> i32 {
    let outer = Outer {
        inner: Inner { x: 5, y: 15 },
        z: false,
    };
    outer.inner.x
}

pub fn write_nested_field() -> Outer {
    let mut outer = Outer {
        inner: Inner { x: 1, y: 2 },
        z: true,
    };
    outer.inner.y = 99;
    outer
}

pub fn struct_to_tuple() -> (i32, i32, bool) {
    let outer = Outer {
        inner: Inner { x: 7, y: 8 },
        z: true,
    };
    (outer.inner.x, outer.inner.y, outer.z)
}
