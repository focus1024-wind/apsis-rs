//! Plane angle quantities.
//!
//! The base unit is the radian. [`Radians`] and [`Degrees`] both measure
//! [`Angle`](crate::apsis_units::dimension::Angle) and convert into each other through
//! [`Quantity::to_unit`](crate::apsis_core::Quantity::to_unit).
//!
//! Degrees are not a power of ten multiple of radians, so the conversion
//! factor is the irrational constant pi. The factor is derived from
//! [`f64::consts::PI`](core::f64::consts::PI) rather than from a rounded
//! literal.
//!
//! 平面角量。
//!
//! 基本单位是弧度。[`Radians`] 与 [`Degrees`] 都度量
//! [`Angle`](crate::apsis_units::dimension::Angle)，并通过
//! [`Quantity::to_unit`](crate::apsis_core::Quantity::to_unit) 相互换算。
//!
//! 度并非弧度的十的整数次幂，故换算因子是无理常数 pi。该因子取自
//! [`f64::consts::PI`](core::f64::consts::PI) 而非某个取整后的字面量。

use crate::apsis_core::{Quantity, Unit};
use crate::apsis_units::dimension::Angle;
use crate::apsis_units::macros::impl_quantity_ops;
use core::f64::consts::PI;
use core::fmt;

/// An angle in radians.
///
/// 以弧度表示的角度。
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Radians(f64);

/// An angle in degrees.
///
/// 以度表示的角度。
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Degrees(f64);

impl Radians {
    /// Creates an angle from a value in radians.
    ///
    /// 由弧度值创建角度。
    #[must_use]
    pub const fn new(value: f64) -> Self {
        Self(value)
    }

    /// Returns this angle in radians.
    ///
    /// 返回该角度的弧度值。
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }

    /// Converts this angle to degrees.
    ///
    /// 把该角度换算为度。
    #[must_use]
    pub fn to_degrees(self) -> Degrees {
        Degrees(self.0 * 180.0 / PI)
    }

    /// Returns this angle wrapped to the half-open interval `[-pi, pi)`.
    ///
    /// Wrapping uses [`f64::rem_euclid`](f64::rem_euclid) so the result is
    /// well defined for negative inputs as well as large ones.
    ///
    /// 返回该角度归约到半开区间 `[-pi, pi)` 后的值。
    ///
    /// 归约使用 [`f64::rem_euclid`](f64::rem_euclid)，因此对负输入与很大输入
    /// 都有良好定义。
    #[must_use]
    pub fn wrap(self) -> Self {
        let two_pi = 2.0 * PI;
        let wrapped = (self.0 + PI).rem_euclid(two_pi) - PI;
        Self(wrapped)
    }
}

impl Degrees {
    /// Creates an angle from a value in degrees.
    ///
    /// 由角度值创建角度。
    #[must_use]
    pub const fn new(value: f64) -> Self {
        Self(value)
    }

    /// Returns this angle in degrees.
    ///
    /// 返回该角度的度值。
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }

    /// Converts this angle to radians.
    ///
    /// 把该角度换算为弧度。
    #[must_use]
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
