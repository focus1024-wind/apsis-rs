//! Strongly typed newtypes for physical quantities.
//!
//! Each quantity is a newtype over `f64` that carries its unit in the type
//! system, so a length cannot be passed where a duration is expected. Units
//! of the same dimension convert through
//! [`Quantity::to_unit`], and each unit also
//! offers direct conversions for the common cases.
//!
//! 物理量的强类型 newtype。
//!
//! 每个量都是 `f64` 上的 newtype，并把单位带进类型系统，因此长度无法被传到
//! 需要时长的位置。同一量纲的单位通过 [`Quantity::to_unit`] 换算，每个单位
//! 另外为常见情形提供直接换算。
//!
//! A newtype implements both [`Unit`] and [`Quantity`], with its own type as
//! the associated unit. For example [`Meters`] is a [`Length`](dimension::Length)
//! unit and a quantity whose unit is itself.
//!
//! The dimensions covered here are length, time, frequency, angle, and
//! thermodynamic temperature. Temperature is the only affine scale in the
//! set, so Celsius is handled by explicit methods on [`Kelvin`] rather than
//! by the linear [`Unit`] conversion factor.
//!
//! 一个 newtype 同时实现 [`Unit`] 与 [`Quantity`]，并以自身类型作为关联单位。
//! 例如 [`Meters`] 既是一个 [`Length`](dimension::Length) 单位，也是单位为
//! 自身的量。
//!
//! 此处覆盖的量纲为长度、时间、频率、角度与热力学温度。温度是集合中唯一的
//! 仿射标度，因此摄氏度由 [`Kelvin`] 上的显式方法处理，而不用 [`Unit`] 的
//! 线性换算因子。
//!
//! ```
//! use apsis_core::Quantity;
//! use apsis_units::{Kilometers, Meters};
//!
//! let distance = Meters::new(2_500.0);
//! let in_kilometers: Kilometers = distance.to_unit();
//! assert!((in_kilometers.value() - 2.5).abs() < 1e-12);
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod dimension;
mod macros;

mod angle;
mod frequency;
mod length;
mod temperature;
mod time;

pub use angle::{Degrees, Radians};
pub use frequency::Hertz;
pub use length::{Kilometers, Meters};
pub use temperature::Kelvin;
pub use time::Seconds;

pub use apsis_core::{Quantity, Unit};
