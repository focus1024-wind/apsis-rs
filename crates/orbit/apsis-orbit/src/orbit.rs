//! The combined orbit type.

use apsis_ephemeris::EphemerisError;
use apsis_kepler::{GravityParameter, KeplerianElements};
use apsis_maneuver::ManeuverError;
use apsis_math::Vec3;
use apsis_sgp4::{Elements, GravityModel, PropagationError, Satellite, State};
use apsis_tle::TwoLineElement;
use core::fmt;

/// A satellite orbit, held as mean elements ready for propagation.
///
/// The type pairs a prepared SGP4 satellite with the gravity model it was
/// initialised against, because the osculating element conversion needs the
/// same gravitational parameter to be consistent with the propagation.
#[derive(Debug, Clone)]
pub struct Orbit {
    satellite: Satellite,
    parameter: GravityParameter,
}

impl Orbit {
    /// Prepares an orbit from a parsed element set and a gravity model.
    ///
    /// The gravity model must match the one the element set was fitted
    /// against, which for a catalogued satellite means
    /// [`GravityModel::WGS72`]. Returns `None` when the elements are not
    /// propagatable.
    pub fn from_element(element: &TwoLineElement, model: GravityModel) -> Option<Self> {
        Self::try_from_element(element, model).ok()
    }

    /// Prepares an orbit from a parsed element set and a gravity model.
    ///
    /// Returns an error describing which element is not propagatable.
    pub fn try_from_element(
        element: &TwoLineElement,
        model: GravityModel,
    ) -> Result<Self, PropagationError> {
        let satellite = Satellite::try_from_element(element, model)?;
        Ok(Self {
            satellite,
            parameter: parameter_for(model),
        })
    }

    /// Prepares an orbit from mean elements already in model units.
    pub fn from_elements(
        elements: &Elements,
        model: GravityModel,
    ) -> Result<Self, PropagationError> {
        let satellite = Satellite::try_from_elements(elements, model)?;
        Ok(Self {
            satellite,
            parameter: parameter_for(model),
        })
    }

    /// Returns the underlying propagator.
    pub fn satellite(&self) -> &Satellite {
        &self.satellite
    }

    /// Returns the gravitational parameter paired with the gravity model.
    ///
    /// The value is the one used for the osculating element conversion, so
    /// that the conversion is consistent with the propagation.
    pub fn gravity_parameter(&self) -> GravityParameter {
        self.parameter
    }

    /// Returns the epoch of the elements as a Modified Julian Date on the
    /// UTC scale.
    ///
    /// The epoch is the value written in the element set, which is the
    /// origin the propagator measures time from.
    pub fn epoch_mjd(&self) -> f64 {
        self.satellite.epoch_mjd()
    }

    /// Returns `true` when the propagator uses the deep space branch.
    pub fn is_deep_space(&self) -> bool {
        self.satellite.is_deep_space()
    }

    /// Returns the state at a number of minutes from the element epoch.
    ///
    /// Position is in kilometers and velocity in kilometers per second, in
    /// the True Equator Mean Equinox frame.
    pub fn propagate_minutes(&self, minutes: f64) -> Result<State, PropagationError> {
        self.satellite.propagate_minutes(minutes)
    }

    /// Returns the state at a Modified Julian Date on the UTC scale.
    pub fn propagate_at(&self, utc_mjd: f64) -> Result<State, PropagationError> {
        self.propagate_minutes((utc_mjd - self.satellite.epoch_mjd()) * 1_440.0)
    }

    /// Returns the osculating Keplerian elements at a number of minutes from
    /// the element epoch.
    ///
    /// The osculating elements describe the two-body orbit that matches the
    /// position and velocity at that instant. They are not the mean elements
    /// of the element set, and they change along the orbit because SGP4
    /// includes perturbations that the two-body model does not.
    ///
    /// Returns an error when the state cannot be propagated, or when the
    /// instantaneous orbit is not elliptical, which can happen for a state
    /// the propagation has pushed onto an escape trajectory.
    pub fn osculating_elements(&self, minutes: f64) -> Result<KeplerianElements, PropagationError> {
        let state = self.propagate_minutes(minutes)?;
        // The two crates define their own state type, so the components are
        // transferred explicitly rather than relying on a shared type.
        let kepler_state = apsis_kepler::State {
            position: state.position,
            velocity: state.velocity,
        };
        KeplerianElements::from_state(&kepler_state, self.parameter).ok_or_else(|| {
            PropagationError::new(apsis_sgp4::PropagationErrorKind::OutOfRange)
                .with_detail("the instantaneous orbit is not elliptical")
        })
    }

    /// Returns the magnitude of the state vector.
    ///
    /// This is a convenience for the common case of wanting only the
    /// distance from the center of the Earth.
    pub fn radius_at(&self, minutes: f64) -> Result<f64, PropagationError> {
        self.propagate_minutes(minutes)
            .map(|state| state.position.norm())
    }

    /// Returns the position at a number of minutes from the element epoch.
    pub fn position_at(&self, minutes: f64) -> Result<Vec3, PropagationError> {
        self.propagate_minutes(minutes).map(|state| state.position)
    }

