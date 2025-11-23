//! Module with type extensions
use anyhow::Result;
use rustc_public::ty::Ty;

pub trait MonoType {
    /// Return the size of the type in bytes.
    fn size(&self) -> Result<usize>;
}

impl MonoType for Ty {
    /// Get the size in bytes of a type
    fn size(&self) -> Result<usize> {
        Ok(self
            .layout()
            .map(|layout| layout.shape().size.bytes())?)
    }
}
