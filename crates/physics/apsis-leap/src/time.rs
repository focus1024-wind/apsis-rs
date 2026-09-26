//! Conversions between UTC and TAI expressed on the Modified Julian Date
//! scale.

use crate::table::{
    LEAP_SECONDS, TABLE_COVERS_FROM_MJD, TAI_MINUS_UTC_AFTER_LAST_ENTRY, mjd_from_ymd,
};

/// The Modified Julian Date of the Unix epoch, 1970-01-01T00:00:00.
///
/// MJD carries no time scale of its own, so this constant is the same
/// number whether the instant is read in UTC, TAI, or any other scale.
pub const MJD_AT_UNIX_EPOCH: f64 = 40_587.0;

/// The number of seconds in a day.
const SECONDS_PER_DAY: f64 = 86_400.0;

/// Returns the value of `TAI - UTC` in seconds at the given UTC Modified
/// Julian Date.
///
/// The offset is a step function: it changes at 00:00:00 UTC on the
/// effective date of each leap second and is constant in between. A query
/// exactly on an effective date returns the new value.
///
/// Returns `None` when the instant precedes 1972-01-01, before which UTC
/// used fractional offsets and this table does not apply.
pub fn tai_minus_utc_at_mjd(utc_mjd: f64) -> Option<f64> {
    if !utc_mjd.is_finite() || utc_mjd < TABLE_COVERS_FROM_MJD {
        return None;
    }
    let mut offset = TAI_MINUS_UTC_AFTER_LAST_ENTRY;
    for entry in LEAP_SECONDS {
        if utc_mjd >= entry.effective_mjd() {
            offset = entry.tai_minus_utc;
        } else {
            break;
        }
    }
    Some(offset)
}

/// Converts a UTC Modified Julian Date to the corresponding TAI Modified
/// Julian Date.
///
/// The conversion adds the offset in force at that instant. Because the
/// offset is a step function evaluated on the UTC side, the conversion is
/// defined for every UTC instant; the inverse conversion is not, because a
/// leap second makes some TAI instants correspond to no UTC instant.
///
/// Returns `None` when the instant precedes 1972-01-01.
pub fn tai_mjd_from_utc_mjd(utc_mjd: f64) -> Option<f64> {
    let offset = tai_minus_utc_at_mjd(utc_mjd)?;
    Some(utc_mjd + offset / SECONDS_PER_DAY)
}

/// Converts a TAI Modified Julian Date to the corresponding UTC Modified
/// Julian Date.
///
/// The offset that applies depends on the UTC instant being sought, so the
/// value is recovered by a monotone iteration. The iteration starts from
/// the smallest possible answer, obtained with the largest offset in the
/// table, and increases until the offset stops changing. Because the offset
/// is a non-decreasing step function of UTC, the sequence is non-decreasing
/// and reaches the fixed point in as many steps as there are offset changes
/// between the seed and the answer, which is a small number.
///
/// Returns `None` when the instant precedes the start of the table, or when
/// the result would fall before that boundary.
pub fn utc_mjd_from_tai_mjd(tai_mjd: f64) -> Option<f64> {
    if !tai_mjd.is_finite() {
        return None;
    }
    // Seed with the largest offset, which gives the smallest candidate UTC.
    let mut utc_mjd = tai_mjd - TAI_MINUS_UTC_AFTER_LAST_ENTRY / SECONDS_PER_DAY;
    for _ in 0..64 {
        let offset = offset_for_iteration(utc_mjd);
        let next = tai_mjd - offset / SECONDS_PER_DAY;
        if next == utc_mjd {
            break;
        }
        utc_mjd = next;
    }
    if utc_mjd < TABLE_COVERS_FROM_MJD {
        return None;
    }
    Some(utc_mjd)
}

/// Returns the offset in force near `utc_mjd`, using the first table entry
/// for instants below the start of the table.
///
/// This is used only while iterating in [`utc_mjd_from_tai_mjd`], where a
/// candidate may transiently precede the boundary even though the answer
/// does not. Callers that need to distinguish an out of range instant from
/// a valid one must use [`tai_minus_utc_at_mjd`].
fn offset_for_iteration(utc_mjd: f64) -> f64 {
    tai_minus_utc_at_mjd(utc_mjd).unwrap_or(LEAP_SECONDS[0].tai_minus_utc)
}

/// Converts a proleptic Gregorian calendar date at 00:00:00 to a Modified
/// Julian Date.
///
/// This is a convenience re-export of the calendar conversion used to place
/// the leap second table entries. It is provided so that callers do not
/// have to reproduce the conversion, and because the calibration scripts in
/// `validation/` need the same function on both sides of a comparison.
pub const fn calendar_mjd(year: i32, month: u32, day: u32) -> f64 {
    mjd_from_ymd(year, month, day)
}
