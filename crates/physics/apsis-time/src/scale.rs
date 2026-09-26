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

use core::fmt;

/// Terrestrial Time minus International Atomic Time, in seconds.
///
/// Exact by the definition of Terrestrial Time.
pub const TT_MINUS_TAI: f64 = 32.184;

/// International Atomic Time minus GPS time, in seconds.
///
/// Exact by the definition of GPS time.
pub const TAI_MINUS_GPS: f64 = 19.0;

/// A time scale.
///
/// The scale determines how a numeric instant is interpreted. It does not
/// change the instant itself; converting an [`Epoch`](crate::Epoch) between
/// scales only changes the number used to display it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TimeScale {
    /// Coordinated Universal Time, adjusted by leap seconds.
    Utc,
    /// International Atomic Time, a uniform scale.
    Tai,
    /// Terrestrial Time, a uniform scale defined for geocentric work.
    Tt,
    /// GPS time, a uniform scale maintained by the GPS constellation.
    Gps,
}

impl TimeScale {
    /// Returns `true` when the scale runs at a uniform rate.
    ///
    /// UTC is the only scale here that is not uniform, because leap seconds
    /// make some of its minutes 61 seconds long.
    pub const fn is_uniform(self) -> bool {
        !matches!(self, Self::Utc)
    }

    /// Returns the short name of the scale, such as `TAI` or `UTC`.
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
