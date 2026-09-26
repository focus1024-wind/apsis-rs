//! The celestial-to-terrestrial transformation.
//!
//! This module composes the frame bias, precession, nutation, Earth rotation
//! angle, and polar motion into the rotation that carries a vector from the
//! inertial frame to the Earth-fixed frame.
//!
//! Two intermediate frames appear in the composition. The **celestial
//! intermediate frame** is the true equator and equinox of date; the
//! rotation from the inertial frame to it is the bias, precession, and
//! nutation. The **terrestrial intermediate frame** is that frame carried to
//! the Earth's prime meridian by the Earth rotation angle. The polar motion
//! is applied last, because it is the motion of the pole within the Earth
//! rather than of the Earth within space.
//!
//! The elementary rotations follow the convention of the reference: `rz`
//! rotates the frame about its third axis, and composing `a * b` applies `b`
//! first. That convention is what makes the products below read in the order
//! the rotations are physically applied.
//!
//! Source: the composition follows the IAU SOFA library and its ERFA
//! derivative. The bias and precession are `eraPmat06`, the nutation matrix
//! is `eraNumat`, the rotation angle is `eraEra00`, the sidereal time for
//! the equinox-based composition is `eraGmst06`, and the polar motion is
//! `eraPom00`, composed by `eraC2t00b`.

use super::nutation::nutation_2000a_with_2006_adjustment;
use super::precession::{frame_to_mean_of_date, mean_obliquity, rotation_z};
use apsis_math::Mat3;
use core::f64::consts::PI;

/// Converts arcseconds to radians.
const ARCSECOND_TO_RADIAN: f64 = PI / (180.0 * 3_600.0);

/// Returns the bias-precession-nutation matrix for a time.
///
/// The matrix carries a vector from the celestial reference frame to the
/// true equator and equinox of date. It is built from the four precession
/// angles with the nutation added to the last two, which is the form the
/// reference uses.
///
/// Source: as implemented by `eraPnm06a` and `eraFw2m`.
pub fn bias_precession_nutation_matrix(t: f64) -> Mat3 {
    let angles = super::precession::precession_angles(t);
    let nutation = nutation_2000a_with_2006_adjustment(t);
    frame_to_mean_of_date(&super::precession::PrecessionAngles {
        gamma_bar: angles.gamma_bar,
        phi_bar: angles.phi_bar,
        psi_bar: angles.psi_bar + nutation.longitude,
        epsilon: angles.epsilon + nutation.obliquity,
    })
}

/// Returns the complementary terms of the equation of the equinoxes.
///
/// The equation of the equinoxes has, besides the projection of the nutation
/// in longitude onto the true equator, a set of small periodic terms from
/// the same theory. They reach about twice the constant term of the
/// osculating nutation, which is a millisecond of time, so they matter for a
/// sidereal time that is quoted to that precision.
///
/// Source: the MHB2000 complementary terms, as implemented by `eraEect00`.
pub fn equation_of_the_equinoxes_complementary_terms(t: f64) -> f64 {
    use super::eect_table::{CONSTANT_TERMS, LINEAR_TERMS};

    // The complementary terms use the IERS 2003 forms of two arguments,
    // which differ from the ones the nutation series was fitted with.
    let arguments = [
        super::fundamental::mean_anomaly_moon(t),
        super::fundamental::mean_anomaly_sun_iers2003(t),
        super::fundamental::mean_latitude_moon(t),
        super::fundamental::mean_elongation_moon_sun_iers2003(t),
        super::fundamental::mean_longitude_node(t),
        super::fundamental::mean_longitude_venus(t),
        super::fundamental::mean_longitude_earth(t),
        super::fundamental::general_precession(t),
    ];

    let mut constant = 0.0;
    for term in CONSTANT_TERMS.iter() {
        let argument: f64 = term
            .argument
            .iter()
            .zip(arguments.iter())
            .map(|(multiplier, value)| f64::from(*multiplier) * value)
            .sum();
        let (sine, cosine) = argument.sin_cos();
        constant += term.coefficients[0] * sine + term.coefficients[1] * cosine;
    }

    let mut linear = 0.0;
    for term in LINEAR_TERMS.iter() {
        let argument: f64 = term
            .argument
            .iter()
            .zip(arguments.iter())
            .map(|(multiplier, value)| f64::from(*multiplier) * value)
            .sum();
        let (sine, cosine) = argument.sin_cos();
        linear += term.coefficients[0] * sine + term.coefficients[1] * cosine;
    }

    // The tabulated coefficients are in arcseconds.
    (constant + linear * t) * ARCSECOND_TO_RADIAN
}

