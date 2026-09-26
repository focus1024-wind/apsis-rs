//! Link budget computation.
//!
//! A link budget adds up every gain and loss between a transmitter and a
//! receiver and compares the resulting carrier power with the noise that
//! competes with it. This crate computes that sum and the figures derived
//! from it: the carrier to noise ratio, the energy per bit to noise density
//! ratio, and the link margin.
//!
//! The order of operations matters, so the chain is stated explicitly:
//!
//! 1. The transmitter power is reduced by the transmit losses and increased
//!    by the transmit antenna gain, giving the equivalent isotropically
//!    radiated power.
//! 2. That is divided by the free space path loss and increased by the
//!    receive antenna gain, giving the carrier power at the receiver input.
//! 3. The system noise temperature is converted to a noise density by
//!    multiplying by the Boltzmann constant.
//! 4. The carrier to noise density ratio is the carrier power divided by
//!    that density, and the carrier to noise ratio divides it further by the
//!    receiver bandwidth.
//!
//! Sources:
//!
//! - The link equation in this form is the one used throughout satellite
//!   communications practice, as described for example in the ITU-R
//!   Handbook on Satellite Communications.
//! - The Boltzmann constant and the speed of light come from the SI
//!   definitions, as re-exported by `apsis-rf`.
//!
//! ```
//! use apsis_link::{LinkBudget, LinkGeometry};
//! use apsis_rf::{Gain, Power};
//!
//! let budget = LinkBudget {
//!     transmit_power: Power::from_dbw(10.0).expect("finite"),
//!     transmit_gain: Gain::from_dbi(30.0).expect("finite"),
//!     transmit_loss: apsis_rf::Decibel::from_db(1.0),
//!     receive_gain: Gain::from_dbi(35.0).expect("finite"),
//!     receive_loss: apsis_rf::Decibel::from_db(0.5),
//!     system_temperature: apsis_noise::NoiseTemperature::from_kelvin(150.0),
//!     bandwidth: 36.0e6,
//!     geometry: LinkGeometry {
//!         distance: 38_000_000.0,
//!         frequency: 12.0e9,
//!     },
//! };
//!
//! // The carrier to noise density ratio is available directly. For this
//! // budget it is about 74.7 dB, which is typical of a Ku band downlink.
//! let ratio = budget.carrier_to_noise_density().expect("valid budget");
//! assert!(ratio.db() > 70.0);
//! assert!(ratio.db() < 80.0);
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod budget;

pub use budget::{
    LinkBudget, LinkGeometry, LinkMargin, carrier_to_noise_density, gain_to_temperature,
    link_margin,
};

pub use apsis_antenna::{
    ElementPattern, ParabolicAperture, Pattern, beam_width, mask_gain, off_axis_attenuation,
};
pub use apsis_noise::{
    NoiseTemperature, REFERENCE_TEMPERATURE_K, Stage, cascade_temperature, thermal_noise_power,
};
pub use apsis_rf::{
    BOLTZMANN, DIPOLE_OVER_ISOTROPIC_DB, Decibel, Gain, Power, SPEED_OF_LIGHT,
    free_space_path_loss, wavelength,
};
