//! Tests for footprint geometry and coverage fractions.
//!
//! 覆盖区几何与覆盖比例的测试。

use apsis::apsis_coverage::{
    Footprint, central_angle_between, footprint, footprint_area, footprint_central_angle,
    footprint_edge_elevation, footprint_fraction, point_in_footprint,
};
use apsis::apsis_math::Vec3;
use core::f64::consts::{FRAC_PI_2, PI};

/// The Earth equatorial radius, in kilometers.
///
/// 地球赤道半径，单位为千米。
const BODY_RADIUS: f64 = 6_378.137;

/// A low Earth orbit at 550 km altitude.
///
/// 550 km 高度的低轨。
const LOW_ORBIT: f64 = BODY_RADIUS + 550.0;

/// A geostationary radius.
///
/// 地球静止轨道半径。
const GEOSTATIONARY: f64 = 42_164.0;

const TOLERANCE: f64 = 1e-9;

#[test]
fn the_footprint_at_a_ten_degree_mask_matches_a_hand_computation() {
    // A 550 km satellite at a ten degree mask has a central angle of
    // 14.9566 degrees, a surface radius of 1664.965 km, and an area of
    // 8 659 493.8 square kilometers.
    // 十度遮蔽下 550 km 卫星的中心角为 14.9566 度，地表半径为 1664.965 km，面积为
    // 8 659 493.8 平方千米。
    let mask = 10.0_f64.to_radians();
    let footprint = footprint(LOW_ORBIT, BODY_RADIUS, mask).expect("valid");
    assert!((footprint.central_angle.to_degrees() - 14.956_635_124_957_29).abs() < 1e-9);
    assert!((footprint.surface_radius() - 1_664.965_006_091_034_9).abs() < 1e-6);
    assert!((footprint.area() - 8_659_493.809_336_036).abs() < 1e-3);
    assert!((footprint.fraction_of_surface() - 0.016_939_280_322_558_092).abs() < 1e-12);
}

#[test]
fn the_area_matches_the_spherical_cap_relation() {
    let footprint = footprint(LOW_ORBIT, BODY_RADIUS, 0.3).expect("valid");
    let expected = 2.0 * PI * BODY_RADIUS * BODY_RADIUS * (1.0 - footprint.central_angle.cos());
    assert!((footprint.area() - expected).abs() < TOLERANCE);
}

#[test]
fn the_area_grows_with_the_central_angle() {
    let mut previous = 0.0;
    for mask_degrees in [30.0_f64, 20.0, 10.0, 5.0, 0.0] {
        let area =
            footprint_area(LOW_ORBIT, BODY_RADIUS, mask_degrees.to_radians()).expect("valid");
        assert!(
            area > previous,
            "mask {mask_degrees}: {area} against {previous}"
        );
        previous = area;
    }
}

#[test]
fn a_footprint_can_never_cover_more_than_a_hemisphere() {
    // The limit is a satellite at the surface with no mask, which sees half
    // the sphere. Any real satellite sees less.
    // 极限是地表上、无遮蔽的卫星，它看到半个球面。任何真实卫星看到的都更少。
    for satellite_radius in [LOW_ORBIT, GEOSTATIONARY, 1.0e6] {
        for mask_degrees in [0.0_f64, 10.0, 45.0] {
            let fraction =
                footprint_fraction(satellite_radius, BODY_RADIUS, mask_degrees.to_radians())
                    .expect("valid");
            assert!(
                fraction > 0.0 && fraction <= 0.5,
                "radius {satellite_radius}, mask {mask_degrees}: {fraction}"
            );
        }
    }
}

#[test]
fn a_geostationary_satellite_sees_about_a_third_of_the_earth() {
    // A geostationary satellite at a zero degree mask sees to about 81.3
    // degrees of central angle, which is a fraction of about 0.42.
    // 零度遮蔽的地球静止卫星看到约 81.3 度的中心角，比例约为 0.42。
    let fraction = footprint_fraction(GEOSTATIONARY, BODY_RADIUS, 0.0).expect("valid");
    assert!(fraction > 0.40 && fraction < 0.43, "fraction {fraction}");
}

#[test]
fn containment_is_inclusive_at_the_edge() {
    let footprint = Footprint::new(0.5, BODY_RADIUS);
    assert!(footprint.contains_central_angle(0.0));
    assert!(footprint.contains_central_angle(0.25));
    assert!(footprint.contains_central_angle(0.5));
    assert!(!footprint.contains_central_angle(0.500_001));
    assert!(!footprint.contains_central_angle(-0.1));
    assert!(!footprint.contains_central_angle(f64::NAN));
}

