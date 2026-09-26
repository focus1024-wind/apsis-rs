//! Ephemeris sampling and interpolation.
//!
//! A propagator gives the state of a satellite at any instant, but calling
//! it millions of times is wasteful when the result is only needed to draw a
//! track or to search for a crossing. This crate records the state at a
//! grid of times, called samples, and answers queries between them by
//! interpolation.
//!
//! The two halves are deliberately independent:
//!
//! - [`Ephemeris`] is a table of times and states. It knows nothing about
//!   where the states came from, so a table propagated by SGP4, computed by
//!   two-body theory, or read from a file all work the same way.
//! - [`Sampler`] fills a table by calling a function that produces a state
//!   from a time. In this workspace that function is normally
//!   `apsis_sgp4::Satellite::propagate_at`, but the sampler only requires
//!   a closure, so a test can use an analytic function.
//!
//! Interpolation is Hermite cubic, which uses both the position and the
//! velocity at each end of an interval. Using the velocity is what makes it
//! accurate: a cubic through two positions alone has no information about
//! how fast the satellite is moving, while Hermite reconstruction matches
//! the slope at both ends and so reproduces a smooth orbit far better for
//! the same sample spacing. On a circular orbit the error is of order the
//! fourth power of the step, so halving the step cuts the error about
//! sixteen fold.
//!
//! Times are in seconds from an origin chosen by the caller, matching the
//! rest of the orbit layer. Positions are in kilometers and velocities in
//! kilometers per second.
//!
//! ```
//! use apsis_ephemeris::{Sample, Sampler};
//! use apsis_kepler::{GravityParameter, KeplerianElements};
//!
//! // A circular orbit at the Earth's equatorial radius.
//! let elements = KeplerianElements {
//!     semi_major_axis: 7_000.0,
//!     eccentricity: 0.0,
//!     inclination: 0.9,
//!     right_ascension: 0.0,
//!     argument_of_perigee: 0.0,
//!     true_anomaly: 0.0,
//! };
//! let state_at = |seconds: f64| {
//!     let mean = 2.0 * core::f64::consts::PI * seconds
//!         / elements.period(GravityParameter::WGS84);
//!     let state = elements
//!         .with_mean_anomaly(mean)
//!         .to_state(GravityParameter::WGS84)
//!         .expect("the orbit is valid");
//!     Sample::new(seconds, state.position, state.velocity)
//! };
//!
//! let sampler = Sampler::new(0.0, 600.0, 60.0);
//! let ephemeris = sampler.sample(state_at).expect("the grid is usable");
//! assert_eq!(ephemeris.len(), 11);
//!
//! // A query between two samples is interpolated.
//! let state = ephemeris.state_at(30.0).expect("the time is covered");
//! assert!(state.position.norm() > 6_900.0);
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod ephemeris;
mod error;
mod sampler;

pub use ephemeris::{Ephemeris, Sample, hermite_position, hermite_state, hermite_velocity};
pub use error::{EphemerisError, EphemerisErrorKind};
pub use sampler::Sampler;

pub use apsis_math::Vec3;
