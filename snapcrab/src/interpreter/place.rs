//! Place handling for MIR interpretation.
//!
//! This module provides utilities for working with MIR places, including
//! resolving place references to memory addresses and handling projections
//! like dereferencing.

use crate::ty::MonoType;
use crate::value::Value;
use anyhow::{Context, Result, bail};
use rustc_public::mir::{Place, ProjectionElem};
use rustc_public::ty::{RigidTy, Ty, TyKind};

use super::function;

impl<'a> function::FnInterpreter<'a> {
    /// Assigns a value to a place (local variable or memory location).
    pub(super) fn assign_to_place(&mut self, place: &Place, value: Value) -> Result<()> {
        let addr = self.resolve_place_addr(place)?;
        let place_ty = place.ty(self.locals())?;
        self.memory.write_addr(addr, value.as_bytes(), place_ty)?;
        Ok(())
    }

    /// Resolves a place to the address of the actual value.
    ///
    /// TODO: This won't quite work for fat pointers. We might need to refactor
    /// this a bit later.
    pub(super) fn resolve_place_addr(&self, place: &Place) -> Result<usize> {
        let initial_addr = self.memory.local_address(place.local)?;
        let initial_ty = self.locals()[place.local].ty;

        let (final_addr, _) = place.projection.iter().try_fold(
            (initial_addr, initial_ty),
            |(current_addr, current_ty), projection| {
                match projection {
                    ProjectionElem::Deref => {
                        // For deref, we need to get the pointee type first
                        let pointee_ty = match current_ty.kind() {
                            TyKind::RigidTy(RigidTy::Ref(_, pointee, _))
                            | TyKind::RigidTy(RigidTy::RawPtr(pointee, _)) => pointee,
                            _ => bail!("Cannot dereference non-pointer type: {:?}", current_ty),
                        };

                        // Read the pointer value at current_addr using memory tracker
                        let ptr_value = self.memory.read_addr(current_addr, current_ty)?;
                        let address = ptr_value
                            .as_type::<usize>()
                            .context("Expected usize pointer value")?;

                        Ok((address, pointee_ty))
                    }
                    ProjectionElem::Field(field_idx, field_ty) => {
                        // Calculate field offset using type layout
                        let layout = current_ty.layout()?;
                        let field_offset = match layout.shape().fields {
                            rustc_public::abi::FieldsShape::Arbitrary { ref offsets } => offsets
                                .get(*field_idx)
                                .with_context(|| {
                                    format!("Field index {} out of bounds", field_idx)
                                })?
                                .bytes(),
                            rustc_public::abi::FieldsShape::Union(_) => {
                                // All union fields start at offset 0
                                0
                            }
                            _ => bail!("Unsupported field layout for type: {:?}", current_ty),
                        };
                        Ok((current_addr + field_offset, *field_ty))
                    }
                    ProjectionElem::Index(local) => {
                        // Get the index value from the local
                        let index_value = self.memory.read_local(*local, Ty::usize_ty())?;
                        let index = index_value
                            .as_type::<usize>()
                            .context("Expected usize index value")?;

                        // Get array element type and stride
                        let (element_ty, stride) = match current_ty.kind() {
                            TyKind::RigidTy(RigidTy::Array(elem_ty, _)) => {
                                let layout = current_ty.layout()?;
                                let stride = match layout.shape().fields {
                                    rustc_public::abi::FieldsShape::Array { stride, .. } => {
                                        stride.bytes()
                                    }
                                    shape => bail!(
                                        "Expected array field shape for `{current_ty:?}`: {shape:?}"
                                    ),
                                };
                                (elem_ty, stride)
                            }
                            _ => bail!("Cannot index non-array type: {current_ty:?}"),
                        };

                        Ok((current_addr + index * stride, element_ty))
                    }
                    _ => bail!("Unsupported place projection: {projection:?}"),
                }
            },
        )?;

        Ok(final_addr)
    }

    /// Reads a value from a place (local variable or memory location).
    pub(super) fn read_from_place(&self, place: &Place) -> Result<Value> {
        let place_ty = place.ty(self.locals())?;
        if place_ty.size()? == 0 {
            return Ok(Value::unit().clone());
        }

        let addr = self.resolve_place_addr(place)?;
        self.memory.read_addr(addr, place_ty)
    }
}
