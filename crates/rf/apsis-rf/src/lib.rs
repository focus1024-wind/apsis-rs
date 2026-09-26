//! Core radio frequency primitives.
//!
//! This crate holds the quantities that the rest of the radio frequency
//! layer is written in: power, gain, and the decibel scale that both are
//! usually quoted on, together with the two relations that every link
//! calculation starts from, the wavelength of a frequency and the free
//! space path loss.
//!
//! Units are SI throughout. Power is in watts, frequency in hertz, distance
//! in meters, and temperature in kelvin. Decibel values are power ratios,
//! computed as ten times the base ten logarithm of the ratio.
//!
//! Two conventions are worth stating because they are easy to mix up:
//!
//! - A decibel is a ratio between two powers. A gain expressed in `dBi` is a
//!   ratio to an isotropic radiator and one in `dBd` is a ratio to a half
//!   wave dipole, which is 2.15 dB more directive than an isotropic
//!   radiator. [`Gain`] carries the ratio and both named forms.
//! - The free space path loss increases with frequency as well as with
//!   distance, because both the effective aperture of a fixed gain antenna
//!   and the solid angle it subtends shrink as the wavelength shortens.
//!
//! ```
//! use apsis_rf::{Power, wavelength};
//!
//! // A noise power of -204 dBW is about 4e-21 W.
//! let noise = Power::from_dbw(-204.0).expect("the reading is finite");
//! assert!(noise.watts() > 0.0);
//!
//! // A one meter wavelength corresponds to a frequency of 299.792 MHz.
//! let lambda = wavelength(299_792_458.0).expect("the frequency is positive");
//! assert!((lambda - 1.0).abs() < 1e-12);
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod decibel;
mod free_space;
mod gain;
mod power;

pub use decibel::Decibel;
pub use free_space::{free_space_path_loss, free_space_received_ratio, wavelength};
pub use gain::{DIPOLE_OVER_ISOTROPIC_DB, Gain, HALF_WAVE_DIPOLE_RATIO};
pub use power::{DBM_REFERENCE, DBW_REFERENCE, Power};

pub use apsis_core::constant::{BOLTZMANN, SPEED_OF_LIGHT};
