pub fn simple_tuple() -> (u8, bool, u32) {
    (42, true, 1000)
}

pub fn nested_tuple() -> ((i16, u8), bool) {
    ((-100, 255), false)
}

pub fn large_tuple() -> (u64, i32, bool, u16, i8) {
    (18446744073709551615, -2147483648, true, 65535, -128)
}

pub fn unit_tuple() -> () {
    ()
}

pub fn single_element_tuple() -> (i32,) {
    (42,)
}

// Different order - same types as simple_tuple but reordered
pub fn reordered_tuple() -> (u32, u8, bool) {
    (1000, 42, true)
}

// Another reordering
pub fn another_order() -> (bool, u32, u8) {
    (true, 1000, 42)
}
