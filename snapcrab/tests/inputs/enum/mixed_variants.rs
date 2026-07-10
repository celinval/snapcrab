#![allow(unused)]

/// Enum mixing unit, tuple, and struct variants.
pub enum Event {
    Nothing,
    Click(i32, i32),
    KeyPress { code: u32 },
    Resize { width: u32, height: u32 },
}

/// Enum with nested enum inside a variant.
pub enum Outer {
    Empty,
    Inner(Inner),
}

pub enum Inner {
    A,
    B(u32),
}

pub fn test_event_nothing() {
    let e = Event::Nothing;
    let val = match e {
        Event::Nothing => 0u32,
        Event::Click(x, y) => (x + y) as u32,
        Event::KeyPress { code } => code,
        Event::Resize { width, height } => width * height,
    };
    if val != 0 { panic!() }
}

pub fn test_event_click() {
    let e = Event::Click(100, 200);
    let val = match e {
        Event::Nothing => 0u32,
        Event::Click(x, y) => (x + y) as u32,
        Event::KeyPress { code } => code,
        Event::Resize { width, height } => width * height,
    };
    if val != 300 { panic!() }
}

pub fn test_event_keypress() {
    let e = Event::KeyPress { code: 65 };
    let val = match e {
        Event::Nothing => 0u32,
        Event::Click(x, y) => (x + y) as u32,
        Event::KeyPress { code } => code,
        Event::Resize { width, height } => width * height,
    };
    if val != 65 { panic!() }
}

pub fn test_event_resize() {
    let e = Event::Resize { width: 1920, height: 1080 };
    let val = match e {
        Event::Nothing => 0u32,
        Event::Click(x, y) => (x + y) as u32,
        Event::KeyPress { code } => code,
        Event::Resize { width, height } => width * height,
    };
    if val != 2073600 { panic!() }
}

pub fn test_nested_outer_empty() {
    let o = Outer::Empty;
    let val = match o {
        Outer::Empty => 0u32,
        Outer::Inner(inner) => match inner {
            Inner::A => 1,
            Inner::B(v) => v,
        },
    };
    if val != 0 { panic!() }
}

pub fn test_nested_inner_a() {
    let o = Outer::Inner(Inner::A);
    let val = match o {
        Outer::Empty => 0u32,
        Outer::Inner(inner) => match inner {
            Inner::A => 1,
            Inner::B(v) => v,
        },
    };
    if val != 1 { panic!() }
}

pub fn test_nested_inner_b() {
    let o = Outer::Inner(Inner::B(99));
    let val = match o {
        Outer::Empty => 0u32,
        Outer::Inner(inner) => match inner {
            Inner::A => 1,
            Inner::B(v) => v,
        },
    };
    if val != 99 { panic!() }
}
