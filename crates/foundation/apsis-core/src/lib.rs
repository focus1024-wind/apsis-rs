//! Shared foundation types for the ApsisRS workspace: error handling,
//! result aliases, physical constants, and the unit marker trait.
//!
//! This crate sits at the bottom of the dependency graph. It has no
//! dependency on any other ApsisRS crate, so every other crate may use it.
//!
//! Three concerns live here:
//!
//! - [`Error`] and [`Result`] give the workspace a single error surface
//!   without pulling in an external error handling crate.
//! - [`constant`] holds physical and astronomical constants expressed in
//!   SI units.
//! - [`Quantity`] marks newtypes that carry a physical unit, so generic
//!   code can accept any quantity and still recover its unit type.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod constant;
mod error;
mod quantity;

pub use error::{Error, ErrorKind, Result};
pub use quantity::{Quantity, Unit};
