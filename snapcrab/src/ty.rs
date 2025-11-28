//! Module with type extensions
use anyhow::Result;
use rustc_public::ty::Ty;

pub trait MonoType {
    /// Return the size of the type in bytes.
    fn size(&self) -> Result<usize>;

    /// Return the alignment of the type in bytes.
    fn alignment(&self) -> Result<usize>;

    /// Check if this is a thin pointer (single usize).
    fn is_thin_ptr(&self) -> bool;

    /// Check if this is a wide pointer (two usize values).
    #[allow(dead_code)]
    fn is_wide_ptr(&self) -> bool;
}

impl MonoType for Ty {
    /// Get the size in bytes of a type
    fn size(&self) -> Result<usize> {
        Ok(self.layout().map(|layout| layout.shape().size.bytes())?)
    }

    /// Get the alignment in bytes of a type
    fn alignment(&self) -> Result<usize> {
        Ok(self
            .layout()
            .map(|layout| layout.shape().abi_align as usize)?)
    }

    fn is_thin_ptr(&self) -> bool {
        let ptr_size = crate::memory::pointer_width();
        self.kind().is_any_ptr() && self.size().ok() == Some(ptr_size)
    }

    fn is_wide_ptr(&self) -> bool {
        let ptr_size = crate::memory::pointer_width();
        self.kind().is_any_ptr() && self.size().ok() == Some(2 * ptr_size)
    }
}