/// Returns the equation of the equinoxes for a time.
///
/// The equation of the equinoxes is the difference between the true and mean
/// equinoxes measured along the true equator. It is the quantity a sidereal
/// time must be corrected by to become the hour angle of the true equinox.
///
/// The value is the projection of the nutation in longitude onto the true
/// equator, `dpsi cos(eps)`, plus the complementary terms of the theory.
pub fn equation_of_the_equinoxes(t: f64) -> f64 {
    let nutation = nutation_2000a_with_2006_adjustment(t);
    let obliquity = mean_obliquity(t);
    nutation.longitude * obliquity.cos() + equation_of_the_equinoxes_complementary_terms(t)
}

/// The polar motion of the Earth, as two angles and the rotation.
///
/// The polar motion is the motion of the rotation pole within the Earth. It
/// is measured daily and published by the International Earth Rotation and
/// Reference Systems Service, so it cannot be computed from a model and is
/// supplied by the caller.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PolarMotion {
    /// The displacement of the pole along the Greenwich meridian, in
    /// radians.
    pub x: f64,
    /// The displacement of the pole along the meridian ninety degrees west,
    /// in radians.
    pub y: f64,
    /// The rotation of the terrestrial frame about the pole, in radians.
    ///
    /// This is the terrestrial intermediate origin locator, which is small
    /// and is often taken as zero.
    pub s: f64,
}

impl PolarMotion {
    /// The zero polar motion, which is the approximation used when the
    /// published values are not available.
    ///
    /// The error from omitting the polar motion is under a tenth of an
    /// arcsecond, well below the nutation but above the precision of a
    /// modern transformation.
    pub const NONE: Self = Self {
        x: 0.0,
        y: 0.0,
        s: 0.0,
    };

    /// Creates a polar motion from its three angles.
    pub const fn new(x: f64, y: f64, s: f64) -> Self {
        Self { x, y, s }
    }

    /// Returns `true` when the angles are usable.
    pub fn is_valid(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.s.is_finite()
    }

    /// Returns the rotation matrix of the polar motion.
    ///
    /// The rotation is a turn about the third axis by the locator, then
    /// about the second by minus the `x` displacement, then about the first
    /// by minus the `y` displacement. The signs are the ones the reference
    /// uses, which carry the terrestrial frame into alignment with the
    /// celestial pole.
    ///
    /// Source: as implemented by `eraPom00`, which applies the rotations in
    /// sequence by composing each on the left of the accumulator.
    pub fn matrix(self) -> Mat3 {
        rotation_x(-self.y) * rotation_y(-self.x) * rotation_z(self.s)
    }
}

/// Returns the intermediate-to-terrestrial matrix for a time.
///
/// The intermediate frame is the true equator and equinox of date, and the
/// terrestrial frame is the Earth-fixed frame at the same date. This uses
/// the equinox-based approach, in which the rotation from one to the other
/// is Greenwich apparent sidereal time.
///
/// The time arguments are the UT1 and TT Julian dates.
///
/// Source: the composition is the one `eraC2t06a` produces for the equinox
/// based chain, with the polar motion applied separately by the caller of
/// the full transformation.
pub fn intermediate_to_terrestrial_matrix(ut1_julian_date: f64, tt_julian_date: f64) -> Mat3 {
    let apparent_sidereal = greenwich_apparent_sidereal_time(ut1_julian_date, tt_julian_date);
    rotation_z(apparent_sidereal)
}

