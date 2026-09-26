//! Noise in a cascade of stages, by the Friis formula.

use crate::temperature::{NoiseTemperature, REFERENCE_TEMPERATURE_K};
use apsis_rf::Gain;

/// One stage of a receiver chain.
///
/// A stage is described by the noise it adds, referred to its own input, and
/// the gain it applies. The noise may be given as a temperature or as a
/// noise figure; both are converted to a temperature before the cascade is
/// evaluated, because temperatures add across the chain while decibel
/// figures do not.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stage {
    /// The noise temperature added by this stage, referred to its input.
    pub temperature: NoiseTemperature,
    /// The available gain of this stage.
    pub gain: Gain,
}

impl Stage {
    /// Creates a stage from a noise temperature and a gain.
    pub const fn new(temperature: NoiseTemperature, gain: Gain) -> Self {
        Self { temperature, gain }
    }

    /// Creates a stage from a noise figure in decibels and a gain in
    /// decibels.
    ///
    /// Returns `None` when either value is not finite.
    pub fn from_noise_figure(noise_figure_db: f64, gain_dbi: f64) -> Option<Self> {
        Some(Self {
            temperature: NoiseTemperature::from_noise_figure_db(noise_figure_db)?,
            gain: Gain::from_dbi(gain_dbi)?,
        })
    }

    /// Creates a noiseless stage with the given gain.
    ///
    /// An ideal amplifier adds no noise, so it contributes nothing to the
    /// cascade except its gain.
    pub const fn ideal(gain: Gain) -> Self {
        Self {
            temperature: NoiseTemperature::from_kelvin(0.0),
            gain,
        }
    }

    /// Returns `true` when the stage is usable.
    pub fn is_valid(self) -> bool {
        self.temperature.is_valid() && self.gain.ratio().is_finite() && self.gain.ratio() > 0.0
    }
}

/// Returns the noise temperature of a cascade of stages, referred to the
/// input of the first stage.
///
/// The Friis formula weights each stage's contribution by the gain that
/// precedes it:
///
/// ```text
/// T = T_1 + T_2 / G_1 + T_3 / (G_1 G_2) + ...
/// ```
///
/// The consequence is that the first stage dominates: a noisy second stage
/// matters much less when the first stage has gain. The total temperature is
/// that of the chain, so it can be compared directly with the available
/// gain to find the noise figure of the chain.
///
/// Returns `None` when any stage is not usable, which includes a gain ratio
/// that is not positive.
pub fn cascade_temperature(stages: &[Stage]) -> Option<NoiseTemperature> {
    let mut total = NoiseTemperature::from_kelvin(0.0);
    let mut cumulative_gain = 1.0_f64;
    for stage in stages {
        if !stage.is_valid() {
            return None;
        }
        total = total.plus(stage.temperature.scaled(1.0 / cumulative_gain));
        cumulative_gain *= stage.gain.ratio();
    }
    Some(total)
}

/// Returns the noise temperature that corresponds to a noise figure in
/// decibels, using the standard reference temperature.
///
/// This is a free function form of
/// [`NoiseTemperature::from_noise_figure_db`](crate::NoiseTemperature::from_noise_figure_db).
///
/// Returns `None` when the figure is not finite.
pub fn noise_figure_to_temperature(noise_figure_db: f64) -> Option<NoiseTemperature> {
    NoiseTemperature::from_noise_figure_db(noise_figure_db)
}

/// Returns the noise figure of a cascade, in decibels.
///
/// The cascade is evaluated as a temperature and then converted to a figure
/// using the standard reference temperature. This is the form in which a
/// receiver chain is usually specified.
///
/// Returns `None` when any stage is not usable.
pub fn cascade_noise_figure_db(stages: &[Stage]) -> Option<f64> {
    cascade_temperature(stages)?.noise_figure_db()
}

/// Returns the reference temperature used by the noise figure definition.
pub const fn reference_temperature() -> f64 {
    REFERENCE_TEMPERATURE_K
}
