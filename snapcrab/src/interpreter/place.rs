//! Place handling for MIR interpretation.
//!
//! This module provides utilities for working with MIR places, including
//! resolving place references to memory addresses and handling projections
//! like dereferencing.

use crate::memory;
use crate::ty::MonoType;
use crate::value::Value;
use anyhow::{Result, bail};
use rustc_public::mir::{Place, ProjectionElem};
use rustc_public::ty::{RigidTy, TyKind};

impl super::function::FnInterpreter {
    /// Assigns a value to a place (local variable or memory location).
    pub(super) fn assign_to_place(&mut self, place: &Place, value: Value) -> Result<()> {
        let addr = self.resolve_place_addr(place)?;
        let place_ty = place.ty(self.locals())?;
        memory::write_addr(addr, value.as_bytes(), place_ty)?;
        Ok(())
    }

    /// Resolves a place to the address of the actual value.
    pub(super) fn resolve_place_addr(&self, place: &Place) -> Result<usize> {
        let initial_addr = self.frame.get_local_address(place.local)?;
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
                        let ptr_value = memory::read_addr(current_addr, current_ty)?;
                        let address = ptr_value
                            .as_type::<usize>()
                            .ok_or_else(|| anyhow::anyhow!("Expected usize pointer value"))?;

                        Ok((address, pointee_ty))
                    }
                    ProjectionElem::Field(field_idx, field_ty) => {
                        // Calculate field offset using type layout
                        let layout = current_ty.layout()?;
                        let field_offset = match layout.shape().fields {
                            rustc_public::abi::FieldsShape::Arbitrary { ref offsets } => offsets
                                .get(*field_idx)
                                .ok_or_else(|| {
                                    anyhow::anyhow!("Field index {} out of bounds", field_idx)
                                })?
                                .bytes(),
                            _ => bail!("Unsupported field layout for type: {:?}", current_ty),
                        };
                        Ok((current_addr + field_offset, *field_ty))
                    }
                    _ => bail!("Unsupported place projection: {:?}", projection),
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
        memory::read_addr(addr, place_ty)
    }
}
