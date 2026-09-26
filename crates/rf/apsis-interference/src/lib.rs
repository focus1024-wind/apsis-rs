//! Interference and carrier-to-interference computation.
//!
//! A wanted signal rarely arrives alone. This crate combines the wanted
//! carrier with one or more interfering carriers and with noise, and reports
//! the ratios that decide whether the wanted link is usable.
//!
//! The figures computed here are:
//!
//! - The carrier to interference ratio, over one interferer or several.
//! - The carrier to noise plus interference ratio, which is what a receiver
//!   actually sees, because it cannot separate the two.
//! - The equivalent noise temperature of an interferer, which lets an
//!   interference contribution be folded into the link budget as if it were
//!   noise.
//!
//! Interference from a second satellite sharing the same frequency is
//! described by the angle between the two satellites as seen from the ground
//! station. The receive antenna has a sidelobe response in that direction,
//! and the ratio of the interferer's power to the wanted power follows from
//! the two gains and the two path losses. This crate takes the two gains as
//! inputs, so it composes with the pattern models in `apsis-antenna`.
//!
//! Source: the definitions of carrier to interference and carrier to noise
//! plus interference, and the practice of expressing interference as an
//! equivalent noise temperature, are standard in satellite link design and
//! appear for example in the ITU-R Handbook on Satellite Communications.
//!
//! ```
//! use apsis_interference::{Interferer, carrier_to_interference, combine};
//! use apsis_rf::{Decibel, Power};
//!
//! let wanted = Power::from_dbw(-130.0).expect("finite");
//! let interferer = Interferer {
//!     power: Power::from_dbw(-140.0).expect("finite"),
//! };
//!
//! // One interferer ten decibels below the carrier leaves a C/I of 10 dB.
//! let ratio = carrier_to_interference(wanted, &[interferer]).expect("valid");
//! assert!((ratio.db() - 10.0).abs() < 1e-9);
//!
//! // Combining with noise can only lower the result.
//! let combined = combine(ratio, Decibel::from_db(15.0)).expect("valid");
//! assert!(combined.db() < ratio.db());
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod ratio;

pub use ratio::{
    InterferenceSummary, Interferer, carrier_to_interference, carrier_to_interference_single,
    carrier_to_noise_plus_interference, combine, equivalent_noise_temperature,
    interference_to_noise_ratio, summarise, total_interference,
};

pub use apsis_core::constant::BOLTZMANN;
pub use apsis_rf::{Decibel, Power};
