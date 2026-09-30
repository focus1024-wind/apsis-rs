//! The SGP4 and SDP4 analytical orbit propagator.
//!
//! The propagator takes the mean orbital elements of a two-line element set
//! and returns the position and velocity of the satellite at a requested
//! instant. It is the model that the element sets are fitted to, so it is
//! the correct way to use them; a general purpose numerical integrator
//! given the same elements would drift away from the published positions.
//!
//! Two branches exist and the propagator selects between them by orbital
//! period. A satellite with a period below 225 minutes is propagated with
//! SGP4, which models the secular effect of the Earth's oblateness and a
//! drag term. A satellite at or above that period is propagated with SDP4,
//! which adds the resonance and the luni-solar perturbations that dominate
//! at that altitude.
//!
//! Inputs and outputs:
//!
//! - Elements are the mean elements from [`apsis_tle::TwoLineElement`].
//!   Angles in the element set are in degrees and mean motion is in
//!   revolutions per day; this crate converts to radians and radians per
//!   minute internally.
//! - The output is expressed in the True Equator Mean Equinox (TEME) frame,
//!   with position in kilometers and velocity in kilometers per second. TEME
//!   is the frame the model defines; converting it to an Earth-fixed frame
//!   requires the Earth rotation and the equation of the equinoxes and is
//!   not part of this crate.
//! - Time is measured from the element set epoch, in minutes.
//!
//! Source: the algorithm follows Vallado et al., *Revisiting Spacetrack
//! Report #3*, AIAA 2006-6753, 2006, and the revised implementation
//! distributed with that paper.
//!
//! ```
//! use apsis_sgp4::{GravityModel, Satellite};
//! use apsis_tle::TwoLineElement;
//!
//! let text = concat!(
//!     "1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927\r\n",
//!     "2 25544  51.6416 247.4627 0006703 130.5360 325.0288 15.72125391563537\r\n",
//! );
//! let element = TwoLineElement::parse(text).expect("sample is well formed");
//! let satellite = Satellite::from_element(&element, GravityModel::WGS72)
//!     .expect("sample elements are propagatable");
//!
//! // Propagate to the epoch of the elements.
//! let state = satellite.propagate_minutes(0.0).expect("epoch state exists");
//! // The position of a low Earth satellite is a few thousand kilometers
//! // from the center of the Earth.
//! assert!(state.position.norm() > 6_600.0);
//! assert!(state.position.norm() < 7_200.0);
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod batch;
mod deep_space;
mod error;
mod init;
mod model;
mod propagate;
mod satellite;
mod state;

pub use batch::{
    propagate_at, propagate_at_parallel, propagate_minutes_at, propagate_minutes_at_parallel,
};
pub use error::{PropagationError, PropagationErrorKind};
pub use init::Elements;
pub use model::GravityModel;
pub use satellite::Satellite;
pub use state::State;

pub use apsis_tle::TwoLineElement;
