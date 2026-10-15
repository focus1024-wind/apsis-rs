//! Interference and carrier-to-interference computation.
//!
//! A wanted signal rarely arrives alone. This module combines the wanted
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
//! the two gains and the two path losses. This module takes the two gains as
//! inputs, so it composes with the pattern models in `apsis_antenna`.
//!
//! Source: the definitions of carrier to interference and carrier to noise
//! plus interference, and the practice of expressing interference as an
//! equivalent noise temperature, are standard in satellite link design and
//! appear for example in the ITU-R Handbook on Satellite Communications.
//!
//! ```
//! use apsis::apsis_interference::{Interferer, carrier_to_interference, combine};
//! use apsis::apsis_rf::{Decibel, Power};
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
//!
//! 干扰与载干比计算。
//!
//! 有用信号很少独自到达。本模块把有用载波与一个或多个干扰载波以及噪声
//! 合并，并报告决定该有用链路是否可用的各比值。
//!
//! 此处计算的数字是：
//!
//! - 载干比，对一个或若干干扰源。
//! - 载波与噪声加干扰之比，这是接收机实际所见，因为它无法把二者分开。
//! - 某干扰源的等效噪声温度，它使干扰贡献可以像噪声一样折算进链路预算。
//!
//! 来自共享同一频率的第二颗卫星的干扰，由自地面站看两颗卫星之间的角描述。
//! 接收天线在该方向上有旁瓣响应，而干扰源功率与有用功率之比由两个增益与
//! 两个路径损耗得出。本模块把两个增益作为输入，故它与 `apsis_antenna`
//! 中的方向图模型可组合。
//!
//! 出处：载干比与载波噪声加干扰比的定义，以及把干扰表达为等效噪声温度的
//! 做法，都是卫星链路设计中的标准内容，例如见于 ITU-R Handbook on Satellite
//! Communications。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod ratio;

pub use ratio::{
    InterferenceSummary, Interferer, carrier_to_interference, carrier_to_interference_single,
    carrier_to_noise_plus_interference, combine, equivalent_noise_temperature,
    interference_to_noise_ratio, summarise, total_interference,
};

pub use crate::apsis_core::constant::BOLTZMANN;
pub use crate::apsis_rf::{Decibel, Power};
