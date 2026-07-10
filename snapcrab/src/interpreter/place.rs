//! Place handling for MIR interpretation.
//!
//! This module provides utilities for working with MIR places, including
//! resolving place references to memory addresses and handling projections
//! like dereferencing.

use crate::ty::MonoType;
use crate::value::Value;
use anyhow::{Context, Result, bail};
use rustc_public::abi::VariantsShape;
use rustc_public::mir::{Place, ProjectionElem};
use rustc_public::ty::{RigidTy, Ty, TyKind, VariantIdx};
use rustc_public_bridge::IndexedVal;

use super::function;

/// State tracked while resolving place projections.
struct PlaceState {
    addr: usize,
    ty: Ty,
    /// Active variant after a Downcast projection; cleared after Field use.
    downcast: Option<VariantIdx>,
}

impl<'a> function::FnInterpreter<'a> {
    /// Assigns a value to a place (local variable or memory location).
    pub(super) fn assign_to_place(&mut self, place: &Place, value: Value) -> Result<()> {
        let addr = self.resolve_place_addr(place)?;
        let place_ty = place.ty(self.locals())?;
        self.memory.write_addr(addr, value.as_bytes(), place_ty)?;
        Ok(())
    }

    /// Resolves a place to the address of the actual value.
    pub(super) fn resolve_place_addr(&self, place: &Place) -> Result<usize> {
        let initial_addr = self.memory.local_address(place.local)?;
        let initial_ty = self.locals()[place.local].ty;

        let state = place.projection.iter().try_fold(
            PlaceState {
                addr: initial_addr,
                ty: initial_ty,
                downcast: None,
            },
            |state, projection| self.apply_projection(state, projection),
        )?;

        Ok(state.addr)
    }

    fn apply_projection(
        &self,
        state: PlaceState,
        projection: &ProjectionElem,
    ) -> Result<PlaceState> {
        let PlaceState {
            addr: current_addr,
            ty: current_ty,
            downcast,
        } = state;
        match projection {
            ProjectionElem::Deref => {
                let pointee_ty = match current_ty.kind() {
                    TyKind::RigidTy(RigidTy::Ref(_, pointee, _))
                    | TyKind::RigidTy(RigidTy::RawPtr(pointee, _)) => pointee,
                    _ => bail!("Cannot dereference non-pointer type: {:?}", current_ty),
                };

                let ptr_value = self.memory.read_addr(current_addr, current_ty)?;
                let address = ptr_value.to_data_addr()?.as_type::<usize>().unwrap();

                Ok(PlaceState {
                    addr: address,
                    ty: pointee_ty,
                    downcast: None,
                })
            }
            ProjectionElem::Field(field_idx, field_ty) => {
                let field_offset = if let Some(variant_idx) = downcast {
                    // After Downcast: use variant-specific field offsets
                    let layout = current_ty.layout()?;
                    let shape = layout.shape();
                    match &shape.variants {
                        VariantsShape::Multiple { variants, .. } => {
                            let variant = &variants[variant_idx.to_index()];
                            variant
                                .offsets
                                .get(*field_idx)
                                .with_context(|| {
                                    format!(
                                        "Field index {} out of bounds for variant {}",
                                        field_idx,
                                        variant_idx.to_index()
                                    )
                                })?
                                .bytes()
                        }
                        VariantsShape::Single { .. } => {
                            // Single variant: use top-level fields
                            match &shape.fields {
                                rustc_public::abi::FieldsShape::Arbitrary { offsets } => offsets
                                    .get(*field_idx)
                                    .with_context(|| {
                                        format!("Field index {} out of bounds", field_idx)
                                    })?
                                    .bytes(),
                                _ => bail!(
                                    "Unsupported field layout for single-variant: {:?}",
                                    current_ty
                                ),
                            }
                        }
                        _ => bail!("Unsupported variant shape for Downcast"),
                    }
                } else {
                    let layout = current_ty.layout()?;
                    match layout.shape().fields {
                        rustc_public::abi::FieldsShape::Arbitrary { ref offsets } => offsets
                            .get(*field_idx)
                            .with_context(|| format!("Field index {} out of bounds", field_idx))?
                            .bytes(),
                        rustc_public::abi::FieldsShape::Union(_) => 0,
                        _ => bail!("Unsupported field layout for type: {:?}", current_ty),
                    }
                };
                Ok(PlaceState {
                    addr: current_addr + field_offset,
                    ty: *field_ty,
                    downcast: None,
                })
            }
            ProjectionElem::Downcast(variant_idx) => Ok(PlaceState {
                addr: current_addr,
                ty: current_ty,
                downcast: Some(*variant_idx),
            }),
            ProjectionElem::Index(local) => {
                let index_value = self.memory.read_local(*local, Ty::usize_ty())?;
                let index = index_value
                    .as_type::<usize>()
                    .context("Expected usize index value")?;

                let (element_ty, stride) = match current_ty.kind() {
                    TyKind::RigidTy(RigidTy::Array(elem_ty, _))
                    | TyKind::RigidTy(RigidTy::Slice(elem_ty)) => {
                        let layout = current_ty.layout()?;
                        let stride = match layout.shape().fields {
                            rustc_public::abi::FieldsShape::Array { stride, .. } => stride.bytes(),
                            shape => {
                                bail!("Expected array field shape for `{current_ty:?}`: {shape:?}")
                            }
                        };
                        (elem_ty, stride)
                    }
                    _ => bail!("Cannot index non-array type: {current_ty:?}"),
                };

                Ok(PlaceState {
                    addr: current_addr + index * stride,
                    ty: element_ty,
                    downcast: None,
                })
            }
            _ => bail!("Unsupported place projection: {projection:?}"),
        }
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
