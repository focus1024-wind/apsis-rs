//! Conversions between UTC and TAI expressed on the Modified Julian Date
//! scale.
//!
//! 在约化儒略日尺度上表达的 UTC 与 TAI 之间的换算。

use crate::table::{
    LEAP_SECONDS, TABLE_COVERS_FROM_MJD, TAI_MINUS_UTC_AFTER_LAST_ENTRY, mjd_from_ymd,
};

/// The Modified Julian Date of the Unix epoch, 1970-01-01T00:00:00.
///
/// MJD carries no time scale of its own, so this constant is the same
/// number whether the instant is read in UTC, TAI, or any other scale.
///
/// Unix 历元的修正儒略日（Modified Julian Date），1970-01-01T00:00:00。
///
/// MJD 本身不携带时间标度，故无论该瞬时按 UTC、TAI 还是其他标度读取，此常量
/// 都是同一个数。
pub const MJD_AT_UNIX_EPOCH: f64 = 40_587.0;

/// The number of seconds in a day.
///
/// 一天中的秒数。
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
///
/// 返回给定 UTC 修正儒略日（Modified Julian Date）处 `TAI - UTC` 的秒值。
///
/// 该偏移是阶跃函数：它在每个闰秒生效日的 00:00:00 UTC 改变，其间保持不变。
/// 恰在生效日上的查询返回新值。
///
/// 当该瞬时早于 1972-01-01 时返回 `None`，因为那之前 UTC 采用小数偏移，本表
/// 不适用。
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
///
/// 把 UTC 修正儒略日（Modified Julian Date）转换为对应的 TAI 修正儒略日。
///
/// 换算加上该瞬时生效的偏移。由于偏移是在 UTC 侧求值的阶跃函数，正向换算对每个
/// UTC 瞬时都有定义；反向换算则不然，因为闰秒使某些 TAI 瞬时不对应任何 UTC
/// 瞬时。
///
/// 当该瞬时早于 1972-01-01 时返回 `None`。
pub fn tai_mjd_from_utc_mjd(utc_mjd: f64) -> Option<f64> {
    let offset = tai_minus_utc_at_mjd(utc_mjd)?;
    Some(utc_mjd + offset / SECONDS_PER_DAY)
}

/// Converts a TAI Modified Julian Date to the corresponding UTC Modified
/// Julian Date.
///
/// Each candidate offset from the table is tried in turn. A candidate is
/// accepted when the offset in force at the resulting UTC instant is the
/// same offset that produced it. At most one offset can satisfy that
/// condition for a given input, so the search is unambiguous.
///
/// Returns `None` in two cases. The first is an instant before the start of
/// the table, where UTC carried fractional offsets that this table does not
/// describe. The second is an instant that falls strictly inside an
/// inserted leap second. Such an instant has no UTC representation, because
/// the UTC clock labels that interval `23:59:60`, a value that does not
/// exist on the continuous Modified Julian Date scale.
///
/// 把 TAI 修正儒略日（Modified Julian Date）转换为对应的 UTC 修正儒略日。
///
/// 表中每个候选偏移依次尝试；当所得 UTC 瞬时处生效的偏移与产生它的偏移相同时，
/// 该候选被接受。对给定输入至多有一个偏移满足该条件，故搜索无歧义。
///
/// 两种情形返回 `None`。其一是早于表中起点的瞬时，那里的 UTC 采用小数偏移，本表
/// 未予描述。其二是严格落在所插入闰秒内部的瞬时，此类瞬时没有 UTC 表示，因为
/// UTC 时钟把该区间标记为 `23:59:60`，而该值在连续的修正儒略日标度上并不存在。
pub fn utc_mjd_from_tai_mjd(tai_mjd: f64) -> Option<f64> {
    if !tai_mjd.is_finite() {
        return None;
    }
    for entry in LEAP_SECONDS {
        let offset = entry.tai_minus_utc;
        let candidate = tai_mjd - offset / SECONDS_PER_DAY;
        if tai_minus_utc_at_mjd(candidate) == Some(offset) {
            return Some(candidate);
        }
    }
    None
}

/// Converts a proleptic Gregorian calendar date at 00:00:00 to a Modified
/// Julian Date.
///
/// This is a convenience re-export of the calendar conversion used to place
/// the leap second table entries. It is provided so that callers do not
/// have to reproduce the conversion, and because the calibration scripts in
/// `validation/` need the same function on both sides of a comparison.
///
/// 把前推格里历日期在 00:00:00 的瞬时转换为修正儒略日（Modified Julian Date）。
///
/// 这是用于定位闰秒表条目的日历换算之便捷再导出。提供它是为了免去调用方重复
/// 实现该换算，也因为 `validation/` 中的标定脚本在比较的两侧都需要同一函数。
pub const fn calendar_mjd(year: i32, month: u32, day: u32) -> f64 {
    mjd_from_ymd(year, month, day)
}
