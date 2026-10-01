//! Marker traits for quantities that carry a physical unit.
//!
//! A quantity newtype stores a numeric value together with its unit in the
//! type. The [`Quantity`] trait exposes that unit through an associated
//! type, so generic code can be written over any quantity and still recover
//! which unit it was given.
//!
//! The unit itself is any type implementing [`Unit`]. A unit knows how many
//! of its kind are in the corresponding base unit, which allows conversion
//! between units of the same dimension. The base unit of a dimension always
//! has a factor of one.
//
//! 单位本身是任意实现 [`Unit`] 的类型。单位知道自己相当于多少个对应的基本单位，
//! 从而可以在同一量纲的单位之间换算。某量纲的基本单位其因子恒为一。

use core::fmt;

/// A unit of a single physical dimension.
///
/// 单一物理量纲的单位。
///
/// Implementors are zero sized marker types. Two units that implement
/// `Unit` for the same dimension convert through
/// [`Quantity::to_unit`](crate::Quantity::to_unit).
pub trait Unit: Copy + fmt::Debug + fmt::Display + 'static {
    /// The physical dimension this unit measures, such as length or time.
    ///
    /// 该单位度量的物理量纲，例如长度或时间。
    type Dimension: fmt::Debug + 'static;

    /// The number of base units in one of this unit.
    ///
    /// 一个该单位相当于多少个基本单位。
    ///
    /// For example a kilometer has a factor of 1000 when the base unit is
    /// the meter. The base unit of a dimension has a factor of exactly one.
    fn factor_to_base() -> f64;

    /// The symbol used to print the unit, such as `m` or `s`.
    ///
    /// 打印该单位所用的符号，例如 `m` 或 `s`。
    fn symbol() -> &'static str;
}

/// A value paired with a unit.
///
/// 与单位配对的一个数值。
///
/// Implementors are newtypes over `f64` and a [`Unit`]. The trait gives
/// generic code access to the stored value and its unit without assuming
/// which newtype it was handed.
pub trait Quantity: Copy {
    /// The unit this quantity is expressed in.
    ///
    /// 该量所表示的单位。
    type U: Unit;

    /// Returns the stored numeric value in this quantity's own unit.
    ///
    /// 以该量自身单位返回所存储的数值。
    fn value(self) -> f64;

    /// Builds a quantity from a numeric value in this quantity's own unit.
    ///
    /// 由该量自身单位的数值构造一个量。
    fn from_value(value: f64) -> Self;

    /// Converts this quantity to another unit of the same dimension.
    ///
    /// 把该量换算为同一量纲的另一个单位。
    ///
    /// The value is scaled by the ratio of the two unit factors. The
    /// conversion is exact only when the factors are exact powers of ten,
    /// so callers that need a specific unit for output should convert once
    /// at the boundary rather than repeatedly.
    fn to_unit<U>(self) -> U
    where
        U: Quantity,
        <U as Quantity>::U: Unit<Dimension = <Self::U as Unit>::Dimension>,
    {
        let base = self.value() * Self::U::factor_to_base();
        U::from_value(base / <U as Quantity>::U::factor_to_base())
    }
}
