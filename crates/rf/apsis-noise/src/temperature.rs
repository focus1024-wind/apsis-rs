//! Noise temperature.

use apsis_core::constant::BOLTZMANN;
use apsis_rf::Power;
use core::fmt;

/// The reference temperature used to define the noise figure, in kelvin.
///
/// The value is 290 K, which is the standard reference temperature of the
/// IEEE definition of noise figure. It is a convention rather than a
/// physical temperature, and the choice affects the noise figure quoted for
/// a given noise temperature.
///
/// 用于定义噪声系数的参考温度，单位为开尔文。
///
/// 该值为 290 K，即 IEEE 噪声系数定义的标准参考温度。它是约定而非物理
/// 温度，这一选择会影响给定噪声温度所对应的噪声系数。
pub const REFERENCE_TEMPERATURE_K: f64 = 290.0;

/// A noise temperature in kelvin.
///
/// A noise temperature is the temperature of a resistor that would deliver
/// the same noise power in the same bandwidth. It is a convenient way to
/// express any noise source, including one that is not thermal in origin,
/// because temperatures add across a cascade while powers do not.
///
/// 以开尔文表示的噪声温度。
///
/// 噪声温度是在同一带宽内会给出相同噪声功率的电阻的温度。它是表达任何
/// 噪声源（包括非热起源者）的便利方式，因为温度在级联中相加而功率
/// 不然。
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct NoiseTemperature(f64);

impl NoiseTemperature {
    /// The standard reference temperature of 290 K.
    ///
    /// 290 K 的标准参考温度。
    pub const REFERENCE: Self = Self(REFERENCE_TEMPERATURE_K);

    /// Creates a noise temperature from a value in kelvin.
    ///
    /// 由以开尔文表示的值创建噪声温度。
    pub const fn from_kelvin(kelvin: f64) -> Self {
        Self(kelvin)
    }

    /// Creates a noise temperature from a noise figure in decibels.
    ///
    /// The relation is `T = T_0 (F - 1)` with `F` the linear noise factor,
    /// so a noise figure of zero decibels gives a temperature of zero.
    ///
    /// Returns `None` when the figure is not finite.
    ///
    /// 由以分贝表示的噪声系数创建噪声温度。
    ///
    /// 关系为 `T = T_0 (F - 1)`，其中 `F` 是线性噪声因子，故零分贝的
    /// 噪声系数给出零温度。
    ///
    /// 该系数非有限时返回 `None`。
    pub fn from_noise_figure_db(decibels: f64) -> Option<Self> {
        if !decibels.is_finite() {
            return None;
        }
        let factor = 10f64.powf(decibels / 10.0);
        Some(Self(REFERENCE_TEMPERATURE_K * (factor - 1.0)))
    }

    /// Creates a noise temperature from a linear noise factor.
    ///
    /// Returns `None` when the factor is not finite or is below one, which
    /// would describe a stage that removes noise.
    ///
    /// 由线性噪声因子创建噪声温度。
    ///
    /// 该因子非有限或小于一时返回 `None`，那将描述一个去除噪声的级。
    pub fn from_noise_factor(factor: f64) -> Option<Self> {
        if factor.is_finite() && factor >= 1.0 {
            Some(Self(REFERENCE_TEMPERATURE_K * (factor - 1.0)))
        } else {
            None
        }
    }

    /// Returns this temperature in kelvin.
    ///
    /// 返回该温度的开尔文值。
    pub const fn kelvin(self) -> f64 {
        self.0
    }

    /// Returns this temperature as a linear noise factor.
    ///
    /// The factor is `T / T_0 + 1`, which is one for zero noise.
    ///
    /// 返回该温度的线性噪声因子。
    ///
    /// 该因子为 `T / T_0 + 1`，零噪声时为一。
    pub fn noise_factor(self) -> f64 {
        self.0 / REFERENCE_TEMPERATURE_K + 1.0
    }

    /// Returns this temperature as a noise figure in decibels.
    ///
    /// Returns `None` when the temperature is not finite or is negative,
    /// which would give a factor below one.
    ///
    /// 返回该温度的噪声系数，以分贝计。
    ///
    /// 温度非有限或为负（会给出小于一的因子）时返回 `None`。
    pub fn noise_figure_db(self) -> Option<f64> {
        let factor = self.noise_factor();
        if factor > 0.0 && factor.is_finite() && self.0 >= 0.0 {
            Some(10.0 * factor.log10())
        } else {
            None
        }
    }

    /// Returns the sum of two temperatures.
    ///
    /// Noise temperatures of independent sources add, which is the reason
    /// the quantity is useful.
    ///
    /// 返回两个温度之和。
    ///
    /// 独立源的噪声温度相加，这正是该量有用的原因。
    pub const fn plus(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }

    /// Returns this temperature scaled by a gain ratio.
    ///
    /// A source followed by a gain contributes its temperature divided by
    /// that gain when referred to the input, so the scaling for a
    /// contribution referred to the input is the reciprocal.
    ///
    /// 返回该温度按某增益比缩放后的值。
    ///
    /// 某源之后接一增益时，折算到输入其贡献为该温度除以该增益，故折算到
    /// 输入的贡献其缩放取倒数。
    pub const fn scaled(self, factor: f64) -> Self {
        Self(self.0 * factor)
    }

    /// Returns `true` when the temperature is finite and not negative.
    ///
    /// 当该温度有限且非负时返回 `true`。
    pub const fn is_valid(self) -> bool {
        self.0.is_finite() && self.0 >= 0.0
    }
}

impl fmt::Display for NoiseTemperature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} K", self.0)
    }
}

impl core::ops::Add for NoiseTemperature {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        self.plus(rhs)
    }
}

/// Returns the thermal noise power in a bandwidth at a noise temperature.
///
/// The relation is `P = k T B`, with `k` the Boltzmann constant. This is the
/// power that a matched resistor at the given temperature would deliver to a
/// load over the stated bandwidth.
///
/// The bandwidth must be positive. Returns `None` for an invalid
/// temperature or bandwidth.
///
/// 返回某噪声温度下某带宽内的热噪声功率。
///
/// 关系为 `P = k T B`，其中 `k` 是玻尔兹曼常量。这是给定温度下的匹配
/// 电阻在所给带宽内向负载输送的功率。
///
/// 带宽必须为正。温度或带宽无效时返回 `None`。
pub fn thermal_noise_power(temperature: NoiseTemperature, bandwidth: f64) -> Option<Power> {
    if !temperature.is_valid() {
        return None;
    }
    if !bandwidth.is_finite() || bandwidth <= 0.0 {
        return None;
    }
    Some(Power::from_watts(
        BOLTZMANN * temperature.kelvin() * bandwidth,
    ))
}
