//! Tests for inter-satellite link geometry.
//!
//! For two satellites at the same radius separated by a central angle
//! `theta`, the closest approach of the chord to the centre is
//! `Rs cos(theta / 2)`. That closed form is used as the reference here.
//!
//! 星间链路几何的测试。
//!
//! 对同一半径、相隔中心角 `theta` 的两颗卫星，弦到中心的最接近距离为
//! `Rs cos(theta / 2)`。此处以该闭式作为参考。

use apsis_isl::{
    LinkState, closest_approach_to_centre, link_geometry, mutual_visibility, sun_blocks_link,
};
use apsis_math::Vec3;
use core::f64::consts::{FRAC_PI_2, PI};

/// The Earth equatorial radius, in kilometers.
///
/// 地球赤道半径，单位为千米。
const BODY_RADIUS: f64 = 6_378.137;

/// A 550 km circular orbit radius, in kilometers.
///
/// 550 km 圆轨道的半径，单位为千米。
const ORBIT_RADIUS: f64 = BODY_RADIUS + 550.0;

/// Returns a satellite on the equator at the given central angle from the
/// first axis.
///
/// 返回赤道上、距第一轴给定中心角的一颗卫星。
fn satellite_at(central_angle: f64) -> Vec3 {
    Vec3::new(
        ORBIT_RADIUS * central_angle.cos(),
        ORBIT_RADIUS * central_angle.sin(),
        0.0,
    )
}

#[test]
fn the_closest_approach_matches_the_chord_geometry() {
    // Two satellites at the same radius separated by a central angle have a
    // closest approach of Rs cos(theta / 2).
    // 同一半径、相隔中心角的两颗卫星其最接近距离为 Rs cos(theta / 2)。
    for degrees in [0.0_f64, 10.0, 30.0, 60.0, 120.0, 180.0] {
        let central = degrees.to_radians();
        let first = satellite_at(0.0);
        let second = satellite_at(central);
        let closest = closest_approach_to_centre(first, second).expect("valid positions");
        let expected = ORBIT_RADIUS * (central / 2.0).cos();
        assert!(
            (closest - expected).abs() < 1e-6,
            "central {degrees}: {closest} against {expected}"
        );
    }
}

#[test]
fn the_closest_approach_is_the_distance_for_coincident_points() {
    let point = Vec3::new(3.0, 4.0, 0.0);
    let closest = closest_approach_to_centre(point, point).expect("valid");
    assert!((closest - 5.0).abs() < 1e-12);
}

#[test]
fn the_closest_approach_is_the_nearer_end_when_the_perpendicular_falls_outside() {
    // Two points on the same side of the origin, so the foot of the
    // perpendicular lies outside the segment.
    // 两点在原点同一侧，故垂足落在该线段之外。
    let first = Vec3::new(10.0, 0.0, 0.0);
    let second = Vec3::new(20.0, 0.0, 0.0);
    let closest = closest_approach_to_centre(first, second).expect("valid");
    assert!((closest - 10.0).abs() < 1e-12);
}

#[test]
fn satellites_on_the_same_side_see_each_other() {
    for degrees in [10.0_f64, 20.0, 30.0] {
        let first = satellite_at(0.0);
        let second = satellite_at(degrees.to_radians());
        assert_eq!(
            mutual_visibility(first, second, BODY_RADIUS),
            Some(true),
            "central {degrees}"
        );
    }
}

#[test]
fn satellites_on_opposite_sides_do_not() {
    for degrees in [60.0_f64, 120.0, 180.0] {
        let first = satellite_at(0.0);
        let second = satellite_at(degrees.to_radians());
        assert_eq!(
            mutual_visibility(first, second, BODY_RADIUS),
            Some(false),
            "central {degrees}"
        );
    }
}

#[test]
fn the_visibility_threshold_is_where_the_chord_grazes_the_surface() {
    // The chord has a closest approach equal to the body radius when
    // cos(theta / 2) = Re / Rs.
    // 当 cos(theta / 2) = Re / Rs 时弦的最接近距离等于天体半径。
    let threshold = 2.0 * (BODY_RADIUS / ORBIT_RADIUS).acos();
    let first = satellite_at(0.0);

    // Just inside the threshold the link is clear.
    // 恰在阈值之内链路通畅。
    let inside = satellite_at(threshold - 1e-6);
    assert_eq!(mutual_visibility(first, inside, BODY_RADIUS), Some(true));

    // Just outside it is blocked.
    // 恰在其外被遮挡。
    let outside = satellite_at(threshold + 1e-6);
    assert_eq!(mutual_visibility(first, outside, BODY_RADIUS), Some(false));

    // Exactly at the threshold the grazing link counts as clear.
    // 恰在阈值处擦过的链路算作通畅。
    let grazing = satellite_at(threshold);
    assert_eq!(mutual_visibility(first, grazing, BODY_RADIUS), Some(true));
}

