//! Link geometry and the Earth blocking test.

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
pub fn closest_approach_to_centre(first: Vec3, second: Vec3) -> Option<f64> {
    if !first.is_finite() || !second.is_finite() {
        return None;
    }
    let segment = second - first;
    let length_squared = segment.norm_squared();
    if length_squared == 0.0 {
        // The two points coincide, so the distance is the radius of that
        // point.
        return Some(first.norm());
    }

    // The parameter of the closest point on the infinite line, clamped to
    // the segment.
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
