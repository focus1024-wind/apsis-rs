//! The compile time leap second table and the calendar helpers that place
//! its entries on the Modified Julian Date scale.
//!
//! 编译期闰秒表，以及把其各项放到约化儒略日尺度上的日历辅助函数。

/// One leap second announcement.
///
/// The entry records the date on which a new value of `TAI - UTC` took
/// effect. It does not record the size of the inserted second, because in
/// every announcement to date the inserted second was exactly one second
/// and the new offset is one greater than the previous one.
///
/// 一条闰秒公告。
///
/// 该条目记录 `TAI - UTC` 新值生效的日期。它不记录所插入闰秒的长度，因为迄今
/// 每一次公告插入的闰秒都恰为一秒，且新偏移比前一个大一。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LeapSecond {
    /// Year of the effective date, in the proleptic Gregorian calendar.
    ///
    /// 生效日期的年，前推格里历。
    pub year: i32,
    /// Month of the effective date, in `1..=12`.
    ///
    /// 生效日期的月，取值 `1..=12`。
    pub month: u32,
    /// Day of the effective date, in `1..=31`.
    ///
    /// 生效日期的日，取值 `1..=31`。
    pub day: u32,
    /// The value of `TAI - UTC` in seconds from this date onward.
    ///
    /// 自该日起生效的 `TAI - UTC` 值，单位为秒。
    pub tai_minus_utc: f64,
}

impl LeapSecond {
    /// Returns the Modified Julian Date of the effective instant.
    ///
    /// Leap seconds take effect at 00:00:00 UTC, so the effective instant
    /// is the start of the given calendar day.
    ///
    /// 返回生效瞬时的修正儒略日（Modified Julian Date）。
    ///
    /// 闰秒在 00:00:00 UTC 生效，故生效瞬时即该日历日的起点。
    #[must_use]
    pub const fn effective_mjd(self) -> f64 {
        mjd_from_ymd(self.year, self.month, self.day)
    }
}

/// The leap second table, ordered by effective date.
///
/// Source: IERS Bulletin C. The first entry is 1972-01-01, when the
/// current scheme began with an initial offset of 10 s inherited from
/// earlier adjustments.
///
/// 闰秒表，按生效日期排序。
///
/// 出处：IERS Bulletin C。首条为 1972-01-01，即现行方案开始之时，其初始偏移
/// 10 s 承自更早的调整。
pub const LEAP_SECONDS: &[LeapSecond] = &[
    LeapSecond {
        year: 1972,
        month: 1,
        day: 1,
        tai_minus_utc: 10.0,
    },
    LeapSecond {
        year: 1972,
        month: 7,
        day: 1,
        tai_minus_utc: 11.0,
    },
    LeapSecond {
        year: 1973,
        month: 1,
        day: 1,
        tai_minus_utc: 12.0,
    },
    LeapSecond {
        year: 1974,
        month: 1,
        day: 1,
        tai_minus_utc: 13.0,
    },
    LeapSecond {
        year: 1975,
        month: 1,
        day: 1,
        tai_minus_utc: 14.0,
    },
    LeapSecond {
        year: 1976,
        month: 1,
        day: 1,
        tai_minus_utc: 15.0,
    },
    LeapSecond {
        year: 1977,
        month: 1,
        day: 1,
        tai_minus_utc: 16.0,
    },
    LeapSecond {
        year: 1978,
        month: 1,
        day: 1,
        tai_minus_utc: 17.0,
    },
    LeapSecond {
        year: 1979,
        month: 1,
        day: 1,
        tai_minus_utc: 18.0,
    },
    LeapSecond {
        year: 1980,
        month: 1,
        day: 1,
        tai_minus_utc: 19.0,
    },
    LeapSecond {
        year: 1981,
        month: 7,
        day: 1,
        tai_minus_utc: 20.0,
    },
    LeapSecond {
        year: 1982,
        month: 7,
        day: 1,
        tai_minus_utc: 21.0,
    },
    LeapSecond {
        year: 1983,
        month: 7,
        day: 1,
        tai_minus_utc: 22.0,
    },
    LeapSecond {
        year: 1985,
        month: 7,
        day: 1,
        tai_minus_utc: 23.0,
    },
    LeapSecond {
        year: 1988,
        month: 1,
        day: 1,
        tai_minus_utc: 24.0,
    },
    LeapSecond {
        year: 1990,
        month: 1,
        day: 1,
        tai_minus_utc: 25.0,
    },
    LeapSecond {
        year: 1991,
        month: 1,
        day: 1,
        tai_minus_utc: 26.0,
    },
    LeapSecond {
        year: 1992,
        month: 7,
        day: 1,
        tai_minus_utc: 27.0,
    },
    LeapSecond {
        year: 1993,
        month: 7,
        day: 1,
        tai_minus_utc: 28.0,
    },
    LeapSecond {
        year: 1994,
        month: 7,
        day: 1,
        tai_minus_utc: 29.0,
    },
    LeapSecond {
        year: 1996,
        month: 1,
        day: 1,
        tai_minus_utc: 30.0,
    },
    LeapSecond {
        year: 1997,
        month: 7,
        day: 1,
        tai_minus_utc: 31.0,
    },
    LeapSecond {
        year: 1999,
        month: 1,
        day: 1,
        tai_minus_utc: 32.0,
    },
    LeapSecond {
        year: 2006,
        month: 1,
        day: 1,
        tai_minus_utc: 33.0,
    },
    LeapSecond {
        year: 2009,
        month: 1,
        day: 1,
        tai_minus_utc: 34.0,
    },
    LeapSecond {
        year: 2012,
        month: 7,
        day: 1,
        tai_minus_utc: 35.0,
    },
    LeapSecond {
        year: 2015,
        month: 7,
        day: 1,
        tai_minus_utc: 36.0,
    },
    LeapSecond {
        year: 2017,
        month: 1,
        day: 1,
        tai_minus_utc: 37.0,
    },
];

