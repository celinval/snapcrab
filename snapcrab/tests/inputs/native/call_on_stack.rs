#![allow(unused)]

use dep_on_stack::SmallStruct;

pub fn test_many_args_then_struct() {
    let s = SmallStruct { a: 10, b: 20 };
    let result = dep_on_stack::many_args_then_struct(1, 2, 3, 4, 5, 6, s);
    assert!(result == 51); // 1+2+3+4+5+6+10+20
}
