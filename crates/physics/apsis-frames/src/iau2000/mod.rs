//! The IAU 2000/2006 Earth orientation models.
//!
//! The transformation from an inertial frame to an Earth-fixed frame is a
//! sequence of rotations, each from a different model:
//!
//! 1. The **frame bias**, a constant rotation from the International
//!    Celestial Reference System to the dynamical mean equator and equinox
//!    of J2000.0.
//! 2. The **precession**, the secular motion of the pole, from the P03
//!    theory adopted in 2006.
//! 3. The **nutation**, the periodic motion of the pole, from the MHB2000
//!    series adopted in 2000.
//! 4. The **Earth rotation angle**, which carries the frame from the
//!    celestial to the terrestrial meridian.
//! 5. The **polar motion**, the small motion of the pole within the Earth,
//!    which a caller supplies because it is published as a daily measured
//!    quantity.
//!
//! The composition of the first three is the celestial-to-intermediate
//! matrix, and the composition with the rotation angle and the polar motion
//! is the celestial-to-terrestrial matrix. For satellite work the polar
//! motion is often omitted, because it is under a tenth of an arcsecond,
//! but the nutation and precession are not: the nutation reaches seventeen
//! arcseconds, which is half a kilometer of ground position for a low Earth
//! satellite.
//!
//! All times are Julian centuries from J2000.0 on the Terrestrial Time
//! scale, unless a function says otherwise. The angles a caller supplies to
//! the polar motion are in radians.
//!
//! Source: the models and the order of the rotations follow the IAU SOFA
//! library and its ERFA derivative. The specific functions are named in each
//! module.

mod eect_table;
mod fundamental;
mod nutation;
mod nutation_table;
mod precession;
mod teme;
mod terrestrial;

pub use fundamental::{
    DelaunayArguments, PlanetaryArguments, general_precession, mean_anomaly_moon, mean_anomaly_sun,
    mean_anomaly_sun_iers2003, mean_elongation_moon_sun, mean_elongation_moon_sun_iers2003,
    mean_latitude_moon, mean_longitude_earth, mean_longitude_jupiter, mean_longitude_mars,
    mean_longitude_mercury, mean_longitude_neptune, mean_longitude_node, mean_longitude_saturn,
    mean_longitude_uranus, mean_longitude_venus,
};
pub use nutation::{Nutation, nutation_2000a, nutation_2000a_with_2006_adjustment};
pub use precession::{
    FRAME_BIAS_DECLINATION, FRAME_BIAS_LONGITUDE, FRAME_BIAS_RIGHT_ASCENSION, PrecessionAngles,
    bias_precession_matrix, frame_bias_matrix, frame_to_mean_of_date, mean_obliquity,
    precession_angles, rotation_z,
};
pub use teme::{greenwich_mean_sidereal_time_1982, teme_to_terrestrial_matrix};
pub use terrestrial::{
    PolarMotion, bias_precession_nutation_matrix, celestial_to_terrestrial_matrix,
    earth_rotation_angle, equation_of_the_equinoxes, equation_of_the_equinoxes_complementary_terms,
    greenwich_apparent_sidereal_time, greenwich_mean_sidereal_time,
    intermediate_to_terrestrial_matrix,
};

/// One term of the luni-solar nutation series.
///
/// The five multipliers are applied to the Delaunay arguments in the order
/// `l`, `l'`, `F`, `D`, `Om`. The six coefficients are the longitude sine,
/// its time derivative, the longitude cosine, the obliquity cosine, its time
/// derivative, and the obliquity sine, in units of 0.1 microarcsecond.
#[derive(Debug, Clone, Copy)]
pub struct SeriesTerm {
    /// The integer multipliers of the five Delaunay arguments.
    pub argument: [i32; 5],
    /// The six coefficients of the term, in the order the reference lists
    /// them.
    pub coefficients: [f64; 6],
}

/// One term of the complementary terms of the equation of the equinoxes.
///
/// The eight multipliers apply to the arguments `l`, `l'`, `F`, `D`, `Om`,
/// the mean longitude of Venus, the mean longitude of the Earth, and the
/// general accumulated precession. The two coefficients are the sine and
/// cosine amplitudes, in arcseconds.
#[derive(Debug, Clone, Copy)]
pub struct ComplementaryTerm {
    /// The integer multipliers of the eight arguments.
    pub argument: [i32; 8],
    /// The sine and cosine coefficients.
    pub coefficients: [f64; 2],
}

/// One term of the planetary nutation series.
///
/// The thirteen multipliers are applied to the planetary arguments in the
/// order `l`, `F`, `D`, `Om`, Mercury through Uranus, Neptune, and the
/// general accumulated precession. The four coefficients are the longitude
/// sine, the longitude cosine, the obliquity sine, and the obliquity cosine,
/// in units of 0.1 microarcsecond.
#[derive(Debug, Clone, Copy)]
pub struct PlanetaryTerm {
    /// The integer multipliers of the thirteen planetary arguments.
    pub argument: [i32; 13],
    /// The four coefficients of the term, in the order the reference lists
    /// them.
    pub coefficients: [f64; 4],
}
