#![allow(unused)]

pub fn test_min() {
    assert!(std::cmp::min(3, 7) == 3);
    assert!(std::cmp::min(10, 2) == 2);
}

pub fn test_max() {
    assert!(std::cmp::max(3, 7) == 7);
    assert!(std::cmp::max(10, 2) == 10);
}

pub fn test_clamp() {
    assert!(5u32.clamp(0, 10) == 5);
    assert!(15u32.clamp(0, 10) == 10);
    assert!(0u32.clamp(3, 10) == 3);
}

pub fn test_ord_methods() {
    assert!(3u32.max(7) == 7);
    assert!(3u32.min(7) == 3);
}
