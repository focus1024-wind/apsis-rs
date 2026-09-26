//! Macros that support the ApsisRS workspace.
//!
//! This crate currently defines no macros. It exists so that the workspace
//! has a single place for macro definitions if a need arises, and so that
//! adding one later does not require a new crate.
//!
//! The decision to leave it empty follows from what the workspace actually
//! needs. Rust's built in `macro_rules!` can already do everything the
//! foundation layer requires:
//!
//! - Substituting a type and a literal to build a quantity, as
//!   `quantity!(Meters, 1.5)`.
//! - Dispatching on a trailing token to accept a unit suffix, as
//!   `to_meters!(2.5 km)`.
//!
//! Both forms are demonstrated by the tests in `tests/probe.rs`. They can be
//! written directly in the crate that uses them, so no shared macro crate is
//! needed.
//!
//! Two limits are worth recording, because they explain why a macro based
//! unit system is not provided:
//!
//! - A `macro_rules!` macro cannot be generic over a unit suffix that is not
//!   an identifier. `1.5 m` and `1.5 km` are accepted by separate rules, so
//!   the macro must enumerate every suffix it supports. The unit newtypes in
//!   `apsis-units` already give compile time unit safety without that list,
//!   so the macro would add surface area without adding checking.
//! - A declarative macro cannot inspect the value of its arguments, so it
//!   cannot fold a unit conversion at compile time beyond what constant
//!   evaluation already does. A literal such as `2.5 * 1000.0` is folded by
//!   the compiler anyway.
//!
//! If the workspace later needs macros that generate items, such as deriving
//! a set of conversions from one definition, that is the point at which this
//! crate should gain its first macro.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
