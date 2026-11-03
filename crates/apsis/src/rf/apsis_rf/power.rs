//! Radio frequency power.
//!
//! 射频功率。

use crate::apsis_rf::decibel::Decibel;
use core::fmt;

/// The reference power of the decibel watt, one watt.
///
/// A power quoted in dBW is a ratio to this value.
///
/// 分贝瓦的参考功率，即一瓦。
///
/// 以 dBW 表示的功率是相对该值的比值。
pub const DBW_REFERENCE: f64 = 1.0;

/// The reference power of the decibel milliwatt, one milliwatt.
///
/// A power quoted in dBm is a ratio to this value, so a value in dBm is
/// thirty decibels above the same value in dBW.
///
/// 分贝毫瓦的参考功率，即一毫瓦。
///
/// 以 dBm 表示的功率是相对该值的比值，故同一数值以 dBm 表示比以 dBW
/// 表示高三十分贝。
pub const DBM_REFERENCE: f64 = 1.0e-3;

/// A power level in watts.
///
/// The type stores watts, which is the unit the decibel references are
/// defined against, and offers conversion to and from the two decibel forms
/// used in link budgets.
///
/// 以瓦特表示的功率电平。
///
/// 该类型以瓦特存储，这是分贝参考所依据的单位，并提供与链路预算中常用
/// 的两种分贝形式之间的换算。
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Power(f64);

impl Power {
    /// The reference power of zero dBW, one watt.
    ///
    /// 零 dBW 的参考功率，即一瓦。
    pub const WATT: Self = Self(1.0);

    /// The reference power of zero dBm, one milliwatt.
    ///
    /// 零 dBm 的参考功率，即一毫瓦。
    pub const MILLIWATT: Self = Self(1.0e-3);

    /// Creates a power level from a value in watts.
    ///
    /// The value is stored as given, so the caller must pass zero or a
    /// positive number. The constructor is `const` so that the reference
    /// constants above can be defined from it, which is why it does not
    /// validate.
    ///
    /// 以瓦特值创建功率电平。
    ///
    /// 该值按原样保存，故调用方须传零或正数。构造函数为 `const`，以便上面的
    /// 参考常量由它定义，因此它不做校验。
    #[must_use]
    pub const fn from_watts(watts: f64) -> Self {
        Self(watts)
    }

    /// Creates a power level from a value in milliwatts.
    ///
    /// 以毫瓦值创建功率电平。
    #[must_use]
    pub const fn from_milliwatts(milliwatts: f64) -> Self {
        Self(milliwatts * 1.0e-3)
    }

    /// Creates a power level from a value in decibel watts.
    ///
    /// Returns `None` when the reading is not finite. Every finite reading
    /// maps to a positive power; a very negative reading maps to a very small
    /// one, so -204 dBW is about 4e-21 W rather than an error.
    ///
    /// 以分贝瓦值创建功率电平。
    ///
    /// 读数非有限时返回 `None`。每个有限读数都映射为正功率；很负的读数映射为
    /// 很小的功率，故 -204 dBW 约为 4e-21 W，而不是错误。
    #[must_use]
    pub fn from_dbw(decibel_watts: f64) -> Option<Self> {
        if !decibel_watts.is_finite() {
            return None;
        }
        Some(Self(DBW_REFERENCE * 10f64.powf(decibel_watts / 10.0)))
    }

    /// Creates a power level from a value in decibel milliwatts.
    ///
    /// Returns `None` when the reading is not finite.
    ///
    /// 以分贝毫瓦值创建功率电平。
    ///
    /// 该读数非有限时返回 `None`。
    #[must_use]
    pub fn from_dbm(decibel_milliwatts: f64) -> Option<Self> {
        if !decibel_milliwatts.is_finite() {
            return None;
        }
        Some(Self(DBM_REFERENCE * 10f64.powf(decibel_milliwatts / 10.0)))
    }

    /// Returns this power in watts.
    ///
    /// 返回该功率的瓦特值。
    #[must_use]
    pub const fn watts(self) -> f64 {
        self.0
    }

    /// Returns this power in milliwatts.
    ///
    /// 返回该功率的毫瓦值。
    #[must_use]
    pub const fn milliwatts(self) -> f64 {
        self.0 * 1.0e3
    }

    /// Returns this power in decibel watts.
    ///
    /// Returns `None` when the power is not positive, because the logarithm
    /// is not defined there.
    ///
    /// 返回该功率的分贝瓦值。
    ///
    /// 功率非正时返回 `None`，因为对数在那里没有定义。
    #[must_use]
    pub fn dbw(self) -> Option<f64> {
        if self.0 > 0.0 && self.0.is_finite() {
            Some(10.0 * (self.0 / DBW_REFERENCE).log10())
        } else {
            None
        }
    }

    /// Returns this power in decibel milliwatts.
    ///
    /// Returns `None` when the power is not positive.
    ///
    /// 返回该功率的分贝毫瓦值。
    ///
    /// 功率非正时返回 `None`。
    #[must_use]
    pub fn dbm(self) -> Option<f64> {
        if self.0 > 0.0 && self.0.is_finite() {
            Some(10.0 * (self.0 / DBM_REFERENCE).log10())
        } else {
            None
        }
    }

    /// Returns the ratio of `self` to `rhs`, in decibels.
    ///
    /// This is the form used in a link budget to express a loss or a gain
    /// between two power levels.
    ///
    /// Returns `None` when either power is not positive.
    ///
    /// 返回 `self` 与 `rhs` 之比，以分贝计。
    ///
    /// 这是链路预算中用来表达两个功率电平之间损耗或增益的形式。
    ///
    /// 两个功率之一非正时返回 `None`。
    #[must_use]
    pub fn ratio_in_decibels(self, rhs: Self) -> Option<Decibel> {
        if self.0 > 0.0 && rhs.0 > 0.0 {
            Decibel::from_ratio(self.0 / rhs.0)
        } else {
            None
        }
    }

    /// Returns this power scaled by a ratio in decibels.
    ///
    /// Returns `None` when the resulting power is not finite, which happens
    /// for a very large gain.
    ///
    /// 返回该功率按某个分贝比值缩放后的值。
    ///
    /// 结果功率非有限时返回 `None`；增益极大时即如此。
    #[must_use]
    pub fn scaled(self, ratio: Decibel) -> Option<Self> {
        if self.0 == 0.0 {
            return Some(self);
        }
        let scaled = self.0 * ratio.ratio();
        scaled.is_finite().then_some(Self(scaled))
    }

    /// Returns `true` when the power is finite and not negative.
    ///
    /// 当该功率有限且非负时返回 `true`。
    #[must_use]
    pub const fn is_valid(self) -> bool {
        self.0.is_finite() && self.0 >= 0.0
    }
}

impl fmt::Display for Power {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.dbw() {
            Some(dbw) => write!(f, "{dbw} dBW"),
            None => write!(f, "{} W", self.0),
        }
    }
}
