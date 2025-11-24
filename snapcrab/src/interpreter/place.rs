//! Place handling for MIR interpretation.
//!
//! This module provides utilities for working with MIR places, including
//! resolving place references to memory addresses and handling projections
//! like dereferencing.

use crate::memory;
use crate::ty::MonoType;
use crate::value::Value;
use anyhow::{Result, anyhow, bail};
use rustc_public::mir::{Place, ProjectionElem};

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
        if place.projection.is_empty() {
            self.frame.get_local_address(place.local)
        } else if place.projection.len() == 1 {
            match &place.projection[0] {
                ProjectionElem::Deref => {
                    let deref_place = Place {
                        local: place.local,
                        projection: place.projection[1..].to_vec(),
                    };
                    let val = self.read_from_place(&deref_place)?;
                    val.as_type()
                        .ok_or_else(|| anyhow!("Expected address, but found {val:?}"))
                }
                _ => bail!("Unsupported place projection: {:?}", place.projection[0]),
            }
        } else {
            bail!("Complex place projections not supported")
        }
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
