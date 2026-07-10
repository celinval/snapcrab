#![allow(unused)]

/// Unit-only enum (C-like) with implicit discriminants.
pub enum Direction {
    North,
    South,
    East,
    West,
}

/// Fieldless enum with explicit discriminants.
pub enum HttpStatus {
    Ok = 200,
    NotFound = 404,
    InternalError = 500,
}

/// Enum with non-contiguous explicit discriminants and gaps.
pub enum Sparse {
    A = 1,
    B = 5,
    C = 100,
}

/// Enum starting at a negative discriminant.
pub enum Signed {
    Neg = -3,
    Zero = 0,
    Pos = 7,
}

/// Single-variant fieldless enum.
pub enum Single {
    Only,
}

pub fn test_direction_discriminants() {
    let n = Direction::North as i32;
    let s = Direction::South as i32;
    let e = Direction::East as i32;
    let w = Direction::West as i32;
    if n != 0 { panic!() }
    if s != 1 { panic!() }
    if e != 2 { panic!() }
    if w != 3 { panic!() }
}

pub fn test_http_status_discriminants() {
    let ok = HttpStatus::Ok as i32;
    let nf = HttpStatus::NotFound as i32;
    let ie = HttpStatus::InternalError as i32;
    if ok != 200 { panic!() }
    if nf != 404 { panic!() }
    if ie != 500 { panic!() }
}

pub fn test_sparse_discriminants() {
    let a = Sparse::A as i32;
    let b = Sparse::B as i32;
    let c = Sparse::C as i32;
    if a != 1 { panic!() }
    if b != 5 { panic!() }
    if c != 100 { panic!() }
}

pub fn test_signed_discriminants() {
    let neg = Signed::Neg as i32;
    let zero = Signed::Zero as i32;
    let pos = Signed::Pos as i32;
    if neg != -3 { panic!() }
    if zero != 0 { panic!() }
    if pos != 7 { panic!() }
}

pub fn test_single_variant() {
    let _s = Single::Only;
}

pub fn test_match_direction() {
    let d = Direction::West;
    let val = match d {
        Direction::North => 10,
        Direction::South => 20,
        Direction::East => 30,
        Direction::West => 40,
    };
    if val != 40 { panic!() }
}

pub fn test_match_http_status() {
    let s = HttpStatus::NotFound;
    let val = match s {
        HttpStatus::Ok => 1,
        HttpStatus::NotFound => 2,
        HttpStatus::InternalError => 3,
    };
    if val != 2 { panic!() }
}
