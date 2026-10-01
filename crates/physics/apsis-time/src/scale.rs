//! Time scales and the fixed offsets between them.
//!
//! Four scales are represented. TAI, TT, and GPS are uniform: one second in
//! each is one SI second, and they differ only by constant offsets fixed by
//! definition. UTC is not uniform, because it is adjusted by leap seconds to
//! stay near the rotation of the Earth.
//!
//! The offsets between the uniform scales are exact:
//!
//! - `TT = TAI + 32.184 s`, by the definition of Terrestrial Time.
//! - `TAI = GPS + 19 s`, by the definition of GPS time, whose origin was
//!   set to coincide with UTC at 1980-01-06.
//!
//! Source: IAU 2000 Resolution B1.9 for Terrestrial Time; the GPS interface
//! specification for GPS time. Neither relation has changed since those
//! definitions took effect, so both are treated as constants here.
//!
//! 时标及它们之间的固定偏移。
//!
//! 表示四种时标。TAI、TT 与 GPS 是均匀的：其中一秒即一个 SI 秒，它们只相差
//! 由定义固定的常值偏移。UTC 不均匀，因为它由闰秒调整以保持接近地球的转动。
//!
//! 均匀时标之间的偏移是精确的：
//!
//! - `TT = TAI + 32.184 s`，由地球时的定义。
//! - `TAI = GPS + 19 s`，由 GPS 时的定义，其原点被设为与 1980-01-06 的 UTC
//!   重合。
//!
//! 出处：地球时见 IAU 2000 Resolution B1.9；GPS 时见 GPS 接口规范。自这些
//! 定义生效以来两个关系都未改变，故此处二者都被当作常量。

use core::fmt;

/// Terrestrial Time minus International Atomic Time, in seconds.
///
/// Exact by the definition of Terrestrial Time.
///
/// 地球时（TT）减国际原子时（TAI），单位为秒。
///
/// 由地球时的定义精确确定。
pub const TT_MINUS_TAI: f64 = 32.184;

/// International Atomic Time minus GPS time, in seconds.
///
/// Exact by the definition of GPS time.
///
/// 国际原子时（TAI）减 GPS 时，单位为秒。
///
/// 由 GPS 时的定义精确确定。
pub const TAI_MINUS_GPS: f64 = 19.0;

/// A time scale.
///
/// The scale determines how a numeric instant is interpreted. It does not
/// change the instant itself; converting an [`Epoch`](crate::Epoch) between
/// scales only changes the number used to display it.
///
/// 时间尺度。
///
/// 标度决定一个数值瞬时如何被解释，但并不改变瞬时本身；把
/// [`Epoch`](crate::Epoch) 在标度之间换算，只改变用于显示它的数值。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TimeScale {
    /// Coordinated Universal Time, adjusted by leap seconds.
    ///
    /// 世界协调时，由闰秒调整。
    Utc,
    /// International Atomic Time, a uniform scale.
    ///
    /// 国际原子时，均匀标度。
    Tai,
    /// Terrestrial Time, a uniform scale defined for geocentric work.
    ///
    /// 地球时，为地心工作定义的均匀标度。
    Tt,
    /// GPS time, a uniform scale maintained by the GPS constellation.
    ///
    /// GPS 时，由 GPS 星座维持的均匀标度。
    Gps,
}

impl TimeScale {
    /// Returns `true` when the scale runs at a uniform rate.
    ///
    /// UTC is the only scale here that is not uniform, because leap seconds
    /// make some of its minutes 61 seconds long.
    ///
    /// 当该标度以均匀速率运行时返回 `true`。
    ///
    /// 此处只有 UTC 不是均匀的，因为闰秒使其部分分钟长达 61 秒。
    pub const fn is_uniform(self) -> bool {
        !matches!(self, Self::Utc)
    }

    /// Returns the short name of the scale, such as `TAI` or `UTC`.
    ///
    /// 返回该标度的简称，例如 `TAI` 或 `UTC`。
    pub const fn name(self) -> &'static str {
        match self {
            Self::Utc => "UTC",
            Self::Tai => "TAI",
            Self::Tt => "TT",
            Self::Gps => "GPS",
        }
    }
}

impl fmt::Display for TimeScale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}
