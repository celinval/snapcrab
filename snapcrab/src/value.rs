//! Abstraction of a value in memory
//!
//! Values are always initialized to avoid reading from uninitialized memory
//! in case the program being interpreted has a safety violation.
//!
//! The value will include padding bytes.
//!
//! # Warning
//!
//! This module currently assumes the target machine is a little endian and
//! matches number of bits from host machine.
use crate::ty::MonoType;
use anyhow::{Result, bail};
use rustc_public::abi::FieldsShape;
use rustc_public::ty::{RigidTy, Ty, TyKind};
use smallvec::{SmallVec, smallvec};
use std::ops::{Index, Range};
use zerocopy::{FromBytes, IntoBytes};

/// Index type for local variables in a function's stack frame.
#[allow(dead_code)]
pub type Local = usize;

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
                    let offset = field_offset.bytes();
                    if offset + field_size <= self.value.len() {
                        return Ok(self.value[offset..offset + field_size].into());
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

impl std::fmt::Display for TypedValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Early length validation - return error for insufficient bytes
        let required_size = self.ty.size().unwrap_or(0);
        if self.value.len() < required_size {
            return write!(
                f,
                "InvalidValue({}, expected {} bytes, got {})",
                self.ty,
                required_size,
                self.value.len()
            );
        }

        match self.ty.kind() {
            // Primitive types using zerocopy for efficient parsing
            TyKind::RigidTy(RigidTy::Bool) => write!(f, "{}", self.value[0] != 0),
            TyKind::RigidTy(RigidTy::Int(int_ty)) => {
                use rustc_public::ty::IntTy;
                match int_ty {
                    IntTy::I8 => write!(f, "{}", i8::read_from_bytes(self.value).unwrap()),
                    IntTy::I16 => write!(f, "{}", i16::read_from_bytes(self.value).unwrap()),
                    IntTy::I32 => write!(f, "{}", i32::read_from_bytes(self.value).unwrap()),
                    IntTy::I64 => {
                        write!(f, "{}", i64::read_from_bytes(self.value).unwrap())
                    }
                    IntTy::Isize => {
                        write!(f, "{}", isize::read_from_bytes(self.value).unwrap())
                    }
                    IntTy::I128 => write!(f, "{}", i128::read_from_bytes(self.value).unwrap()),
                }
            }
            TyKind::RigidTy(RigidTy::Uint(uint_ty)) => {
                use rustc_public::ty::UintTy;
                match uint_ty {
                    UintTy::U8 => write!(f, "{}", u8::read_from_bytes(self.value).unwrap()),
                    UintTy::U16 => write!(f, "{}", u16::read_from_bytes(self.value).unwrap()),
                    UintTy::U32 => write!(f, "{}", u32::read_from_bytes(self.value).unwrap()),
                    UintTy::U64 => {
                        write!(f, "{}", u64::read_from_bytes(self.value).unwrap())
                    }
                    UintTy::Usize => {
                        write!(f, "{}", usize::read_from_bytes(self.value).unwrap())
                    }
                    UintTy::U128 => write!(f, "{}", u128::read_from_bytes(self.value).unwrap()),
                }
            }
            TyKind::RigidTy(RigidTy::Tuple(fields)) if fields.is_empty() => write!(f, "()"),
            TyKind::RigidTy(RigidTy::Tuple(fields)) => {
                write!(f, "(")?;
                for (i, field_ty) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    let field_value = self
                        .extract_field_value(field_ty, i)
                        .unwrap_or_else(|_| Value::unit().clone());
                    let typed_field = TypedValue {
                        ty: *field_ty,
                        value: &field_value.data,
                    };
                    write!(f, "{}", typed_field)?;
                }
                write!(f, ")")
            }
            TyKind::RigidTy(RigidTy::RawPtr(_, _)) | TyKind::RigidTy(RigidTy::Ref(_, _, _)) => {
                write!(f, "0x{:x}", usize::read_from_bytes(self.value).unwrap())
            }
            TyKind::RigidTy(RigidTy::Array(elem_ty, len)) => {
                write!(f, "[{}; {:?}]", elem_ty, len)
            }
            TyKind::RigidTy(RigidTy::Str) => {
                write!(f, "\"<string>\"")
            }
            _ => {
                write!(f, "Unsupported({})", self.ty)
            }
        }
    }
}

