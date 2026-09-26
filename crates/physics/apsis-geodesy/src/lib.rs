//! Reference ellipsoids and geodetic computations.
//!
//! A geodetic position is written as latitude, longitude, and height above
//! an ellipsoid. Latitude here is always geodetic, the angle between the
//! ellipsoid normal and the equatorial plane, not geocentric. The two
//! differ by up to about 0.19 degrees at mid latitudes, so the distinction
//! matters for pointing and ranging.
//!
//! [`WGS84`] is the ellipsoid used by GPS and by almost all satellite work.
//! Its defining parameters are the semi-major axis and the inverse
//! flattening; every other quantity is derived from those two, so the
//! derived values agree with the ones published in the standard to the
//! precision of the defining parameters.
//!
//! Source: National Imagery and Mapping Agency, *Department of Defense World
//! Geodetic System 1984*, Technical Report TR8350.2, third edition, 2000.
//!
//! ```
//! use apsis_geodesy::{Geodetic, WGS84};
//!
//! // The equator has zero height and a radius equal to the semi-major axis.
//! let point = Geodetic::new(0.0, 0.0, 0.0);
//! let cartesian = point.to_cartesian(WGS84);
//! assert!((cartesian.norm() - WGS84.semi_major_axis()).abs() < 1e-9);
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod ellipsoid;
mod geodetic;

pub use ellipsoid::{Ellipsoid, WGS84, WGS84_GRAVITATIONAL_PARAMETER};
pub use geodetic::{Geodetic, cartesian_from_geodetic, geodetic_from_cartesian};
