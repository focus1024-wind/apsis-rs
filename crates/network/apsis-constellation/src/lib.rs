//! Constellation definition and instantiation.
//!
//! A constellation is a set of satellites described by a small number of
//! parameters rather than as a list of individual orbits. This crate models
//! two forms of construction that together cover the constellations in
//! operational use:
//!
//! - A **Walker constellation**, in which satellites are spread evenly over
//!   a set of orbital planes that share an inclination and a semi-major
//!   axis, with a fixed phase relationship between adjacent planes.
//! - A **custom constellation**, where each shell is described separately so
//!   that a design that does not follow the Walker pattern can still be
//!   instantiated.
//!
//! Both forms produce a list of [`KeplerianElements`] at a reference time,
//! which the orbit layer can propagate. The constellations in use for
//! satellite internet are Walker patterns at several altitudes, so the
//! Walker constructor is the one most callers need.
//!
//! The Walker notation is written `i: t/p/f`, where `i` is the inclination
//! in degrees, `t` the total number of satellites, `p` the number of planes,
//! and `f` the phase factor. The satellites are placed so that
//! `t / p` satellites share each plane and adjacent planes are offset in
//! mean anomaly by `f * 360 / t` degrees.
//!
//! Source: the Walker notation and the phasing rule are the ones defined in
//! Walker, J. G., *Circular Orbit Patterns Providing Continuous Whole Earth
//! Coverage*, Royal Aircraft Establishment Technical Report 70211, 1970.
//!
//! ```
//! use apsis_constellation::{Constellation, WalkerPattern};
//!
//! // A Walker 53 degree pattern with 24 satellites in 3 planes.
//! let pattern = WalkerPattern {
//!     total_satellites: 24,
//!     planes: 3,
//!     phase_factor: 1,
//!     inclination: 53.0_f64.to_radians(),
//!     semi_major_axis: 6_878.137,
//! };
//! let constellation =
//!     Constellation::walker(pattern, 0.0).expect("the pattern is consistent");
//! assert_eq!(constellation.satellites().len(), 24);
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod error;
mod walker;

pub use error::{ConstellationError, ConstellationErrorKind};
pub use walker::{Constellation, Satellite, WalkerPattern};

pub use apsis_kepler::{GravityParameter, KeplerianElements};
pub use apsis_sgp4::GravityModel;
