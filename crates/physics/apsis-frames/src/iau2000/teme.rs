//! The True Equator Mean Equinox frame and its transformation to the
//! Earth-fixed frame.
//!
//! The SGP4 propagator produces its state in the True Equator Mean Equinox
//! (TEME) frame. TEME is the true equator of date, so its third axis is the
//! true celestial pole and it carries the nutation, but its first axis
//! points at the mean equinox rather than at the true equinox.
//!
//! Because the first axis is the mean equinox, the transformation to the
//! Earth-fixed frame uses Greenwich mean sidereal time, not the apparent
//! value. That is the detail that is easy to get wrong: subtracting the
//! equation of the equinoxes as well, as one would for a true-equinox frame,
//! rotates the result by about a thousandth of a degree, which is a
//! kilometre of ground position for a low Earth satellite.
//!
//! The time is therefore the UT1 Julian date, from which the mean sidereal
//! time follows, and the polar motion is applied last.
//!
//! Source: the relation between TEME and the Earth-fixed frame is given in
//! Vallado, D. A., *Fundamentals of Astrodynamics and Applications*, 4th
//! edition, microcosm Press, 2013, section 3.7. The sidereal time
//! expression is the one used by the SGP4 reference implementation, as
//! given in the same work and implemented by `eraGmst82`.

use super::terrestrial::PolarMotion;
use apsis_math::Mat3;
use core::f64::consts::PI;

/// The number of seconds in a day.
const SECONDS_PER_DAY: f64 = 86_400.0;

/// Converts seconds of time to radians.
///
/// A full turn is 86 400 seconds of time, so one second of time is
/// `2 pi / 86400` radians. Sidereal time is quoted in seconds of time, not in
/// arcseconds, and the two differ by a factor of fifteen.
const SECONDS_OF_TIME_TO_RADIAN: f64 = 2.0 * PI / SECONDS_PER_DAY;

/// Returns Greenwich mean sidereal time from the 1982 expression, in
/// radians.
///
/// The expression is a polynomial in the number of Julian centuries from
/// J2000.0 plus the fraction of the day in seconds of time. It is the
/// sidereal time the SGP4 theory was fitted against, which is why it is used
/// here rather than the modern expression: the element sets are consistent
/// with it.
///
/// The Julian date is taken in two parts, a whole number of days and a
/// fraction, because a single `f64` near 2 461 000 has a resolution of about
/// 40 microseconds and the sidereal time is wanted to better than that. The
/// caller that has only one value may pass the whole part and the fraction
/// separately, or pass a value twice: `(jd, 0.0)`, `(2400000.5, mjd)`, and
/// `(jd - 0.5, 0.5)` are all accepted, but the split with the smaller whole
/// part preserves more of the fraction.
///
/// Source: as implemented by `eraGmst82`.
pub fn greenwich_mean_sidereal_time_1982(day: f64, fraction: f64) -> f64 {
    // The constant absorbs the half day between the start of the Julian
    // date and the start of the day.
    let a = 24_110.548_41 - SECONDS_PER_DAY / 2.0;
    let b = 8_640_184.812_866;
    let c = 0.093_104;
    let d = -6.2e-6;

    // The reference orders the two parts so that the smaller is the first,
    // which is the arrangement that preserves the most precision.
    let (first, second) = if day < fraction {
        (day, fraction)
    } else {
        (fraction, day)
    };

    let t = (first + (second - 2_451_545.0)) / 36_525.0;
    let seconds_of_time = SECONDS_PER_DAY * (first.rem_euclid(1.0) + second.rem_euclid(1.0));
    let seconds = (a + (b + (c + d * t) * t) * t) + seconds_of_time;
    (seconds * SECONDS_OF_TIME_TO_RADIAN).rem_euclid(2.0 * PI)
}

/// Returns the rotation from the True Equator Mean Equinox frame to the
/// Earth-fixed frame.
///
/// The argument is the UT1 Julian date, because the rotation is the Earth's
/// rotation, and the polar motion is supplied by the caller because it is
/// measured rather than modelled.
///
/// This is the transformation to apply to the position and velocity that
/// `apsis-sgp4` produces.
pub fn teme_to_terrestrial_matrix(
    ut1_day: f64,
    ut1_fraction: f64,
    polar_motion: PolarMotion,
) -> Mat3 {
    let sidereal = greenwich_mean_sidereal_time_1982(ut1_day, ut1_fraction);
    // The composition applies the Earth rotation first and the polar motion
    // last, with each new rotation on the left of the accumulator.
    polar_motion.matrix() * rotation_z(sidereal)
}

/// Returns a rotation of `angle` about the third axis, in radians.
///
/// The convention is the one the reference helpers use: the rotation is
/// applied to the frame, so composing `a * b` applies `b` first.
fn rotation_z(angle: f64) -> Mat3 {
    super::precession::rotation_z(angle)
}
