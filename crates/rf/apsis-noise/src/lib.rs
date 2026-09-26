//! Noise temperature and noise figure models.
//!
//! Noise enters a receiver from three places: the source, the antenna and
//! its feed, and the receiver itself. They are combined by adding noise
//! temperatures rather than powers, because a temperature is a power
//! referred to a fixed bandwidth through the Boltzmann relation, and a
//! cascade of stages at different gains does not simply add powers.
//!
//! The central relations are:
//!
//! - Thermal noise power in a bandwidth, `P = k T B`, with `k` the Boltzmann
//!   constant.
//! - Noise figure as a temperature, `T = T_0 (F - 1)`, with `T_0` the
//!   reference temperature of 290 K.
//! - The Friis cascade, which weights each stage's noise by the gain that
//!   precedes it.
//!
//! Sources:
//!
//! - The reference temperature of 290 K and the noise figure definition are
//!   from the IEEE standard definition of noise figure, IEEE Std 161.
//! - The cascade is the Friis formula for noise, as given in any receiver
//!   design text.
//!
//! ```
//! use apsis_noise::{NoiseTemperature, thermal_noise_power};
//!
//! // A 36 MHz bandwidth at 290 K carries about -128 dBW of thermal noise.
//! let temperature = NoiseTemperature::from_kelvin(290.0);
//! let power = thermal_noise_power(temperature, 36.0e6).expect("valid arguments");
//! assert!((power.dbw().expect("positive") + 128.0).abs() < 1.0);
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod cascade;
mod temperature;

pub use cascade::{
    Stage, cascade_noise_figure_db, cascade_temperature, noise_figure_to_temperature,
    reference_temperature,
};
pub use temperature::{NoiseTemperature, REFERENCE_TEMPERATURE_K, thermal_noise_power};

pub use apsis_core::constant::BOLTZMANN;