/// The Modified Julian Date of the first table entry, 1972-01-01.
///
/// Queries earlier than this instant are rejected because UTC did not yet
/// use integer second offsets.
///
/// 表中首条的修正儒略日（Modified Julian Date），1972-01-01。
///
/// 早于该瞬时的查询会被拒绝，因为那时的 UTC 尚未采用整数秒偏移。
pub const TABLE_COVERS_FROM_MJD: f64 = mjd_from_ymd(
    LEAP_SECONDS[0].year,
    LEAP_SECONDS[0].month,
    LEAP_SECONDS[0].day,
);

/// The value of `TAI - UTC` on and after the last table entry.
///
/// This is the offset that remains in force until a new leap second is
/// announced.
///
/// 自表中末条起生效的 `TAI - UTC` 值。
///
/// 在新的闰秒公布之前，该偏移持续有效。
pub const TAI_MINUS_UTC_AFTER_LAST_ENTRY: f64 = LEAP_SECONDS[LEAP_SECONDS.len() - 1].tai_minus_utc;

/// Converts a proleptic Gregorian calendar date to a Modified Julian Date.
///
/// The result is the MJD of 00:00:00 on the given day. The algorithm is the
/// Fliegel-Van Flandern integer conversion, which is exact for every date
/// the Gregorian calendar is defined on and does not need a lookup table.
///
/// The computation is done in `i64` and converted once at the end, so the
/// result is exact for the range of dates in which a leap second can
/// plausibly occur.
///
/// 把前推格里历日期转换为修正儒略日（Modified Julian Date）。
///
/// 结果是给定日期 00:00:00 的 MJD。算法采用 Fliegel-Van Flandern 整数换算，
/// 对格里历定义范围内的每个日期都精确，且无需查表。
///
/// 计算在 `i64` 中进行并仅在末尾换算一次，故在闰秒可能发生的日期范围内结果
/// 精确。
#[must_use]
pub const fn mjd_from_ymd(year: i32, month: u32, day: u32) -> f64 {
    let a = (14 - month as i64) / 12;
    let y = year as i64 + 4800 - a;
    let m = month as i64 + 12 * a - 3;
    let jdn = day as i64 + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400 - 32045;
    // The Julian Day Number counts days from noon, and MJD 0 is the
    // midnight starting 1858-11-17, whose Julian Day Number is 2 400 001.
    // 儒略日数从正午起计，而 MJD 0 是始于 1858-11-17 的午夜，其儒略日数为
    // 2 400 001。
    (jdn - 2_400_001) as f64
}
