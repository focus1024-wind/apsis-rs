//! Frequency quantities.
//!
//! The base unit is the hertz, one cycle per second. [`Hertz`] measures
//! [`Frequency`](crate::dimension::Frequency).
//!
//! 频率量。
//!
//! 基本单位是赫兹，即每秒一个周期。[`Hertz`] 度量
//! [`Frequency`](crate::dimension::Frequency)。

use crate::dimension::Frequency;
use crate::macros::impl_quantity_ops;
use apsis_core::{Quantity, Unit};
use core::fmt;

/// A frequency in hertz.
///
/// 以赫兹表示的频率。
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Hertz(f64);

impl Hertz {
    /// Creates a frequency from a value in hertz.
    ///
    /// 由赫兹值创建频率。
    #[must_use]
    pub const fn new(value: f64) -> Self {
        Self(value)
    }

    /// Returns this frequency in hertz.
    ///
    /// 返回该频率的赫兹值。
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }

    /// Returns the period corresponding to this frequency, in seconds.
    ///
    /// Returns `f64::INFINITY` when the frequency is zero, which is the
    /// correct limit of an infinite period.
    ///
    /// 返回与该频率对应的周期，单位为秒。
    ///
    /// 频率为零时返回 `f64::INFINITY`，这正是周期趋于无穷的正确极限。
    #[must_use]
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
