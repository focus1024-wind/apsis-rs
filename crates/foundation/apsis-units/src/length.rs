//! Length quantities.
//!
//! The base unit is the meter. [`Meters`] and [`Kilometers`] both measure
//! [`Length`](crate::dimension::Length) and convert into each other through
//! [`Quantity::to_unit`](apsis_core::Quantity::to_unit).
//!
//! 长度量。
//!
//! 基本单位是米。[`Meters`] 与 [`Kilometers`] 都度量
//! [`Length`](crate::dimension::Length)，并通过
//! [`Quantity::to_unit`](apsis_core::Quantity::to_unit) 相互换算。

use crate::dimension::Length;
use crate::macros::impl_quantity_ops;
use apsis_core::{Quantity, Unit};
use core::fmt;

/// A length in meters.
///
/// 以米为单位的长度。
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Meters(f64);

/// A length in kilometers.
///
/// 以千米为单位的长度。
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Kilometers(f64);

impl Meters {
    /// Creates a length from a value in meters.
    ///
    /// 由米值创建长度。
    #[must_use]
    pub const fn new(value: f64) -> Self {
        Self(value)
    }

    /// Returns this length in meters.
    ///
    /// 返回该长度的米值。
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }

    /// Converts this length to kilometers.
    ///
    /// 把该长度换算为千米。
    #[must_use]
    pub fn to_kilometers(self) -> Kilometers {
        Kilometers(self.0 / 1_000.0)
    }
}

impl Kilometers {
    /// Creates a length from a value in kilometers.
    ///
    /// 由千米值创建长度。
    #[must_use]
    pub const fn new(value: f64) -> Self {
        Self(value)
    }

    /// Returns this length in kilometers.
    ///
    /// 返回该长度的千米值。
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }

    /// Converts this length to meters.
    ///
    /// 把该长度换算为米。
    #[must_use]
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
