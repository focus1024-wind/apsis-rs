//! Antenna gain.
//!
//! 天线增益。

use crate::decibel::Decibel;
use core::f64::consts::PI;
use core::fmt;

/// The gain of a half wave dipole relative to an isotropic radiator, in
/// decibels.
///
/// The exact directivity of an ideal half wave dipole is 1.6409, which is
/// 2.1509 dB. Engineering practice rounds this to 2.15 dB, and that rounded
/// value is what the `dBd` and `dBi` conventions are defined against. This
/// crate uses the conventional value so that a conversion between the two
/// named forms is exact, and it derives the linear ratio from it.
///
/// 半波振子相对各向同性辐射体的增益，单位为分贝。
///
/// 理想半波振子的精确方向性为 1.6409，即 2.1509 dB。工程实践将其取整为
/// 2.15 dB，`dBd` 与 `dBi` 两种约定即以该取整值定义。本 crate 采用
/// 该约定值，使两种具名形式之间的换算精确，并由其导出线性比值。
pub const DIPOLE_OVER_ISOTROPIC_DB: f64 = 2.15;

/// The linear gain ratio of a half wave dipole, `10^(2.15 / 10)`.
///
/// This is about 1.64059, within 0.02 percent of the exact directivity of an
/// ideal dipole, which is 1.6409.
///
/// 半波振子的线性增益比值，`10^(2.15 / 10)`。
///
/// 该值约为 1.64059，与理想振子 1.6409 的精确方向性相差在 0.02% 以内。
pub const HALF_WAVE_DIPOLE_RATIO: f64 = 1.640_589_773_199_539_4;

/// An antenna gain expressed as a ratio to an isotropic radiator.
///
/// The type stores the linear ratio. The two named decibel forms are offered
/// as conversions, because the choice is a matter of convention rather than
/// of the gain itself:
///
/// - `dBi` is the ratio to an isotropic radiator, which radiates equally in
///   every direction.
/// - `dBd` is the ratio to a half wave dipole, which is
///   [`DIPOLE_OVER_ISOTROPIC_DB`] decibels above an isotropic radiator.
///
/// 以相对各向同性辐射体的比值表示的天线增益。
///
/// 该类型以线性比值存储。两种具名分贝形式以换算形式提供，因为这一
/// 选择关乎约定而非增益本身：
///
/// - `dBi` 是相对各向同性辐射体的比值，后者向各方向均匀辐射。
/// - `dBd` 是相对半波振子的比值，后者比各向同性辐射体高
///   [`DIPOLE_OVER_ISOTROPIC_DB`] 分贝。
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Gain(f64);

impl Default for Gain {
    /// Returns unit gain, matching [`Gain::ISOTROPIC`].
    ///
    /// This is written by hand rather than derived, because the default for
    /// a ratio type is not its zero but its unit.
    ///
    /// 返回单位增益，与 [`Gain::ISOTROPIC`] 相同。
    ///
    /// 此处手写而非推导，因为比值类型的缺省值不是其零而是其单位。
    fn default() -> Self {
        Self::ISOTROPIC
    }
}

impl Gain {
    /// Unit gain, as an isotropic radiator has.
    ///
    /// 单位增益，如各向同性辐射体所有。
    pub const ISOTROPIC: Self = Self(1.0);

    /// The gain of a half wave dipole, about 1.6406.
    ///
    /// 半波振子的增益，约为 1.6406。
    pub const HALF_WAVE_DIPOLE: Self = Self(HALF_WAVE_DIPOLE_RATIO);

    /// Creates a gain from a linear ratio.
    ///
    /// 由线性比值创建增益。
    pub const fn from_ratio(ratio: f64) -> Self {
        Self(ratio)
    }

    /// Creates a gain from a value in decibels relative to an isotropic
    /// radiator.
    ///
    /// A gain ratio is never negative, so any finite reading is accepted.
    ///
    /// 由相对各向同性辐射体的分贝值创建增益。
    ///
    /// 增益比值不会为负，故任何有限读数都被接受。
    pub fn from_dbi(dbi: f64) -> Option<Self> {
        if !dbi.is_finite() {
            return None;
        }
        Some(Self(Decibel::from_db(dbi).ratio()))
    }

