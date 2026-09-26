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

#[cfg(test)]
mod tests {
    use super::Kelvin;
    use crate::dimension::Temperature;
    use apsis_core::Unit;
    use apsis_core::constant::ZERO_CELSIUS_IN_KELVIN;

    #[test]
    fn stores_the_given_value() {
        assert_eq!(Kelvin::new(290.0).value(), 290.0);
    }

    #[test]
    fn celsius_conversion_uses_the_exact_offset() {
        let freezing = Kelvin::from_celsius(0.0);
        assert_eq!(freezing.value(), ZERO_CELSIUS_IN_KELVIN);
        assert_eq!(freezing.to_celsius(), 0.0);
        let boiling = Kelvin::from_celsius(100.0);
        assert!((boiling.to_celsius() - 100.0).abs() < 1e-12);
    }

    #[test]
    fn celsius_conversion_round_trips() {
        let original = Kelvin::new(273.15 + 21.5);
        assert!(
            (Kelvin::from_celsius(original.to_celsius()).value() - original.value()).abs() < 1e-12
        );
    }

    #[test]
    fn unit_metadata_is_available() {
        assert_eq!(Kelvin::symbol(), "K");
        assert_eq!(Kelvin::factor_to_base(), 1.0);
        fn assert_temperature<U: Unit<Dimension = Temperature>>() {}
        assert_temperature::<Kelvin>();
    }

    #[test]
    fn display_includes_the_symbol() {
        assert_eq!(Kelvin::new(300.0).to_string(), "300 K");
    }
}
