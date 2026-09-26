//! The combined orbit type.

use apsis_kepler::{GravityParameter, KeplerianElements};
use apsis_math::Vec3;
use apsis_sgp4::{Elements, GravityModel, PropagationError, Satellite, State};
use apsis_tle::TwoLineElement;

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
}

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
