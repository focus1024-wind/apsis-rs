//! Carrier to interference ratios and their combination with noise.
//!
//! 载干比及其与噪声的合成。

use crate::apsis_core::constant::BOLTZMANN;
use crate::apsis_rf::{Decibel, Power};

/// One interfering carrier seen at the receiver input.
///
/// The power is the interfering power that reaches the receiver, already
/// including the transmit power, the transmit gain, the path loss, the
/// receive antenna gain in the direction of the interferer, and any
/// propagation loss. This module combines the powers; it does not compute the
/// geometry.
///
/// 在接收机输入端看到的一个干扰载波。
///
/// 功率是到达接收机的干扰功率，其中已含发射功率、发射增益、路径损耗、
/// 干扰源方向上的接收天线增益，以及任何传播损耗。本模块只合并功率；
/// 它不计算几何。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interferer {
    /// The interfering power at the receiver input, in watts.
    ///
    /// 接收机输入端的干扰功率，单位为瓦特。
    pub power: Power,
}

impl Interferer {
    /// Creates an interferer from a power at the receiver input.
    ///
    /// 由接收机输入端的功率创建干扰源。
    #[must_use]
    pub const fn new(power: Power) -> Self {
        Self { power }
    }
}

/// The combined result of a wanted carrier and its interference.
///
/// 有用载波与其干扰合并所得的结果。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InterferenceSummary {
    /// The wanted carrier power, in watts.
    ///
    /// 有用载波功率，单位为瓦特。
    pub carrier: Power,
    /// The total interfering power, in watts.
    ///
    /// 总干扰功率，单位为瓦特。
    pub interference: Power,
    /// The carrier to interference ratio.
    ///
    /// 载干比。
    pub carrier_to_interference: Decibel,
    /// The carrier to noise plus interference ratio, when a noise power was
    /// given.
    ///
    /// 载波与噪声加干扰之比，当给出了噪声功率时。
    pub carrier_to_noise_plus_interference: Option<Decibel>,
}

impl InterferenceSummary {
    /// Returns the interference to carrier ratio.
    ///
    /// This is the reciprocal of the carrier to interference ratio and is
    /// the form some specifications use.
    ///
    /// 返回干扰与载波之比。
    ///
    /// 它是载干比的倒数，是某些规范所用的形式。
    #[must_use]
    pub fn interference_to_carrier(self) -> Decibel {
        self.carrier_to_interference.negated()
    }
}

/// Returns the total interfering power from a set of interferers.
///
/// Interfering powers add, because they are independent signals and their
/// power is what matters, not their amplitudes. Two equal interferers
/// therefore give twice the power of one, which is three decibels and not
/// six.
///
/// Returns `None` when any interferer has an invalid power.
///
/// 返回一组干扰源的总干扰功率。
///
/// 各干扰功率相加，因为它们是独立信号，要紧的是其功率而非幅度。故两个
/// 相等的干扰源给出单个的两倍功率，即三分贝而非六分贝。
///
/// 任一干扰源的功率无效时返回 `None`。
#[must_use]
pub fn total_interference(interferers: &[Interferer]) -> Option<Power> {
    let mut total = 0.0;
    for interferer in interferers {
        if !interferer.power.is_valid() {
            return None;
        }
        total += interferer.power.watts();
    }
    Some(Power::from_watts(total))
}

/// Returns the carrier to interference ratio for a carrier and a set of
/// interferers.
///
/// The ratio is the wanted carrier power divided by the sum of the
/// interfering powers. A large ratio means the interferers are weak.
///
/// Returns `None` when the carrier is not positive, when any interferer is
/// invalid, or when the total interference is zero, which would give an
/// infinite ratio.
///
/// 返回某载波与一组干扰源的载干比。
///
/// 该比是有用载波功率除以各干扰功率之和。比大意味着干扰源弱。
///
/// 载波非正、任一干扰源无效，或总干扰为零（会给出无穷的比）时返回
/// `None`。
#[must_use]
pub fn carrier_to_interference(carrier: Power, interferers: &[Interferer]) -> Option<Decibel> {
    if !carrier.is_valid() || carrier.watts() <= 0.0 {
        return None;
    }
    let interference = total_interference(interferers)?;
    if interference.watts() <= 0.0 {
        return None;
    }
    Decibel::from_ratio(carrier.watts() / interference.watts())
}

/// Returns the carrier to interference ratio for a single interferer.
///
/// This is a convenience for the common case of one dominant interferer.
///
/// Returns `None` when either power is not positive.
///
/// 返回单个干扰源下的载干比。
///
/// 这是为一个占主导的干扰源这一常见情形提供的便利。
///
/// 两个功率之一非正时返回 `None`。
#[must_use]
pub fn carrier_to_interference_single(carrier: Power, interferer: Interferer) -> Option<Decibel> {
    carrier_to_interference(carrier, &[interferer])
}