#[test]
fn a_higher_orbit_allows_a_wider_link() {
    // Two satellites at the same radius can see each other while the central
    // angle between them is less than 2 acos(Re / Rs). For a 550 km orbit
    // that limit is 45.97 degrees, and for a 20 200 km orbit it is 152.23
    // degrees. A separation of 90 degrees therefore falls between the two.
    // 同一半径的两颗卫星在它们之间的中心角小于 2 acos(Re / Rs) 时互相可见。对
    // 550 km 轨道该极限为 45.97 度，对 20 200 km 轨道为 152.23 度。故 90 度的间隔
    // 落在两者之间。
    let central = FRAC_PI_2;
    let low_radius = BODY_RADIUS + 550.0;
    let high_radius = BODY_RADIUS + 20_200.0;

    let low_first = Vec3::new(low_radius, 0.0, 0.0);
    let low_second = Vec3::new(low_radius * central.cos(), low_radius * central.sin(), 0.0);
    assert_eq!(
        mutual_visibility(low_first, low_second, BODY_RADIUS),
        Some(false)
    );

    let high_first = Vec3::new(high_radius, 0.0, 0.0);
    let high_second = Vec3::new(
        high_radius * central.cos(),
        high_radius * central.sin(),
        0.0,
    );
    assert_eq!(
        mutual_visibility(high_first, high_second, BODY_RADIUS),
        Some(true)
    );
}

#[test]
fn the_link_geometry_reports_range_and_directions() {
    let first = Vec3::new(7_000.0, 0.0, 0.0);
    let second = Vec3::new(7_000.0, 3_000.0, 0.0);
    let geometry = link_geometry(first, second, BODY_RADIUS).expect("valid");

    // The range is the distance between the positions.
    // 距离是两位置之间的距离。
    assert!((geometry.range_km - first.distance(second)).abs() < 1e-12);
    // The two directions are opposite unit vectors.
    // 两个方向是相反的单位矢量。
    assert!((geometry.direction_first_to_second.norm() - 1.0).abs() < 1e-12);
    assert!((geometry.direction_second_to_first.norm() - 1.0).abs() < 1e-12);
    assert!(
        (geometry.direction_first_to_second + geometry.direction_second_to_first).norm() < 1e-12
    );
    // The direction points from the first to the second.
    // 方向从第一颗指向第二颗。
    let expected = (second - first).normalize().expect("non-zero separation");
    assert!((geometry.direction_first_to_second - expected).norm() < 1e-12);
}

#[test]
fn the_link_geometry_reports_the_surface_clearance() {
    let first = satellite_at(0.0);
    let second = satellite_at(20.0_f64.to_radians());
    let geometry = link_geometry(first, second, BODY_RADIUS).expect("valid");
    let clearance = geometry.surface_clearance(BODY_RADIUS);
    assert!(clearance > 0.0, "clearance {clearance}");
    assert!(geometry.is_clear(BODY_RADIUS));

    let blocked = link_geometry(first, satellite_at(PI), BODY_RADIUS).expect("valid");
    assert!(blocked.surface_clearance(BODY_RADIUS) < 0.0);
    assert!(!blocked.is_clear(BODY_RADIUS));
}

#[test]
fn coincident_satellites_have_no_link() {
    let point = Vec3::new(7_000.0, 0.0, 0.0);
    assert_eq!(link_geometry(point, point, BODY_RADIUS), None);
}

#[test]
fn invalid_link_arguments_are_rejected() {
    let first = satellite_at(0.0);
    let second = satellite_at(0.1);
    assert_eq!(mutual_visibility(first, second, 0.0), None);
    assert_eq!(mutual_visibility(first, second, -1.0), None);
    assert_eq!(
        mutual_visibility(Vec3::splat(f64::NAN), second, BODY_RADIUS),
        None
    );
    assert_eq!(
        mutual_visibility(first, Vec3::splat(f64::INFINITY), BODY_RADIUS),
        None
    );
    assert_eq!(link_geometry(first, second, -1.0), None);
    assert_eq!(
        closest_approach_to_centre(Vec3::splat(f64::NAN), second),
        None
    );
}