#[test]
fn the_central_angle_between_two_surface_points_is_the_included_angle() {
    // Two points on the equator ninety degrees apart.
    // 赤道上相隔九十度的两个点。
    let first = Vec3::new(BODY_RADIUS, 0.0, 0.0);
    let second = Vec3::new(0.0, BODY_RADIUS, 0.0);
    let angle = central_angle_between(first, second).expect("valid");
    assert!((angle - FRAC_PI_2).abs() < TOLERANCE);

    // A point with itself gives zero.
    // 点与其自身给出零。
    assert!(central_angle_between(first, first).expect("valid").abs() < TOLERANCE);

    // Opposite points give a straight angle.
    // 相对的点给出平角。
    let opposite = Vec3::new(-BODY_RADIUS, 0.0, 0.0);
    assert!((central_angle_between(first, opposite).expect("valid") - PI).abs() < TOLERANCE);
}

#[test]
fn the_central_angle_between_points_is_scale_invariant() {
    // The angle depends only on the directions, not on the radii.
    // 该角只依赖于方向，而不依赖于半径。
    let near = Vec3::new(1.0, 0.5, 0.25);
    let far = near * 1_000.0;
    let other = Vec3::new(0.0, 1.0, 0.0);
    let first = central_angle_between(near, other).expect("valid");
    let second = central_angle_between(far, other).expect("valid");
    assert!((first - second).abs() < 1e-12);
}

#[test]
fn a_degenerate_vector_has_no_central_angle() {
    assert_eq!(central_angle_between(Vec3::ZERO, Vec3::X), None);
    assert_eq!(central_angle_between(Vec3::X, Vec3::ZERO), None);
    assert_eq!(central_angle_between(Vec3::splat(f64::NAN), Vec3::X), None);
}

#[test]
fn point_in_footprint_uses_the_central_angle_from_the_sub_satellite_point() {
    let footprint = Footprint::new(0.1, BODY_RADIUS);
    // The sub-satellite point on the equator.
    // 赤道上的星下点。
    let sub_satellite = Vec3::new(BODY_RADIUS, 0.0, 0.0);

    // A point within the footprint, about 0.05 radians away.
    // 覆盖区内的一个点，约 0.05 弧度之外。
    let inside = Vec3::new(
        BODY_RADIUS * 0.05_f64.cos(),
        BODY_RADIUS * 0.05_f64.sin(),
        0.0,
    );
    assert_eq!(
        point_in_footprint(&footprint, inside, sub_satellite),
        Some(true)
    );

    // A point well outside it, about a quarter radian away.
    // 远在其外的一个点，约四分之一弧度之外。
    let outside = Vec3::new(
        BODY_RADIUS * 0.25_f64.cos(),
        BODY_RADIUS * 0.25_f64.sin(),
        0.0,
    );
    assert_eq!(
        point_in_footprint(&footprint, outside, sub_satellite),
        Some(false)
    );
}

#[test]
fn the_edge_elevation_inverts_the_footprint_central_angle() {
    let mask = 12.0_f64.to_radians();
    let footprint = footprint(LOW_ORBIT, BODY_RADIUS, mask).expect("valid");
    let recovered = footprint_edge_elevation(LOW_ORBIT, &footprint).expect("valid");
    assert!(
        (recovered - mask).abs() < 1e-9,
        "{recovered} against {mask}"
    );
}

#[test]
fn the_footprint_central_angle_helper_matches_the_constructor() {
    let mask = 15.0_f64.to_radians();
    let central = footprint_central_angle(LOW_ORBIT, BODY_RADIUS, mask).expect("valid");
    let footprint = footprint(LOW_ORBIT, BODY_RADIUS, mask).expect("valid");
    assert!((central - footprint.central_angle).abs() < TOLERANCE);
}

#[test]
fn invalid_geometry_is_rejected() {
    assert_eq!(footprint(LOW_ORBIT, 0.0, 0.1), None);
    assert_eq!(footprint(BODY_RADIUS - 1.0, BODY_RADIUS, 0.1), None);
    assert_eq!(footprint(LOW_ORBIT, BODY_RADIUS, f64::NAN), None);
    assert_eq!(footprint_area(LOW_ORBIT, BODY_RADIUS, f64::INFINITY), None);
    assert_eq!(footprint_fraction(LOW_ORBIT, -1.0, 0.1), None);
}
