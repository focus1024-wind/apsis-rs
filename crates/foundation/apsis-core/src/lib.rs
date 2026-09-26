//! Shared foundation types for the ApsisRS workspace: a general purpose
//! error type, result aliases, physical constants, and the unit marker
//! trait.
//!
//! This crate sits at the bottom of the dependency graph. It has no
//! dependency on any other ApsisRS crate, so every other crate may use it.
//!
//! Three concerns live here:
//!
//! - [`Error`] and [`Result`] are a general purpose error carrying a coarse
//!   [`ErrorKind`]. They exist so that a function whose failure has no
//!   interesting structure can report it without inventing a type, and so
//!   that the workspace has a shape to fall back on. They are deliberately
//!   *not* the error type of the layers: a layer with a domain specific
//!   failure defines its own `Error` and `ErrorKind`, whose variants name
//!   the concrete conditions a caller would want to match on. See
//!   `docs/design/errors.md` for why, and for the conventions the domain
//!   errors follow.
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
