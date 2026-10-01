//! Carrier to interference ratios and their combination with noise.

use apsis_core::constant::BOLTZMANN;
use apsis_rf::{Decibel, Power};

/// One interfering carrier seen at the receiver input.
///
/// The power is the interfering power that reaches the receiver, already
/// including the transmit power, the transmit gain, the path loss, the
/// receive antenna gain in the direction of the interferer, and any
/// propagation loss. This crate combines the powers; it does not compute the
/// geometry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interferer {
    /// The interfering power at the receiver input, in watts.
    pub power: Power,
}

impl Interferer {
    /// Creates an interferer from a power at the receiver input.
    pub const fn new(power: Power) -> Self {
        Self { power }
    }
}

/// The combined result of a wanted carrier and its interference.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InterferenceSummary {
    /// The wanted carrier power, in watts.
    pub carrier: Power,
    /// The total interfering power, in watts.
    pub interference: Power,
    /// The carrier to interference ratio.
    pub carrier_to_interference: Decibel,
    /// The carrier to noise plus interference ratio, when a noise power was
    /// given.
    pub carrier_to_noise_plus_interference: Option<Decibel>,
}

impl InterferenceSummary {
    /// Returns the interference to carrier ratio.
    ///
    /// This is the reciprocal of the carrier to interference ratio and is
    /// the form some specifications use.
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
/// 两个功率之一非正时返回 `None`。
pub fn carrier_to_interference_single(carrier: Power, interferer: Interferer) -> Option<Decibel> {
    carrier_to_interference(carrier, &[interferer])
}

/// Combines a ratio with another ratio of the same kind.
///
/// When two independent impairments act together, their effects add as
/// reciprocal ratios, which in decibels means
///
/// ```text
/// 1 / R = 1 / R_1 + 1 / R_2
/// ```
///
/// The result is always below the smaller of the two, so adding a second
/// impairment can only lower the combined ratio. This is how a carrier to
/// noise ratio and a carrier to interference ratio are combined into a
/// carrier to noise plus interference ratio.
///
/// Returns `None` when either value is not finite.
pub fn combine(first: Decibel, second: Decibel) -> Option<Decibel> {
    if !first.is_finite() || !second.is_finite() {
        return None;
    }
    // A ratio of zero in decibels means an infinite impairment, so the
    // combination is also infinite.
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
/// 两个功率之一非正时返回 `None`。
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
/// ```text
/// T = P / (k B)
/// ```
///
/// with `P` the interfering power, `k` the Boltzmann constant, and `B` the
/// receiver noise bandwidth. The temperature is the value that, added to the
/// system temperature, would raise the noise by the same amount as the
/// interferer.
///
/// Returns `None` when the power is not positive or the bandwidth is not
/// positive and finite.
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
