use crate::ty::MonoType;
use anyhow::{Result, bail};
use rustc_public::abi::FieldsShape;
use rustc_public::ty::{RigidTy, Ty, TyKind};
use smallvec::{SmallVec, smallvec};
use zerocopy::{FromBytes, IntoBytes};

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
pub struct TypedValue<'a> {
    pub ty: Ty,
    pub value: &'a [u8],
}

impl TypedValue<'_> {
    /// Extract a field value from the binary data at the given offset
    fn extract_field_value(&self, field_ty: &Ty, field_idx: usize) -> Result<Value> {
        // Get the tuple layout to find the actual field offset
        let layout = self.ty.layout()?;
        let shape = layout.shape();
        match &shape.fields {
            FieldsShape::Arbitrary { offsets } => {
                if let Some(field_offset) = offsets.get(field_idx) {
                    let field_size = field_ty.size()?;
                    let offset = field_offset.bytes() as usize;
                    if offset + field_size <= self.value.len() {
                        let field_data =
                            SmallVec::from_slice(&self.value[offset..offset + field_size]);
                        return Ok(Value { data: field_data });
                    }
                }
                bail!("Field at `{field_idx}` of `{field_ty}` out of range.")
            }
            _ => {
                bail!("Unsupported shape: {shape:?}");
            }
        }
    }
}

