//! The IAU 2006 precession model and the frame bias.
//!
//! Precession is the slow, secular motion of the Earth's rotation axis about
//! the pole of the ecliptic, caused mainly by the torque of the Moon and the
//! Sun on the equatorial bulge. The IAU 2006 model expresses it as a
//! rotation from the celestial reference frame to the mean equator and
//! equinox of date, built from four angles.
//!
//! The frame bias accounts for the small offset between the orientation of
//! the International Celestial Reference System and the dynamical mean
//! equator and equinox of J2000.0. It is a constant rotation, and omitting
//! it places a position about twenty milliarcseconds away, which is well
//! above the precision of a modern transformation.
//!
//! Source: the P03 precession theory of Capitaine, Wallace and Chapront,
//! adopted by the International Astronomical Union in 2006, as implemented
//! in the IAU SOFA library and its ERFA derivative, functions `eraPfw06`,
//! `eraPmat06`, `eraFw2m`, `eraBi00` and `eraObl06`.

use apsis_math::Mat3;
use core::f64::consts::PI;

/// Converts arcseconds to radians.
const ARCSECOND_TO_RADIAN: f64 = PI / (180.0 * 3_600.0);

/// The frame bias in right ascension, in radians.
///
/// The offset between the origin of the International Celestial Reference
/// System and the dynamical equinox of J2000.0. The value is
/// minus 0.0146 arcseconds.
pub const FRAME_BIAS_RIGHT_ASCENSION: f64 = -0.014_6 * ARCSECOND_TO_RADIAN;

/// The frame bias in declination, in radians.
///
/// The offset between the pole of the International Celestial Reference
/// System and the mean pole of J2000.0. The value is minus 0.0068192
/// arcseconds.
pub const FRAME_BIAS_DECLINATION: f64 = -0.006_819_2 * ARCSECOND_TO_RADIAN;

/// The frame bias in longitude, in radians.
///
/// The offset between the two origins measured along the equator. The value
/// is minus 0.041775 arcseconds.
pub const FRAME_BIAS_LONGITUDE: f64 = -0.041_775 * ARCSECOND_TO_RADIAN;

/// The four angles that describe the precession from J2000.0 to a date.
///
/// They are applied in the order the reference uses: a rotation by
/// `gamma_bar` about the third axis, then by `phi_bar` about the first, then
/// by minus `psi_bar` about the third, then by minus `epsilon_a` about the
/// first.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrecessionAngles {
    /// The precession of the equator along the ecliptic, in radians.
    pub gamma_bar: f64,
    /// The inclination of the moving equator to the ecliptic, in radians.
    pub phi_bar: f64,
    /// The accumulated precession in longitude, in radians.
    pub psi_bar: f64,
    /// The mean obliquity at the date, in radians.
    pub epsilon: f64,
}

/// Returns the mean obliquity of the ecliptic, in radians.
///
/// The polynomial is the IAU 2006 expression, evaluated for a time in Julian
/// centuries from J2000.0 on the Terrestrial Time scale. The constant term
/// is 84381.406 arcseconds, which is the obliquity at J2000.0.
///
/// Source: the IAU 2006 precession, as implemented by `eraObl06`.
pub fn mean_obliquity(t: f64) -> f64 {
    let arcseconds = 84_381.406
        + t * (-46.836_769
            + t * (-0.000_183_1
                + t * (0.002_003_40 + t * (-0.000_000_576 + t * (-0.000_000_043_4)))));
    arcseconds * ARCSECOND_TO_RADIAN
}

/// Returns the four precession angles for a time.
///
/// The time is in Julian centuries from J2000.0 on the Terrestrial Time
/// scale.
///
/// Source: the P03 precession, as implemented by `eraPfw06`.
pub fn precession_angles(t: f64) -> PrecessionAngles {
    let gamma_bar = -0.052_928
        + t * (10.556_378
            + t * (0.493_204_4
                + t * (-0.000_312_38 + t * (-0.000_002_788 + t * (0.000_000_026_0)))));
    let phi_bar = 84_381.412_819
        + t * (-46.811_016
            + t * (0.051_126_8
                + t * (0.000_532_89 + t * (-0.000_000_440 + t * (-0.000_000_017_6)))));
    let psi_bar = -0.041_775
        + t * (5_038.481_484
            + t * (1.558_417_5
                + t * (-0.000_185_22 + t * (-0.000_026_452 + t * (-0.000_000_014_8)))));

    PrecessionAngles {
        gamma_bar: gamma_bar * ARCSECOND_TO_RADIAN,
        phi_bar: phi_bar * ARCSECOND_TO_RADIAN,
        psi_bar: psi_bar * ARCSECOND_TO_RADIAN,
        epsilon: mean_obliquity(t),
    }
}

/// Returns the frame bias rotation matrix.
///
/// The bias is a constant rotation from the International Celestial
/// Reference System to the dynamical mean equator and equinox of J2000.0.
/// Its three angles are far below a tenth of an arcsecond, but omitting it
/// would leave a systematic error that is larger than the precision of the
/// nutation series.
///
/// Source: as implemented by `eraBi00`, which gives the bias as three small
/// rotations rather than as a matrix.
pub fn frame_bias_matrix() -> Mat3 {
    // The reference builds the bias from three elementary rotations, and
    // the result is reproduced here in the same order.
    rotation_z(FRAME_BIAS_LONGITUDE)
        * rotation_x(FRAME_BIAS_DECLINATION)
        * rotation_y(FRAME_BIAS_RIGHT_ASCENSION)
}

/// Returns the rotation from the celestial reference frame to the mean
/// equator and equinox of date.
///
/// This is the bias-precession matrix. Nutation is applied on top of it to
/// reach the true equator and equinox of date.
///
/// The time is in Julian centuries from J2000.0 on the Terrestrial Time
/// scale.
///
/// Source: as implemented by `eraPmat06`.
pub fn bias_precession_matrix(t: f64) -> Mat3 {
    let angles = precession_angles(t);
    frame_to_mean_of_date(&angles)
}

/// Returns the rotation from the celestial reference frame to the mean
/// equator and equinox of date, given precomputed precession angles.
///
/// Source: as implemented by `eraFw2m`, which applies the four elementary
/// rotations in sequence. The reference helper composes each new rotation
/// on the left of the accumulator, so the product here is written with the
/// last rotation applied first, in the same order.
pub fn frame_to_mean_of_date(angles: &PrecessionAngles) -> Mat3 {
    rotation_x(-angles.epsilon)
        * rotation_z(-angles.psi_bar)
        * rotation_x(angles.phi_bar)
        * rotation_z(angles.gamma_bar)
}

/// Returns a rotation of `angle` about the first axis, in radians.
///
/// The convention matches the reference helpers `eraRx` and `eraRz`: the
/// rotation is applied to the frame rather than to a vector, so the matrix
/// is the transpose of the one that would rotate a vector by the same angle.
/// Composing `a * b` therefore applies `b` first, and the products in this
/// module read in the order the rotations are physically applied.
pub(crate) fn rotation_x(angle: f64) -> Mat3 {
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

/// Returns a rotation of `angle` about the third axis, in radians.
///
/// The convention is the one of [`rotation_x`].
pub fn rotation_z(angle: f64) -> Mat3 {
    let (sine, cosine) = angle.sin_cos();
    Mat3::from_rows([cosine, sine, 0.0], [-sine, cosine, 0.0], [0.0, 0.0, 1.0])
}
