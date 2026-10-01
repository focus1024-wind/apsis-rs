//! Line of sight and visibility from a ground point.

use crate::geometry::earth_central_angle;
use apsis_geodesy::{Ellipsoid, Geodetic, WGS84};
use apsis_math::Vec3;
use core::f64::consts::FRAC_PI_2;

/// A ground station, given as a geodetic position on an ellipsoid.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GroundPoint {
    /// The geodetic position of the station.
    pub position: Geodetic,
    /// The ellipsoid the position is referred to.
    pub ellipsoid: Ellipsoid,
    /// The sensor elevation mask, in radians.
    ///
    /// The minimum elevation above the local horizon at which the station
    /// can see a satellite. A mask of zero means the station sees down to
    /// its own horizon.
    pub elevation_mask: f64,
}

impl GroundPoint {
    /// Creates a ground point on the WGS 84 ellipsoid with a given mask.
    pub fn new(position: Geodetic, elevation_mask: f64) -> Self {
        Self {
            position,
            ellipsoid: WGS84,
            elevation_mask,
        }
    }

    /// Returns the position of the station in the Earth-fixed Cartesian
    /// frame, in kilometers.
    ///
    /// The geodetic conversion yields meters, so the result is divided by a
    /// thousand to match the frame the orbit layer uses.
    pub fn ecef(&self) -> Vec3 {
        self.position.to_cartesian(self.ellipsoid) / 1_000.0
    }

    /// Returns the distance from the centre of the Earth, in kilometers.
    ///
    /// This is the local radius of the ellipsoid at the station's latitude
    /// plus its height, so it includes the effect of the flattening.
    pub fn radius(&self) -> f64 {
        self.ecef().norm()
    }

    /// Returns `true` when the station and its mask describe a usable
    /// sensor.
    pub fn is_valid(&self) -> bool {
        self.position.is_valid()
            && self.elevation_mask.is_finite()
            && (-FRAC_PI_2..=FRAC_PI_2).contains(&self.elevation_mask)
            && self.ellipsoid.semi_major_axis() > 0.0
    }
}

/// A sensor at a ground point, with a pointing direction.
///
/// The direction is a unit vector in the Earth-fixed frame. It is optional,
/// because a station that can point anywhere with sufficient gain is
/// modelled by its elevation mask alone; the direction matters only when the
/// station has a fixed beam.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sensor {
    /// The station.
    pub ground: GroundPoint,
    /// The pointing direction in the Earth-fixed frame, if the station is
    /// directional.
    pub pointing: Option<Vec3>,
}

impl Sensor {
    /// Creates a station that can point anywhere above its mask.
    pub const fn omnidirectional(ground: GroundPoint) -> Self {
        Self {
            ground,
            pointing: None,
        }
    }

    /// Creates a station with a fixed pointing direction.
    ///
    /// The direction is normalized when it is used, so the caller may pass
    /// an unnormalized vector.
    pub const fn directional(ground: GroundPoint, pointing: Vec3) -> Self {
        Self {
            ground,
            pointing: Some(pointing),
        }
    }
}

/// The point on the Earth's surface directly below a satellite.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SubSatellitePoint {
    /// The geodetic position of the sub-satellite point.
    pub position: Geodetic,
    /// The ellipsoid the position is referred to.
    pub ellipsoid: Ellipsoid,
}

/// Returns the sub-satellite point for a position in the Earth-fixed frame.
///
/// The satellite position is in kilometers. The sub-satellite point is the
/// intersection of the line from the centre of the Earth through the
/// satellite with the ellipsoid, which is the geodetic position of the
/// satellite with its height set to zero at the same latitude and longitude.
///
/// This is the geocentric projection, not the point at which the ellipsoid
/// normal through the satellite meets the surface. The difference is at most
/// about twenty kilometers for a low Earth satellite, and the geocentric
/// form is the one the footprint geometry uses.
///
/// Returns `None` when the position is not finite or lies on the polar axis,
/// where no longitude is defined.
pub fn sub_satellite_point(
    position_ecef_km: Vec3,
    ellipsoid: Ellipsoid,
) -> Option<SubSatellitePoint> {
    if !position_ecef_km.is_finite() {
        return None;
    }
    let position_m = position_ecef_km * 1_000.0;
    let geodetic = apsis_geodesy::geodetic_from_cartesian(position_m, ellipsoid)?;
    Some(SubSatellitePoint {
        position: Geodetic::new(geodetic.latitude, geodetic.longitude, 0.0),
        ellipsoid,
    })
}

