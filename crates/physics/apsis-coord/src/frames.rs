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
///
/// 地心惯性系中的笛卡尔位置。
///
/// 该系与 J2000.0 的平赤道平春分点对齐，并视为惯性系，故不随地球自转。
/// 坐标单位为米。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Eci(Vec3);

/// A Cartesian position in the Earth-centered Earth-fixed frame.
///
/// The frame rotates with the Earth, so a point fixed to the ground has
/// constant coordinates in it. Coordinates are in meters.
///
/// 地心地固系中的笛卡尔位置。
///
/// 该系随地球自转，故地面上固定的点在其中坐标恒定。坐标单位为米。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ecef(Vec3);

/// A position in the Earth-fixed frame given as latitude, longitude, and
/// height above the WGS 84 ellipsoid.
///
/// Latitude and longitude are in radians and height is in meters. Latitude
/// is geodetic, measured between the ellipsoid normal and the equatorial
/// plane.
///
/// 以纬度、经度与 WGS 84 椭球高给出的地固系位置。
///
/// 纬度与经度用弧度，高用米。纬度为大地纬度，在椭球法线与赤道面之间量取。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Geodetic(EllipsoidalPosition);

impl Eci {
    /// Creates an inertial position from a vector in meters.
    ///
    /// 由米为单位的向量创建惯性位置。
    pub const fn new(vector: Vec3) -> Self {
        Self(vector)
    }

    /// Returns the position in meters.
    ///
    /// 返回以米为单位的位置。
    pub const fn vector(self) -> Vec3 {
        self.0
    }

    /// Converts this inertial position to the Earth-fixed frame at the given
    /// Earth Rotation Angle.
    ///
    /// 按给定的地球自转角把该惯性位置转换到地固系。
    pub fn to_ecef(self, angle: EarthRotationAngle) -> Ecef {
        Ecef(rotation_eci_to_ecef(angle) * self.0)
    }
}

impl Ecef {
    /// Creates an Earth-fixed position from a vector in meters.
    ///
    /// 由米为单位的向量创建地固位置。
    pub const fn new(vector: Vec3) -> Self {
        Self(vector)
    }

    /// Returns the position in meters.
    ///
    /// 返回以米为单位的位置。
    pub const fn vector(self) -> Vec3 {
        self.0
    }

    /// Converts this Earth-fixed position to the inertial frame at the given
    /// Earth Rotation Angle.
    ///
    /// 按给定的地球自转角把该地固位置转换到惯性系。
    pub fn to_eci(self, angle: EarthRotationAngle) -> Eci {
        Eci(rotation_ecef_to_eci(angle) * self.0)
    }

    /// Converts this Earth-fixed position to latitude, longitude, and height
    /// above the given ellipsoid.
    ///
    /// Returns `None` when the position lies on the polar axis, where no
    /// longitude is defined.
    ///
    /// 把该地固位置转换为给定椭球之上的纬度、经度与高。
    ///
    /// 当位置落在极轴上时返回 `None`，因为那里经度未定义。
    pub fn to_geodetic(self, ellipsoid: Ellipsoid) -> Option<Geodetic> {
        geodetic_from_cartesian(self.0, ellipsoid).map(Geodetic)
    }
}

impl Geodetic {
    /// Creates a geodetic position from latitude, longitude, and height.
    ///
    /// Latitude and longitude are in radians and height is in meters.
    ///
    /// 由纬度、经度与高创建大地位置。
    ///
    /// 纬度与经度用弧度，高用米。
    pub const fn new(latitude: f64, longitude: f64, height: f64) -> Self {
        Self(EllipsoidalPosition::new(latitude, longitude, height))
    }

    /// Returns the geodetic latitude in radians.
    ///
    /// 返回大地纬度，单位为弧度。
    pub const fn latitude(self) -> f64 {
        self.0.latitude
    }

    /// Returns the longitude in radians, positive east.
    ///
    /// 返回经度，单位为弧度，向东为正。
    pub const fn longitude(self) -> f64 {
        self.0.longitude
    }

    /// Returns the height above the ellipsoid in meters.
    ///
    /// 返回椭球高，单位为米。
    pub const fn height(self) -> f64 {
        self.0.height
    }

    /// Returns the underlying ellipsoidal position.
    ///
    /// 返回底层的椭球位置。
    pub const fn ellipsoidal(self) -> EllipsoidalPosition {
        self.0
    }

    /// Converts this position to the Earth-fixed frame on the WGS 84
    /// ellipsoid.
    ///
    /// This is the common case. Use [`Geodetic::to_ecef_on`] for a different
    /// ellipsoid.
    ///
    /// 把该位置转换到 WGS 84 椭球的地固系。
    ///
    /// 这是常见情形。需用其他椭球时请用 [`Geodetic::to_ecef_on`]。
    pub fn to_ecef(self) -> Ecef {
        self.to_ecef_on(WGS84)
    }

    /// Converts this position to the Earth-fixed frame on the given
    /// ellipsoid.
    ///
    /// 把该位置转换到给定椭球的地固系。
    pub fn to_ecef_on(self, ellipsoid: Ellipsoid) -> Ecef {
        Ecef(cartesian_from_geodetic(self.0, ellipsoid))
    }
}
