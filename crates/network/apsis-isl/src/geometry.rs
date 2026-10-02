//! Link geometry and the Earth blocking test.
//!
//! 链路几何与地球遮挡判据。

use crate::link::LinkGeometry;
use apsis_math::Vec3;

/// Returns the closest distance from the origin to the line segment between
/// two points.
///
/// This is the perpendicular distance when the closest point lies between
/// the two ends, and the distance to the nearer end otherwise. For an
/// inter-satellite link the first case is the one that matters, because the
/// blocking body lies between the terminals.
///
/// Returns `None` when either point is not finite.
///
/// 返回原点到两点连线的最短距离。
///
/// 最近点落在两端之间时它是垂距，否则是到较近一端的距离。对星间链路
/// 而言前一种情形是要紧的，因为遮挡天体位于两端之间。
///
/// 任一点非有限时返回 `None`。
#[must_use]
pub fn closest_approach_to_centre(first: Vec3, second: Vec3) -> Option<f64> {
    if !first.is_finite() || !second.is_finite() {
        return None;
    }
    let segment = second - first;
    let length_squared = segment.norm_squared();
    if length_squared == 0.0 {
        // The two points coincide, so the distance is the radius of that
        // point.
        // 两点重合，故距离即该点的半径。
        return Some(first.norm());
    }

    // The parameter of the closest point on the infinite line, clamped to
    // the segment.
    // 无限直线上最接近点的参数，被夹逼到该线段。
    let parameter = (-first.dot(segment) / length_squared).clamp(0.0, 1.0);
    let closest = first + segment * parameter;
    Some(closest.norm())
}

/// Returns `true` when the two satellites can see each other past the body.
///
/// The test is whether the line segment between the two positions stays
/// outside a sphere of the given radius. A link that grazes the surface
/// exactly is treated as clear, because the limit is the geometric horizon.
///
/// Returns `None` when either position is not finite or the body radius is
/// not positive.
///
/// 当两颗卫星能越过该天体互相可见时返回 `true`。
///
/// 判据是两位置之间的线段是否保持在给定半径的球体之外。恰好擦过表面
/// 的链路被视为通畅，因为该极限即几何地平线。
///
/// 任一位置非有限或天体半径非正时返回 `None`。
#[must_use]
pub fn mutual_visibility(first: Vec3, second: Vec3, body_radius: f64) -> Option<bool> {
    if !body_radius.is_finite() || body_radius <= 0.0 {
        return None;
    }
    let closest = closest_approach_to_centre(first, second)?;
    Some(closest >= body_radius)
}

/// Returns the full geometry of a link between two satellites.
///
/// The directions are unit vectors from each satellite toward the other. The
/// closest approach to the centre is included so that the caller can judge
/// how much margin the link has.
///
/// Returns `None` when either position is not finite, when the two coincide,
/// or when the body radius is not usable. A cell that reports its own
/// position for both ends has no link, so that case is rejected rather than
/// returned with a zero range.
///
/// 返回两颗卫星之间链路的完整几何。
///
/// 各方向是自每颗卫星指向对方的单位矢量。其中含对中心的最接近距离，使
/// 调用方能判断该链路有多少余量。
///
/// 任一位置非有限、两者重合或天体半径不可用时返回 `None`。两端回报
/// 自身位置的单元没有链路，故该情形被拒绝而非以零距离返回。
#[must_use]
pub fn link_geometry(first: Vec3, second: Vec3, body_radius: f64) -> Option<LinkGeometry> {
    if !body_radius.is_finite() || body_radius < 0.0 {
        return None;
    }
    if !first.is_finite() || !second.is_finite() {
        return None;
    }
    let separation = second - first;
    let range = separation.norm();
    if range == 0.0 {
        return None;
    }
    Some(LinkGeometry {
        range_km: range,
        direction_first_to_second: separation / range,
        direction_second_to_first: -separation / range,
        centre_distance_km: closest_approach_to_centre(first, second)?,
    })
}

/// Returns `true` when the Sun is within the field of view of an optical
/// link.
///
/// The test is the angle between the link direction and the direction to the
/// Sun. If that angle is within the field of view at either terminal, the
/// Sun is in the beam and the link cannot be used.
///
/// The field of view is given as a half angle in radians, which is the
/// acceptance cone of the telescope. A typical optical terminal has a half
/// angle of a few microradians.
///
/// The direction to the Sun is a unit vector in the same frame as the
/// positions. This crate does not compute it, because that needs an
/// ephemeris of the Sun, which the orbit layer does not currently provide; a
/// caller with such an ephemeris supplies the direction.
///
/// Returns `None` when the directions or the field of view are not usable.
///
/// 当太阳位于某光链路的视场内时返回 `true`。
///
/// 判据是链路方向与太阳方向之间的夹角。若该夹角落在任一终端的视场内，则太阳进入
/// 波束，该链路不可用。
///
/// 视场以半角（弧度）给出，即望远镜的接受锥角。典型光终端的半角为若干微弧度。
///
/// 太阳方向是与位置同系的单位矢量。本 crate 不计算它，因为这需要太阳星历，而
/// orbit 层当前不提供；拥有该星历的调用方自行提供该方向。
///
/// 方向或视场不可用时返回 `None`。
#[must_use]
pub fn sun_blocks_link(
    link: &LinkGeometry,
    sun_direction: Vec3,
    field_of_view: f64,
) -> Option<bool> {
    if !sun_direction.is_finite() || !field_of_view.is_finite() {
        return None;
    }
    if field_of_view < 0.0 {
        return None;
    }
    let sun = sun_direction.normalize()?;
    let forward = link.direction_first_to_second.normalize()?;
    let backward = link.direction_second_to_first.normalize()?;
    let forward_angle = forward.dot(sun).clamp(-1.0, 1.0).acos();
    let backward_angle = backward.dot(sun).clamp(-1.0, 1.0).acos();
    Some(forward_angle <= field_of_view || backward_angle <= field_of_view)
}
