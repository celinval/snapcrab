fn add(a: u32, b: u32) -> u32 {
    a + b
}

fn main() {
    // Wrong on purpose: the interpreter should report this as a panic.
    assert!(add(2, 3) == 6);
}