#[test]
fn a_link_along_a_radius_has_the_altitude_as_its_closest_approach() {
    // Two satellites on the same radial line, one above the other. The
    // lower one is the closest point of the segment.
    // 同一径向线上的两颗卫星，一颗在另一颗之上。较低者是该线段的最接近点。
    let low = Vec3::new(7_000.0, 0.0, 0.0);
    let high = Vec3::new(8_000.0, 0.0, 0.0);
    let closest = closest_approach_to_centre(low, high).expect("valid");
    assert!((closest - 7_000.0).abs() < 1e-12);
}

#[test]
fn the_sun_blocks_a_link_when_it_lies_along_it() {
    let first = Vec3::new(7_000.0, 0.0, 0.0);
    let second = Vec3::new(7_000.0, 1_000.0, 0.0);
    let geometry = link_geometry(first, second, BODY_RADIUS).expect("valid");
    let field_of_view = 1e-5;

    // The Sun in the same direction as the link, seen from the first
    // terminal.
    // 从第一终端看，太阳与链路同方向。
    let along = geometry.direction_first_to_second;
    assert_eq!(sun_blocks_link(&geometry, along, field_of_view), Some(true));

    // The Sun behind the link, seen from the second terminal.
    // 从第二终端看，太阳在链路之后。
    let behind = geometry.direction_second_to_first;
    assert_eq!(
        sun_blocks_link(&geometry, behind, field_of_view),
        Some(true)
    );

    // The Sun at right angles to the link.
    // 太阳与链路成直角。
    let transverse = Vec3::new(0.0, 0.0, 1.0);
    assert_eq!(
        sun_blocks_link(&geometry, transverse, field_of_view),
        Some(false)
    );
}

#[test]
fn a_zero_field_of_view_blocks_only_an_exact_alignment() {
    let geometry = link_geometry(
        Vec3::new(7_000.0, 0.0, 0.0),
        Vec3::new(7_000.0, 1_000.0, 0.0),
        BODY_RADIUS,
    )
    .expect("valid");
    // Exactly along the link, the angle is zero, which is within a zero
    // field of view.
    // 恰沿链路时夹角为零，落在零视场内。
    let along = geometry.direction_first_to_second;
    assert_eq!(sun_blocks_link(&geometry, along, 0.0), Some(true));
    // A tiny offset is outside a zero field of view.
    // 极小偏移就在零视场之外。
    let offset = (along + Vec3::new(0.0, 0.0, 0.01))
        .normalize()
        .expect("valid");
    assert_eq!(sun_blocks_link(&geometry, offset, 0.0), Some(false));
}

#[test]
fn the_sun_test_rejects_invalid_arguments() {
    let geometry = link_geometry(
        Vec3::new(7_000.0, 0.0, 0.0),
        Vec3::new(7_000.0, 1_000.0, 0.0),
        BODY_RADIUS,
    )
    .expect("valid");
    assert_eq!(
        sun_blocks_link(&geometry, Vec3::splat(f64::NAN), 1e-5),
        None
    );
    assert_eq!(
        sun_blocks_link(&geometry, Vec3::new(0.0, 0.0, 1.0), -1.0),
        None
    );
    assert_eq!(sun_blocks_link(&geometry, Vec3::ZERO, 1e-5), None);
}

#[test]
fn the_link_state_reports_usability() {
    assert!(LinkState::Available.is_usable());
    assert!(!LinkState::Blocked.is_usable());
    assert!(!LinkState::SunInField.is_usable());
    assert!(!LinkState::NotEstablished.is_usable());

    assert_eq!(LinkState::Available.name(), "available");
    assert_eq!(LinkState::Blocked.name(), "blocked");
    assert_eq!(LinkState::SunInField.name(), "sun in field");
    assert_eq!(LinkState::NotEstablished.name(), "not established");
}

#[test]
fn a_link_between_neighbouring_planes_is_clear() {
    // Two satellites in adjacent planes of a Walker pattern at 550 km are
    // separated by about 120 degrees of right ascension at the equator,
    // which at the same radius would be blocked. Satellites that are near
    // each other in time are much closer, which is what makes cross-plane
    // links viable only between nearby satellites.
    // 550 km 的 Walker 模式中相邻两面的两颗卫星，在赤道处相隔约 120 度赤经，同一
    // 半径下这会被遮挡。在时间上彼此接近的卫星要近得多，这正是跨面链路只在邻近卫星
    // 之间可行的原因。
    let first = satellite_at(0.0);
    let neighbour = satellite_at(5.0_f64.to_radians());
    assert_eq!(mutual_visibility(first, neighbour, BODY_RADIUS), Some(true));
    let far = satellite_at(FRAC_PI_2);
    assert_eq!(mutual_visibility(first, far, BODY_RADIUS), Some(false));
}
