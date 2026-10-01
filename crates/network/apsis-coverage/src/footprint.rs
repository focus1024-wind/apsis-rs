//! Footprint geometry and coverage fractions.

use crate::geometry::{earth_central_angle, elevation_from_central_angle};
use apsis_math::Vec3;
use core::f64::consts::PI;

/// A circular ground footprint.
///
/// The footprint is the part of the Earth's surface from which a satellite
/// is visible above a given elevation mask. On a sphere it is an exact
/// circle, described here by the central angle from the sub-satellite point
/// and the radius of the body.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Footprint {
    /// The central half angle of the footprint, in radians.
    ///
    /// This is the angle at the centre of the Earth from the sub-satellite
    /// point to the edge of the footprint.
    pub central_angle: f64,
    /// The radius of the body, in kilometers.
    pub body_radius: f64,
}

impl Footprint {
    /// Creates a footprint from its central angle and body radius.
    pub const fn new(central_angle: f64, body_radius: f64) -> Self {
        Self {
            central_angle,
            body_radius,
        }
    }

    /// Returns the footprint radius measured along the surface, in
    /// kilometers.
    pub fn surface_radius(&self) -> f64 {
        self.central_angle * self.body_radius
    }

    /// Returns the area of the footprint, in square kilometers.
    ///
    /// The area is the part of a sphere cut off by a cone,
    ///
    /// ```text
    /// A = 2 pi R^2 (1 - cos(central))
    /// ```
    ///
    /// which is exact for a spherical body.
    pub fn area(&self) -> f64 {
        2.0 * PI * self.body_radius * self.body_radius * (1.0 - self.central_angle.cos())
    }

    /// Returns the fraction of the whole sphere covered by the footprint.
    ///
    /// The value is between zero and one half, because a footprint from a
    /// point above the surface can never cover more than a hemisphere.
    pub fn fraction_of_surface(&self) -> f64 {
        self.area() / (4.0 * PI * self.body_radius * self.body_radius)
    }

    /// Returns `true` when a ground point at the given central angle from
    /// the sub-satellite point lies inside the footprint.
    ///
    /// The comparison is inclusive, so a point exactly on the edge is
    /// inside.
    pub fn contains_central_angle(&self, central_angle: f64) -> bool {
        central_angle.is_finite() && central_angle >= 0.0 && central_angle <= self.central_angle
    }
}

/// Returns the central half angle of the footprint, in radians.
///
/// This is the largest central angle at which the satellite is still above
/// the elevation mask.
///
/// Returns `None` when the geometry is not usable.
///
/// 几何不可用时返回 `None`。
pub fn footprint_central_angle(
    satellite_radius: f64,
    body_radius: f64,
    elevation_mask: f64,
) -> Option<f64> {
    let central = earth_central_angle(satellite_radius, body_radius, elevation_mask)?;
    if central.is_finite() && central >= 0.0 {
        Some(central)
    } else {
        None
    }
}

/// Returns the footprint for a satellite and an elevation mask.
///
/// Returns `None` when the geometry is not usable.
///
/// 几何不可用时返回 `None`。
pub fn footprint(
    satellite_radius: f64,
    body_radius: f64,
    elevation_mask: f64,
) -> Option<Footprint> {
    Some(Footprint::new(
        footprint_central_angle(satellite_radius, body_radius, elevation_mask)?,
        body_radius,
    ))
}

/// Returns the area of a footprint, in square kilometers.
///
/// This is a free function form of [`Footprint::area`].
///
/// Returns `None` when the geometry is not usable.
///
/// 几何不可用时返回 `None`。
pub fn footprint_area(satellite_radius: f64, body_radius: f64, elevation_mask: f64) -> Option<f64> {
    Some(footprint(satellite_radius, body_radius, elevation_mask)?.area())
}

/// Returns the fraction of the Earth's surface covered by a footprint.
///
/// Returns `None` when the geometry is not usable.
///
/// 几何不可用时返回 `None`。
pub fn footprint_fraction(
    satellite_radius: f64,
    body_radius: f64,
    elevation_mask: f64,
) -> Option<f64> {
    Some(footprint(satellite_radius, body_radius, elevation_mask)?.fraction_of_surface())
}

/// Returns the angle between two surface points, in radians.
///
/// The vectors must be positions relative to the centre of the body, in the
/// same frame. The angle is the central angle between them, as seen from the
/// centre.
///
/// Returns `None` when either vector has zero length or a non-finite
/// component.
pub fn central_angle_between(first: Vec3, second: Vec3) -> Option<f64> {
    if !first.is_finite() || !second.is_finite() {
        return None;
    }
    let first_norm = first.norm();
    let second_norm = second.norm();
    if first_norm == 0.0 || second_norm == 0.0 {
        return None;
    }
    Some(
        (first.dot(second) / (first_norm * second_norm))
            .clamp(-1.0, 1.0)
            .acos(),
    )
}

/// Returns `true` when a ground point lies inside a footprint.
///
/// The point and the sub-satellite point are positions relative to the
/// centre of the body, in kilometers. The footprint supplies the body radius
/// and the central half angle.
///
/// Returns `None` when either vector is degenerate or not finite.
pub fn point_in_footprint(
    footprint: &Footprint,
    point: Vec3,
    sub_satellite_point: Vec3,
) -> Option<bool> {
    let central = central_angle_between(point, sub_satellite_point)?;
    Some(footprint.contains_central_angle(central))
}

/// Returns the elevation at which a footprint edge is seen from the ground,
/// in radians.
///
/// This is the inverse of [`footprint_central_angle`], useful when a
/// footprint is known and the mask it corresponds to is wanted.
///
/// Returns `None` when the geometry is not usable.
///
/// 几何不可用时返回 `None`。
pub fn footprint_edge_elevation(satellite_radius: f64, footprint: &Footprint) -> Option<f64> {
    elevation_from_central_angle(
        satellite_radius,
        footprint.body_radius,
        footprint.central_angle,
    )
}
