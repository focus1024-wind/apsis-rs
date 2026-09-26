//! Reference ellipsoids.

/// A biaxial reference ellipsoid: an ellipse of revolution about its minor
/// axis.
///
/// The shape is fixed by two numbers. This crate stores the semi-major axis
/// and the inverse flattening, which are the two defining parameters of the
/// WGS 84 ellipsoid in the standard. Every other quantity is derived, so
/// there is no risk of the parameters disagreeing with each other.
///
/// Lengths are in meters and angles are in radians.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ellipsoid {
    semi_major_axis: f64,
    inverse_flattening: f64,
}

impl Ellipsoid {
    /// Creates an ellipsoid from its semi-major axis and inverse
    /// flattening.
    ///
    /// Lengths are in meters. The inverse flattening must be positive and
    /// finite; a value of infinity describes a sphere.
    pub const fn new(semi_major_axis: f64, inverse_flattening: f64) -> Self {
        Self {
            semi_major_axis,
            inverse_flattening,
        }
    }

    /// Returns the semi-major axis, in meters.
    pub const fn semi_major_axis(self) -> f64 {
        self.semi_major_axis
    }

    /// Returns the inverse flattening.
    pub const fn inverse_flattening(self) -> f64 {
        self.inverse_flattening
    }

    /// Returns the flattening.
    ///
    /// This is zero for a sphere.
    pub fn flattening(self) -> f64 {
        1.0 / self.inverse_flattening
    }

    /// Returns the semi-minor axis, in meters.
    pub fn semi_minor_axis(self) -> f64 {
        self.semi_major_axis * (1.0 - self.flattening())
    }

    /// Returns the first eccentricity squared, a dimensionless number in
    /// `[0, 1)`.
    ///
    /// It is `2f - f^2` for flattening `f`.
    pub fn eccentricity_squared(self) -> f64 {
        let flattening = self.flattening();
        2.0 * flattening - flattening * flattening
    }

    /// Returns the first eccentricity, a dimensionless number in `[0, 1)`.
    pub fn eccentricity(self) -> f64 {
        self.eccentricity_squared().sqrt()
    }

    /// Returns the second eccentricity squared, a dimensionless number.
    ///
    /// It is `e^2 / (1 - e^2)`, which appears in the closed form for
    /// converting a geodetic position to Cartesian coordinates.
    pub fn second_eccentricity_squared(self) -> f64 {
        let squared = self.eccentricity_squared();
        squared / (1.0 - squared)
    }

    /// Returns the polar radius of curvature, in meters.
    ///
    /// This is the radius of curvature in the meridian at the pole, equal
    /// to `a / sqrt(1 - e^2)`.
    pub fn polar_radius_of_curvature(self) -> f64 {
        self.semi_major_axis / (1.0 - self.eccentricity_squared()).sqrt()
    }

    /// Returns the radius of curvature in the prime vertical at the given
    /// geodetic latitude, in meters.
    ///
    /// This is the distance from the surface to the polar axis measured
    /// along the ellipsoid normal, and it grows from the semi-major axis at
    /// the equator to [`Ellipsoid::polar_radius_of_curvature`] at the pole.
    ///
    /// The latitude is in radians.
    pub fn prime_vertical_radius(self, latitude: f64) -> f64 {
        let sin = latitude.sin();
        self.semi_major_axis / (1.0 - self.eccentricity_squared() * sin * sin).sqrt()
    }

    /// Returns the radius of curvature in the meridian at the given
    /// geodetic latitude, in meters.
    ///
    /// The latitude is in radians.
    pub fn meridian_radius(self, latitude: f64) -> f64 {
        let sin = latitude.sin();
        let squared = self.eccentricity_squared();
        let denominator = 1.0 - squared * sin * sin;
        self.semi_major_axis * (1.0 - squared) / denominator.powf(1.5)
    }

    /// Returns the geocentric latitude corresponding to a geodetic
    /// latitude, in radians.
    ///
    /// The geocentric latitude is measured from the equatorial plane to the
    /// radius vector, while the geodetic latitude is measured to the
    /// ellipsoid normal. The geocentric value is the smaller of the two
    /// away from the equator and the poles.
    pub fn geocentric_latitude(self, geodetic_latitude: f64) -> f64 {
        let squared = self.eccentricity_squared();
        let tangent = (1.0 - squared) * geodetic_latitude.tan();
        tangent.atan()
    }
}

/// The WGS 84 reference ellipsoid.
///
/// The two defining parameters are the semi-major axis `a = 6 378 137 m` and
/// the inverse flattening `1 / f = 298.257 223 563`.
///
/// Source: NIMA TR8350.2, third edition, 2000.
pub const WGS84: Ellipsoid = Ellipsoid::new(6_378_137.0, 298.257_223_563);

/// The Earth gravitational parameter for WGS 84, in cubic meters per second
/// squared.
///
/// This is held here rather than in `apsis-core` so that a caller working
/// with the ellipsoid can use the matching gravity field. The value is
/// `3.986 004 418e14 m^3/s^2`.
pub const WGS84_GRAVITATIONAL_PARAMETER: f64 = 3.986_004_418e14;
