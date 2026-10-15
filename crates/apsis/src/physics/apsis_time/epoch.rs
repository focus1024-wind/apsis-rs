//! The instant type and its scale conversions.
//!
//! An [`Epoch`] stores an instant as a Modified Julian Date in International
//! Atomic Time. TAI is chosen as the internal representation because it is
//! uniform and because every other scale in this module is either a constant
//! offset from it or, for UTC, a step offset that can be looked up.
//!
//! Storing the unified scale makes comparison and differencing between two
//! epochs exact regardless of the scales they were created in. The scale is
//! therefore not part of an epoch's identity; it is a parameter of the
//! conversion methods that read or write a number.
//!
//! 瞬时类型及其时标换算。
//!
//! [`Epoch`] 把瞬时存为国际原子时中的约化儒略日。选用 TAI 作为内部表示，
//! 因为它均匀，且本模块中其余每个时标要么是相对它的常值偏移，要么（对
//! UTC 而言）是可查表的阶跃偏移。
//!
//! 存储统一时标使两个历元之间的比较与求差精确，与它们创建时所处的时标无关。
//! 因此时标不属于历元的身份；它是读取或写入某数的各换算方法的参数。

use crate::apsis_leap::{tai_minus_utc_at_mjd, utc_mjd_from_tai_mjd};
use crate::apsis_time::calendar::{
    Calendar, SECONDS_PER_DAY, mjd_from_ymd, seconds_of_day_from_mjd, ymd_from_mjd,
};
use crate::apsis_time::scale::{TAI_MINUS_GPS, TT_MINUS_TAI, TimeScale};
use core::fmt;

/// An instant on the TAI time scale, stored as a Modified Julian Date.
///
/// The field is private so that the stored scale is guaranteed. Use
/// [`Epoch::from_mjd`] or the named constructors to build one.
///
/// TAI 时间尺度上的一个瞬时，以修正儒略日（Modified Julian Date）存储。
///
/// 字段私有以保证所存标度确定。用 [`Epoch::from_mjd`] 或具名构造函数创建。
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Epoch {
    tai_mjd: f64,
}

impl Epoch {
    /// The instant 2000-01-01T12:00:00 TT, which defines the J2000.0 epoch.
    ///
    /// The value is `2451545.0` on the TT scale, offset back to TAI.
    ///
    /// 瞬时 2000-01-01T12:00:00 TT，定义了 J2000.0 历元。
    ///
    /// 该值在 TT 标度上为 `2451545.0`，并回退到 TAI。
    pub const J2000: Self = Self::from_tai_mjd_const(51_544.5 - TT_MINUS_TAI / SECONDS_PER_DAY);

    /// Creates an epoch from a Modified Julian Date on the TAI scale.
    ///
    /// 由 TAI 标度上的修正儒略日创建历元。
    #[must_use]
    pub const fn from_tai_mjd(tai_mjd: f64) -> Self {
        Self { tai_mjd }
    }

    /// Const constructor used by the associated constants.
    ///
    /// 供各关联常量使用的 const 构造函数。
    const fn from_tai_mjd_const(tai_mjd: f64) -> Self {
        Self { tai_mjd }
    }

    /// Creates an epoch from a Modified Julian Date on the given scale.
    ///
    /// Returns `None` when the instant cannot be expressed on the requested
    /// scale. This happens for UTC before 1972, where the leap second table
    /// used to convert from UTC does not apply.
    ///
    /// 由给定标度上的修正儒略日（Modified Julian Date）创建历元。
    ///
    /// 当该瞬时无法在所请求的标度上表达时返回 `None`。这发生在 1972 年之前的
    /// UTC，因为用于从 UTC 换算的闰秒表不适用。
    #[must_use]
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
    ///
    /// 由日历日期创建历元，并按给定标度解释该日期。
    ///
    /// 当各字段不构成有效日期，或该瞬时无法在所请求的标度上表达时返回 `None`。
    #[must_use]
    pub fn from_calendar(calendar: Calendar, scale: TimeScale) -> Option<Self> {
        if !calendar.is_valid() {
            return None;
        }
        let mjd = mjd_from_ymd(calendar.year, calendar.month, calendar.day)
            + calendar.seconds_of_day() / SECONDS_PER_DAY;
        Self::from_mjd(mjd, scale)
    }

    /// Returns the instant as a Modified Julian Date on the TAI scale.
    ///
    /// 返回该瞬时在 TAI 标度上的修正儒略日（Modified Julian Date）。
    #[must_use]
    pub const fn tai_mjd(self) -> f64 {
        self.tai_mjd
    }

    /// Returns the instant as a Modified Julian Date on the given scale.
    ///
    /// Returns `None` when the instant precedes 1972 and the UTC scale is
    /// requested, because the leap second table does not cover that range.
    ///
    /// 返回该瞬时在给定标度上的修正儒略日（Modified Julian Date）。
    ///
    /// 当该瞬时早于 1972 年且请求 UTC 标度时返回 `None`，因为闰秒表不覆盖该
    /// 区间。
    #[must_use]
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
    ///
    /// 返回该瞬时在给定标度上的日历日期。
    ///
    /// 当该瞬时不可进行标度换算时返回 `None`；参见 [`Epoch::mjd`]。
    #[must_use]
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
    ///
    /// 返回从 `self` 到 `other` 的带符号秒数。
    ///
    /// 当 `other` 更晚时结果为正。由于两个历元都存于均匀的 TAI 标度，差值
    /// 精确，且与创建二者时所用的标度无关。
    #[must_use]
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
    ///
    /// 返回平移 `seconds` 后的历元。
    ///
    /// 平移量以均匀 TAI 标度上的 SI 秒计，故瞬时恰好前进相同的流逝时间。这是
    /// 唯一提供的平移方式：若按 UTC 时钟平移，当区间跨越插入的闰秒时结果会
    /// 不同，而 UTC 时钟在该瞬时前后没有单一良定义的数值。需要按 UTC 标度做
    /// 日历运算的调用方，应先转为 [`Calendar`]、调整后再转回。
    #[must_use]
    pub fn add_seconds(self, seconds: f64) -> Self {
        Self {
            tai_mjd: self.tai_mjd + seconds / SECONDS_PER_DAY,
        }
    }

    /// Returns `true` when the stored Modified Julian Date is finite.
    ///
    /// 当所存修正儒略日（Modified Julian Date）有限时返回 `true`。
    #[must_use]
    pub const fn is_finite(self) -> bool {
        self.tai_mjd.is_finite()
    }
}

impl fmt::Display for Epoch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} TAI", self.tai_mjd)
    }
}
