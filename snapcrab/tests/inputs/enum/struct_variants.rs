#![allow(unused)]

/// Enum with struct (named-field) variants.
pub enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write { text_len: u32 },
    Color { r: u8, g: u8, b: u8 },
}

/// Enum with a single struct variant.
pub enum SingleStruct {
    Point { x: i32, y: i32 },
}

pub fn test_message_quit() {
    let m = Message::Quit;
    let val = match m {
        Message::Quit => 0,
        Message::Move { x, y } => x + y,
        Message::Write { text_len } => text_len as i32,
        Message::Color { r, g, b } => (r as i32) + (g as i32) + (b as i32),
    };
    if val != 0 { panic!() }
}

pub fn test_message_move() {
    let m = Message::Move { x: 10, y: -3 };
    let val = match m {
        Message::Quit => 0,
        Message::Move { x, y } => x + y,
        Message::Write { text_len } => text_len as i32,
        Message::Color { r, g, b } => (r as i32) + (g as i32) + (b as i32),
    };
    if val != 7 { panic!() }
}

pub fn test_message_write() {
    let m = Message::Write { text_len: 42 };
    let val = match m {
        Message::Quit => 0,
        Message::Move { x, y } => x + y,
        Message::Write { text_len } => text_len as i32,
        Message::Color { r, g, b } => (r as i32) + (g as i32) + (b as i32),
    };
    if val != 42 { panic!() }
}

pub fn test_message_color() {
    let m = Message::Color { r: 255, g: 128, b: 0 };
    let val = match m {
        Message::Quit => 0,
        Message::Move { x, y } => x + y,
        Message::Write { text_len } => text_len as i32,
        Message::Color { r, g, b } => (r as i32) + (g as i32) + (b as i32),
    };
    if val != 383 { panic!() }
}

pub fn test_single_struct_variant() {
    let p = SingleStruct::Point { x: 5, y: 12 };
    let sum = match p {
        SingleStruct::Point { x, y } => x + y,
    };
    if sum != 17 { panic!() }
}
