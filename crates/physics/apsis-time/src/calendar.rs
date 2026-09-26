//! Proleptic Gregorian calendar conversions.
//!
//! The calendar is the proleptic Gregorian calendar, extended backwards
//! before its historical adoption. A day is identified by its Modified
//! Julian Date (MJD), which counts days from 1858-11-17. MJD 0 is the start
//! of that day, so the MJD of a day is an integer and an instant within the
//! day adds a fraction.
//!
//! The conversion uses the algorithm published by Howard Hinnant in
//! *chrono-Compatible Low-Level Date Algorithms* (2015), which is exact for
//! the whole range of the Gregorian calendar and needs no lookup table.
//!
//! The seconds field is a floating point number so that a leap second can
//! be represented as second 60. Values are not constrained here; callers
//! that require a valid calendar date must check the fields they care
//! about.

use core::fmt;

/// The number of seconds in a day, not counting leap seconds.
pub const SECONDS_PER_DAY: f64 = 86_400.0;

/// A calendar date and time of day.
///
/// Fields are the proleptic Gregorian calendar fields. `second` is a
/// floating point number in the range `[0, 60)`.
///
/// The second may not reach 60. A leap second is written as second 60 in
/// UTC, but that label does not have a unique position on a continuous time
/// scale: it occupies the same interval that the following midnight would
/// otherwise occupy. This crate therefore treats leap seconds only where
/// they are unambiguous, in the `TAI - UTC` offset lookup, and not as
/// calendar instants.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Calendar {
    /// Year, negative for years before year 1.
    pub year: i32,
    /// Month in `1..=12`.
    pub month: u32,
    /// Day of month in `1..=31`.
    pub day: u32,
    /// Hour in `0..=23`.
    pub hour: u32,
    /// Minute in `0..=59`.
    pub minute: u32,
    /// Second in `[0, 61)`.
    pub second: f64,
}

impl Calendar {
    /// Creates a calendar date from its fields.
    pub const fn new(year: i32, month: u32, day: u32, hour: u32, minute: u32, second: f64) -> Self {
        Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
        }
    }

    /// Creates a calendar date at the start of the given day.
    pub const fn from_ymd(year: i32, month: u32, day: u32) -> Self {
        Self::new(year, month, day, 0, 0, 0.0)
    }

    /// Returns the seconds elapsed since the start of the day.
    pub fn seconds_of_day(self) -> f64 {
        f64::from(self.hour) * 3_600.0 + f64::from(self.minute) * 60.0 + self.second
    }

    /// Returns `true` when the fields form a date that exists in the
    /// proleptic Gregorian calendar.
    pub fn is_valid(self) -> bool {
        (1..=12).contains(&self.month)
            && self.day >= 1
            && self.day <= days_in_month(self.year, self.month)
            && self.hour <= 23
            && self.minute <= 59
            && self.second >= 0.0
            && self.second < 60.0
    }
}

impl fmt::Display for Calendar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:06.3}",
            self.year, self.month, self.day, self.hour, self.minute, self.second
        )
    }
}

/// Returns `true` when `year` is a leap year in the proleptic Gregorian
/// calendar.
pub const fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Returns the number of days in the given month of the given year.
///
/// Returns zero when `month` is outside `1..=12`.
pub const fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// Converts a calendar date, ignoring the time of day, to a Modified Julian
/// Date.
///
/// The result is the MJD of the start of the given day, so it is an
/// integer. The time of day is ignored; use [`Calendar::seconds_of_day`] to
/// account for it.
pub const fn mjd_from_ymd(year: i32, month: u32, day: u32) -> f64 {
    // Shift the year so that March is the first month, which places the
    // leap day at the end of the shifted year and removes a special case
    // from the day count.
    let a = (14 - month as i64) / 12;
    let y = year as i64 + 4800 - a;
    let m = month as i64 + 12 * a - 3;
    let jdn = day as i64 + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400 - 32045;
    (jdn - 2_400_001) as f64
}

/// Converts a Modified Julian Date to a calendar date.
///
/// The fractional part is ignored, so the result is the calendar day
/// containing the instant.
pub fn ymd_from_mjd(mjd: f64) -> (i32, u32, u32) {
    let jdn = mjd.floor() as i64 + 2_400_001;
    let a = jdn + 32_044;
    let b = (4 * a + 3) / 146_097;
    let c = a - (146_097 * b) / 4;
    let d = (4 * c + 3) / 1_461;
    let e = c - (1_461 * d) / 4;
    let m = (5 * e + 2) / 153;
    let day = (e - (153 * m + 2) / 5 + 1) as u32;
    let month = (m + 3 - 12 * (m / 10)) as u32;
    let year = (100 * b + d - 4_800 + (m / 10)) as i32;
    (year, month, day)
}

/// Keeps only the fractional part of an MJD as seconds since midnight.
///
/// The result lies in `[0, 86400)` for any finite input.
pub fn seconds_of_day_from_mjd(mjd: f64) -> f64 {
    let fraction = mjd - mjd.floor();
    fraction * SECONDS_PER_DAY
}
