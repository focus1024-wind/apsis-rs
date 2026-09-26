//! Thermodynamic temperature quantities.
//!
//! The base unit is the kelvin. [`Kelvin`] measures
//! [`Temperature`](crate::dimension::Temperature).
//!
//! Temperature is the one affine scale in this crate. The Celsius scale
//! shares the kelvin's magnitude but not its zero, so converting between
//! them requires an offset and cannot be expressed by the linear
//! [`Unit`] factor. Celsius is therefore offered
//! through explicit methods rather than as a second [`Unit`] type.

use crate::dimension::Temperature;
use crate::macros::impl_quantity_ops;
use apsis_core::{Quantity, Unit};
use core::fmt;

/// A thermodynamic temperature in kelvin.
///
/// Values below absolute zero are not rejected, because intermediate
/// arithmetic may legitimately pass through them; callers that require a
/// physical temperature should check the value.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Kelvin(f64);

impl Kelvin {
    /// Creates a temperature from a value in kelvin.
    pub const fn new(value: f64) -> Self {
        Self(value)
    }

    /// Creates a temperature from a value in degrees Celsius.
    ///
    /// Uses the exact offset defined by the Celsius scale.
    pub fn from_celsius(celsius: f64) -> Self {
        Self(celsius + apsis_core::constant::ZERO_CELSIUS_IN_KELVIN)
    }

    /// Returns this temperature in kelvin.
    pub const fn value(self) -> f64 {
        self.0
    }

    /// Converts this temperature to degrees Celsius.
    pub fn to_celsius(self) -> f64 {
        self.0 - apsis_core::constant::ZERO_CELSIUS_IN_KELVIN
    }
}

impl Unit for Kelvin {
    type Dimension = Temperature;

    fn factor_to_base() -> f64 {
        1.0
    }

    fn symbol() -> &'static str {
        "K"
    }
}

impl Quantity for Kelvin {
    type U = Kelvin;

    fn value(self) -> f64 {
        self.0
    }

    fn from_value(value: f64) -> Self {
        Self(value)
    }
}

impl fmt::Display for Kelvin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} K", self.0)
    }
}

impl_quantity_ops!(Kelvin);