/// Returns the elevation of a satellite as seen from a station, in radians.
///
/// The satellite position is in the Earth-fixed frame, in kilometers. The
/// elevation is measured from the local horizon, which is the plane
/// perpendicular to the ellipsoid normal at the station.
///
/// A positive value means the satellite is above the horizon, a value of a
/// right angle means it is at the zenith, and a negative value means it is
/// below the horizon.
///
/// Returns `None` when either position is not usable.
pub fn elevation_angle(satellite_ecef_km: Vec3, ground: &GroundPoint) -> Option<f64> {
    if !satellite_ecef_km.is_finite() || !ground.is_valid() {
        return None;
    }
    let station = ground.ecef();
    let line_of_sight = satellite_ecef_km - station;
    if line_of_sight.norm() == 0.0 {
        return None;
    }
    // The local up direction is the ellipsoid normal at the station, which
    // for a geodetic position is the direction the surface normal points.
    let up = local_up(ground);
    let sine = line_of_sight.normalize()?.dot(up.normalize()?);
    Some(sine.clamp(-1.0, 1.0).asin())
}

/// Returns the unit vector along the ellipsoid normal at a station.
///
/// This is the local vertical, the direction a plumb line points. For a
/// geodetic position it follows from the latitude and longitude, because the
/// normal of a biaxial ellipsoid lies in the meridian plane and makes the
/// geodetic latitude with the equatorial plane.
pub fn local_up(ground: &GroundPoint) -> Vec3 {
    let latitude = ground.position.latitude;
    let longitude = ground.position.longitude;
    let (sin_latitude, cos_latitude) = latitude.sin_cos();
    let (sin_longitude, cos_longitude) = longitude.sin_cos();
    Vec3::new(
        cos_latitude * cos_longitude,
        cos_latitude * sin_longitude,
        sin_latitude,
    )
}

/// Returns the elevation of a satellite as seen from a station, in radians.
///
/// This is the same as [`elevation_angle`], named for symmetry with the
/// other visibility functions.
///
/// Returns `None` when either position is not usable.
pub fn line_of_sight(satellite_ecef_km: Vec3, ground: &GroundPoint) -> Option<f64> {
    elevation_angle(satellite_ecef_km, ground)
}

/// Returns `true` when a satellite is visible from a sensor.
///
/// The satellite is visible when its elevation at the station is at least
/// the sensor's mask, and, for a directional sensor, when the direction to
/// the satellite is within a right angle of the pointing direction, so that
/// the satellite is in the forward hemisphere of the beam.
///
/// Returns `None` when the satellite position or the sensor is not usable.
pub fn visible_from(satellite_ecef_km: Vec3, sensor: &Sensor) -> Option<bool> {
    let elevation = elevation_angle(satellite_ecef_km, &sensor.ground)?;
    if elevation < sensor.ground.elevation_mask {
        return Some(false);
    }
    match sensor.pointing {
        None => Some(true),
        Some(pointing) => {
            let to_satellite = satellite_ecef_km - sensor.ground.ecef();
            let pointing = pointing.normalize()?;
            let to_satellite = to_satellite.normalize()?;
            Some(pointing.dot(to_satellite) > 0.0)
        }
    }
}

/// Returns the elevation of a satellite as seen from a geodetic station.
///
/// This is a convenience for a station given directly as a geodetic
/// position, without building a [`GroundPoint`].
///
/// Returns `None` when the satellite position or the station is not usable.
pub fn visible_from_geodetic(
    satellite_ecef_km: Vec3,
    position: Geodetic,
    elevation_mask: f64,
) -> Option<bool> {
    let ground = GroundPoint::new(position, elevation_mask);
    visible_from(satellite_ecef_km, &Sensor::omnidirectional(ground))
}

/// Returns the central angle between a satellite and a ground station, in
/// radians.
///
/// This uses the spherical relation between the two radii and the elevation,
/// so it is consistent with the footprint geometry. The satellite position
/// is used only for its radius.
///
/// Returns `None` when the geometry is not usable.
///
/// 几何不可用时返回 `None`。
pub fn central_angle_to_station(
    satellite_ecef_km: Vec3,
    ground: &GroundPoint,
    elevation: f64,
) -> Option<f64> {
    if !satellite_ecef_km.is_finite() || !ground.is_valid() {
        return None;
    }
    earth_central_angle(satellite_ecef_km.norm(), ground.radius(), elevation)
}