/// Returns Greenwich apparent sidereal time for a UT1 and a TT Julian date.
///
/// This is the mean sidereal time plus the equation of the equinoxes, so it
/// is the hour angle of the true equinox.
///
/// Source: as implemented by `eraGst06a`.
pub fn greenwich_apparent_sidereal_time(ut1_julian_date: f64, tt_julian_date: f64) -> f64 {
    let t = (tt_julian_date - 2_451_545.0) / 36_525.0;
    let mean = greenwich_mean_sidereal_time(ut1_julian_date, tt_julian_date);
    (mean + equation_of_the_equinoxes(t)).rem_euclid(2.0 * PI)
}

/// Returns Greenwich mean sidereal time for a UT1 and a TT Julian date.
///
/// The relation is the Earth rotation angle plus the accumulated precession
/// in right ascension, which is the quantity that relates the equinox to the
/// non-rotating origin.
///
/// Source: as implemented by `eraGmst06`.
pub fn greenwich_mean_sidereal_time(ut1_julian_date: f64, tt_julian_date: f64) -> f64 {
    let t = (tt_julian_date - 2_451_545.0) / 36_525.0;
    // The accumulated precession in right ascension, in arcseconds.
    let arcseconds = 0.014_506
        + t * (4_612.156_534
            + t * (1.391_581_7
                + t * (-0.000_000_44 + t * (-0.000_029_956 + t * (-0.000_000_036_8)))));
    (earth_rotation_angle(ut1_julian_date) + arcseconds * ARCSECOND_TO_RADIAN).rem_euclid(2.0 * PI)
}

/// Returns the Earth rotation angle for a UT1 Julian date.
///
/// This is the linear expression of the International Astronomical Union,
/// which is exact to the precision the angle is known.
///
/// Source: IERS Conventions (2010), equation 8.32.
pub fn earth_rotation_angle(ut1_julian_date: f64) -> f64 {
    let days = ut1_julian_date - 2_451_545.0;
    let turns = 0.779_057_273_264_0 + 1.002_737_811_911_354_6 * days;
    (turns.rem_euclid(1.0)) * 2.0 * PI
}

/// Returns the celestial-to-terrestrial matrix for a time.
///
/// This composes the whole chain: the frame bias, the precession, the
/// nutation, the Earth rotation, and the polar motion.
///
/// The time arguments are the UT1 and TT Julian dates, and the polar motion
/// is supplied by the caller because it is measured rather than modelled.
///
/// Source: the composition is the one `eraC2t06a` produces.
pub fn celestial_to_terrestrial_matrix(
    ut1_julian_date: f64,
    tt_julian_date: f64,
    polar_motion: PolarMotion,
) -> Mat3 {
    let t = (tt_julian_date - 2_451_545.0) / 36_525.0;
    let bias_precession_nutation = bias_precession_nutation_matrix(t);
    let sidereal = greenwich_apparent_sidereal_time(ut1_julian_date, tt_julian_date);
    // The composition is the one `eraC2tcio` produces: the bias, precession,
    // and nutation first, then the Earth rotation, then the polar motion,
    // with each new rotation composed on the left of the accumulator.
    polar_motion.matrix() * rotation_z(sidereal) * bias_precession_nutation
}

/// Returns a rotation of `angle` about the first axis, in radians.
///
/// The convention matches the reference helper `eraRx`: the rotation is
/// applied to the frame, so the matrix is the transpose of the one that
/// would rotate a vector by the same angle.
fn rotation_x(angle: f64) -> Mat3 {
    let (sine, cosine) = angle.sin_cos();
    Mat3::from_rows([1.0, 0.0, 0.0], [0.0, cosine, sine], [0.0, -sine, cosine])
}

/// Returns a rotation of `angle` about the second axis, in radians.
///
/// The convention is the one of [`rotation_x`].
fn rotation_y(angle: f64) -> Mat3 {
    let (sine, cosine) = angle.sin_cos();
    Mat3::from_rows([cosine, 0.0, -sine], [0.0, 1.0, 0.0], [sine, 0.0, cosine])
}
