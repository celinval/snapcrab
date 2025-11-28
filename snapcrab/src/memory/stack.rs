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

use crate::memory::sanitizer::MemorySanitizer;
use crate::memory::{MemoryAccessError, MemorySegment};
use crate::ty::MonoType;
use crate::value::Value;
use anyhow::Result;
use rustc_public::mir::Body;
use rustc_public::mir::mono::Instance;
use std::pin::Pin;

use super::ThreadMemory;

/// Stack memory manager containing sanitizer and stack frames
#[derive(Default, Debug)]
pub struct Stack {
    sanitizer: MemorySanitizer,
    frames: Vec<StackFrame>,
}

unsafe impl MemorySegment for Stack {
    fn read_addr(&self, address: usize, size: usize) -> Result<&[u8], MemoryAccessError> {
        if self.sanitizer.contains(address, size) {
            // SAFETY: sanitizer verified the address range is valid
            Ok(unsafe { std::slice::from_raw_parts(address as *const u8, size) })
        } else {
            Err(MemoryAccessError::OutOfBounds)
        }
    }

    fn write_addr(&self, address: usize, data: &[u8]) -> Result<(), MemoryAccessError> {
        if self.sanitizer.contains(address, data.len()) {
            // SAFETY: sanitizer verified the address range is valid
            unsafe { std::ptr::copy(data.as_ptr(), address as *mut u8, data.len()) };
            Ok(())
        } else {
            Err(MemoryAccessError::OutOfBounds)
        }
    }
}

impl Stack {
    /// Runs a method with their own stack frame.
    ///
    /// This ensures that the stack frame is allocated just for the duration
    /// of the execution, and sanitizer is kept up-to-date.
    pub fn with_stack_frame<F, R>(instance: Instance, memory: &mut ThreadMemory, func: F) -> R
    where
        F: FnOnce(&Body, &mut ThreadMemory) -> R,
    {
        // Create frame and register in the sanitizer.
        let body = instance.body().expect("Caller should ensure body exists");
        let frame = StackFrame::new(&body);
        let address = frame.data.as_ptr();
        memory.stack.sanitizer.register_alloc(&frame.data);
        memory.stack.frames.push(frame);

        // Call function.
        let result = func(&body, memory);

        // Remove from the sanitizer and pop the frame.
        let data = &memory.stack.frames.last().unwrap().data;
        assert_eq!(
            data.as_ptr() as usize,
            address as usize,
            "Unexpected stack frame"
        );
        memory.stack.sanitizer.deregister_alloc(data);
        memory.stack.frames.pop();

        // Return actual result.
        result
    }

    #[allow(dead_code)]
    pub fn read_local(&self, local: usize) -> Result<Value> {
        self.frames.last().unwrap().read_local(local)
    }

    pub fn write_local(&mut self, local: usize, value: Value) -> Result<()> {
        self.frames.last_mut().unwrap().write_local(local, value)
    }

    pub fn local_address(&self, local: usize) -> Result<usize> {
        self.frames.last().unwrap().local_address(local)
    }
}

/// Stack frame for function execution.
///
/// Contains a contiguous block of memory for all local variables.
/// Variables are stored as raw bytes at calculated offsets.
#[derive(Debug)]
pub struct StackFrame {
    /// Holds the stack data. We require this data to stay in the same location
    data: Pin<Box<[u8]>>,
    /// Maps local to the data[offset].
    offsets: Vec<usize>,
}

impl StackFrame {
    /// Creates a new stack frame for the given function body.
    ///
    /// Calculates the total size needed for all local variables and allocates
    /// a contiguous block of memory to store them.
    pub fn new(body: &Body) -> Self {
        let mut offsets = Vec::new();
        let mut current_offset = 0;

        for local in body.locals() {
            let size = local
                .ty
                .size()
                .unwrap_or_else(|e| panic!("Locals should be sized. {e}"));
            let alignment = local
                .ty
                .alignment()
                .unwrap_or_else(|e| panic!("Locals should have alignment. {e}"));

            // Align current_offset to the required alignment (power of 2)
            current_offset = (current_offset + alignment - 1) & !(alignment - 1);

            offsets.push(current_offset);
            current_offset += size;
        }

        // We should replace this with Box::new_zeroed_slice once it's stable.
        let buffer = vec![0; current_offset];
        let data = Box::into_pin(buffer.into_boxed_slice());

        Self { data, offsets }
    }

    /// Sets a local variable to the given value
    pub fn write_local(&mut self, local: usize, value: Value) -> Result<()> {
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
    pub fn read_local(&self, local: usize) -> Result<Value> {
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
    pub fn local_address(&self, local: usize) -> Result<usize> {
        if local >= self.offsets.len() {
            anyhow::bail!("Local index {} out of bounds", local);
        }
        let offset = self.offsets[local];
        Ok(self.data.as_ptr() as usize + offset)
    }
}
