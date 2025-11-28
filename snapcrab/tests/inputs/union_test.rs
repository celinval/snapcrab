#![allow(unused)]

pub union SimpleUnion {
    int_val: i32,
    float_val: f32,
}

pub fn create_union_int() -> SimpleUnion {
    SimpleUnion { int_val: 42 }
}

pub fn create_union_float() -> SimpleUnion {
    SimpleUnion { float_val: 3.14 }
}

pub union EmptyVariantUnion {
    empty: (),
    value: i32,
}

pub fn create_union_empty() -> EmptyVariantUnion {
    EmptyVariantUnion { empty: () }
}

pub fn create_union_value() -> EmptyVariantUnion {
    EmptyVariantUnion { value: 100 }
}

pub union MultiFieldUnion {
    byte: u8,
    word: u16,
    dword: u32,
}

pub fn assign_all_fields() -> u32 {
    unsafe {
        let mut u = MultiFieldUnion { byte: 0xFF };
        u.word = 0xABCD;
        u.dword = 0x12345678;
        u.dword
    }
}
