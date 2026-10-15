//! Footprint geometry and coverage fractions.
//!
//! 覆盖区几何与覆盖比例。

use crate::apsis_coverage::geometry::{earth_central_angle, elevation_from_central_angle};
use crate::apsis_math::Vec3;
use core::f64::consts::PI;

/// A circular ground footprint.
///
/// The footprint is the part of the Earth's surface from which a satellite
/// is visible above a given elevation mask. On a sphere it is an exact
/// circle, described here by the central angle from the sub-satellite point
/// and the radius of the body.
///
/// 圆形地面覆盖区。
///
/// 覆盖区是能在地平遮蔽以上看见某卫星的那部分地表。在球面上它是精确
/// 的圆，此处以自星下点的中心角和天体半径描述。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Footprint {
    /// The central half angle of the footprint, in radians.
    ///
    /// This is the angle at the centre of the Earth from the sub-satellite
    /// point to the edge of the footprint.
    ///
    /// 覆盖区的中心半角，单位为弧度。
    ///
    /// 这是在地心处自星下点到覆盖区边缘的角度。
    pub central_angle: f64,
    /// The radius of the body, in kilometers.
    ///
    /// 天体的半径，单位为千米。
    pub body_radius: f64,
}

impl Footprint {
    /// Creates a footprint from its central angle and body radius.
    ///
    /// 由其中心角与天体半径创建覆盖区。
    #[must_use]
    pub const fn new(central_angle: f64, body_radius: f64) -> Self {
        Self {
            central_angle,
            body_radius,
        }
    }

    /// Returns the footprint radius measured along the surface, in
    /// kilometers.
    ///
    /// 返回沿地表量得的覆盖区半径，单位为千米。
    #[must_use]
    pub fn surface_radius(&self) -> f64 {
        self.central_angle * self.body_radius
    }

    /// Returns the area of the footprint, in square kilometers.
    ///
    /// The area is the part of a sphere cut off by a cone,
    ///
    /// <div>
    /// $$
    /// A = 2 \pi R^2 \left(1 - \cos(\text{central})\right)
    /// $$
    /// </div>
    ///
    /// which is exact for a spherical body.
    ///
    /// 返回覆盖区的面积，单位为平方千米。
    ///
    /// 该面积是被圆锥截去的球面部分，
    ///
    /// <div>
    /// $$
    /// A = 2 \pi R^2 \left(1 - \cos(\text{central})\right)
    /// $$
    /// </div>
    ///
    /// 对球形天体这是精确的。
    #[must_use]
    pub fn area(&self) -> f64 {
        2.0 * PI * self.body_radius * self.body_radius * (1.0 - self.central_angle.cos())
    }

    /// Returns the fraction of the whole sphere covered by the footprint.
    ///
    /// The value is between zero and one half, because a footprint from a
    /// point above the surface can never cover more than a hemisphere.
    ///
    /// 返回覆盖区占整个球面的比例。
    ///
    /// 该值介于零与二分之一之间，因为来自地表之上某点的覆盖区覆盖范围
    /// 绝不会超过一个半球。
    #[must_use]
    pub fn fraction_of_surface(&self) -> f64 {
        self.area() / (4.0 * PI * self.body_radius * self.body_radius)
    }

    /// Returns `true` when a ground point at the given central angle from
    /// the sub-satellite point lies inside the footprint.
    ///
    /// The comparison is inclusive, so a point exactly on the edge is
    /// inside.
    ///
    /// 当距星下点给定中心角的地面点位于覆盖区内时返回 `true`。
    ///
    /// 比较是含端点的，故恰在边缘上的点算在内部。
    #[must_use]
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
/// 返回覆盖区的中心半角，单位为弧度。
///
/// 这是卫星仍高于仰角遮蔽时的最大中心角。
///
/// 几何不可用时返回 `None`。
#[must_use]
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
/// 返回卫星与仰角遮蔽所对应的覆盖区。
///
/// 几何不可用时返回 `None`。
#[must_use]
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
/// 返回覆盖区的面积，单位为平方千米。
///
/// 这是 [`Footprint::area`] 的自由函数形式。
///
/// 几何不可用时返回 `None`。
#[must_use]
pub fn footprint_area(satellite_radius: f64, body_radius: f64, elevation_mask: f64) -> Option<f64> {
    Some(footprint(satellite_radius, body_radius, elevation_mask)?.area())
}

/// Returns the fraction of the Earth's surface covered by a footprint.
///
/// Returns `None` when the geometry is not usable.
///
/// 返回覆盖区所占地球表面的比例。
///
/// 几何不可用时返回 `None`。
#[must_use]
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
///
/// 返回两个地表点之间的角度，单位为弧度。
///
/// 两矢量必须是相对天体中心、在同一坐标系中的位置。该角度是自中心看
/// 它们之间的中心角。
///
/// 任一矢量为零长或含非有限分量时返回 `None`。
#[must_use]
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
///
/// 当某地面点位于某覆盖区内时返回 `true`。
///
/// 该点与星下点都是相对天体中心的位置，单位为千米。覆盖区提供天体半径
/// 与中心半角。
///
/// 任一矢量退化或非有限时返回 `None`。
#[must_use]
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
/// 返回自地面看覆盖区边缘的仰角，单位为弧度。
///
/// 它是 [`footprint_central_angle`] 的逆，当已知覆盖区而想求它对应的遮蔽时
/// 有用。
///
/// 几何不可用时返回 `None`。
#[must_use]
pub fn footprint_edge_elevation(satellite_radius: f64, footprint: &Footprint) -> Option<f64> {
    elevation_from_central_angle(
        satellite_radius,
        footprint.body_radius,
        footprint.central_angle,
    )
}
