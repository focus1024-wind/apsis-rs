//! Geodetic positions and their conversion to Earth-centered,
//! Earth-fixed Cartesian coordinates.

use crate::ellipsoid::Ellipsoid;
use apsis_math::Vec3;

/// A geodetic position: latitude, longitude, and height above an ellipsoid.
///
/// Latitude and longitude are in radians and height is in meters. Latitude
/// is geodetic, measured between the ellipsoid normal and the equatorial
/// plane, and lies in `[-pi/2, pi/2]`. Longitude is measured east from the
/// prime meridian. Height is the distance along the ellipsoid normal, taken
/// as positive outside the ellipsoid.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Geodetic {
    /// Geodetic latitude, in radians.
    pub latitude: f64,
    /// Longitude, in radians, positive east.
    pub longitude: f64,
    /// Height above the ellipsoid, in meters.
    pub height: f64,
}

impl Geodetic {
    /// Creates a geodetic position from latitude, longitude, and height.
    pub const fn new(latitude: f64, longitude: f64, height: f64) -> Self {
        Self {
            latitude,
            longitude,
            height,
        }
    }

    /// Converts this geodetic position to Earth-centered, Earth-fixed
    /// Cartesian coordinates on the given ellipsoid.
    ///
    /// The frame has its third axis along the ellipsoid's minor axis
    /// through the north pole, its first axis through the intersection of
    /// the equator and the prime meridian, and its second axis completing
    /// a right-handed set. The result is in meters.
    ///
    /// The closed form is
    ///
    /// ```text
    /// N = a / sqrt(1 - e^2 sin^2(lat))
    /// x = (N + h) cos(lat) cos(lon)
    /// y = (N + h) cos(lat) sin(lon)
    /// z = (N (1 - e^2) + h) sin(lat)
    /// ```
    ///
    /// where `N` is the prime vertical radius of curvature. This form is
    /// exact everywhere, including at the poles, where the cosine factors
    /// vanish.
    pub fn to_ecef(self, ellipsoid: Ellipsoid) -> Ecef {
        ecef_from_geodetic(self, ellipsoid)
    }

    /// Returns `true` when every field is finite and the latitude and
    /// longitude are inside their ranges.
    pub fn is_valid(self) -> bool {
        let half_pi = core::f64::consts::FRAC_PI_2;
        self.latitude.is_finite()
            && self.longitude.is_finite()
            && self.height.is_finite()
            && self.latitude >= -half_pi
            && self.latitude <= half_pi
            && self.longitude >= -core::f64::consts::PI
            && self.longitude <= core::f64::consts::PI
    }
}

/// An Earth-centered, Earth-fixed Cartesian position.
///
/// The frame is the one described by [`Geodetic::to_ecef`]. The wrapped
/// [`Vec3`] holds the position in meters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ecef(pub Vec3);

impl Ecef {
    /// Creates an Earth-centered, Earth-fixed position from a vector in
    /// meters.
    pub const fn new(position: Vec3) -> Self {
        Self(position)
    }

    /// Returns the position in meters.
    pub const fn position(self) -> Vec3 {
        self.0
    }

    /// Converts this position to a geodetic position on the given
    /// ellipsoid.
    ///
    /// Returns `None` when the position is not finite, or when it lies on
    /// the polar axis so that no longitude is defined.
    pub fn to_geodetic(self, ellipsoid: Ellipsoid) -> Option<Geodetic> {
        geodetic_from_ecef(self, ellipsoid)
    }
}

/// Converts a geodetic position to Earth-centered, Earth-fixed Cartesian
/// coordinates.
///
/// See [`Geodetic::to_ecef`] for the frame definition and the closed form.
pub fn ecef_from_geodetic(geodetic: Geodetic, ellipsoid: Ellipsoid) -> Ecef {
    let Geodetic {
        latitude,
        longitude,
        height,
    } = geodetic;
    let (sin_latitude, cos_latitude) = latitude.sin_cos();
    let (sin_longitude, cos_longitude) = longitude.sin_cos();
    let prime_vertical = ellipsoid.prime_vertical_radius(latitude);
    let eccentricity_squared = ellipsoid.eccentricity_squared();

    let horizontal = prime_vertical + height;
    let x = horizontal * cos_latitude * cos_longitude;
    let y = horizontal * cos_latitude * sin_longitude;
    let z = (prime_vertical * (1.0 - eccentricity_squared) + height) * sin_latitude;
    Ecef(Vec3::new(x, y, z))
}

/// Converts Earth-centered, Earth-fixed Cartesian coordinates to a geodetic
/// position.
///
/// The latitude and height are recovered by Bowring's method, which solves
/// the meridian equation in closed form and then refines the result with a
/// few fixed point steps. The iteration converges to well below a
/// micrometer for positions from below the ellipsoid out to geostationary
/// radius.
///
/// Returns `None` when the position is not finite, or when it lies on the
/// polar axis so that no longitude is defined.
pub fn geodetic_from_ecef(ecef: Ecef, ellipsoid: Ellipsoid) -> Option<Geodetic> {
    let Vec3 { x, y, z } = ecef.0;
    if !ecef.0.is_finite() {
        return None;
    }

    let squared = ellipsoid.eccentricity_squared();
    let second_squared = ellipsoid.second_eccentricity_squared();
    let semi_major = ellipsoid.semi_major_axis();
    let semi_minor = ellipsoid.semi_minor_axis();

    let horizontal_squared = x * x + y * y;
    if horizontal_squared == 0.0 {
        // On the polar axis the longitude is undefined.
        return None;
    }
    let horizontal = horizontal_squared.sqrt();
    let longitude = y.atan2(x);

    // Bowring's initial latitude estimate, obtained by treating the point
    // as if it lay on a locally spherical surface.
    let theta = (z * semi_major).atan2(horizontal * semi_minor);
    let (sin_theta, cos_theta) = theta.sin_cos();
    let mut latitude = (z + second_squared * semi_minor * sin_theta * sin_theta * sin_theta)
        .atan2(horizontal - squared * semi_major * cos_theta * cos_theta * cos_theta);

    // Refine with fixed point steps on the ellipsoid equation. Three steps
    // reduce the error to the floating point noise floor for every
    // terrestrial altitude.
    for _ in 0..3 {
        let (sin_latitude, _) = latitude.sin_cos();
        latitude = (z + squared * ellipsoid.prime_vertical_radius(latitude) * sin_latitude)
            .atan2(horizontal);
    }

    let (sin_latitude, cos_latitude) = latitude.sin_cos();
    // Guard against a latitude that has converged exactly to a pole, where
    // the division below would be singular.
    if cos_latitude == 0.0 {
        return None;
    }
    let height = if cos_latitude.abs() > 0.5 {
        horizontal / cos_latitude - ellipsoid.prime_vertical_radius(latitude)
    } else {
        z / sin_latitude - ellipsoid.prime_vertical_radius(latitude) * (1.0 - squared)
    };

    Some(Geodetic::new(latitude, longitude, height))
}
