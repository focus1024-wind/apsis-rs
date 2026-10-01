//! Conversions between Modified Julian Date and Julian Date.
//!
//! A Julian Date counts days from noon on 4713-01-01 in the proleptic
//! Julian calendar. A Modified Julian Date counts days from midnight on
//! 1858-11-17. The two therefore differ by a fixed number of days, and the
//! half day offset accounts for the different times of day at which each
//! scale starts a day.
//!
//! The offset is exact, so these conversions introduce no rounding beyond
//! the representation of the input.

/// The number of days between the start of the Julian Date scale and the
/// start of the Modified Julian Date scale.
///
/// 儒略日标度起点与修正儒略日标度起点之间的天数。
pub const JULIAN_DATE_OFFSET: f64 = 2_400_000.5;

/// Converts a Julian Date to a Modified Julian Date.
///
/// 把儒略日（Julian Date）转换为修正儒略日（Modified Julian Date）。
pub const fn mjd_from_julian_date(julian_date: f64) -> f64 {
    julian_date - JULIAN_DATE_OFFSET
}

/// Converts a Modified Julian Date to a Julian Date.
///
/// 把修正儒略日（Modified Julian Date）转换为儒略日（Julian Date）。
pub const fn julian_date_from_mjd(mjd: f64) -> f64 {
    mjd + JULIAN_DATE_OFFSET
}
