//! Duration quantities.
//!
//! The base unit is the second. [`Seconds`] is the only unit defined here;
//! it measures [`Time`](crate::dimension::Time).

use crate::dimension::Time;
use crate::macros::impl_quantity_ops;
use apsis_core::{Quantity, Unit};
use core::fmt;

/// A duration in seconds.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Seconds(f64);

impl Seconds {
    /// Creates a duration from a value in seconds.
    pub const fn new(value: f64) -> Self {
        Self(value)
    }

    /// Returns this duration in seconds.
    pub const fn value(self) -> f64 {
        self.0
    }
}

impl Unit for Seconds {
    type Dimension = Time;

    fn factor_to_base() -> f64 {
        1.0
    }

    fn symbol() -> &'static str {
        "s"
    }
}

impl Quantity for Seconds {
    type U = Seconds;

    fn value(self) -> f64 {
        self.0
    }

    fn from_value(value: f64) -> Self {
        Self(value)
    }
}

impl fmt::Display for Seconds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} s", self.0)
    }
}

impl_quantity_ops!(Seconds);

#[cfg(test)]
mod tests {
    use super::Seconds;
    use crate::dimension::Time;
    use apsis_core::Unit;

    #[test]
    fn stores_the_given_value() {
        assert_eq!(Seconds::new(0.5).value(), 0.5);
    }

    #[test]
    fn unit_metadata_is_available() {
        assert_eq!(Seconds::symbol(), "s");
        assert_eq!(Seconds::factor_to_base(), 1.0);
        fn assert_time<U: Unit<Dimension = Time>>() {}
        assert_time::<Seconds>();
    }

    #[test]
    fn arithmetic_operates_on_the_stored_value() {
        assert_eq!((Seconds::new(1.0) + Seconds::new(2.0)).value(), 3.0);
        assert_eq!((Seconds::new(3.0) - Seconds::new(2.0)).value(), 1.0);
        assert_eq!((Seconds::new(3.0) * 2.0).value(), 6.0);
        assert_eq!((Seconds::new(3.0) / 3.0).value(), 1.0);
        assert_eq!(Seconds::new(6.0) / Seconds::new(3.0), 2.0);
    }

    #[test]
    fn display_includes_the_symbol() {
        assert_eq!(Seconds::new(1.5).to_string(), "1.5 s");
    }
}
