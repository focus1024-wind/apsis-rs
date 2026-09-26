//! Parsing of two-line element sets.
//!
//! A two-line element set (TLE) describes a satellite orbit in the form
//! used by the North American Aerospace Defense Command and distributed by
//! Space-Track. The name of the satellite is carried on an optional first
//! line, and the two following lines carry the orbital data.
//!
//! The format is fixed width. Every field occupies a fixed range of columns
//! and the checksum of each line is the sum of its digits modulo ten, where
//! a minus sign counts as one. This crate parses a TLE into structured
//! fields and verifies the line checksums and the matching element set
//! number. It does not interpret the fields; the mean elements and their
//! units are the concern of `apsis-sgp4`.
//!
//! Column positions are one based, matching the documentation of the format.
//! Two conventions from the format are worth stating explicitly, because
//! they are easy to misread:
//!
//! - An exponent field is written as a signed single digit followed by the
//!   magnitude. For example ` 12345-3` means `0.12345e-3`, not `12345e-3`.
//! - An assumed decimal point is written without a decimal point. For
//!   example the eccentricity field `0006703` means `0.0006703`.
//!
//! Source: the field layout follows the format description published by
//! CelesTrak, *FAQs: Two-Line Element Set Format*, which documents the
//! columns defined by the original NORAD specification. The checksum rule
//! is the one stated there and used by Space-Track.
//!
//! ```
//! use apsis_tle::TwoLineElement;
//!
//! let text = concat!(
//!     "1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927\r\n",
//!     "2 25544  51.6416 247.4627 0006703 130.5360 325.0288 15.72125391563537\r\n",
//! );
//! let element = TwoLineElement::parse(text).expect("sample is well formed");
//! assert_eq!(element.satellite_number, 25544);
//! assert!((element.eccentricity - 0.0006703).abs() < 1e-12);
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod checksum;
mod element;
mod field;

pub use checksum::{line_checksum, verify_line_checksum};
pub use element::{Classification, TwoLineElement};
pub use field::{ElementError, ElementErrorKind, Result};
