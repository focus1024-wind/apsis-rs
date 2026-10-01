//! The link budget and the figures derived from it.

use apsis_core::constant::BOLTZMANN;
use apsis_noise::NoiseTemperature;
use apsis_rf::{Decibel, Gain, Power, free_space_path_loss, wavelength};

/// The geometric part of a link: how far apart the terminals are and at what
/// frequency they operate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinkGeometry {
    /// The distance between the terminals, in meters.
    pub distance: f64,
    /// The carrier frequency, in hertz.
    pub frequency: f64,
}

impl LinkGeometry {
    /// Returns the free space path loss for this geometry.
    ///
    /// Returns `None` when the distance or frequency is not usable.
    pub fn path_loss(self) -> Option<Decibel> {
        free_space_path_loss(self.distance, self.frequency)
    }

    /// Returns `true` when the geometry is usable.
    pub fn is_valid(self) -> bool {
        self.distance.is_finite()
            && self.distance > 0.0
            && self.frequency.is_finite()
            && self.frequency > 0.0
    }
}

/// A complete link budget for one direction of a radio link.
///
/// The losses are given as positive decibel values that are subtracted,
/// which is the convention in link budgets: a three decibel loss is entered
/// as three decibels, not as minus three.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinkBudget {
    /// The power delivered to the transmit antenna, in watts.
    pub transmit_power: Power,
    /// The transmit antenna gain on the relevant path.
    pub transmit_gain: Gain,
    /// Every loss between the transmitter and the transmit antenna, as a
    /// positive decibel value.
    pub transmit_loss: Decibel,
    /// The receive antenna gain on the relevant path.
    pub receive_gain: Gain,
    /// Every loss between the receive antenna and the receiver input, as a
    /// positive decibel value.
    pub receive_loss: Decibel,
    /// The system noise temperature referred to the receiver input.
    pub system_temperature: NoiseTemperature,
    /// The receiver noise bandwidth, in hertz.
    pub bandwidth: f64,
    /// The distance and frequency of the link.
    pub geometry: LinkGeometry,
}

impl LinkBudget {
    /// Returns the equivalent isotropically radiated power, in watts.
    ///
    /// This is the transmit power reduced by the transmit losses and
    /// increased by the transmit antenna gain. It is the power that an
    /// isotropic radiator would have to radiate to produce the same flux
    /// density in the direction of the receiver.
    ///
    /// Returns `None` when any input is not usable.
    ///
    /// 任一输入不可用时返回 `None`。
    pub fn eirp(&self) -> Option<Power> {
        if !self.transmit_loss.is_finite() || !self.geometry.is_valid() {
            return None;
        }
        let gain = self.transmit_gain.as_decibel()?;
        let net = gain - self.transmit_loss;
        self.transmit_power.scaled(net)
    }

    /// Returns the carrier power at the receiver input, in watts.
    ///
    /// Returns `None` when any input is not usable.
    ///
    /// 任一输入不可用时返回 `None`。
    pub fn received_power(&self) -> Option<Power> {
        if !self.receive_loss.is_finite() {
            return None;
        }
        let eirp = self.eirp()?;
        let receive_gain = self.receive_gain.as_decibel()?;
        let loss = self.geometry.path_loss()?;
        eirp.scaled(receive_gain - loss - self.receive_loss)
    }

    /// Returns the noise power density at the receiver input, in watts per
    /// hertz.
    ///
    /// This is `k T`, the noise temperature converted to a power per unit
    /// bandwidth. It does not depend on the receiver bandwidth, which is
    /// what makes the carrier to noise density ratio a property of the link
    /// rather than of the receiver.
    ///
    /// Returns `None` when the system temperature is not usable.
    pub fn noise_density(&self) -> Option<f64> {
        if !self.system_temperature.is_valid() {
            return None;
        }
        Some(BOLTZMANN * self.system_temperature.kelvin())
    }

    /// Returns the carrier to noise density ratio.
    ///
    /// This is the carrier power divided by the noise power in one hertz.
    /// It is the form used to compare a link against the energy per bit
    /// requirement of a modulation, because the data rate is accounted for
    /// separately.
    ///
    /// Returns `None` when any input is not usable.
    ///
    /// 任一输入不可用时返回 `None`。
    pub fn carrier_to_noise_density(&self) -> Option<Decibel> {
        let carrier = self.received_power()?;
        let density = self.noise_density()?;
        if density <= 0.0 {
            return None;
        }
        // The ratio is a power ratio, and the numerator and denominator
        // carry different bandwidths, so the ratio is formed directly.
        let ratio = carrier.watts() / density;
        Decibel::from_ratio(ratio)
    }

