//! Strongly typed newtypes for physical quantities.
//!
//! Each quantity is a newtype over `f64` that carries its unit in the type
//! system, so a length cannot be passed where a duration is expected. Units
//! of the same dimension convert through
//! [`Quantity::to_unit`](apsis_core::Quantity::to_unit), and each unit also
//! offers direct conversions for the common cases.
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
