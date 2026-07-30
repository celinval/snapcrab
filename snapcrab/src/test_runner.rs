//! Test discovery and execution.
//!
//! When a crate is compiled with `--test`, rustc generates a harness `main`
//! (the entry function) that builds an array of `test::TestDescAndFn` values
//! and passes it to `test::test_main_static`. Each entry carries a test's
//! name and a function pointer to its body.
//!
//! Rather than run the harness (which would pull in the native `test`
//! runtime), we read the harness `main`'s MIR to recover the list of tests,
//! then interpret each test function directly.
//!
//! Only the standard `#[test]` harness is supported: `StaticTestName` +
//! `StaticTestFn`. Dynamically-named tests, benches, and custom harnesses
//! are rejected.

use crate::interpreter::function::invoke_fn;
use crate::memory::ThreadMemory;
use crate::{CheckConfig, load_native_libs};
use anyhow::{Result, bail};
use rustc_public::CrateDef;
use rustc_public::abi::FieldsShape;
use rustc_public::mir::alloc::{AllocId, GlobalAlloc};
use rustc_public::mir::mono::Instance;
use rustc_public::ty::{RigidTy, Ty, TyKind};
use rustc_public::{CrateItem, entry_fn};
use std::path::Path;
use tracing::{debug, info};

/// A discovered test: its name and the function that runs it.
///
/// TODO: Add whether should_panic, skip and other test attributes.
struct TestCase {
    name: String,
    func: Instance,
}

/// Discover the crate's tests and interpret those matching `filter`.
///
/// Returns `Ok(true)` if all executed tests passed, `Ok(false)` if any
/// failed. Errors if the test harness cannot be decoded.
pub fn run_tests(
    filter: Option<&str>,
    check_config: CheckConfig,
    native_libs: &[impl AsRef<Path>],
) -> Result<bool> {
    load_native_libs(native_libs)?;

    let tests = discover_tests()?;
    let selected: Vec<_> = tests
        .into_iter()
        .filter(|t| filter.is_none_or(|f| t.name.contains(f)))
        .collect();

    info!("running {} test(s)", selected.len());
    let mut passed = 0;
    let mut failed = 0;
    for test in &selected {
        match run_one(test, &check_config) {
            true => {
                println!("test {} ... ok", test.name);
                passed += 1;
            }
            false => {
                println!("test {} ... FAILED", test.name);
                failed += 1;
            }
        }
    }

    println!("\ntest result: {passed} passed; {failed} failed");
    Ok(failed == 0)
}

/// Interpret a single test, catching panics. Returns whether it passed.
fn run_one(test: &TestCase, check_config: &CheckConfig) -> bool {
    let mut memory = ThreadMemory::new();
    memory.check_config = check_config.clone();
    // The test fn is a closure's `call_once`, which takes the closure
    // environment as its receiver. `#[test]` closures capture nothing, so the
    // environment is a ZST — pass an empty value.
    let env = crate::value::Value::unit().clone();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        invoke_fn(test.func, &mut memory, vec![env], &mut None)
    }));
    match result {
        Ok(Ok(_)) => true,
        Ok(Err(e)) => {
            debug!("test {} errored: {e}", test.name);
            false
        }
        Err(_) => false,
    }
}

/// Walk the harness `main` to recover the list of `#[test]` functions.
fn discover_tests() -> Result<Vec<TestCase>> {
    let entry = entry_fn().ok_or_else(|| {
        anyhow::anyhow!("no entry function found (is the crate compiled with --test?)")
    })?;

    let (array_id, test_ty) = test_array(entry)?;
    let GlobalAlloc::Memory(array) = GlobalAlloc::from(array_id) else {
        bail!("test array allocation is not memory");
    };

    // The array holds one `&TestDescAndFn` pointer per test; each provenance
    // entry points at a `TestDescAndFn` allocation.
    let mut tests = Vec::new();
    for (_, prov) in &array.provenance.ptrs {
        tests.push(decode_test(prov.0, test_ty)?);
    }
    Ok(tests)
}

/// Find the promoted `&[&TestDescAndFn; N]` array in the harness main and
/// return its alloc id plus the `TestDescAndFn` element type.
fn test_array(entry: CrateItem) -> Result<(AllocId, Ty)> {
    use rustc_public::mir::{Operand, Rvalue, StatementKind};
    use rustc_public::ty::ConstantKind;

    let instance = Instance::try_from(entry)
        .map_err(|e| anyhow::anyhow!("cannot make instance from entry: {e:?}"))?;
    let body = instance
        .body()
        .ok_or_else(|| anyhow::anyhow!("entry function has no body"))?;

    for block in &body.blocks {
        for stmt in &block.statements {
            let StatementKind::Assign(_, Rvalue::Use(Operand::Constant(c), _)) = &stmt.kind else {
                continue;
            };
            let ConstantKind::Allocated(alloc) = c.const_.kind() else {
                continue;
            };
            let Some(test_ty) = element_adt(c.const_.ty()) else {
                continue;
            };
            if is_test_desc(test_ty)
                && let Some((_, prov)) = alloc.provenance.ptrs.first()
            {
                return Ok((prov.0, test_ty));
            }
        }
    }
    bail!("could not find the test harness array (unsupported test setup?)")
}

