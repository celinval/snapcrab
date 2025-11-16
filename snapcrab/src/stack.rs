use smallvec::{SmallVec, smallvec};

/// Index type for local variables in a function's stack frame.
#[allow(dead_code)]
pub type Local = usize;

/// Type information for values during interpretation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValueType {
    /// Signed integer types
    Int,
    /// Unsigned integer types  
    Uint,
    /// Boolean type
    Bool,
    /// Unit type () - zero-sized
    Unit,
}

/// Runtime value with binary representation and size information.
///
/// Uses SmallVec to avoid heap allocations for values ≤16 bytes,
/// which covers most primitive types (i128, u128, pointers, etc.).
#[derive(Debug, Clone, PartialEq)]
pub struct Value {
    /// Raw bytes - inline for values ≤16 bytes, heap for larger
    data: SmallVec<[u8; 16]>,
}

impl Value {
    /// Create a new value from raw bytes
    pub fn new(data: SmallVec<[u8; 16]>) -> Self {
        Self { data }
    }

    /// Create from signed integer
    pub fn from_i128(val: i128) -> Self {
        Self {
            data: SmallVec::from_slice(&val.to_le_bytes()),
        }
    }

    /// Create from unsigned integer
    pub fn from_u128(val: u128) -> Self {
        Self {
            data: SmallVec::from_slice(&val.to_le_bytes()),
        }
    }

    /// Create from boolean
    pub fn from_bool(val: bool) -> Self {
        Self {
            data: smallvec![val as u8],
        }
    }

    /// Get reference to the unit value (zero-sized)
    pub fn unit() -> &'static Self {
        use std::sync::LazyLock;
        static UNIT: LazyLock<Value> = LazyLock::new(|| Value {
            data: SmallVec::new(),
        });
        &UNIT
    }

    /// Extract as signed integer
    pub fn as_i128(&self) -> Option<i128> {
        if self.data.len() == 16 {
            let bytes: [u8; 16] = self.data.as_slice().try_into().ok()?;
            Some(i128::from_le_bytes(bytes))
        } else {
            None
        }
    }

    /// Extract as unsigned integer
    pub fn as_u128(&self) -> Option<u128> {
        if self.data.len() == 16 {
            let bytes: [u8; 16] = self.data.as_slice().try_into().ok()?;
            Some(u128::from_le_bytes(bytes))
        } else {
            None
        }
    }

    /// Extract as boolean
    pub fn as_bool(&self) -> Option<bool> {
        if self.data.len() == 1 {
            Some(self.data[0] != 0)
        } else {
            None
        }
    }

    /// Check if this is a unit value
    pub fn is_unit(&self) -> bool {
        self.data.is_empty()
    }

    /// Get raw bytes
    pub fn bytes(&self) -> &[u8] {
        &self.data
    }

    /// Get size in bytes
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

/// Stack frame containing local variable slots for a function.
///
/// Each slot can either contain a value (Some) or be uninitialized (None).
/// The frame is indexed by local variable indices from the MIR.
pub type StackFrame = Vec<Option<Value>>;
