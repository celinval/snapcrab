pub fn returns_true() -> bool {
    true
}

/// Bitwise operators on `bool`.
pub fn bool_bit_ops() {
    let t = true;
    let f = false;
    assert!((t ^ f) == true);
    assert!((t ^ t) == false);
    assert!((t & f) == false);
    assert!((t | f) == true);
}
