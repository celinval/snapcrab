//! This module provides the core interpretation logic for the interpreter.
//!
//! It executes Rust programs at the MIR (Mid-level Intermediate Representation) level.
//! The interpreter evaluates MIR instructions directly, handling function calls,
//! control flow, and memory operations without code generation overhead.

pub mod function;
mod place;
mod rvalue;
