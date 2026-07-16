/// A simple dependency crate compiled as dylib+rlib.
/// Non-generic functions are called natively; generic/inline ones get interpreted.

pub fn add(a: u32, b: u32) -> u32 {
    a + b
}

pub fn multiply(a: u64, b: u64) -> u64 {
    a * b
}

pub fn negate(x: i32) -> i32 {
    -x
}

pub fn subtract(a: u32, b: u32) -> u32 {
    a - b
}

/// This is generic — its MIR will be monomorphized in the caller crate
/// and interpreted, not native-called.
pub fn generic_add<T: std::ops::Add<Output = T>>(a: T, b: T) -> T {
    a + b
}

/// Inline function — MIR is available in the rlib metadata.
#[inline]
pub fn inline_double(x: u32) -> u32 {
    x * 2
}

pub fn sum_slice(data: &[u32]) -> u32 {
    let mut sum = 0;
    let mut i = 0;
    while i < data.len() {
        sum += data[i];
        i += 1;
    }
    sum
}
