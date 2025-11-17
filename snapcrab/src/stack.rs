use crate::value::Value;

/// Stack frame for function execution.
///
/// Contains local variable slots indexed by Local. Each slot can be None (uninitialized)
/// or Some(Value) (initialized with a value).
pub type StackFrame = Vec<Option<Value>>;
