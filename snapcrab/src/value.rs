use smallvec::{SmallVec, smallvec};
use rustc_public::ty::{RigidTy, Ty, TyKind};

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

/// A typed value combining MIR type information with runtime value.
#[derive(Debug, Clone)]
pub struct ValueTyped {
    pub ty: Ty,
    pub value: Value,
}

impl ToString for ValueTyped {
    fn to_string(&self) -> String {
        match self.ty.kind() {
            // Primitive types
            TyKind::RigidTy(RigidTy::Bool) => {
                self.value.as_bool().map_or("invalid_bool".to_string(), |b| b.to_string())
            }
            TyKind::RigidTy(RigidTy::Int(int_ty)) => {
                use rustc_public::ty::IntTy;
                match int_ty {
                    IntTy::I8 => self.value.as_i8().map_or("invalid_i8".to_string(), |i| i.to_string()),
                    IntTy::I16 => self.value.as_i16().map_or("invalid_i16".to_string(), |i| i.to_string()),
                    IntTy::I32 => self.value.as_i32().map_or("invalid_i32".to_string(), |i| i.to_string()),
                    IntTy::I64 => self.value.as_i64().map_or("invalid_i64".to_string(), |i| i.to_string()),
                    IntTy::I128 => self.value.as_i128().map_or("invalid_i128".to_string(), |i| i.to_string()),
                    IntTy::Isize => self.value.as_i64().map_or("invalid_isize".to_string(), |i| i.to_string()),
                }
            }
            TyKind::RigidTy(RigidTy::Uint(uint_ty)) => {
                use rustc_public::ty::UintTy;
                match uint_ty {
                    UintTy::U8 => self.value.as_u8().map_or("invalid_u8".to_string(), |u| u.to_string()),
                    UintTy::U16 => self.value.as_u16().map_or("invalid_u16".to_string(), |u| u.to_string()),
                    UintTy::U32 => self.value.as_u32().map_or("invalid_u32".to_string(), |u| u.to_string()),
                    UintTy::U64 => self.value.as_u64().map_or("invalid_u64".to_string(), |u| u.to_string()),
                    UintTy::U128 => self.value.as_u128().map_or("invalid_u128".to_string(), |u| u.to_string()),
                    UintTy::Usize => self.value.as_u64().map_or("invalid_usize".to_string(), |u| u.to_string()),
                }
            }
            TyKind::RigidTy(RigidTy::Tuple(fields)) if fields.is_empty() => {
                "()".to_string()
            }
            // Pointers and references
            TyKind::RigidTy(RigidTy::RawPtr(_, _)) | TyKind::RigidTy(RigidTy::Ref(_, _, _)) => {
                if self.value.data.len() >= 8 {
                    let mut bytes = [0u8; 8];
                    bytes.copy_from_slice(&self.value.data[..8]);
                    let addr = usize::from_le_bytes(bytes);
                    format!("0x{:x}", addr)
                } else {
                    "invalid_ptr".to_string()
                }
            }
            // Arrays
            TyKind::RigidTy(RigidTy::Array(elem_ty, len)) => {
                // For now, just show array info - full element printing would need more complex logic
                format!("[{}; {:?}]", elem_ty, len)
            }
            // Strings (for future implementation)
            TyKind::RigidTy(RigidTy::Str) => {
                // Would need to interpret binary representation as UTF-8
                "\"<string>\"".to_string()
            }
            // Unsupported types
            _ => {
                format!("Unsupported({})", self.ty)
            }
        }
    }
}

impl Value {
    /// Create value from signed 8-bit integer
    pub fn from_i8(value: i8) -> Self {
        Self {
            data: SmallVec::from_slice(&value.to_le_bytes()),
        }
    }

    /// Create value from signed 16-bit integer
    pub fn from_i16(value: i16) -> Self {
        Self {
            data: SmallVec::from_slice(&value.to_le_bytes()),
        }
    }

    /// Create value from signed 32-bit integer
    pub fn from_i32(value: i32) -> Self {
        Self {
            data: SmallVec::from_slice(&value.to_le_bytes()),
        }
    }

    /// Create value from signed 64-bit integer
    pub fn from_i64(value: i64) -> Self {
        Self {
            data: SmallVec::from_slice(&value.to_le_bytes()),
        }
    }
    
