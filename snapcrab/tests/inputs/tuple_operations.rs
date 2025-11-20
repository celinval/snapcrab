fn simple_tuple() -> (u8, bool, u32) {
    (42, true, 1000)
}

fn nested_tuple() -> ((i16, u8), bool) {
    ((-100, 255), false)
}

fn large_tuple() -> (u64, i32, bool, u16, i8) {
    (18446744073709551615, -2147483648, true, 65535, -128)
}

fn unit_tuple() -> () {
    ()
}

fn single_element_tuple() -> (i32,) {
    (42,)
}

// Different order - same types as simple_tuple but reordered
fn reordered_tuple() -> (u32, u8, bool) {
    (1000, 42, true)
}

// Another reordering
fn another_order() -> (bool, u32, u8) {
    (true, 1000, 42)
}

fn main() {
    println!("Tuple operations test");
}
