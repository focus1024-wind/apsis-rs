//! The decibel scale.

use core::fmt;

/// A ratio expressed in decibels.
///
/// 以分贝表示的比值。
///
/// A decibel is ten times the base ten logarithm of a power ratio, so a
/// value of zero is unity gain or loss, a value of three is very close to a
/// doubling, and a value of minus three is very close to a halving.
///
/// The type stores the decibel value rather than the ratio, because a chain
/// of gains and losses is then a sum, which is both faster and less prone to
/// overflow than a product of ratios.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Decibel(f64);

impl Decibel {
    /// The value for a ratio of one, which is zero decibels.
    ///
    /// 比值为一时的取值，即零分贝。
    pub const UNITY: Self = Self(0.0);

    /// The value for a ratio of two, which is about 3.0103 decibels.
    ///
    /// 比值为二时的取值，约为 3.0103 分贝。
    pub const DOUBLING: Self = Self(3.010_299_956_639_812);

    /// Creates a value from a decibel reading.
    ///
    /// 由分贝读数创建一个取值。
    pub const fn from_db(decibels: f64) -> Self {
        Self(decibels)
    }

    /// Creates a value from a power ratio.
    ///
    /// 由功率比创建一个取值。
    ///
    /// Returns `None` when the ratio is not positive and finite, because the
    /// logarithm is not defined there.
    pub fn from_ratio(ratio: f64) -> Option<Self> {
        if ratio > 0.0 && ratio.is_finite() {
            Some(Self(10.0 * ratio.log10()))
        } else {
            None
        }
    }

    /// Creates a value from an amplitude ratio.
    ///
    /// 由幅度比创建一个取值。
    ///
    /// An amplitude, unlike a power, scales the decibel value by twenty
    /// rather than ten, because power is proportional to amplitude squared.
    /// Use this for a voltage or field ratio.
    ///
    /// Returns `None` when the ratio is not positive and finite.
    ///
    /// 比值非正或非有限时返回 `None`。
    pub fn from_amplitude_ratio(ratio: f64) -> Option<Self> {
        if ratio > 0.0 && ratio.is_finite() {
            Some(Self(20.0 * ratio.log10()))
        } else {
            None
        }
    }

    /// Returns the reading in decibels.
    ///
    /// 返回该分贝读数。
    pub const fn db(self) -> f64 {
        self.0
    }

    /// Returns the power ratio this reading represents.
    ///
    /// 返回该读数所表示的功率比。
    pub fn ratio(self) -> f64 {
        10f64.powf(self.0 / 10.0)
    }

    /// Returns the amplitude ratio this reading represents.
    ///
    /// 返回该读数所表示的幅度比。
    pub fn amplitude_ratio(self) -> f64 {
        10f64.powf(self.0 / 20.0)
    }

    /// Returns the sum of two readings.
    ///
    /// Adding decibel values multiplies the ratios they represent, which is
    /// how a cascade of gains and losses is combined.
    ///
    /// 返回两个读数之和。
    ///
    /// 分贝值相加即其所表示的比值相乘，这正是级联增益与损耗的合成
    /// 方式。
    pub const fn plus(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }

    /// Returns the difference of two readings.
    ///
    /// Subtracting decibel values divides the ratios they represent, which
    /// is how a loss is removed from a gain.
    ///
    /// 返回两个读数之差。
    ///
    /// 分贝值相减即其所表示的比值相除，这正是从增益中去掉损耗的方式。
    pub const fn minus(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }

    /// Returns this reading negated, which inverts the ratio.
    ///
    /// 返回该读数的相反数，即比值的倒数。
    pub const fn negated(self) -> Self {
        Self(-self.0)
    }

    /// Returns `true` when the reading is finite.
    ///
    /// 当该读数有限时返回 `true`。
    pub const fn is_finite(self) -> bool {
        self.0.is_finite()
    }
}

impl fmt::Display for Decibel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} dB", self.0)
    }
}

impl core::ops::Add for Decibel {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        self.plus(rhs)
    }
}

impl core::ops::Sub for Decibel {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        self.minus(rhs)
    }
}

impl core::ops::Neg for Decibel {
    type Output = Self;

    fn neg(self) -> Self {
        self.negated()
    }
}