    /// Create value from signed 128-bit integer
    pub fn from_i128(value: i128) -> Self {
        Self {
            data: SmallVec::from_slice(&value.to_le_bytes()),
        }
    }

    /// Create value from unsigned 8-bit integer
    pub fn from_u8(value: u8) -> Self {
        Self {
            data: SmallVec::from_slice(&value.to_le_bytes()),
        }
    }

    /// Create value from unsigned 16-bit integer
    pub fn from_u16(value: u16) -> Self {
        Self {
            data: SmallVec::from_slice(&value.to_le_bytes()),
        }
    }

    /// Create value from unsigned 32-bit integer
    pub fn from_u32(value: u32) -> Self {
        Self {
            data: SmallVec::from_slice(&value.to_le_bytes()),
        }
    }

    /// Create value from unsigned 64-bit integer
    pub fn from_u64(value: u64) -> Self {
        Self {
            data: SmallVec::from_slice(&value.to_le_bytes()),
        }
    }
    
    /// Create value from unsigned 128-bit integer
    pub fn from_u128(value: u128) -> Self {
        Self {
            data: SmallVec::from_slice(&value.to_le_bytes()),
        }
    }
    
    /// Create unit value (zero-sized)
    pub fn unit() -> &'static Self {
        use std::sync::LazyLock;
        static UNIT: LazyLock<Value> = LazyLock::new(|| Value { data: SmallVec::new() });
        &UNIT
    }
    
    /// Create value from boolean
    pub fn from_bool(value: bool) -> Self {
        Self {
            data: smallvec![if value { 1 } else { 0 }],
        }
    }
    
    /// Try to interpret as signed 8-bit integer
    pub fn as_i8(&self) -> Option<i8> {
        if self.data.len() == 1 {
            Some(i8::from_le_bytes([self.data[0]]))
        } else {
            None
        }
    }

    /// Try to interpret as signed 16-bit integer
    pub fn as_i16(&self) -> Option<i16> {
        if self.data.len() == 2 {
            let mut bytes = [0u8; 2];
            bytes.copy_from_slice(&self.data);
            Some(i16::from_le_bytes(bytes))
        } else {
            None
        }
    }

    /// Try to interpret as signed 32-bit integer
    pub fn as_i32(&self) -> Option<i32> {
        if self.data.len() == 4 {
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&self.data);
            Some(i32::from_le_bytes(bytes))
        } else {
            None
        }
    }

    /// Try to interpret as signed 64-bit integer
    pub fn as_i64(&self) -> Option<i64> {
        if self.data.len() == 8 {
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&self.data);
            Some(i64::from_le_bytes(bytes))
        } else {
            None
        }
    }
    
    /// Try to interpret as signed 128-bit integer
    pub fn as_i128(&self) -> Option<i128> {
        if self.data.len() == 16 {
            let mut bytes = [0u8; 16];
            bytes.copy_from_slice(&self.data);
            Some(i128::from_le_bytes(bytes))
        } else {
            None
        }
    }

    /// Try to interpret as unsigned 8-bit integer
    pub fn as_u8(&self) -> Option<u8> {
        if self.data.len() == 1 {
            Some(self.data[0])
        } else {
            None
        }
    }

    /// Try to interpret as unsigned 16-bit integer
    pub fn as_u16(&self) -> Option<u16> {
        if self.data.len() == 2 {
            let mut bytes = [0u8; 2];
            bytes.copy_from_slice(&self.data);
            Some(u16::from_le_bytes(bytes))
        } else {
            None
        }
    }

    /// Try to interpret as unsigned 32-bit integer
    pub fn as_u32(&self) -> Option<u32> {
        if self.data.len() == 4 {
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&self.data);
            Some(u32::from_le_bytes(bytes))
        } else {
            None
        }
    }

    /// Try to interpret as unsigned 64-bit integer
    pub fn as_u64(&self) -> Option<u64> {
        if self.data.len() == 8 {
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&self.data);
            Some(u64::from_le_bytes(bytes))
        } else {
            None
        }
    }
    
    /// Try to interpret as unsigned 128-bit integer  
    pub fn as_u128(&self) -> Option<u128> {
        if self.data.len() == 16 {
            let mut bytes = [0u8; 16];
            bytes.copy_from_slice(&self.data);
            Some(u128::from_le_bytes(bytes))
        } else {
            None
        }
    }
    
    /// Check if this is a unit value
    pub fn is_unit(&self) -> bool {
        self.data.is_empty()
    }
    
    /// Try to interpret as boolean
    pub fn as_bool(&self) -> Option<bool> {
        if self.data.len() == 1 {
            Some(self.data[0] != 0)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_from_bool() {
        let true_val = Value::from_bool(true);
        assert_eq!(true_val.as_bool(), Some(true));
        assert!(!true_val.is_unit());
        assert_eq!(true_val.data.len(), 1);
        
        let false_val = Value::from_bool(false);
        assert_eq!(false_val.as_bool(), Some(false));
        assert_eq!(false_val.data.len(), 1);
    }

    #[test]
    fn test_value_signed_integers() {
        // Test i8
        let i8_val = Value::from_i8(-42);
        assert_eq!(i8_val.as_i8(), Some(-42));
        assert_eq!(i8_val.data.len(), 1);
        
        // Test i16
        let i16_val = Value::from_i16(-1000);
        assert_eq!(i16_val.as_i16(), Some(-1000));
        assert_eq!(i16_val.data.len(), 2);
        
        // Test i32
        let i32_val = Value::from_i32(-100000);
        assert_eq!(i32_val.as_i32(), Some(-100000));
        assert_eq!(i32_val.data.len(), 4);
        
        // Test i64
        let i64_val = Value::from_i64(-1000000000);
        assert_eq!(i64_val.as_i64(), Some(-1000000000));
        assert_eq!(i64_val.data.len(), 8);
        
        // Test i128
        let i128_val = Value::from_i128(-123);
        assert_eq!(i128_val.as_i128(), Some(-123));
        assert_eq!(i128_val.data.len(), 16);
    }

    #[test]
    fn test_value_unsigned_integers() {
        // Test u8
        let u8_val = Value::from_u8(255);
        assert_eq!(u8_val.as_u8(), Some(255));
        assert_eq!(u8_val.data.len(), 1);
        
        // Test u16
        let u16_val = Value::from_u16(65535);
        assert_eq!(u16_val.as_u16(), Some(65535));
        assert_eq!(u16_val.data.len(), 2);
        
        // Test u32
        let u32_val = Value::from_u32(4294967295);
        assert_eq!(u32_val.as_u32(), Some(4294967295));
        assert_eq!(u32_val.data.len(), 4);
        
        // Test u64
        let u64_val = Value::from_u64(18446744073709551615);
        assert_eq!(u64_val.as_u64(), Some(18446744073709551615));
        assert_eq!(u64_val.data.len(), 8);
        
        // Test u128
        let u128_val = Value::from_u128(999);
        assert_eq!(u128_val.as_u128(), Some(999));
        assert_eq!(u128_val.data.len(), 16);
    }

    #[test]
    fn test_value_unit() {
        let unit_val = Value::unit();
        assert!(unit_val.is_unit());
        assert_eq!(unit_val.data.len(), 0);
        assert_eq!(unit_val.as_bool(), None);
        assert_eq!(unit_val.as_i8(), None);
        assert_eq!(unit_val.as_u8(), None);
    }

    #[test]
    fn test_value_type_safety() {
        // Test that wrong size conversions return None
        let i8_val = Value::from_i8(42);
        assert_eq!(i8_val.as_i16(), None);
        assert_eq!(i8_val.as_i32(), None);
        assert_eq!(i8_val.as_u8(), Some(42)); // Same size, different interpretation
        
        let i32_val = Value::from_i32(42);
        assert_eq!(i32_val.as_i8(), None);
        assert_eq!(i32_val.as_i16(), None);
        assert_eq!(i32_val.as_u32(), Some(42)); // Same size, different interpretation
    }

    #[test]
    fn test_value_equality() {
        let val1 = Value::from_i32(42);
        let val2 = Value::from_i32(42);
        let val3 = Value::from_i32(43);
        let val4 = Value::from_i8(42); // Different size, same value
        
        assert_eq!(val1, val2);
        assert_ne!(val1, val3);
        assert_ne!(val1, val4); // Different sizes are not equal
    }
}
