//! Function stack frame implementation.
//!
//! A stack frame represents the local variable storage for a single function call.
//! Instead of storing each local as a separate `Value`, the frame uses a contiguous
//! byte array where variables are stored at calculated offsets based on their types.
//!
//! This approach:
//! - Reduces memory overhead by eliminating per-variable allocations
//! - Matches how actual stack frames work in compiled code
//! - Allows for efficient memory layout based on type sizes
//!
//! Downside:
//! - It makes it harder to check for buffer overflow.

use crate::value::Value;
use crate::{memory::global_memory_tracker, ty::MonoType};
use anyhow::Result;
use rustc_public::mir::Body;

/// Stack frame for function execution.
///
/// Contains a contiguous block of memory for all local variables.
/// Variables are stored as raw bytes at calculated offsets.
#[derive(Debug)]
pub struct StackFrame {
    data: Vec<u8>,
    offsets: Vec<usize>,
}

impl StackFrame {
    /// Creates a new stack frame for the given function body.
    ///
    /// Calculates the total size needed for all local variables and allocates
    /// a contiguous block of memory to store them.
    pub fn new(body: &Body) -> Result<Self> {
        let mut offsets = Vec::new();
        let mut current_offset = 0;

        for local in body.locals() {
            offsets.push(current_offset);
            current_offset += local.ty.size()?;
        }

        let data = vec![0; current_offset];

        // Register with global memory tracker
        let address = data.as_ptr() as usize;
        global_memory_tracker()
            .lock()
            .unwrap()
            .allocate(address, data.len())?;

        Ok(Self { data, offsets })
    }

    /// Sets a local variable to the given value
    pub fn set_local(&mut self, local: usize, value: Value) -> Result<()> {
        if local >= self.offsets.len() {
            anyhow::bail!("Local index {} out of bounds", local);
        }

        let offset = self.offsets[local];
        let bytes = value.as_bytes();
        let end = offset + bytes.len();
        if end > self.data.len() {
            anyhow::bail!("Value too large for local {}", local);
        }
        self.data[offset..end].copy_from_slice(bytes);
        Ok(())
    }

    /// Gets a local variable value
    pub fn get_local(&self, local: usize) -> Result<Value> {
        if local >= self.offsets.len() {
            anyhow::bail!("Local index {} out of bounds", local);
        }

        let offset = self.offsets[local];
        let next_offset = if local + 1 < self.offsets.len() {
            self.offsets[local + 1]
        } else {
            self.data.len()
        };

        let bytes = &self.data[offset..next_offset];
        Ok(Value::from_bytes(bytes))
    }

    /// Gets the address of a local variable
    pub fn get_local_address(&self, local: usize) -> Result<usize> {
        if local >= self.offsets.len() {
            anyhow::bail!("Local index {} out of bounds", local);
        }
        let offset = self.offsets[local];
        Ok(self.data.as_ptr() as usize + offset)
    }
}

impl Drop for StackFrame {
    fn drop(&mut self) {
        let address = self.data.as_ptr() as usize;
        if let Ok(mut tracker) = global_memory_tracker().lock() {
            let _ = tracker.deallocate(address);
        }
    }
}
