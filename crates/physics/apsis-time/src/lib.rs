//! Time scales, epochs, and calendar conversions.
//!
//! This crate provides the time model used by the rest of the workspace.
//! An [`Epoch`] is an instant stored as a Modified Julian Date on the
//! International Atomic Time scale, and [`TimeScale`] names the scale a
//! number is expressed in.
//!
//! Storing a single uniform scale makes the arithmetic exact: the
//! difference between two epochs is the elapsed SI time between them,
//! independent of the scales they were created in. UTC is not uniform,
//! because leap seconds are inserted to keep it near the rotation of the
//! Earth, so UTC conversions are routed through the table in `apsis-leap`.
//!
//! ```
//! use apsis_time::{Calendar, Epoch, TimeScale};
//!
//! // Build an instant from a UTC calendar date and read it back.
//! let calendar = Calendar::from_ymd(2024, 6, 15);
//! let epoch = Epoch::from_calendar(calendar, TimeScale::Utc).expect("covered by the leap table");
//! let back = epoch.calendar(TimeScale::Utc).expect("covered by the leap table");
//! assert_eq!(back.year, 2024);
//! assert_eq!(back.month, 6);
//! assert_eq!(back.day, 15);
//! ```
//!
//! Sources:
//!
//! - Julian date and Modified Julian date conventions: IAU, as described in
//!   the *Explanatory Supplement to the Astronomical Almanac*.
//! - Terrestrial Time offset from TAI: IAU 2000 Resolution B1.9.
//! - Leap seconds: IERS Bulletin C, embedded by `apsis-leap`.
//! - Calendar algorithms: Howard Hinnant, *chrono-Compatible Low-Level Date
//!   Algorithms*, 2015.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod calendar;
mod epoch;
mod julian;
mod scale;

pub use calendar::{
    Calendar, SECONDS_PER_DAY, days_in_month, is_leap_year, mjd_from_ymd, seconds_of_day_from_mjd,
    ymd_from_mjd,
};
pub use epoch::Epoch;
pub use julian::{JULIAN_DATE_OFFSET, julian_date_from_mjd, mjd_from_julian_date};
pub use scale::{TAI_MINUS_GPS, TT_MINUS_TAI, TimeScale};
