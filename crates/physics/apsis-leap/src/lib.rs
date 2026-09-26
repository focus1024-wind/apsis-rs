//! Embedded leap second table for the UTC time scale.
//!
//! Coordinated Universal Time (UTC) is kept within 0.9 s of UT1 by
//! inserting or removing leap seconds. Because UTC is not a uniform time
//! scale, converting between UTC and the uniform scales TAI, TT, and GPS
//! requires the offset `TAI - UTC` in force at the instant of interest.
//!
//! This crate embeds that offset as a compile time table and answers
//! queries against it. Dates are handled as Modified Julian Date (MJD),
//! which is the Julian Date minus 2 400 000.5. An MJD is a count of days
//! since 1858-11-17T00:00:00, so it carries no time scale of its own; the
//! caller states the scale by choosing which function to call.
//!
//! The table begins at 1972-01-01, when UTC adopted the current integer
//! second offset scheme. Offsets before that date were fractional and
//! followed a different rule, so queries earlier than the first entry
//! return `None` rather than a misleading value.
//!
//! Source: IERS Bulletin C, which announces leap seconds after
//! confirmation by the International Earth Rotation and Reference Systems
//! Service. The most recent entry in this table is the leap second of
//! 2017-01-01; no leap second has been announced since.
//!
//! ```
//! use apsis_leap::{tai_minus_utc_at_mjd, LEAP_SECONDS};
//!
//! // The last entry in the embedded table fixes the current offset.
//! let last = LEAP_SECONDS.last().expect("table is not empty");
//! assert_eq!(last.tai_minus_utc, 37.0);
//! assert_eq!(tai_minus_utc_at_mjd(60_000.0), Some(37.0));
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod table;
mod time;

pub use table::{
    LEAP_SECONDS, LeapSecond, TABLE_COVERS_FROM_MJD, TAI_MINUS_UTC_AFTER_LAST_ENTRY, mjd_from_ymd,
};
pub use time::{
    MJD_AT_UNIX_EPOCH, calendar_mjd, tai_minus_utc_at_mjd, tai_mjd_from_utc_mjd,
    utc_mjd_from_tai_mjd,
};
