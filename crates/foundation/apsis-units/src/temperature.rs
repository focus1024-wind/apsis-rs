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
//!
//! 热力学温度量。
//!
//! 基本单位是开尔文。[`Kelvin`] 度量
//! [`Temperature`](crate::dimension::Temperature)。
//!
//! 温度是本 crate 中唯一的仿射标度。摄氏标度与开尔文大小相同但零点不同，二者
//! 换算需要一个偏移，无法用线性的 [`Unit`] 因子表达。因此摄氏度以显式方法
//! 提供，而不是作为第二个 [`Unit`] 类型。

use crate::dimension::Temperature;
use crate::macros::impl_quantity_ops;
use apsis_core::{Quantity, Unit};
use core::fmt;

/// A thermodynamic temperature in kelvin.
///
/// 以开尔文表示的热力学温度。
///
/// Values below absolute zero are not rejected, because intermediate
/// arithmetic may legitimately pass through them; callers that require a
/// physical temperature should check the value.
///
/// 不拒绝低于绝对零的值，因为中间运算可能合法地经过该区域；需要物理温度的调用方
/// 应自行检查。
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Kelvin(f64);

impl Kelvin {
    /// Creates a temperature from a value in kelvin.
    ///
    /// 由开尔文值创建温度。
    pub const fn new(value: f64) -> Self {
        Self(value)
    }

    /// Creates a temperature from a value in degrees Celsius.
    ///
    /// Uses the exact offset defined by the Celsius scale.
    ///
    /// 由摄氏度值创建温度。
    ///
    /// 使用摄氏标度所定义的精确偏移。
    pub fn from_celsius(celsius: f64) -> Self {
        Self(celsius + apsis_core::constant::ZERO_CELSIUS_IN_KELVIN)
    }

    /// Returns this temperature in kelvin.
    ///
    /// 返回该温度的开尔文值。
    pub const fn value(self) -> f64 {
        self.0
    }

    /// Converts this temperature to degrees Celsius.
    ///
    /// 把该温度换算为摄氏度。
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
