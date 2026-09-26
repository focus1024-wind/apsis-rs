//! Orbital state facade that integrates the orbit layer.
//!
//! This crate is the single entry point to the orbit layer. It re-exports
//! the element parsing, the analytical propagator, and the two-body model,
//! and adds one type that ties them together.
//!
//! [`Orbit`] holds a satellite prepared for propagation together with the
//! gravity model it was initialised against, and offers the conversions
//! between the two ways of describing an orbit that the layer supports:
//!
//! - Mean elements propagated by SGP4, which is the model a two-line
//!   element set is written for and the only way to reproduce the published
//!   positions of a catalogued satellite.
//! - Instantaneous Keplerian elements, which describe the osculating
//!   two-body orbit at one instant and are useful for inspection and for
//!   planning a maneuver.
//!
//! The two descriptions are not interchangeable. Propagating mean elements
//! with the two-body model, or osculating elements with SGP4, gives a
//! different and wrong answer; the conversions here are explicit about which
//! direction they go.
//!
//! Two further operations are built on the propagation and exposed here
//! because they need the same gravity parameter to stay consistent:
//!
//! - [`Orbit::sample_ephemeris`] records the track on a time grid so that it
//!   can be queried by interpolation instead of by propagation, which is
//!   what a coverage or access search wants.
//! - [`Orbit::apply_burn`] applies a finite maneuver to the osculating state
//!   and returns the resulting elements, which is what a maneuver plan
//!   produces.
//!
//! ```
//! use apsis_orbit::{GravityModel, Orbit};
//! use apsis_tle::TwoLineElement;
//!
//! let text = concat!(
//!     "1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927\r\n",
//!     "2 25544  51.6416 247.4627 0006703 130.5360 325.0288 15.72125391563537\r\n",
//! );
//! let element = TwoLineElement::parse(text).expect("sample is well formed");
//! let orbit = Orbit::from_element(&element, GravityModel::WGS72)
//!     .expect("sample elements are propagatable");
//!
//! let state = orbit.propagate_minutes(60.0).expect("state exists");
//! assert!(state.speed() > 7.0);
//!
//! // The osculating elements at that instant.
//! let osculating = orbit.osculating_elements(60.0).expect("state exists");
//! assert!(osculating.eccentricity < 0.01);
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod orbit;

pub use orbit::{EphemerisSamplingError, ManeuverApplicationError, Orbit};

pub use apsis_ephemeris::{Ephemeris, EphemerisError, EphemerisErrorKind, Sample, Sampler};
pub use apsis_kepler::{GravityParameter, KeplerianElements};
pub use apsis_maneuver::{
    BurnOutcome, FiniteBurn, HohmannTransfer, Impulse, ManeuverError, ManeuverErrorKind,
    hohmann_transfer,
};
pub use apsis_sgp4::{
    Elements, GravityModel, PropagationError, PropagationErrorKind, Satellite, State,
};
pub use apsis_tle::{Classification, ElementError, ElementErrorKind, TwoLineElement};