/// Decode one `TestDescAndFn` allocation into a `TestCase`.
fn decode_test(id: AllocId, test_ty: Ty) -> Result<TestCase> {
    let GlobalAlloc::Memory(alloc) = GlobalAlloc::from(id) else {
        bail!("TestDescAndFn allocation is not memory");
    };

    // `testfn` field holds a `TestFn`; its `StaticTestFn` payload is a fn
    // pointer, found via a provenance entry inside the field's byte range.
    let (testfn_off, testfn_ty) = field(test_ty, "testfn")?;
    let func_id = ptr_in_range(&alloc, testfn_off, size_of_ty(testfn_ty)?)?;
    let GlobalAlloc::Function(func) = GlobalAlloc::from(func_id) else {
        bail!("test function is not a StaticTestFn (unsupported test kind)");
    };

    // `desc.name` holds a `TestName`; its `StaticTestName` payload is a &str.
    let (desc_off, desc_ty) = field(test_ty, "desc")?;
    let (name_off, name_ty) = field(desc_ty, "name")?;
    let abs_name_off = desc_off + name_off;
    let name_id = ptr_in_range(&alloc, abs_name_off, size_of_ty(name_ty)?)?;
    let name = read_str(name_id)?;

    Ok(TestCase { name, func })
}

/// Read the single provenance pointer whose offset falls in `[off, off+size)`.
fn ptr_in_range(alloc: &rustc_public::ty::Allocation, off: usize, size: usize) -> Result<AllocId> {
    let mut found = None;
    for (o, prov) in &alloc.provenance.ptrs {
        let o = *o;
        if o >= off && o < off + size {
            if found.is_some() {
                bail!("multiple pointers in field range; cannot decode");
            }
            found = Some(prov.0);
        }
    }
    found.ok_or_else(|| anyhow::anyhow!("no pointer found in field range (unsupported test kind)"))
}

/// Read a `&str`'s bytes from a string allocation.
fn read_str(id: AllocId) -> Result<String> {
    let GlobalAlloc::Memory(alloc) = GlobalAlloc::from(id) else {
        bail!("test name is not a static string");
    };
    let bytes: Vec<u8> = alloc.bytes.iter().map(|b| b.unwrap_or(0)).collect();
    String::from_utf8(bytes).map_err(|e| anyhow::anyhow!("test name is not valid UTF-8: {e}"))
}

/// Look up a struct field's byte offset and type by name.
fn field(ty: Ty, name: &str) -> Result<(usize, Ty)> {
    let TyKind::RigidTy(RigidTy::Adt(adt, args)) = ty.kind() else {
        bail!("expected a struct type for field `{name}`");
    };
    let variant = adt
        .variants()
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("type has no variant"))?;
    let idx = variant
        .fields()
        .iter()
        .position(|f| f.name.as_str() == name)
        .ok_or_else(|| anyhow::anyhow!("no field `{name}`"))?;
    let field_ty = variant.fields()[idx].ty_with_args(&args);

    let FieldsShape::Arbitrary { offsets } = ty
        .layout()
        .map_err(|e| anyhow::anyhow!("no layout: {e:?}"))?
        .shape()
        .fields
    else {
        bail!("field `{name}` has no arbitrary layout");
    };
    let off = offsets
        .get(idx)
        .ok_or_else(|| anyhow::anyhow!("no offset for field `{name}`"))?
        .bytes();
    Ok((off, field_ty))
}

/// The size in bytes of a type.
fn size_of_ty(ty: Ty) -> Result<usize> {
    Ok(ty
        .layout()
        .map_err(|e| anyhow::anyhow!("no layout: {e:?}"))?
        .shape()
        .size
        .bytes())
}

/// Peel `&[&T; N]` / `&[&T]` to the element ADT type `T`.
fn element_adt(ty: Ty) -> Option<Ty> {
    let mut cur = ty;
    loop {
        match cur.kind() {
            TyKind::RigidTy(RigidTy::Ref(_, inner, _) | RigidTy::RawPtr(inner, _)) => cur = inner,
            TyKind::RigidTy(RigidTy::Array(inner, _) | RigidTy::Slice(inner)) => cur = inner,
            TyKind::RigidTy(RigidTy::Adt(..)) => return Some(cur),
            _ => return None,
        }
    }
}

/// Whether an ADT type is `test::TestDescAndFn`.
fn is_test_desc(ty: Ty) -> bool {
    let TyKind::RigidTy(RigidTy::Adt(adt, _)) = ty.kind() else {
        return false;
    };
    adt.name().ends_with("test::TestDescAndFn")
}