    /// Creates a gain from a value in decibels relative to a half wave
    /// dipole.
    ///
    /// Returns `None` when the reading is not finite.
    ///
    /// 由相对半波振子的分贝值创建增益。
    ///
    /// 该读数非有限时返回 `None`。
    pub fn from_dbd(dbd: f64) -> Option<Self> {
        if !dbd.is_finite() {
            return None;
        }
        Self::from_dbi(dbd + DIPOLE_OVER_ISOTROPIC_DB)
    }

    /// Returns the linear ratio to an isotropic radiator.
    ///
    /// 返回相对各向同性辐射体的线性比值。
    pub const fn ratio(self) -> f64 {
        self.0
    }

    /// Returns this gain in decibels relative to an isotropic radiator.
    ///
    /// Returns `None` when the ratio is not positive.
    ///
    /// 返回该增益，相对各向同性辐射体以分贝计。
    ///
    /// 比值非正时返回 `None`。
    pub fn dbi(self) -> Option<f64> {
        if self.0 > 0.0 && self.0.is_finite() {
            Some(10.0 * self.0.log10())
        } else {
            None
        }
    }

    /// Returns this gain in decibels relative to a half wave dipole.
    ///
    /// Returns `None` when the ratio is not positive.
    ///
    /// 返回该增益，相对半波振子以分贝计。
    ///
    /// 比值非正时返回 `None`。
    pub fn dbd(self) -> Option<f64> {
        self.dbi().map(|dbi| dbi - DIPOLE_OVER_ISOTROPIC_DB)
    }

    /// Returns this gain as a ratio in decibels.
    ///
    /// Returns `None` when the ratio is not positive.
    ///
    /// 返回该增益，以分贝比表示。
    ///
    /// 比值非正时返回 `None`。
    pub fn as_decibel(self) -> Option<Decibel> {
        Decibel::from_ratio(self.0)
    }

    /// Returns the effective aperture of an antenna with this gain at the
    /// given wavelength, in square meters.
    ///
    /// The relation is `A = G lambda^2 / (4 pi)`. It follows from the
    /// requirement that an antenna in a thermal bath receive as much power
    /// as it radiates.
    ///
    /// Returns `None` when the wavelength is not positive and finite.
    ///
    /// 返回具有该增益的天线在给定波长处的有效孔径，单位为平方米。
    ///
    /// 关系为 `A = G lambda^2 / (4 pi)`。它由热浴中的天线接收功率须等于
    /// 其辐射功率这一要求得出。
    ///
    /// 波长非正且非有限时返回 `None`。
    pub fn effective_aperture(self, wavelength: f64) -> Option<f64> {
        if wavelength > 0.0 && wavelength.is_finite() && self.0.is_finite() {
            Some(self.0 * wavelength * wavelength / (4.0 * PI))
        } else {
            None
        }
    }

    /// Returns the gain of an antenna with this effective aperture at the
    /// given wavelength.
    ///
    /// This is the inverse of [`Gain::effective_aperture`].
    ///
    /// Returns `None` when the wavelength is not positive and finite, or
    /// when the aperture is negative.
    ///
    /// 返回具有该有效孔径的天线在给定波长处的增益。
    ///
    /// 它是 [`Gain::effective_aperture`] 的逆。
    ///
    /// 波长非正且非有限，或孔径为负时返回 `None`。
    pub fn from_effective_aperture(aperture: f64, wavelength: f64) -> Option<Self> {
        if wavelength > 0.0 && wavelength.is_finite() && aperture >= 0.0 && aperture.is_finite() {
            Some(Self(4.0 * PI * aperture / (wavelength * wavelength)))
        } else {
            None
        }
    }
}

impl fmt::Display for Gain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.dbi() {
            Some(dbi) => write!(f, "{dbi} dBi"),
            None => write!(f, "{} (ratio)", self.0),
        }
    }
}
