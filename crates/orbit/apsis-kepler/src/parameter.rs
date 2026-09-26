//! The Earth gravitational parameter.

/// The gravitational parameter of the central body, in cubic kilometers per
/// second squared.
///
/// Two values are offered because the workspace supports both gravity
/// models, and the correct choice is the one that matches the source of the
/// orbital elements. For elements derived from a two-line element set, the
/// WGS 72 value is the one the SGP4 theory was fitted to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GravityParameter(pub f64);

impl GravityParameter {
    /// The WGS 72 gravitational parameter, `398 600.8 km^3/s^2`.
    pub const WGS72: Self = Self(398_600.8);

    /// The WGS 84 gravitational parameter, `398 600.5 km^3/s^2`.
    pub const WGS84: Self = Self(398_600.5);

    /// Returns the value in cubic kilometers per second squared.
    pub const fn value(self) -> f64 {
        self.0
    }

    /// Returns the square root of the parameter, in kilometers to the three
    /// halves per second.
    pub fn sqrt(self) -> f64 {
        self.0.sqrt()
    }
}
