#![allow(unused)]

pub enum Direction {
    North,
    South,
    East,
    West,
}

pub enum Option<T> {
    None,
    Some(T),
}

pub enum Either {
    Left(i32),
    Right(u8),
}

pub enum Multi {
    Empty,
    Single(i32),
    Pair(i32, i32),
}

// --- C-like enum tests ---

pub fn create_north() -> Direction {
    Direction::North
}

pub fn create_west() -> Direction {
    Direction::West
}

pub fn direction_to_int(d: Direction) -> i32 {
    match d {
        Direction::North => 0,
        Direction::South => 1,
        Direction::East => 2,
        Direction::West => 3,
    }
}

pub fn match_north() -> i32 {
    let d = Direction::North;
    direction_to_int(d)
}

pub fn match_west() -> i32 {
    let d = Direction::West;
    direction_to_int(d)
}

// --- Option-like enum tests ---

pub fn create_some() -> Option<i32> {
    Option::Some(42)
}

pub fn create_none() -> Option<i32> {
    Option::None
}

pub fn unwrap_some() -> i32 {
    let opt = Option::Some(99);
    match opt {
        Option::Some(v) => v,
        Option::None => -1,
    }
}

pub fn unwrap_none() -> i32 {
    let opt: Option<i32> = Option::None;
    match opt {
        Option::Some(v) => v,
        Option::None => -1,
    }
}

// --- Either enum tests ---

pub fn create_left() -> Either {
    Either::Left(10)
}

pub fn create_right() -> Either {
    Either::Right(255)
}

pub fn match_left() -> i32 {
    let e = Either::Left(7);
    match e {
        Either::Left(v) => v,
        Either::Right(b) => b as i32,
    }
}

pub fn match_right() -> i32 {
    let e = Either::Right(200);
    match e {
        Either::Left(v) => v,
        Either::Right(b) => b as i32,
    }
}

// --- Multi-variant tests ---

pub fn create_empty() -> Multi {
    Multi::Empty
}

pub fn create_single() -> Multi {
    Multi::Single(77)
}

pub fn create_pair() -> Multi {
    Multi::Pair(3, 4)
}

pub fn match_empty() -> i32 {
    let m = Multi::Empty;
    match m {
        Multi::Empty => 0,
        Multi::Single(v) => v,
        Multi::Pair(a, b) => a + b,
    }
}

pub fn match_single() -> i32 {
    let m = Multi::Single(55);
    match m {
        Multi::Empty => 0,
        Multi::Single(v) => v,
        Multi::Pair(a, b) => a + b,
    }
}

pub fn match_pair() -> i32 {
    let m = Multi::Pair(10, 20);
    match m {
        Multi::Empty => 0,
        Multi::Single(v) => v,
        Multi::Pair(a, b) => a + b,
    }
}
