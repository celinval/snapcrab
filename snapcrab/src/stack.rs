/// Index type for local variables in a function's stack frame.
#[allow(dead_code)]
pub type Local = usize;

/// Runtime values that can be stored and manipulated by the interpreter.
///
/// This enum represents all the basic value types that the interpreter
/// can handle during execution.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    /// Signed integer value (covers i8, i16, i32, i64, i128, isize)
    Int(i128),
    /// Unsigned integer value (covers u8, u16, u32, u64, u128, usize)
    Uint(u128),
    /// Boolean value
    Bool(bool),
    /// Unit value () - represents zero-sized types
    Unit,
}

/// Stack frame containing local variable slots for a function.
///
/// Each slot can either contain a value (Some) or be uninitialized (None).
/// The frame is indexed by local variable indices from the MIR.
pub type StackFrame = Vec<Option<Value>>;
