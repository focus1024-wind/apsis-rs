//! Frame marker types.

use apsis_math::Vec3;

/// A position or vector expressed in the Earth-centered inertial frame.
///
/// The frame is aligned with the mean equator and equinox of J2000.0. It is
/// treated as inertial here, so it does not rotate with the Earth.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Eci(pub Vec3);

/// A position or vector expressed in the Earth-centered Earth-fixed frame.
///
/// The frame rotates with the Earth, so a point fixed to the ground has
/// constant coordinates in it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ecef(pub Vec3);

impl Eci {
    /// Creates an inertial frame vector from its components.
    pub const fn new(vector: Vec3) -> Self {
        Self(vector)
    }

    /// Returns the underlying vector.
    pub const fn vector(self) -> Vec3 {
        self.0
    }
}

impl Ecef {
    /// Creates an Earth-fixed frame vector from its components.
    pub const fn new(vector: Vec3) -> Self {
        Self(vector)
    }

    /// Returns the underlying vector.
    pub const fn vector(self) -> Vec3 {
        self.0
    }
}
