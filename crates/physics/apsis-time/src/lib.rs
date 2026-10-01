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
//!
//! 时标、历元与日历换算。
//!
//! 本 crate 提供工作区其余部分所用的时间模型。[`Epoch`] 是以国际原子时时标
//! 上的约化儒略日存储的瞬时，[`TimeScale`] 指名某数所表达的时标。
//!
//! 存储单一的均匀时标使算术精确：两个历元之差即它们之间流逝的 SI 时间，
//! 与它们创建时所处的时标无关。UTC 不均匀，因为插入闰秒以使其接近地球的
//! 转动，故 UTC 换算经由 `apsis-leap` 中的表格。
//!
//! 出处：
//!
//! - 儒略日与约化儒略日约定：IAU，如 *Explanatory Supplement to the
//!   Astronomical Almanac* 所述。
//! - 地球时相对 TAI 的偏移：IAU 2000 Resolution B1.9。
//! - 闰秒：IERS Bulletin C，由 `apsis-leap` 内嵌。
//! - 日历算法：Howard Hinnant，*chrono-Compatible Low-Level Date
//!   Algorithms*，2015。

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
