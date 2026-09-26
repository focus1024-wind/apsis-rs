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
