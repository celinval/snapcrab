#[allow(dead_code)]
pub type Local = usize;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    Int(i128),    // All signed integers
    Uint(u128),   // All unsigned integers
    Bool(bool),
    Unit,
}

pub type StackFrame = Vec<Option<Value>>;
