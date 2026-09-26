//! Impulsive and finite orbital maneuver models.
//!
//! A maneuver changes an orbit by applying thrust. The two models here are
//! the two ends of the spectrum.
//!
//! An impulsive maneuver, [`Impulse`], applies a velocity change
//! instantaneously, so the position is unchanged and only the velocity
//! jumps. This is the model for a chemical burn, whose duration is short
//! compared with the orbital period, and it is the model every textbook
//! delta-v budget is written in. It is exact in the limit of zero burn time
//! and the source of the closed form results for transfers between two
//! orbits.
//!
//! A finite maneuver, [`FiniteBurn`], applies a constant acceleration over a
//! duration, so the position drifts during the burn and the achieved
//! velocity change is less than the thrust would give in free space when
//! the thrust has to fight gravity. It is the model for an electric
//! thruster, whose burn may last a large fraction of an orbit, and for any
//! burn where the short duration approximation is not good enough.
//!
//! The two agree in the limit: a finite burn of acceleration `a` over a
//! duration `t` delivers the impulse `a t`, so a [`FiniteBurn`] with a
//! vanishing duration converges on the matching [`Impulse`]. The tests check
//! that limit, which is what ties the two models together.
//!
//! The impulsive model also provides the classic two impulse transfer
//! between two coplanar circular orbits, [`hohmann_transfer`], which is the
//! reference every low thrust result is compared against. It gives the two
//! burns and the total delta-v for a transfer between two circular orbits of
//! given radii, together with the transfer time, the half period of the
//! transfer ellipse.
//!
//! Conventions match the rest of the orbit layer: lengths are in kilometers,
//! velocities in kilometers per second, and angles in radians.
//!
//! ```
//! use apsis_maneuver::{Impulse, hohmann_transfer};
//! use apsis_kepler::GravityParameter;
//! use apsis_math::Vec3;
//!
//! // An impulsive burn straight along the velocity.
//! let impulse = Impulse::new(Vec3::new(0.0, 1.0, 0.0)).expect("the change is usable");
//! assert!((impulse.magnitude() - 1.0).abs() < 1e-12);
//!
//! // The two burns that raise a circular orbit from 7000 to 8000 km.
//! let transfer = hohmann_transfer(7_000.0, 8_000.0, GravityParameter::WGS84)
//!     .expect("the radii are usable");
//! assert!(transfer.first_burn > 0.0);
//! assert!(transfer.total_delta_v() > 0.0);
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod burn;
mod error;
mod hohmann;
mod impulse;

pub use burn::{BurnOutcome, FiniteBurn};
pub use error::{ManeuverError, ManeuverErrorKind};
pub use hohmann::{HohmannTransfer, hohmann_transfer};
pub use impulse::Impulse;

pub use apsis_kepler::{GravityParameter, KeplerianElements};
pub use apsis_math::Vec3;
