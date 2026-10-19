//! Link budget computation.
//!
//! A link budget adds up every gain and loss between a transmitter and a
//! receiver and compares the resulting carrier power with the noise that
//! competes with it. This module computes that sum and the figures derived
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
//!   definitions, as re-exported by `apsis_rf`.
//!
//! ```
//! use apsis::apsis_link::{LinkBudget, LinkGeometry};
//! use apsis::apsis_rf::{Gain, Power};
//!
//! let budget = LinkBudget {
//!     transmit_power: Power::from_dbw(10.0).expect("finite"),
//!     transmit_gain: Gain::from_dbi(30.0).expect("finite"),
//!     transmit_loss: apsis::apsis_rf::Decibel::from_db(1.0),
//!     receive_gain: Gain::from_dbi(35.0).expect("finite"),
//!     receive_loss: apsis::apsis_rf::Decibel::from_db(0.5),
//!     system_temperature: apsis::apsis_noise::NoiseTemperature::from_kelvin(150.0),
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
//!
//! 链路预算计算。
//!
//! 链路预算把发射机与接收机之间的每项增益与损耗相加，并把所得载波功率与同
//! 它竞争的噪声比较。本模块计算该和以及由它导出的各数字：载噪比、每比特
//! 能量与噪声密度之比，以及链路余量。
//!
//! 运算次序要紧，故该链被明确写出：
//!
//! 1. 发射机功率减去发射损耗、加上发射天线增益，给出等效全向辐射功率。
//! 2. 它除以自由空间路径损耗、加上接收天线增益，给出接收机输入端的载波
//!    功率。
//! 3. 系统噪声温度乘以玻尔兹曼常量，换算为噪声密度。
//! 4. 载噪密度比是载波功率除以该密度，载噪比则进一步除以接收机带宽。
//!
//! 出处：
//!
//! - 此形式的链路方程是卫星通信实践中通篇所用的，例如见 ITU-R Handbook on
//!   Satellite Communications。
//! - 玻尔兹曼常量与光速来自 SI 定义，由 `apsis_rf` 重导出。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod budget;
mod mission;

pub use budget::{
    LinkBudget, LinkGeometry, LinkMargin, carrier_to_noise_density, gain_to_temperature,
    link_margin,
};
pub use mission::{LinkParameters, LinkResult, link_budget};

pub use crate::apsis_antenna::{
    ElementPattern, ParabolicAperture, Pattern, beam_width, mask_gain, off_axis_attenuation,
};
pub use crate::apsis_noise::{
    NoiseTemperature, REFERENCE_TEMPERATURE_K, Stage, cascade_temperature, thermal_noise_power,
};
pub use crate::apsis_rf::{
    BOLTZMANN, DIPOLE_OVER_ISOTROPIC_DB, Decibel, Gain, Power, SPEED_OF_LIGHT,
    free_space_path_loss, wavelength,
};
