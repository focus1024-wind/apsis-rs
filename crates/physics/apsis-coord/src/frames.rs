//! Positions that carry their reference frame in the type.

use apsis_frames::{EarthRotationAngle, rotation_ecef_to_eci, rotation_eci_to_ecef};
use apsis_geodesy::{
    Ellipsoid, Geodetic as EllipsoidalPosition, WGS84, cartesian_from_geodetic,
    geodetic_from_cartesian,
};
use apsis_math::Vec3;

/// A Cartesian position in the Earth-centered inertial frame.
///
/// The frame is aligned with the mean equator and equinox of J2000.0 and is
/// treated as inertial, so it does not rotate with the Earth. Coordinates
/// are in meters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Eci(Vec3);

/// A Cartesian position in the Earth-centered Earth-fixed frame.
///
/// The frame rotates with the Earth, so a point fixed to the ground has
/// constant coordinates in it. Coordinates are in meters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ecef(Vec3);

/// A position in the Earth-fixed frame given as latitude, longitude, and
/// height above the WGS 84 ellipsoid.
///
/// Latitude and longitude are in radians and height is in meters. Latitude
/// is geodetic, measured between the ellipsoid normal and the equatorial
/// plane.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Geodetic(EllipsoidalPosition);

impl Eci {
    /// Creates an inertial position from a vector in meters.
    pub const fn new(vector: Vec3) -> Self {
        Self(vector)
    }

    /// Returns the position in meters.
    pub const fn vector(self) -> Vec3 {
        self.0
    }

    /// Converts this inertial position to the Earth-fixed frame at the given
    /// Earth Rotation Angle.
    pub fn to_ecef(self, angle: EarthRotationAngle) -> Ecef {
        Ecef(rotation_eci_to_ecef(angle) * self.0)
    }
}

impl Ecef {
    /// Creates an Earth-fixed position from a vector in meters.
    pub const fn new(vector: Vec3) -> Self {
        Self(vector)
    }

    /// Returns the position in meters.
    pub const fn vector(self) -> Vec3 {
        self.0
    }

    /// Converts this Earth-fixed position to the inertial frame at the given
    /// Earth Rotation Angle.
    pub fn to_eci(self, angle: EarthRotationAngle) -> Eci {
        Eci(rotation_ecef_to_eci(angle) * self.0)
    }

    /// Converts this Earth-fixed position to latitude, longitude, and height
    /// above the given ellipsoid.
    ///
    /// Returns `None` when the position lies on the polar axis, where no
    /// longitude is defined.
    pub fn to_geodetic(self, ellipsoid: Ellipsoid) -> Option<Geodetic> {
        geodetic_from_cartesian(self.0, ellipsoid).map(Geodetic)
    }
}

impl Geodetic {
    /// Creates a geodetic position from latitude, longitude, and height.
    ///
    /// Latitude and longitude are in radians and height is in meters.
    pub const fn new(latitude: f64, longitude: f64, height: f64) -> Self {
        Self(EllipsoidalPosition::new(latitude, longitude, height))
    }

    /// Returns the geodetic latitude in radians.
    pub const fn latitude(self) -> f64 {
        self.0.latitude
    }

    /// Returns the longitude in radians, positive east.
    pub const fn longitude(self) -> f64 {
        self.0.longitude
    }

    /// Returns the height above the ellipsoid in meters.
    pub const fn height(self) -> f64 {
        self.0.height
    }

    /// Returns the underlying ellipsoidal position.
    pub const fn ellipsoidal(self) -> EllipsoidalPosition {
        self.0
    }

    /// Converts this position to the Earth-fixed frame on the WGS 84
    /// ellipsoid.
    ///
    /// This is the common case. Use [`Geodetic::to_ecef_on`] for a different
    /// ellipsoid.
    pub fn to_ecef(self) -> Ecef {
        self.to_ecef_on(WGS84)
    }

    /// Converts this position to the Earth-fixed frame on the given
    /// ellipsoid.
    pub fn to_ecef_on(self, ellipsoid: Ellipsoid) -> Ecef {
        Ecef(cartesian_from_geodetic(self.0, ellipsoid))
    }
}
