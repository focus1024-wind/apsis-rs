//! Ground station access windows.
//!
//! A satellite is accessible from a ground station during the intervals when
//! it rises above the station's elevation mask. Each such interval is a
//! pass, or access window, and the sequence of windows over a period is the
//! access schedule. This crate finds those windows from a propagated orbit
//! and reports their start, end, duration, and the geometry at their peak.
//!
//! The windows are found by sampling the elevation angle at a fixed step and
//! refining each crossing where the elevation passes the mask. The sampling
//! step sets the shortest pass that can be found, and the refinement gives
//! the crossing times to a tolerance independent of the step.
//!
//! The elevation is evaluated from the satellite position in the
//! Earth-fixed frame, so the caller supplies the transformation from the
//! orbit frame. In this workspace that is the conversion in `apsis-coord`
//! with the Earth rotation angle from `apsis-frames`.
//!
//! ```
//! use apsis_access::{AccessFinder, Pass};
//! use apsis_coverage::GroundPoint;
//! use apsis_geodesy::Geodetic;
//!
//! // A station on the equator with a ten degree mask.
//! let station = GroundPoint::new(Geodetic::new(0.0, 0.0, 0.0), 10.0_f64.to_radians());
//! let finder = AccessFinder {
//!     station,
//!     sample_step_seconds: 60.0,
//!     crossing_tolerance_seconds: 0.001,
//! };
//! assert!(finder.is_valid());
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod error;
mod finder;
mod pass;

pub use error::{AccessError, AccessErrorKind};
pub use finder::{AccessFinder, OrbitSample};
pub use pass::{Pass, PassSummary};

pub use apsis_coverage::{GroundPoint, Sensor, elevation_angle};
pub use apsis_math::Vec3;