    /// Returns the carrier to noise ratio in the receiver bandwidth.
    ///
    /// This is the carrier to noise density ratio reduced by the bandwidth.
    /// Unlike the density ratio it depends on the receiver, which is why the
    /// same link can show different carrier to noise ratios in receivers of
    /// different bandwidth.
    ///
    /// Returns `None` when any input is not usable.
    ///
    /// 任一输入不可用时返回 `None`。
    pub fn carrier_to_noise(&self) -> Option<Decibel> {
        if !self.bandwidth.is_finite() || self.bandwidth <= 0.0 {
            return None;
        }
        let density = self.carrier_to_noise_density()?;
        Some(density - Decibel::from_ratio(self.bandwidth)?)
    }

    /// Returns the energy per bit to noise density ratio for a data rate.
    ///
    /// This is the carrier to noise density ratio reduced by the data rate,
    /// because energy per bit is carrier power divided by bit rate.
    ///
    /// Returns `None` when the data rate is not positive, or when the
    /// carrier to noise density ratio is not usable.
    pub fn energy_per_bit_over_noise(&self, data_rate: f64) -> Option<Decibel> {
        if !data_rate.is_finite() || data_rate <= 0.0 {
            return None;
        }
        let density = self.carrier_to_noise_density()?;
        Some(density - Decibel::from_ratio(data_rate)?)
    }

    /// Returns the rise in system noise caused by the receive antenna
    /// viewing a noise source of the given temperature.
    ///
    /// The antenna noise is added to the system temperature in power, which
    /// in temperature is a sum. This is the form used to account for the
    /// Earth, the Sun, or a rain cell in the beam.
    ///
    /// Returns `None` when the added temperature is not usable.
    pub fn with_added_noise(&self, added: NoiseTemperature) -> Option<Self> {
        if !added.is_valid() {
            return None;
        }
        Some(Self {
            system_temperature: self.system_temperature + added,
            ..*self
        })
    }

    /// Returns the wavelength of the carrier, in meters.
    ///
    /// Returns `None` when the frequency is not usable.
    pub fn carrier_wavelength(&self) -> Option<f64> {
        wavelength(self.geometry.frequency)
    }

    /// Returns `true` when every input describes a usable link.
    pub fn is_valid(&self) -> bool {
        self.transmit_power.is_valid()
            && self.transmit_gain.dbi().is_some()
            && self.transmit_loss.is_finite()
            && self.receive_gain.dbi().is_some()
            && self.receive_loss.is_finite()
            && self.system_temperature.is_valid()
            && self.bandwidth.is_finite()
            && self.bandwidth > 0.0
            && self.geometry.is_valid()
    }
}

/// The result of comparing a link against a requirement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinkMargin {
    /// The ratio the link achieves, in decibels.
    pub achieved: Decibel,
    /// The ratio the link requires, in decibels.
    pub required: Decibel,
}

impl LinkMargin {
    /// Returns the margin, in decibels.
    ///
    /// A positive margin means the link closes with room to spare; a
    /// negative margin means it does not.
    pub fn margin(self) -> Decibel {
        self.achieved - self.required
    }

    /// Returns `true` when the margin is not negative.
    pub fn closes(self) -> bool {
        self.margin().db() >= 0.0
    }
}

/// Returns the margin between an achieved ratio and a required one.
///
/// This is the comparison that decides whether a link closes.
pub fn link_margin(achieved: Decibel, required: Decibel) -> LinkMargin {
    LinkMargin { achieved, required }
}

/// Returns the carrier to noise density ratio for a budget.
///
/// This is a free function form of
/// [`LinkBudget::carrier_to_noise_density`].
///
/// Returns `None` when the budget is not usable.
pub fn carrier_to_noise_density(budget: &LinkBudget) -> Option<Decibel> {
    budget.carrier_to_noise_density()
}

/// Returns the gain to system temperature ratio of a receive chain, in
/// reciprocal kelvin.
///
/// This figure of merit is written `G / T` and is the standard way to
/// compare receive stations. It combines the receive antenna gain with the
/// system noise temperature, so a station with a bigger dish and a quieter
/// receiver scores higher.
///
/// Expressed in decibels, `G / T` is the receive gain in dBi minus ten times
/// the logarithm of the temperature, which is the form usually quoted.
///
/// Returns `None` when the gain or temperature is not usable.
pub fn gain_to_temperature(gain: Gain, temperature: NoiseTemperature) -> Option<f64> {
    if !temperature.is_valid() || temperature.kelvin() <= 0.0 {
        return None;
    }
    let ratio = gain.ratio() / temperature.kelvin();
    if ratio.is_finite() && ratio > 0.0 {
        Some(ratio)
    } else {
        None
    }
}
