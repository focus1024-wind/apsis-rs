//! Length quantities.
//!
//! The base unit is the meter. [`Meters`] and [`Kilometers`] both measure
//! [`Length`](crate::dimension::Length) and convert into each other through
//! [`Quantity::to_unit`](apsis_core::Quantity::to_unit).

use crate::dimension::Length;
use crate::macros::impl_quantity_ops;
use apsis_core::{Quantity, Unit};
use core::fmt;

/// A length in meters.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Meters(f64);

/// A length in kilometers.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Kilometers(f64);

impl Meters {
    /// Creates a length from a value in meters.
    pub const fn new(value: f64) -> Self {
        Self(value)
    }

    /// Returns this length in meters.
    pub const fn value(self) -> f64 {
        self.0
    }

    /// Converts this length to kilometers.
    pub fn to_kilometers(self) -> Kilometers {
        Kilometers(self.0 / 1_000.0)
    }
}

impl Kilometers {
    /// Creates a length from a value in kilometers.
    pub const fn new(value: f64) -> Self {
        Self(value)
    }

    /// Returns this length in kilometers.
    pub const fn value(self) -> f64 {
        self.0
    }

    /// Converts this length to meters.
    pub fn to_meters(self) -> Meters {
        Meters(self.0 * 1_000.0)
    }
}

impl Unit for Meters {
    type Dimension = Length;

    fn factor_to_base() -> f64 {
        1.0
    }

    fn symbol() -> &'static str {
        "m"
    }
}

impl Unit for Kilometers {
    type Dimension = Length;

    fn factor_to_base() -> f64 {
        1_000.0
    }

    fn symbol() -> &'static str {
        "km"
    }
}

impl Quantity for Meters {
    type U = Meters;

    fn value(self) -> f64 {
        self.0
    }

    fn from_value(value: f64) -> Self {
        Self(value)
    }
}

impl Quantity for Kilometers {
    type U = Kilometers;

    fn value(self) -> f64 {
        self.0
    }

    fn from_value(value: f64) -> Self {
        Self(value)
    }
}

impl fmt::Display for Meters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} m", self.0)
    }
}

impl fmt::Display for Kilometers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} km", self.0)
    }
}

impl_quantity_ops!(Meters);
impl_quantity_ops!(Kilometers);

#[cfg(test)]
mod tests {
    use super::{Kilometers, Meters};
    use crate::dimension::Length;
    use apsis_core::{Quantity, Unit};

    #[test]
    fn stores_the_given_value() {
        assert_eq!(Meters::new(12.5).value(), 12.5);
        assert_eq!(Kilometers::new(-3.0).value(), -3.0);
    }

    #[test]
    fn direct_conversions_are_consistent() {
        assert_eq!(Meters::new(2_500.0).to_kilometers().value(), 2.5);
        assert_eq!(Kilometers::new(2.5).to_meters().value(), 2_500.0);
    }

    #[test]
    fn generic_conversion_matches_direct_conversion() {
        let distance = Meters::new(2_500.0);
        let via_trait: Kilometers = distance.to_unit();
        assert_eq!(via_trait, distance.to_kilometers());
    }

    #[test]
    fn conversion_round_trips() {
        let original = Kilometers::new(42.195);
        let round_tripped: Kilometers = original.to_unit::<Meters>().to_unit();
        assert!((round_tripped.value() - original.value()).abs() < 1e-12);
    }

    #[test]
    fn units_report_their_metadata() {
        assert_eq!(Meters::symbol(), "m");
        assert_eq!(Kilometers::symbol(), "km");
        assert_eq!(Meters::factor_to_base(), 1.0);
        assert_eq!(Kilometers::factor_to_base(), 1_000.0);
        // Both units measure the same dimension, which is what makes the
        // generic conversion above type check.
        fn assert_length<U: Unit<Dimension = Length>>() {}
        assert_length::<Meters>();
        assert_length::<Kilometers>();
    }

    #[test]
    fn arithmetic_operates_on_the_stored_value() {
        let sum = Meters::new(1.5) + Meters::new(2.5);
        assert_eq!(sum.value(), 4.0);
        let difference = Meters::new(4.0) - Meters::new(1.0);
        assert_eq!(difference.value(), 3.0);
        let scaled = Meters::new(3.0) * 2.0;
        assert_eq!(scaled.value(), 6.0);
        let straddled = 2.0 * Meters::new(3.0);
        assert_eq!(straddled.value(), 6.0);
        let halved = Meters::new(3.0) / 2.0;
        assert_eq!(halved.value(), 1.5);
        let ratio = Meters::new(3.0) / Meters::new(1.5);
        assert_eq!(ratio, 2.0);
        assert_eq!((-Meters::new(1.0)).value(), -1.0);
    }

    #[test]
    fn display_includes_the_symbol() {
        assert_eq!(Meters::new(1.0).to_string(), "1 m");
        assert_eq!(Kilometers::new(2.0).to_string(), "2 km");
    }
}
