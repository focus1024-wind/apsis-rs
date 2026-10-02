//! Noise in a cascade of stages, by the Friis formula.
//!
//! 按 Friis 公式计算的级联各级噪声。

use crate::temperature::{NoiseTemperature, REFERENCE_TEMPERATURE_K};
use apsis_rf::Gain;

/// One stage of a receiver chain.
///
/// A stage is described by the noise it adds, referred to its own input, and
/// the gain it applies. The noise may be given as a temperature or as a
/// noise figure; both are converted to a temperature before the cascade is
/// evaluated, because temperatures add across the chain while decibel
/// figures do not.
///
/// 接收链的一级。
///
/// 一级由其折算到自身输入所加的噪声与其所施加的增益描述。噪声可以温度或
/// 噪声系数给出；两者都在评估级联之前换算为温度，因为温度在链中相加而
/// 分贝系数不然。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stage {
    /// The noise temperature added by this stage, referred to its input.
    ///
    /// 本级折算到其输入所加的噪声温度。
    pub temperature: NoiseTemperature,
    /// The available gain of this stage.
    ///
    /// 本级的可用增益。
    pub gain: Gain,
}

impl Stage {
    /// Creates a stage from a noise temperature and a gain.
    ///
    /// 由噪声温度与增益创建一级。
    #[must_use]
    pub const fn new(temperature: NoiseTemperature, gain: Gain) -> Self {
        Self { temperature, gain }
    }

    /// Creates a stage from a noise figure in decibels and a gain in
    /// decibels.
    ///
    /// Returns `None` when either value is not finite.
    ///
    /// 由以分贝表示的噪声系数与以分贝表示的增益创建一级。
    ///
    /// 任一值非有限时返回 `None`。
    #[must_use]
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
    ///
    /// 以给定增益创建无噪声的一级。
    ///
    /// 理想放大器不加噪声，故除其增益外它对级联别无贡献。
    #[must_use]
    pub const fn ideal(gain: Gain) -> Self {
        Self {
            temperature: NoiseTemperature::from_kelvin(0.0),
            gain,
        }
    }

    /// Returns `true` when the stage is usable.
    ///
    /// 当该级可用时返回 `true`。
    #[must_use]
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
/// <div>
/// $$
/// T = T_1 + \frac{T_2}{G_1} + \frac{T_3}{G_1 G_2} + \cdots
/// $$
/// </div>
///
/// The consequence is that the first stage dominates: a noisy second stage
/// matters much less when the first stage has gain. The total temperature is
/// that of the chain, so it can be compared directly with the available
/// gain to find the noise figure of the chain.
///
/// Returns `None` when any stage is not usable, which includes a gain ratio
/// that is not positive.
///
/// 返回各级级联的噪声温度，折算到第一级的输入。
///
/// Friis 公式按本级之前的增益对每级的贡献加权：
///
/// <div>
/// $$
/// T = T_1 + \frac{T_2}{G_1} + \frac{T_3}{G_1 G_2} + \cdots
/// $$
/// </div>
///
/// 其结果是第一级占主导：当第一级有增益时，噪声大的第二级影响小得多。
/// 总温度即整链的温度，故可直接与可用增益比较以求得整链的噪声系数。
///
/// 任一级不可用（包括增益比非正）时返回 `None`。
#[must_use]
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
///
/// 使用标准参考温度，返回与某分贝噪声系数对应的噪声温度。
///
/// 这是
/// [`NoiseTemperature::from_noise_figure_db`](crate::NoiseTemperature::from_noise_figure_db)
/// 的自由函数形式。
///
/// 该系数非有限时返回 `None`。
#[must_use]
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
///
/// 返回级联的噪声系数，以分贝计。
///
/// 级联先按温度求值，再用标准参考温度换算为系数。这是接收链通常规定的
/// 形式。
///
/// 任一级不可用时返回 `None`。
#[must_use]
pub fn cascade_noise_figure_db(stages: &[Stage]) -> Option<f64> {
    cascade_temperature(stages)?.noise_figure_db()
}

/// Returns the reference temperature used by the noise figure definition.
///
/// 返回噪声系数定义所用的参考温度。
#[must_use]
pub const fn reference_temperature() -> f64 {
    REFERENCE_TEMPERATURE_K
}