impl Value {
    /// Get the length of the value (= size in bytes)
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Get the size in bytes of this value
    ///
    /// Same as `[Value::len()]`.
    #[inline]
    #[allow(unused)]
    pub fn size(&self) -> usize {
        self.data.len()
    }

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

    /// Create a initialized Value with the requested number of bytes
    ///
    /// We currently initialize it to zero.
    pub fn with_size(num_bytes: usize) -> Self {
        Self {
            data: smallvec![0; num_bytes],
        }
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
        if let FieldsShape::Arbitrary { offsets } = &shape.fields {
            let total_size = shape.size.bytes();
            let mut result = Self::with_size(total_size);

            for (i, value) in values.iter().enumerate() {
                if value.len() > 0
                    && let Some(offset) = offsets.get(i)
                {
                    let offset = offset.bytes();
                    let end = offset + value.data.len();
                    debug_assert!(end <= total_size);
                    result.data[offset..end].copy_from_slice(&value.data);
                }
            }
            Ok(result)
        } else {
            // TODO: Create a panic hook for handling internal errors.
            panic!("Expected tuple with arbitrary layout, but found `{shape:?}` for {ty}");
        }
    }

    /// Create value from raw bytes
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            data: SmallVec::from_slice(bytes),
        }
    }

    /// Create array by repeating a value
    pub fn from_repeated(value: &Value, count: usize) -> Self {
        Self {
            data: SmallVec::from_vec(value.data.as_slice().repeat(count)),
        }
    }

    /// Create array from values
    pub fn from_array(values: &[Value]) -> Self {
        let mut data = SmallVec::new();
        for value in values {
            data.extend_from_slice(&value.data);
        }
        Self { data }
    }

    /// Create value from raw bytes with additional padding at the end
    pub fn from_val_with_padding(src: &Value, len: usize) -> Self {
        if src.len() == len {
            // simply move
            src.clone()
        } else if src.len() == 0 {
            // Only padding bytes needed
            Self::with_size(len)
        } else {
            debug_assert!(
                src.len() < len,
                "Expected at least `{}` bytes. Found `{}`",
                len + 1,
                src.len()
            );
            let mut new_val = Self::with_size(len);
            new_val.data[0..src.len()].copy_from_slice(&src.data);
            new_val
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

    /// Method for creating fat pointers
    ///
    /// Metadata can be either pointer to vtable or length of a slice.
    pub fn new_wide_ptr(data_addr: usize, metadata: usize) -> Self {
        Self::from_type([data_addr, metadata])
    }

    /// Get metadata from a possibly wide pointer
    ///
    /// - Wide pointers are represented as [data_addr: usize, metadata: usize]
    /// - Thin pointers are represented as [address: usize]
    pub fn ptr_metadata(&self) -> Result<Self> {
        let ptr_size = size_of::<usize>();
        if self.len() == ptr_size {
            // Thin pointer, return an empty value.
            Ok(Value::unit().clone())
        } else if self.len() == 2 * ptr_size {
            Ok(self.data[ptr_size..2 * ptr_size].into())
        } else {
            bail!("Expected pointer, got {} bytes", self.len())
        }
    }

    /// Get thin pointer from a possibly wide pointer
    ///
    /// - Wide pointers are represented as [data_address: usize, metadata: usize]
    /// - Thin pointers are represented as [address: usize]
    #[allow(clippy::wrong_self_convention)]
    pub fn to_data_addr(mut self) -> Result<Self> {
        let ptr_size = size_of::<usize>();
        if self.len() == ptr_size {
            // Already thin pointer
            Ok(self)
        } else if self.len() == 2 * ptr_size {
            self.data.truncate(ptr_size);
            Ok(self)
        } else {
            bail!("Expected pointer, got {} bytes", self.len())
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

impl From<&[u8]> for Value {
    fn from(bytes: &[u8]) -> Self {
        Self::from_bytes(bytes)
    }
}

impl Index<usize> for Value {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl Index<Range<usize>> for Value {
    type Output = [u8];

    fn index(&self, range: Range<usize>) -> &Self::Output {
        &self.data[range]
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

    #[test]
    fn test_from_val_with_padding_same_size() {
        let src = Value::from_type(42i32);
        let result = Value::from_val_with_padding(&src, 4);
        assert_eq!(result.as_bytes(), &[42, 0, 0, 0]);
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn test_from_val_with_padding_add_padding() {
        let src = Value::from_type(42i32);
        let result = Value::from_val_with_padding(&src, 8);
        assert_eq!(result.as_bytes(), &[42, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(result.len(), 8);
    }

    #[test]
    fn test_from_val_with_padding_empty_src() {
        let src = Value::with_size(0);
        let result = Value::from_val_with_padding(&src, 4);
        assert_eq!(result.as_bytes(), &[0u8, 0, 0, 0]);
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn test_from_val_with_padding_empty_to_empty() {
        let src = Value::with_size(0);
        let result = Value::from_val_with_padding(&src, 0);
        assert_eq!(result.as_bytes(), &[] as &[u8]);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_from_val_with_padding_one_byte() {
        let src = Value::from_type(255u8);
        let result = Value::from_val_with_padding(&src, 4);
        assert_eq!(result.as_bytes(), &[255, 0, 0, 0]);
        assert_eq!(result.len(), 4);
    }

    #[test]
    #[should_panic(expected = "Expected at least")]
    fn test_from_val_with_padding_src_too_large() {
        let src = Value::from_type(42i32);
        let _ = Value::from_val_with_padding(&src, 2);
    }

    #[test]
    fn test_new_wide_ptr() {
        let ptr = Value::new_wide_ptr(0x1000, 42);
        assert_eq!(ptr.len(), 2 * size_of::<usize>());
        assert_eq!(ptr.as_type::<[usize; 2]>(), Some([0x1000, 42]));
    }

    #[test]
    fn test_ptr_metadata_thin_pointer() {
        let thin_ptr = Value::from_type(0x1000usize);
        let metadata = thin_ptr.ptr_metadata().unwrap();
        assert!(metadata.is_unit());
    }

    #[test]
    fn test_ptr_metadata_wide_pointer() {
        let wide_ptr = Value::new_wide_ptr(0x1000, 42);
        let metadata = wide_ptr.ptr_metadata().unwrap();
        assert_eq!(metadata.as_type::<usize>(), Some(42));
    }

    #[test]
    fn test_ptr_metadata_invalid_size() {
        let invalid = Value::from_type(42u32);
        assert!(invalid.ptr_metadata().is_err());
    }

    #[test]
    fn test_to_data_addr_thin_pointer() {
        let thin_ptr = Value::from_type(0x1000usize);
        let data_addr = thin_ptr.to_data_addr().unwrap();
        assert_eq!(data_addr.as_type::<usize>(), Some(0x1000));
    }

    #[test]
    fn test_to_data_addr_wide_pointer() {
        let wide_ptr = Value::new_wide_ptr(0x2000, 100);
        let data_addr = wide_ptr.to_data_addr().unwrap();
        assert_eq!(data_addr.len(), size_of::<usize>());
        assert_eq!(data_addr.as_type::<usize>(), Some(0x2000));
    }

    #[test]
    fn test_to_data_addr_invalid_size() {
        let invalid = Value::from_type(42u32);
        assert!(invalid.to_data_addr().is_err());
    }
}
