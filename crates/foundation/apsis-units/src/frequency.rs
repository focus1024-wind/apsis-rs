//! Frequency quantities.
//!
//! The base unit is the hertz, one cycle per second. [`Hertz`] measures
//! [`Frequency`](crate::dimension::Frequency).

use crate::dimension::Frequency;
use crate::macros::impl_quantity_ops;
use apsis_core::{Quantity, Unit};
use core::fmt;

/// A frequency in hertz.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Hertz(f64);

impl Hertz {
    /// Creates a frequency from a value in hertz.
    pub const fn new(value: f64) -> Self {
        Self(value)
    }

    /// Returns this frequency in hertz.
    pub const fn value(self) -> f64 {
        self.0
    }

    /// Returns the period corresponding to this frequency, in seconds.
    ///
    /// Returns `f64::INFINITY` when the frequency is zero, which is the
    /// correct limit of an infinite period.
    pub fn period(self) -> f64 {
        1.0 / self.0
    }
}

impl Unit for Hertz {
    type Dimension = Frequency;

    fn factor_to_base() -> f64 {
        1.0
    }

    fn symbol() -> &'static str {
        "Hz"
    }
}

impl Quantity for Hertz {
    type U = Hertz;

    fn value(self) -> f64 {
        self.0
    }

    fn from_value(value: f64) -> Self {
        Self(value)
    }
}

impl fmt::Display for Hertz {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} Hz", self.0)
    }
}

impl_quantity_ops!(Hertz);

#[cfg(test)]
mod tests {
    use super::Hertz;
    use crate::dimension::Frequency;
    use apsis_core::Unit;

    #[test]
    fn stores_the_given_value() {
        assert_eq!(Hertz::new(1_575.42).value(), 1_575.42);
    }

    #[test]
    fn period_is_the_reciprocal() {
        assert!((Hertz::new(2.0).period() - 0.5).abs() < 1e-12);
        assert!(Hertz::new(0.0).period().is_infinite());
    }

    #[test]
    fn unit_metadata_is_available() {
        assert_eq!(Hertz::symbol(), "Hz");
        assert_eq!(Hertz::factor_to_base(), 1.0);
        fn assert_frequency<U: Unit<Dimension = Frequency>>() {}
        assert_frequency::<Hertz>();
    }

    #[test]
    fn display_includes_the_symbol() {
        assert_eq!(Hertz::new(1_000.0).to_string(), "1000 Hz");
    }
}
