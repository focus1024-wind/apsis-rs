//! Plane angle quantities.
//!
//! The base unit is the radian. [`Radians`] and [`Degrees`] both measure
//! [`Angle`](crate::dimension::Angle) and convert into each other through
//! [`Quantity::to_unit`](apsis_core::Quantity::to_unit).
//!
//! Degrees are not a power of ten multiple of radians, so the conversion
//! factor is the irrational constant pi. The factor is derived from
//! [`f64::consts::PI`](core::f64::consts::PI) rather than from a rounded
//! literal.

use crate::dimension::Angle;
use crate::macros::impl_quantity_ops;
use apsis_core::{Quantity, Unit};
use core::f64::consts::PI;
use core::fmt;

/// An angle in radians.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Radians(f64);

/// An angle in degrees.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Degrees(f64);

impl Radians {
    /// Creates an angle from a value in radians.
    pub const fn new(value: f64) -> Self {
        Self(value)
    }

    /// Returns this angle in radians.
    pub const fn value(self) -> f64 {
        self.0
    }

    /// Converts this angle to degrees.
    pub fn to_degrees(self) -> Degrees {
        Degrees(self.0 * 180.0 / PI)
    }

    /// Returns this angle wrapped to the half-open interval `[-pi, pi)`.
    ///
    /// Wrapping uses [`f64::rem_euclid`](f64::rem_euclid) so the result is
    /// well defined for negative inputs as well as large ones.
    pub fn wrap(self) -> Self {
        let two_pi = 2.0 * PI;
        let wrapped = (self.0 + PI).rem_euclid(two_pi) - PI;
        Self(wrapped)
    }
}

impl Degrees {
    /// Creates an angle from a value in degrees.
    pub const fn new(value: f64) -> Self {
        Self(value)
    }

    /// Returns this angle in degrees.
    pub const fn value(self) -> f64 {
        self.0
    }

    /// Converts this angle to radians.
    pub fn to_radians(self) -> Radians {
        Radians(self.0 * PI / 180.0)
    }
}

impl Unit for Radians {
    type Dimension = Angle;

    fn factor_to_base() -> f64 {
        1.0
    }

    fn symbol() -> &'static str {
        "rad"
    }
}

impl Unit for Degrees {
    type Dimension = Angle;

    fn factor_to_base() -> f64 {
        PI / 180.0
    }

    fn symbol() -> &'static str {
        "deg"
    }
}

impl Quantity for Radians {
    type U = Radians;

    fn value(self) -> f64 {
        self.0
    }

    fn from_value(value: f64) -> Self {
        Self(value)
    }
}

impl Quantity for Degrees {
    type U = Degrees;

    fn value(self) -> f64 {
        self.0
    }

    fn from_value(value: f64) -> Self {
        Self(value)
    }
}

impl fmt::Display for Radians {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} rad", self.0)
    }
}

impl fmt::Display for Degrees {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} deg", self.0)
    }
}

impl_quantity_ops!(Radians);
impl_quantity_ops!(Degrees);
