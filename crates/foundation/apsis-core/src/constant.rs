//! Physical and astronomical constants used across the workspace.
//!
//! All values are expressed in SI units unless the name or doc comment
//! states otherwise. Values that are fixed by definition are given exactly;
//! measured values carry the number of significant digits justified by the
//! public source.
//!
//! Sources:
//!
//! - SI defining constants: BIPM, *The International System of Units (SI)*,
//!   9th edition, 2019.
//! - CODATA 2018 recommended values: E. Tiesinga et al., *CODATA
//!   recommended values of the fundamental physical constants: 2018*,
//!   Rev. Mod. Phys. 93, 025010, 2021.
//! - Earth gravitational parameter and equatorial radius: WGS 84,
//!   NIMA TR8350.2, 2000.
//! - Astronomical unit: IAU 2012 Resolution B2.
//!
//! Where a value is defined exactly by a standard, the doc comment says so,
//! because several derived quantities depend on the exact value rather than
//! the rounded printed one.

/// Speed of light in vacuum, in meters per second.
///
/// Exact by definition of the metre in the SI.
pub const SPEED_OF_LIGHT: f64 = 299_792_458.0;

/// Newtonian constant of gravitation, in cubic meters per kilogram per
/// second squared.
///
/// CODATA 2018 value with its standard uncertainty of
/// `1.5e-11 m^3 kg^-1 s^-2`.
pub const GRAVITATIONAL_CONSTANT: f64 = 6.674_30e-11;

/// Boltzmann constant, in joules per kelvin.
///
/// Exact by definition of the kelvin in the SI.
pub const BOLTZMANN: f64 = 1.380_649e-23;

/// Planck constant, in joule seconds.
///
/// Exact by definition of the kilogram in the SI.
pub const PLANCK: f64 = 6.626_070_15e-34;

/// Elementary charge, in coulombs.
///
/// Exact by definition of the ampere in the SI.
pub const ELEMENTARY_CHARGE: f64 = 1.602_176_634e-19;

/// Astronomical unit, in meters.
///
/// Exact by IAU 2012 Resolution B2.
pub const ASTRONOMICAL_UNIT: f64 = 149_597_870_700.0;

/// Standard gravity, in meters per second squared.
///
/// Exact by definition of the standard atmosphere.
pub const STANDARD_GRAVITY: f64 = 9.806_65;

/// One minute, in seconds.
pub const MINUTE: f64 = 60.0;

/// One hour, in seconds.
pub const HOUR: f64 = 3_600.0;

/// One day of 24 hours, in seconds.
///
/// This is the SI day. It is not the mean solar day, which is slightly
/// longer, and not the Julian day of 86 400 seconds used in Julian date
/// arithmetic, which is identical to this value.
pub const DAY: f64 = 86_400.0;

/// One Julian year of 365.25 days, in seconds.
///
/// Exact by definition of the Julian year.
pub const JULIAN_YEAR: f64 = 31_557_600.0;

/// One Julian century of 36 525 days, in seconds.
///
/// Exact by definition of the Julian year, used by precession and secular
/// rate expressions quoted per Julian century.
pub const JULIAN_CENTURY: f64 = 3_155_760_000.0;

/// Julian date of the Unix epoch, 1970-01-01T00:00:00 UTC.
///
/// This is `2 440 587.5` in the Julian date system.
pub const JULIAN_DATE_UNIX_EPOCH: f64 = 2_440_587.5;

/// Julian date of the J2000.0 epoch, 2000-01-01T12:00:00 TT.
///
/// Exact by definition of the J2000.0 epoch.
pub const JULIAN_DATE_J2000: f64 = 2_451_545.0;

/// The number of seconds between the J2000.0 epoch and the Unix epoch.
///
/// Derived from [`JULIAN_DATE_J2000`] and [`JULIAN_DATE_UNIX_EPOCH`].
pub const SECONDS_J2000_TO_UNIX: f64 = (JULIAN_DATE_J2000 - JULIAN_DATE_UNIX_EPOCH) * DAY;

/// Earth gravitational parameter, in cubic kilometers per second squared.
///
/// WGS 84 value, `GM = 398 600.4418 km^3/s^2`, with an uncertainty that is
/// dominated by the uncertainty in the Newtonian constant of gravitation.
/// Orbit propagation uses kilometers, so the value is given in those units
/// rather than in SI.
pub const EARTH_MU: f64 = 398_600.441_8;

/// Earth equatorial radius, in kilometers.
///
/// WGS 84 semi-major axis, `a = 6 378.137 km`.
pub const EARTH_EQUATORIAL_RADIUS: f64 = 6_378.137;

/// Earth flattening, dimensionless.
///
/// WGS 84 value, `f = 1 / 298.257 223 563`.
pub const EARTH_FLATTENING: f64 = 1.0 / 298.257_223_563;

/// Earth rotation rate, in radians per second.
///
/// WGS 84 value. This is the nominal mean angular velocity of the Earth,
/// `omega = 7.292 115e-5 rad/s`.
pub const EARTH_ROTATION_RATE: f64 = 7.292_115_0e-5;

/// Earth second zonal harmonic, dimensionless.
///
/// WGS 84 value, `J2 = 1.082 626 683e-3`.
pub const EARTH_J2: f64 = 1.082_626_683e-3;

/// Standard atmosphere sea level pressure, in pascals.
///
/// Exact by definition of the standard atmosphere.
pub const STANDARD_ATMOSPHERE: f64 = 101_325.0;

/// Zero degrees Celsius on the thermodynamic temperature scale, in kelvin.
///
/// Exact by definition of the Celsius scale.
pub const ZERO_CELSIUS_IN_KELVIN: f64 = 273.15;
