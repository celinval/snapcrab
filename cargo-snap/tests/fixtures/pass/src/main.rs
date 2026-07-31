fn add(a: u32, b: u32) -> u32 {
    a + b
}

fn main() {
    assert!(add(2, 3) == 5);
}

#[test]
fn test_add() {
    assert!(add(2, 3) == 5);
}
