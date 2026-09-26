//! The instant type and its scale conversions.
//!
//! An [`Epoch`] stores an instant as a Modified Julian Date in International
//! Atomic Time. TAI is chosen as the internal representation because it is
//! uniform and because every other scale in this crate is either a constant
//! offset from it or, for UTC, a step offset that can be looked up.
//!
//! Storing the unified scale makes comparison and differencing between two
//! epochs exact regardless of the scales they were created in. The scale is
//! therefore not part of an epoch's identity; it is a parameter of the
//! conversion methods that read or write a number.

use crate::calendar::{
    Calendar, SECONDS_PER_DAY, mjd_from_ymd, seconds_of_day_from_mjd, ymd_from_mjd,
};
use crate::scale::{TAI_MINUS_GPS, TT_MINUS_TAI, TimeScale};
use apsis_leap::{tai_minus_utc_at_mjd, utc_mjd_from_tai_mjd};
use core::fmt;

/// An instant on the TAI time scale, stored as a Modified Julian Date.
///
/// The field is private so that the stored scale is guaranteed. Use
/// [`Epoch::from_mjd`] or the named constructors to build one.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Epoch {
    tai_mjd: f64,
}

impl Epoch {
    /// The instant 2000-01-01T12:00:00 TT, which defines the J2000.0 epoch.
    ///
    /// The value is `2451545.0` on the TT scale, offset back to TAI.
    pub const J2000: Self = Self::from_tai_mjd_const(51_544.5 - TT_MINUS_TAI / SECONDS_PER_DAY);

    /// Creates an epoch from a Modified Julian Date on the TAI scale.
    pub const fn from_tai_mjd(tai_mjd: f64) -> Self {
        Self { tai_mjd }
    }

    /// Const constructor used by the associated constants.
    const fn from_tai_mjd_const(tai_mjd: f64) -> Self {
        Self { tai_mjd }
    }

    /// Creates an epoch from a Modified Julian Date on the given scale.
    ///
    /// Returns `None` when the instant cannot be expressed on the requested
    /// scale. This happens for UTC before 1972, where the leap second table
    /// used to convert from UTC does not apply.
    pub fn from_mjd(mjd: f64, scale: TimeScale) -> Option<Self> {
        if !mjd.is_finite() {
            return None;
        }
        let tai_mjd = match scale {
            TimeScale::Tai => mjd,
            TimeScale::Tt => mjd - TT_MINUS_TAI / SECONDS_PER_DAY,
            TimeScale::Gps => mjd + TAI_MINUS_GPS / SECONDS_PER_DAY,
            TimeScale::Utc => {
                let offset = tai_minus_utc_at_mjd(mjd)?;
                mjd + offset / SECONDS_PER_DAY
            }
        };
        Some(Self { tai_mjd })
    }

    /// Creates an epoch from a calendar date, interpreting it on the given
    /// scale.
    ///
    /// Returns `None` when the fields do not form a valid date, or when the
    /// instant cannot be expressed on the requested scale.
    pub fn from_calendar(calendar: Calendar, scale: TimeScale) -> Option<Self> {
        if !calendar.is_valid() {
            return None;
        }
        let mjd = mjd_from_ymd(calendar.year, calendar.month, calendar.day)
            + calendar.seconds_of_day() / SECONDS_PER_DAY;
        Self::from_mjd(mjd, scale)
    }

    /// Returns the instant as a Modified Julian Date on the TAI scale.
    pub const fn tai_mjd(self) -> f64 {
        self.tai_mjd
    }

    /// Returns the instant as a Modified Julian Date on the given scale.
    ///
    /// Returns `None` when the instant precedes 1972 and the UTC scale is
    /// requested, because the leap second table does not cover that range.
    pub fn mjd(self, scale: TimeScale) -> Option<f64> {
        match scale {
            TimeScale::Tai => Some(self.tai_mjd),
            TimeScale::Tt => Some(self.tai_mjd + TT_MINUS_TAI / SECONDS_PER_DAY),
            TimeScale::Gps => Some(self.tai_mjd - TAI_MINUS_GPS / SECONDS_PER_DAY),
            TimeScale::Utc => utc_mjd_from_tai_mjd(self.tai_mjd),
        }
    }

    /// Returns the instant as a calendar date on the given scale.
    ///
    /// Returns `None` when the scale conversion is not available for this
    /// instant; see [`Epoch::mjd`].
    pub fn calendar(self, scale: TimeScale) -> Option<Calendar> {
        let mjd = self.mjd(scale)?;
        let (year, month, day) = ymd_from_mjd(mjd);
        let seconds = seconds_of_day_from_mjd(mjd);
        let hour = (seconds / 3_600.0).floor();
        let minute = ((seconds - hour * 3_600.0) / 60.0).floor();
        let second = seconds - hour * 3_600.0 - minute * 60.0;
        Some(Calendar::new(
            year,
            month,
            day,
            hour as u32,
            minute as u32,
            second,
        ))
    }

    /// Returns the signed number of seconds from `self` to `other`.
    ///
    /// The result is positive when `other` is later. Because both epochs
    /// are stored on the uniform TAI scale, the difference is exact and
    /// does not depend on the scales used to create either epoch.
    pub fn seconds_until(self, other: Self) -> f64 {
        (other.tai_mjd - self.tai_mjd) * SECONDS_PER_DAY
    }

    /// Returns an epoch shifted by `seconds`.
    ///
    /// The shift is measured in SI seconds on the uniform TAI scale, so it
    /// advances the instant by exactly that much elapsed time. This is the
    /// only shift offered: a shift measured on the UTC clock would differ
    /// when the interval spans an inserted leap second, and the UTC clock
    /// does not have a single well defined numeric value across such an
    /// instant. Callers that need calendar arithmetic on the UTC scale
    /// should convert to a [`Calendar`], adjust it, and convert back.
    pub fn add_seconds(self, seconds: f64) -> Self {
        Self {
            tai_mjd: self.tai_mjd + seconds / SECONDS_PER_DAY,
        }
    }

    /// Returns `true` when the stored Modified Julian Date is finite.
    pub const fn is_finite(self) -> bool {
        self.tai_mjd.is_finite()
    }
}

impl fmt::Display for Epoch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} TAI", self.tai_mjd)
    }
}
