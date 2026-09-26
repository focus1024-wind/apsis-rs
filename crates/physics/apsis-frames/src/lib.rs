//! Reference frame definitions and the transformations between them.
//!
//! A reference frame fixes the orientation and origin that a vector is
//! expressed relative to. This crate defines the two frames used for
//! near-Earth work and the rotation between them.
//!
//! The inertial frame is the Earth-centered inertial (ECI) frame aligned
//! with the mean equator and equinox of J2000.0. Its axes are fixed in
//! inertial space, so an orbit propagated in it is not affected by the
//! rotation of the Earth.
//!
//! The rotating frame is the Earth-centered Earth-fixed (ECEF) frame. Its
//! third axis is the Earth's rotation axis and its first axis follows the
//! prime meridian, so a point fixed to the ground has constant
//! coordinates in it.
//!
//! The two are related by a rotation of [`EarthRotationAngle`] about the
//! common third axis. This crate models the Earth's rotation with the
//! Earth Rotation Angle, which is a function of UT1 only. It does not
//! apply precession or nutation, because an angle accurate to a few
//! hundredths of a degree is sufficient for pointing and coverage work and
//! the full model needs the orientation of the celestial pole, which is a
//! separate computation.
//!
//! Source: the Earth Rotation Angle expression is from the International
//! Astronomical Union, as given in the *IERS Conventions (2010)*, equation
//! 8.32.
//!
//! ```
//! use apsis_frames::{EarthRotationAngle, rotation_eci_to_ecef};
//!
//! // The angle advances by roughly 360.9856 degrees per day.
//! let first = EarthRotationAngle::from_ut1_days(0.0);
//! let second = EarthRotationAngle::from_ut1_days(1.0);
//! let advance = (second.radians() - first.radians()).to_degrees();
//! assert!((advance - 360.985_612_288_088).abs() < 1e-6);
//!
//! // The rotation is ready to apply to a vector.
//! let _ = rotation_eci_to_ecef(first);
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod earth_rotation;
mod frame;

pub use earth_rotation::{
    ERA_J2000_TURNS, ERA_RATE_TURNS_PER_DAY, EarthRotationAngle, rotation_ecef_to_eci,
    rotation_eci_to_ecef, ut1_days_from_j2000,
};
pub use frame::{Ecef, Eci};