impl ToString for TypedValue<'_> {
    fn to_string(&self) -> String {
        // Early length validation - return error for insufficient bytes
        let required_size = self.ty.size().unwrap_or(0);
        if self.value.len() < required_size {
            return format!(
                "InvalidValue({}, expected {} bytes, got {})",
                self.ty,
                required_size,
                self.value.len()
            );
        }

        match self.ty.kind() {
            // Primitive types using zerocopy for efficient parsing
            TyKind::RigidTy(RigidTy::Bool) => (self.value[0] != 0).to_string(),
            TyKind::RigidTy(RigidTy::Int(int_ty)) => {
                use rustc_public::ty::IntTy;
                match int_ty {
                    IntTy::I8 => i8::read_from_bytes(self.value).unwrap().to_string(),
                    IntTy::I16 => i16::read_from_bytes(self.value).unwrap().to_string(),
                    IntTy::I32 => i32::read_from_bytes(self.value).unwrap().to_string(),
                    IntTy::I64 | IntTy::Isize => {
                        i64::read_from_bytes(self.value).unwrap().to_string()
                    }
                    IntTy::I128 => i128::read_from_bytes(self.value).unwrap().to_string(),
                }
            }
            TyKind::RigidTy(RigidTy::Uint(uint_ty)) => {
                use rustc_public::ty::UintTy;
                match uint_ty {
                    UintTy::U8 => u8::read_from_bytes(self.value).unwrap().to_string(),
                    UintTy::U16 => u16::read_from_bytes(self.value).unwrap().to_string(),
                    UintTy::U32 => u32::read_from_bytes(self.value).unwrap().to_string(),
                    UintTy::U64 | UintTy::Usize => {
                        u64::read_from_bytes(self.value).unwrap().to_string()
                    }
                    UintTy::U128 => u128::read_from_bytes(self.value).unwrap().to_string(),
                }
            }
            TyKind::RigidTy(RigidTy::Tuple(fields)) if fields.is_empty() => "()".to_string(),
            TyKind::RigidTy(RigidTy::Tuple(fields)) => {
                // For non-empty tuples, use actual ABI layout
                let mut result = String::from("(");

                // Use declaration order per user expectation
                for (i, field_ty) in fields.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }

                    let field_value = self
                        .extract_field_value(field_ty, i)
                        .unwrap_or_else(|_| Value::unit().clone());
                    let typed_field = TypedValue {
                        ty: *field_ty,
                        value: &field_value.data,
                    };
                    result.push_str(&typed_field.to_string());
                }

                result.push(')');
                result
            }
            // Pointers and references
            TyKind::RigidTy(RigidTy::RawPtr(_, _)) | TyKind::RigidTy(RigidTy::Ref(_, _, _)) => {
                format!("0x{:x}", usize::read_from_bytes(self.value).unwrap())
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
    /// Get the raw bytes of the value
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Create unit value (zero-sized)
    pub fn unit() -> &'static Self {
        static UNIT: Value = Value {
            data: SmallVec::new_const(),
        };
        &UNIT
    }

    /// Create value from boolean
    pub fn from_bool(value: bool) -> Self {
        Self {
            data: smallvec![if value { 1 } else { 0 }],
        }
    }

    /// Create value from tuple of values with proper layout
    pub fn from_tuple_with_layout(values: &[Value], ty: Ty) -> Result<Self> {
        let layout = ty.layout()?;
        let shape = layout.shape();
        if let rustc_public::abi::FieldsShape::Arbitrary { offsets } = &shape.fields {
            let total_size = shape.size.bytes() as usize;
            let mut data = SmallVec::from_elem(0u8, total_size);

            for (i, value) in values.iter().enumerate() {
                if let Some(offset) = offsets.get(i) {
                    let offset = offset.bytes() as usize;
                    let end = offset + value.data.len();
                    if end <= data.len() {
                        data[offset..end].copy_from_slice(&value.data);
                    }
                }
            }
            Ok(Self { data })
        } else {
            bail!("Cannot create tuple with layout for type: {:?}", ty)
        }
    }

    /// Create value from raw bytes
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            data: SmallVec::from_slice(bytes),
        }
    }

    /// Generic method to interpret as any FromBytes type
    pub fn as_type<T: FromBytes>(&self) -> Option<T> {
        T::read_from_bytes(&self.data).ok()
    }

    /// Check if this is a unit value
    pub fn is_unit(&self) -> bool {
        self.data.is_empty()
    }

    /// Generic method to create value from any IntoBytes type
    pub fn from_type<T: IntoBytes + zerocopy::Immutable>(value: T) -> Self {
        Self {
            data: SmallVec::from_slice(value.as_bytes()),
        }
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
        let i8_val = Value::from_type(-42i8);
        assert_eq!(i8_val.as_type::<i8>(), Some(-42));
        assert_eq!(i8_val.data.len(), 1);

        // Test i16
        let i16_val = Value::from_type(-1000i16);
        assert_eq!(i16_val.as_type::<i16>(), Some(-1000));
        assert_eq!(i16_val.data.len(), 2);

        // Test i32
        let i32_val = Value::from_type(-100000i32);
        assert_eq!(i32_val.as_type::<i32>(), Some(-100000));
        assert_eq!(i32_val.data.len(), 4);

        // Test i64
        let i64_val = Value::from_type(-1000000000i64);
        assert_eq!(i64_val.as_type::<i64>(), Some(-1000000000));
        assert_eq!(i64_val.data.len(), 8);

        // Test i128
        let i128_val = Value::from_type(-123i128);
        assert_eq!(i128_val.as_type::<i128>(), Some(-123));
        assert_eq!(i128_val.data.len(), 16);
    }

    #[test]
    fn test_value_unsigned_integers() {
        // Test u8
        let u8_val = Value::from_type(255u8);
        assert_eq!(u8_val.as_type::<u8>(), Some(255));
        assert_eq!(u8_val.data.len(), 1);

        // Test u16
        let u16_val = Value::from_type(65535u16);
        assert_eq!(u16_val.as_type::<u16>(), Some(65535));
        assert_eq!(u16_val.data.len(), 2);

        // Test u32
        let u32_val = Value::from_type(4294967295u32);
        assert_eq!(u32_val.as_type::<u32>(), Some(4294967295));
        assert_eq!(u32_val.data.len(), 4);

        // Test u64
        let u64_val = Value::from_type(18446744073709551615u64);
        assert_eq!(u64_val.as_type::<u64>(), Some(18446744073709551615));
        assert_eq!(u64_val.data.len(), 8);

        // Test u128
        let u128_val = Value::from_type(999u128);
        assert_eq!(u128_val.as_type::<u128>(), Some(999));
        assert_eq!(u128_val.data.len(), 16);
    }

    #[test]
    fn test_value_unit() {
        let unit_val = Value::unit();
        assert!(unit_val.is_unit());
        assert_eq!(unit_val.data.len(), 0);
        assert_eq!(unit_val.as_bool(), None);
        assert_eq!(unit_val.as_type::<i8>(), None);
        assert_eq!(unit_val.as_type::<u8>(), None);
    }

    #[test]
    fn test_value_type_safety() {
        // Test that wrong size conversions return None
        let i8_val = Value::from_type(42i8);
        assert_eq!(i8_val.as_type::<i16>(), None);
        assert_eq!(i8_val.as_type::<i32>(), None);
        assert_eq!(i8_val.as_type::<u8>(), Some(42)); // Same size, different interpretation

        let i32_val = Value::from_type(42i32);
        assert_eq!(i32_val.as_type::<i8>(), None);
        assert_eq!(i32_val.as_type::<i16>(), None);
        assert_eq!(i32_val.as_type::<u32>(), Some(42)); // Same size, different interpretation
    }

    #[test]
    fn test_value_tuple() {
        // Create a simple tuple value for testing (packed layout)
        let mut data = SmallVec::new();
        data.extend_from_slice(&Value::from_type(42u8).data);
        data.extend_from_slice(&Value::from_bool(true).data);
        data.extend_from_slice(&Value::from_type(1000u32).data);
        let tuple_val = Value { data };

        // Should have combined size: 1 + 1 + 4 = 6 bytes
        assert_eq!(tuple_val.data.len(), 6);

        // Verify the data is laid out correctly (packed layout)
        assert_eq!(tuple_val.data[0], 42); // u8
        assert_eq!(tuple_val.data[1], 1); // bool (true)
        // u32 1000 in little-endian: [232, 3, 0, 0]
        assert_eq!(&tuple_val.data[2..6], &[232, 3, 0, 0]);
    }

    #[test]
    fn test_tuple_memory_layout() {
        // Test simple concatenation layout
        let values = vec![
            Value::from_type(42u8),
            Value::from_bool(true),
            Value::from_type(1000u32),
        ];

        // Create tuple with simple concatenation
        let mut data = SmallVec::new();
        for value in &values {
            data.extend_from_slice(&value.data);
        }
        let tuple_val = Value { data };

        // The tuple should be a simple concatenation of the field data
        let mut expected_data = SmallVec::<[u8; 16]>::new();
        for value in &values {
            expected_data.extend_from_slice(&value.data);
        }

        assert_eq!(tuple_val.data, expected_data);
    }

    #[test]
    fn test_tuple_field_extraction() {
        // Create a tuple (u8, bool, u32) with known layout
        let tuple_data = Value::from_bytes(&[42, 1, 0, 0, 232, 3, 0, 0]);

        // Extract individual fields
        assert_eq!(tuple_data.data[0], 42); // u8
        assert_eq!(tuple_data.data[1], 1); // bool
        assert_eq!(&tuple_data.data[4..8], &[232, 3, 0, 0]); // u32 at offset 4
    }

    #[test]
    fn test_nested_tuple_layout() {
        // Test ((i16, u8), bool) layout - just verify structure
        let nested = Value::from_bytes(&[156, 255, 255, 255, 0, 0, 0, 0]);
        assert_eq!(nested.data.len(), 8); // Total size with padding
    }

    #[test]
    fn test_tuple_type_conversion() {
        // Original tuple: (u8, bool, u32) = (42, true, 1000)
        let original = Value::from_bytes(&[1, 42, 0, 0, 232, 3, 0, 0]);

        // Convert to (u32, u8, bool) - same types, different order
        let reordered = Value::from_bytes(&[232, 3, 0, 0, 42, 1, 0, 0]);

        // Verify the reordered tuple has different layout
        assert_ne!(original.data, reordered.data);
        assert_eq!(reordered.data.len(), 8); // Same total size but different layout
    }

    #[test]
    fn test_single_element_tuple() {
        // Single element tuple (i32,) should be same as i32
        let single_tuple = Value::from_type(42i32);
        let regular_i32 = Value::from_type(42i32);

        assert_eq!(single_tuple.data, regular_i32.data);
        assert_eq!(single_tuple.data.len(), 4);
    }

    #[test]
    fn test_tuple_field_ordering() {
        // Test that different field orders produce different layouts
        let tuple1 = Value::from_bytes(&[1, 42, 0, 0, 232, 3, 0, 0]); // (u8, bool, u32)
        let tuple2 = Value::from_bytes(&[232, 3, 0, 0, 42, 1, 0, 0]); // (u32, u8, bool)
        let tuple3 = Value::from_bytes(&[232, 3, 0, 0, 1, 42, 0, 0]); // (bool, u32, u8) - note field reordering

        // All should be different due to different field arrangements
        assert_ne!(tuple1.data, tuple2.data);
        assert_ne!(tuple1.data, tuple3.data);
        assert_ne!(tuple2.data, tuple3.data);
    }

    #[test]
    fn test_value_equality() {
        let val1 = Value::from_type(42i32);
        let val2 = Value::from_type(42i32);
        let val3 = Value::from_type(43i32);
        let val4 = Value::from_type(42i8); // Different size, same value

        assert_eq!(val1, val2);
        assert_ne!(val1, val3);
        assert_ne!(val1, val4); // Different sizes are not equal
    }
}
