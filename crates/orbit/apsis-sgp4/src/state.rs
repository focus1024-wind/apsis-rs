//! The propagated state of a satellite.

use apsis_math::Vec3;

/// The position and velocity of a satellite in the True Equator Mean Equinox
/// frame.
///
/// Position is in kilometers and velocity is in kilometers per second. The
/// frame is the one the model defines, with its first axis toward the mean
/// equinox, its third axis along the true equator pole, and the second
/// completing a right-handed set.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct State {
    /// Position, in kilometers.
    pub position: Vec3,
    /// Velocity, in kilometers per second.
    pub velocity: Vec3,
    /// The propagated semi-major axis, in kilometers.
    ///
    /// This is the semi-major axis after the secular drag term has been
    /// applied, so it differs from the value implied by the mean motion in
    /// the element set.
    pub semi_major_axis: f64,
}

impl State {
    /// Returns the distance from the center of the Earth, in kilometers.
    pub fn altitude_radius(self) -> f64 {
        self.position.norm()
    }

    /// Returns the speed relative to the Earth's center, in kilometers per
    /// second.
    pub fn speed(self) -> f64 {
        self.velocity.norm()
    }
}
