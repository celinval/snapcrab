//! SnapCrab Interpreter
//!
//! A rustc wrapper that leverages `rustc_public` to interpret Rust code at the MIR level.
//! This component executes Rust code without LLVM code generation and linking overhead,
//! enabling rapid development iteration.

fn main() {
    println!("SnapCrab Interpreter v{}", env!("CARGO_PKG_VERSION"));
    println!("Experimental Rust interpreter for fast development iteration");
}
