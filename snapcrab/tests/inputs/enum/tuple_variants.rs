#![allow(unused)]

/// Classic Option-like enum with one unit and one tuple variant.
pub enum MyOption<T> {
    None,
    Some(T),
}

/// Enum where variants hold different tuple arities.
pub enum Shape {
    Circle(u32),
    Rectangle(u32, u32),
    Triangle(u32, u32, u32),
}

/// Enum with a single tuple variant.
pub enum Wrapper {
    Val(i64),
}

/// Enum where all variants are tuples (no fieldless variant).
pub enum AllTuples {
    One(u8),
    Two(u16, u16),
    Three(u32, u32, u32),
}

pub fn test_option_some() {
    let opt = MyOption::Some(42i32);
    let val = match opt {
        MyOption::Some(v) => v,
        MyOption::None => -1,
    };
    if val != 42 { panic!() }
}

pub fn test_option_none() {
    let opt: MyOption<i32> = MyOption::None;
    let val = match opt {
        MyOption::Some(v) => v,
        MyOption::None => -1,
    };
    if val != -1 { panic!() }
}

pub fn test_shape_circle() {
    let s = Shape::Circle(10);
    let area_approx = match s {
        Shape::Circle(r) => r * r * 3,
        Shape::Rectangle(w, h) => w * h,
        Shape::Triangle(a, b, _c) => a * b / 2,
    };
    if area_approx != 300 { panic!() }
}

pub fn test_shape_rectangle() {
    let s = Shape::Rectangle(5, 8);
    let area = match s {
        Shape::Circle(r) => r * r * 3,
        Shape::Rectangle(w, h) => w * h,
        Shape::Triangle(a, b, _c) => a * b / 2,
    };
    if area != 40 { panic!() }
}

pub fn test_shape_triangle() {
    let s = Shape::Triangle(6, 4, 5);
    let area = match s {
        Shape::Circle(r) => r * r * 3,
        Shape::Rectangle(w, h) => w * h,
        Shape::Triangle(a, b, _c) => a * b / 2,
    };
    if area != 12 { panic!() }
}

pub fn test_single_tuple_variant() {
    let w = Wrapper::Val(123456789i64);
    let val = match w {
        Wrapper::Val(v) => v,
    };
    if val != 123456789 { panic!() }
}

pub fn test_all_tuples_one() {
    let t = AllTuples::One(255u8);
    let val = match t {
        AllTuples::One(a) => a as u32,
        AllTuples::Two(a, b) => (a + b) as u32,
        AllTuples::Three(a, b, c) => a + b + c,
    };
    if val != 255 { panic!() }
}

pub fn test_all_tuples_three() {
    let t = AllTuples::Three(10, 20, 30);
    let val = match t {
        AllTuples::One(a) => a as u32,
        AllTuples::Two(a, b) => (a + b) as u32,
        AllTuples::Three(a, b, c) => a + b + c,
    };
    if val != 60 { panic!() }
}
