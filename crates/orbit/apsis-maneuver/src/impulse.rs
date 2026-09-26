//! The impulsive maneuver.

use crate::error::{ManeuverError, ManeuverErrorKind};
use apsis_math::Vec3;

/// An instantaneous change in velocity.
///
/// An impulse is a velocity change applied in zero time, so it changes the
/// velocity of a state but not its position. That is the model for a burn
/// short compared with the orbital period, and it is how a delta-v budget is
/// written.
///
/// The change is stored as a vector in the same frame as the state it will
/// be applied to, so the caller is responsible for expressing it in that
/// frame. The magnitude is the delta-v, the quantity a propellant budget is
/// measured in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Impulse {
    delta_v: Vec3,
}

impl Impulse {
    /// Creates an impulse from a velocity change vector, in kilometers per
    /// second.
    ///
    /// Returns an error when a component is not finite or when the vector is
    /// zero, because a zero change is not a maneuver and is almost always a
    /// mistake in a plan.
    pub fn new(delta_v: Vec3) -> Result<Self, ManeuverError> {
        if !delta_v.is_finite() {
            return Err(ManeuverError::new(ManeuverErrorKind::NotFinite)
                .with_detail("the velocity change is not finite"));
        }
        if delta_v.norm() == 0.0 {
            return Err(ManeuverError::new(ManeuverErrorKind::InvalidState)
                .with_detail("the velocity change is zero"));
        }
        Ok(Self { delta_v })
    }

    /// Creates an impulse of a given magnitude along a direction.
    ///
    /// The direction is normalized, so the magnitude is the delta-v whatever
    /// the length of the direction vector. Returns an error when the
    /// magnitude is not positive and finite or when the direction is zero or
    /// not finite.
    pub fn along(direction: Vec3, magnitude: f64) -> Result<Self, ManeuverError> {
        if !magnitude.is_finite() || magnitude <= 0.0 {
            return Err(
                ManeuverError::new(ManeuverErrorKind::NonPositiveAcceleration)
                    .with_detail(format!("magnitude {magnitude}")),
            );
        }
        let unit = direction.normalize().ok_or_else(|| {
            ManeuverError::new(ManeuverErrorKind::InvalidState)
                .with_detail("the direction is zero or not finite")
        })?;
        Self::new(unit * magnitude)
    }

    /// Returns the velocity change vector, in kilometers per second.
    pub const fn delta_v(self) -> Vec3 {
        self.delta_v
    }

    /// Returns the magnitude of the change, in kilometers per second.
    ///
    /// This is the delta-v the burn costs.
    pub fn magnitude(self) -> f64 {
        self.delta_v.norm()
    }

    /// Returns a copy of the change scaled by a factor.
    ///
    /// A factor of zero produces a zero impulse, which [`Impulse::new`]
    /// rejects, so this returns an error for it rather than a value that
    /// cannot be built.
    pub fn scaled(self, factor: f64) -> Result<Self, ManeuverError> {
        Self::new(self.delta_v * factor)
    }

    /// Returns a copy of the change reversed, which undoes the burn.
    pub fn reversed(self) -> Self {
        Self {
            delta_v: -self.delta_v,
        }
    }

    /// Applies the impulse to a velocity, returning the new velocity.
    pub fn apply_to_velocity(self, velocity: Vec3) -> Vec3 {
        velocity + self.delta_v
    }
}

impl core::fmt::Display for Impulse {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "impulse of {:.6} km/s", self.magnitude())
    }
}
