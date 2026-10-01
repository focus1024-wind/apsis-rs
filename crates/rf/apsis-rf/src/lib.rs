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
//!
//! 核心射频原语。
//!
//! 本 crate 保存射频层其余部分据以书写的各量：功率、增益，以及二者通常所
//! 用的分贝尺度，连同每次链路计算所起的两条关系——某频率的波长与自由空间
//! 路径损耗。
//!
//! 全程使用 SI 单位。功率以瓦特计，频率以赫兹计，距离以米计，温度以开尔文
//! 计。分贝值是功率比，按该比的十进对数的十倍计算。
//!
//! 有两条约定值得写出，因为它们容易混淆：
//!
//! - 分贝是两个功率之比。以 `dBi` 表示的增益是相对各向同性辐射体的比值，
//!   以 `dBd` 表示的则是相对半波振子的比值，后者比各向同性辐射体更具方向性
//!   2.15 dB。[`Gain`] 携带该比值以及两种具名形式。
//! - 自由空间路径损耗既随频率也随距离增大，因为固定增益天线的有效孔径与其
//!   所张立体角都随波长变短而缩小。

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
