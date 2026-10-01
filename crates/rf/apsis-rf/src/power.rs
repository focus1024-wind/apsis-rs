//! Radio frequency power.

use crate::decibel::Decibel;
use core::fmt;

/// The reference power of the decibel watt, one watt.
///
/// A power quoted in dBW is a ratio to this value.
pub const DBW_REFERENCE: f64 = 1.0;

/// The reference power of the decibel milliwatt, one milliwatt.
///
/// A power quoted in dBm is a ratio to this value, so a value in dBm is
/// thirty decibels above the same value in dBW.
pub const DBM_REFERENCE: f64 = 1.0e-3;

/// A power level in watts.
///
/// The type stores watts, which is the unit the decibel references are
/// defined against, and offers conversion to and from the two decibel forms
/// used in link budgets.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Power(f64);

impl Power {
    /// The reference power of zero dBW, one watt.
    pub const WATT: Self = Self(1.0);

    /// The reference power of zero dBm, one milliwatt.
    pub const MILLIWATT: Self = Self(1.0e-3);

    /// Creates a power level from a value in watts.
    ///
    /// A power may be zero, which represents no power, but not negative.
    pub const fn from_watts(watts: f64) -> Self {
        Self(watts)
    }

    /// Creates a power level from a value in milliwatts.
    pub const fn from_milliwatts(milliwatts: f64) -> Self {
        Self(milliwatts * 1.0e-3)
    }

    /// Creates a power level from a value in decibel watts.
    ///
    /// A zero or negative reading cannot be expressed as a finite power, so
    /// no validation is needed.
    pub fn from_dbw(decibel_watts: f64) -> Option<Self> {
        if !decibel_watts.is_finite() {
            return None;
        }
        Some(Self(DBW_REFERENCE * 10f64.powf(decibel_watts / 10.0)))
    }

    /// Creates a power level from a value in decibel milliwatts.
    ///
    /// Returns `None` when the reading is not finite.
    pub fn from_dbm(decibel_milliwatts: f64) -> Option<Self> {
        if !decibel_milliwatts.is_finite() {
            return None;
        }
        Some(Self(DBM_REFERENCE * 10f64.powf(decibel_milliwatts / 10.0)))
    }

    /// Returns this power in watts.
    pub const fn watts(self) -> f64 {
        self.0
    }

    /// Returns this power in milliwatts.
    pub const fn milliwatts(self) -> f64 {
        self.0 * 1.0e3
    }

    /// Returns this power in decibel watts.
    ///
    /// Returns `None` when the power is not positive, because the logarithm
    /// is not defined there.
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
    /// 两个功率之一非正时返回 `None`。
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
    pub fn scaled(self, ratio: Decibel) -> Option<Self> {
        if self.0 == 0.0 {
            return Some(self);
        }
        let scaled = self.0 * ratio.ratio();
        scaled.is_finite().then_some(Self(scaled))
    }

    /// Returns `true` when the power is finite and not negative.
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