/// Combines a ratio with another ratio of the same kind.
///
/// When two independent impairments act together, their effects add as
/// reciprocal ratios, which in decibels means
///
/// <div>
/// $$
/// \frac{1}{R} = \frac{1}{R_1} + \frac{1}{R_2}
/// $$
/// </div>
///
/// The result is always below the smaller of the two, so adding a second
/// impairment can only lower the combined ratio. This is how a carrier to
/// noise ratio and a carrier to interference ratio are combined into a
/// carrier to noise plus interference ratio.
///
/// Returns `None` when either value is not finite.
///
/// 把一个比与另一个同类之比合并。
///
/// 当两种独立损伤共同作用时，它们的效果按倒数之比相加，以分贝表示即
///
/// <div>
/// $$
/// \frac{1}{R} = \frac{1}{R_1} + \frac{1}{R_2}
/// $$
/// </div>
///
/// 结果总低于两者中较小者，故加入第二种损伤只会降低合并之比。载噪比与
/// 载干比正是这样合并为载波与噪声加干扰之比的。
///
/// 任一值非有限时返回 `None`。
#[must_use]
pub fn combine(first: Decibel, second: Decibel) -> Option<Decibel> {
    if !first.is_finite() || !second.is_finite() {
        return None;
    }
    // A ratio of zero in decibels means an infinite impairment, so the
    // combination is also infinite.
    // 分贝表示的比为零意味着无穷的损伤，故合并也是无穷的。
    let first_ratio = first.ratio();
    let second_ratio = second.ratio();
    if !first_ratio.is_finite() || !second_ratio.is_finite() {
        return None;
    }
    Decibel::from_ratio(1.0 / (1.0 / first_ratio + 1.0 / second_ratio))
}

/// Returns the carrier to noise plus interference ratio.
///
/// This combines a carrier to noise ratio and a carrier to interference
/// ratio through [`combine`]. It is the ratio a receiver actually
/// experiences, because it cannot separate thermal noise from a competing
/// carrier.
///
/// Returns `None` when either ratio is not finite.
///
/// 返回载波与噪声加干扰之比。
///
/// 它通过 [`combine`] 把载噪比与载干比合并。这是接收机实际经历的比，因为
/// 它无法把热噪声与竞争载波分开。
///
/// 任一比非有限时返回 `None`。
#[must_use]
pub fn carrier_to_noise_plus_interference(
    carrier_to_noise: Decibel,
    carrier_to_interference: Decibel,
) -> Option<Decibel> {
    combine(carrier_to_noise, carrier_to_interference)
}

/// Returns the interference to noise ratio.
///
/// This compares the interfering power with the thermal noise power in the
/// receiver. A value well below zero decibels means the interference is
/// negligible against the noise and can be ignored; a value near or above
/// zero means it must be accounted for.
///
/// Returns `None` when either power is not positive.
///
/// 返回干扰与噪声之比。
///
/// 它把干扰功率与接收机中的热噪声功率相比较。远低于零分贝的值意味着干扰相对
/// 噪声可忽略、可以不计；接近或高于零的值意味着必须计入它。
///
/// 两个功率之一非正时返回 `None`。
#[must_use]
pub fn interference_to_noise_ratio(interference: Power, noise: Power) -> Option<Decibel> {
    if !interference.is_valid() || !noise.is_valid() {
        return None;
    }
    if interference.watts() <= 0.0 || noise.watts() <= 0.0 {
        return None;
    }
    Decibel::from_ratio(interference.watts() / noise.watts())
}

/// Returns the noise temperature equivalent to an interfering power.
///
/// An interferer that a receiver treats as noise can be folded into the link
/// budget as an increase in system noise temperature. In temperature the
/// relation is
///
/// <div>
/// $$
/// T = \frac{P}{k B}
/// $$
/// </div>
///
/// with `P` the interfering power, `k` the Boltzmann constant, and `B` the
/// receiver noise bandwidth. The temperature is the value that, added to the
/// system temperature, would raise the noise by the same amount as the
/// interferer.
///
/// Returns `None` when the power is not positive or the bandwidth is not
/// positive and finite.
///
/// 返回与某干扰功率等效的噪声温度。
///
/// 接收机当作噪声处理的干扰源可以折算为系统噪声温度的增量计入链路
/// 预算。在温度上关系为
///
/// <div>
/// $$
/// T = \frac{P}{k B}
/// $$
/// </div>
///
/// 其中 `P` 是干扰功率，`k` 是玻尔兹曼常量，`B` 是接收机噪声带宽。该
/// 温度即加到系统温度上会使噪声上升与干扰源相同量的值。
///
/// 功率非正，或带宽非正且非有限时返回 `None`。
#[must_use]
pub fn equivalent_noise_temperature(interference: Power, bandwidth: f64) -> Option<f64> {
    if !interference.is_valid() || interference.watts() <= 0.0 {
        return None;
    }
    if !bandwidth.is_finite() || bandwidth <= 0.0 {
        return None;
    }
    Some(interference.watts() / (BOLTZMANN * bandwidth))
}

/// Returns a summary of a carrier with its interference.
///
/// When a noise power is given, the carrier to noise plus interference ratio
/// is computed as well. Pass `None` for the noise power when only the
/// interference is of interest.
///
/// Returns `None` when the carrier is not positive, when any interferer is
/// invalid, or when the total interference is zero.
///
/// 返回某载波及其干扰的摘要。
///
/// 当给出噪声功率时，载波与噪声加干扰之比也一并算出。当只关注干扰时，
/// 噪声功率传 `None`。
///
/// 载波非正、任一干扰源无效，或总干扰为零时返回 `None`。
#[must_use]
pub fn summarise(
    carrier: Power,
    interferers: &[Interferer],
    noise: Option<Power>,
) -> Option<InterferenceSummary> {
    let interference = total_interference(interferers)?;
    let carrier_to_interference = carrier_to_interference(carrier, interferers)?;

    let carrier_to_noise_plus_interference = match noise {
        Some(noise) => {
            if !noise.is_valid() || noise.watts() <= 0.0 {
                return None;
            }
            let carrier_to_noise = Decibel::from_ratio(carrier.watts() / noise.watts())?;
            Some(combine(carrier_to_noise, carrier_to_interference)?)
        }
        None => None,
    };

    Some(InterferenceSummary {
        carrier,
        interference,
        carrier_to_interference,
        carrier_to_noise_plus_interference,
    })
}
