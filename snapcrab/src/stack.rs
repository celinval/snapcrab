#[allow(dead_code)]
pub type Local = usize;

#[derive(Debug, Clone, Copy)]
pub enum Value {
    I32(i32),
    Bool(bool),
    Unit,
}

pub type StackFrame = Vec<Option<Value>>;
