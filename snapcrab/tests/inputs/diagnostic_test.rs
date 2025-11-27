#[inline(never)]
pub fn div_by_zero() -> i32 {
    division(0)
}

#[inline(never)]
fn division(n: i32) -> i32 {
    10 / n
}

pub fn error_deep_call() {
    let _unused = tick_bomb(10);
}

fn tick_bomb(n: i32) -> i32 {
    let _ = division(n);
    if n > 0 {
        tick_bomb(n.wrapping_sub(1))
    } else {
        0
    }
}
