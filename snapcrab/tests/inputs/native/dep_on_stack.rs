/// Test for Indirect { on_stack: true } pass mode.
/// On x86-64, this happens in certain ABIs or when registers are exhausted.

#[repr(C)]
pub struct SmallStruct {
    pub a: u32,
    pub b: u32,
}

/// stdcall on x86-64 is the same as C, but on x86 it uses stack for all args.
/// Let's try exhausting registers: 6 integer args + a struct.
pub extern "C" fn many_args_then_struct(
    a: u64, b: u64, c: u64, d: u64, e: u64, f: u64,
    s: SmallStruct,
) -> u64 {
    a + b + c + d + e + f + s.a as u64 + s.b as u64
}
