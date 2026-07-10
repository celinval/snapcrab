#![allow(unused)]

use std::mem;
use std::num::NonZeroU32;
use std::num::NonZeroU8;
use std::num::NonZeroUsize;

/// Niche optimization: Option<NonZeroU32> should be same size as u32.
pub fn test_option_nonzero_u32_size() {
    let size_opt = mem::size_of::<Option<NonZeroU32>>();
    let size_u32 = mem::size_of::<u32>();
    if size_opt != size_u32 { panic!() }
}

/// Niche optimization: Option<NonZeroU8> should be same size as u8.
pub fn test_option_nonzero_u8_size() {
    let size_opt = mem::size_of::<Option<NonZeroU8>>();
    let size_u8 = mem::size_of::<u8>();
    if size_opt != size_u8 { panic!() }
}

/// Niche optimization: Option<NonZeroUsize> should be same size as usize.
pub fn test_option_nonzero_usize_size() {
    let size_opt = mem::size_of::<Option<NonZeroUsize>>();
    let size_usize = mem::size_of::<usize>();
    if size_opt != size_usize { panic!() }
}

/// Niche optimization: Option<bool> should be 1 byte (bool has niche at 2..=255).
pub fn test_option_bool_size() {
    let size = mem::size_of::<Option<bool>>();
    if size != 1 { panic!() }
}

/// Niche optimization: Option<&T> should be same size as &T (null niche).
pub fn test_option_ref_size() {
    let size_opt = mem::size_of::<Option<&u32>>();
    let size_ref = mem::size_of::<&u32>();
    if size_opt != size_ref { panic!() }
}

/// Double niche: Option<Option<bool>> should be 1 byte.
pub fn test_option_option_bool_size() {
    let size = mem::size_of::<Option<Option<bool>>>();
    if size != 1 { panic!() }
}

/// Option<NonZeroU32> with Some value.
pub fn test_option_nonzero_some() {
    let val = NonZeroU32::new(42);
    let result = match val {
        Some(v) => v.get(),
        None => 0,
    };
    if result != 42 { panic!() }
}

/// Option<NonZeroU32> with None (from zero input).
pub fn test_option_nonzero_none() {
    let val = NonZeroU32::new(0);
    let result = match val {
        Some(v) => v.get(),
        None => 0,
    };
    if result != 0 { panic!() }
}

/// Option<bool> values.
pub fn test_option_bool_values() {
    let some_true: Option<bool> = Some(true);
    let some_false: Option<bool> = Some(false);
    let none: Option<bool> = None;

    let r1 = match some_true {
        Some(true) => 1u32,
        Some(false) => 2,
        None => 3,
    };
    if r1 != 1 { panic!() }

    let r2 = match some_false {
        Some(true) => 1u32,
        Some(false) => 2,
        None => 3,
    };
    if r2 != 2 { panic!() }

    let r3 = match none {
        Some(true) => 1u32,
        Some(false) => 2,
        None => 3,
    };
    if r3 != 3 { panic!() }
}

/// Option<&T> with Some reference.
pub fn test_option_ref_some() {
    let x = 99u32;
    let opt: Option<&u32> = Some(&x);
    let val = match opt {
        Some(r) => *r,
        None => 0,
    };
    if val != 99 { panic!() }
}

/// Option<&T> with None.
pub fn test_option_ref_none() {
    let opt: Option<&u32> = None;
    let val = match opt {
        Some(r) => *r,
        None => 0,
    };
    if val != 0 { panic!() }
}

/// Nested Option with niche: Option<Option<NonZeroU32>>.
pub fn test_nested_option_nonzero() {
    let size = mem::size_of::<Option<Option<NonZeroU32>>>();
    let size_u32 = mem::size_of::<u32>();
    // Option<Option<NonZeroU32>> fits in u32 due to two niche values (0, max or similar)
    // Actually, Option<NonZeroU32> = u32, and Option<Option<NonZeroU32>> = 8 bytes
    // because Option<NonZeroU32> only has one niche value used.
    // Let's just test the matching works correctly.
    let inner_some = Some(NonZeroU32::new(10).unwrap());
    let outer_some: Option<Option<NonZeroU32>> = Some(inner_some);
    let outer_none: Option<Option<NonZeroU32>> = None;
    let inner_none: Option<Option<NonZeroU32>> = Some(None);

    let r1 = match outer_some {
        Some(Some(v)) => v.get(),
        Some(None) => 0,
        None => 999,
    };
    if r1 != 10 { panic!() }

    let r2 = match outer_none {
        Some(Some(v)) => v.get(),
        Some(None) => 0,
        None => 999,
    };
    if r2 != 999 { panic!() }

    let r3 = match inner_none {
        Some(Some(v)) => v.get(),
        Some(None) => 0,
        None => 999,
    };
    if r3 != 0 { panic!() }
}