    /// Samples the orbit into an ephemeris over a range of minutes.
    ///
    /// The propagation is the expensive part of a track, so a table built
    /// once and queried many times is worth the memory. The sample times are
    /// minutes from the element epoch, matching the rest of this type, while
    /// the returned ephemeris keeps seconds, matching `apsis-ephemeris`.
    ///
    /// The table is built with position and velocity from the propagator,
    /// so the Hermite interpolation has exact endpoint slopes and the error
    /// between samples is tiny; there is no need to use the position only
    /// route except for a source that has no velocity.
    ///
    /// Returns an error when the grid is not usable or when a sample cannot
    /// be propagated, which happens when the elements have decayed or the
    /// range leaves the region where the model is defined.
    pub fn sample_ephemeris(
        &self,
        start_minutes: f64,
        end_minutes: f64,
        step_minutes: f64,
    ) -> Result<apsis_ephemeris::Ephemeris, EphemerisSamplingError> {
        let sampler = apsis_ephemeris::Sampler::new(
            start_minutes * 60.0,
            end_minutes * 60.0,
            step_minutes * 60.0,
        );
        if !sampler.is_valid() {
            return Err(EphemerisSamplingError::InvalidGrid);
        }
        let mut failure = None;
        let ephemeris = sampler.sample(|seconds| {
            // The minutes are recovered from the seconds the sampler uses,
            // so a non-integer step stays consistent with the grid.
            match self.propagate_minutes(seconds / 60.0) {
                Ok(state) => apsis_ephemeris::Sample::new(seconds, state.position, state.velocity),
                Err(error) => {
                    // The sampler cannot report the propagation failure, so
                    // it is recorded and the sample is filled with a value
                    // the table constructor will then reject.
                    failure = Some(error);
                    apsis_ephemeris::Sample::new(
                        seconds,
                        Vec3::new(f64::NAN, f64::NAN, f64::NAN),
                        Vec3::new(f64::NAN, f64::NAN, f64::NAN),
                    )
                }
            }
        });
        if let Some(error) = failure {
            return Err(EphemerisSamplingError::Propagation(error));
        }
        ephemeris.map_err(EphemerisSamplingError::Ephemeris)
    }

    /// Applies a finite burn and returns the resulting osculating elements.
    ///
    /// The burn is applied to the osculating state at the given time, so the
    /// manoeuvre is described relative to the orbit the satellite is
    /// actually on rather than the mean elements. The resulting elements are
    /// the osculating elements of the post burn state, which is what a
    /// following propagation needs. Because the underlying propagator is
    /// built for one element set, this is a planning result rather than a
    /// new propagator; a caller that wants to propagate the new orbit starts
    /// a fresh `Orbit` from the returned elements.
    pub fn apply_burn(
        &self,
        minutes: f64,
        burn: apsis_maneuver::FiniteBurn,
        steps: usize,
    ) -> Result<KeplerianElements, ManeuverApplicationError> {
        let state = self
            .propagate_minutes(minutes)
            .map_err(ManeuverApplicationError::Propagation)?;
        let kepler_state = apsis_kepler::State {
            position: state.position,
            velocity: state.velocity,
        };
        let outcome = burn
            .apply(kepler_state, self.parameter, steps)
            .map_err(ManeuverApplicationError::Maneuver)?;
        KeplerianElements::from_state(&outcome.state, self.parameter)
            .ok_or(ManeuverApplicationError::NotElliptical)
    }
}

/// A failure while sampling an orbit into an ephemeris.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EphemerisSamplingError {
    /// The requested grid is not usable: the step is not positive or the
    /// range is empty or reversed.
    InvalidGrid,
    /// A sample could not be propagated.
    Propagation(PropagationError),
    /// The table rejected the samples, which should not happen for a valid
    /// grid and is reported rather than hidden.
    Ephemeris(EphemerisError),
}

impl fmt::Display for EphemerisSamplingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidGrid => f.write_str("the sampling grid is not usable"),
            Self::Propagation(error) => write!(f, "a sample could not be propagated: {error}"),
            Self::Ephemeris(error) => write!(f, "the ephemeris rejected the samples: {error}"),
        }
    }
}

impl core::error::Error for EphemerisSamplingError {}

/// A failure while applying a burn to an orbit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManeuverApplicationError {
    /// The state before the burn could not be propagated.
    Propagation(PropagationError),
    /// The burn itself was not usable.
    Maneuver(ManeuverError),
    /// The post burn state is not an elliptical orbit, which happens when
    /// the burn is large enough to escape.
    NotElliptical,
}

impl fmt::Display for ManeuverApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Propagation(error) => write!(f, "the state could not be propagated: {error}"),
            Self::Maneuver(error) => write!(f, "the burn is not usable: {error}"),
            Self::NotElliptical => f.write_str("the post burn orbit is not elliptical"),
        }
    }
}

impl core::error::Error for ManeuverApplicationError {}

/// Returns the gravitational parameter that matches a gravity model.
///
/// The two models differ slightly in their value of the gravitational
/// parameter, and using the wrong one shifts the osculating semi-major axis
/// by a small amount. The values are taken from the model definitions rather
/// than duplicated, so a change to a model propagates here.
fn parameter_for(model: GravityModel) -> GravityParameter {
    // The two parameters differ by 0.3 km^3/s^2, about one part in a
    // million. The comparison is on the model's defining value, which is
    // exact for the constants defined in this workspace.
    if model.mu == GravityModel::WGS84.mu {
        GravityParameter::WGS84
    } else {
        GravityParameter::WGS72
    }
}
