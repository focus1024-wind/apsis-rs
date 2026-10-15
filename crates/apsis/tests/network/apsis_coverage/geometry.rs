//! Tests for the satellite geometry of a ground coverage calculation.
//!
//! The expected values are computed independently from the law of sines
//! applied to the triangle formed by the satellite, the ground point, and
//! the centre of the Earth.
//!
//! 地面覆盖计算中卫星几何的测试。
//!
//! 期望值由正弦定理独立计算，应用于卫星、地面点与地心构成的三角形。

use apsis::apsis_coverage::{
    earth_central_angle, elevation_from_central_angle, nadir_angle, slant_range,
};
use core::f64::consts::FRAC_PI_2;

/// The Earth equatorial radius, in kilometers.
///
/// 地球赤道半径，单位为千米。
const BODY_RADIUS: f64 = 6_378.137;

/// A low Earth orbit at 550 km altitude.
///
/// 550 km 高度的低轨。
const LOW_ORBIT: f64 = BODY_RADIUS + 550.0;

/// A medium Earth orbit at 20 200 km altitude, near the navigation
/// constellations.
///
/// 20 200 km 高度的中轨，接近导航星座。
const MEDIUM_ORBIT: f64 = BODY_RADIUS + 20_200.0;

/// A geostationary radius.
///
/// 地球静止轨道半径。
const GEOSTATIONARY: f64 = 42_164.0;

const TOLERANCE: f64 = 1e-9;

#[test]
fn the_angles_at_the_zenith_are_consistent() {
    // Directly below the satellite the elevation is a right angle, the
    // nadir angle is zero, and the central angle is zero.
    // 卫星正下方处仰角为直角，天底角为零，中心角为零。
    assert!(
        nadir_angle(LOW_ORBIT, BODY_RADIUS, FRAC_PI_2)
            .expect("valid")
            .abs()
            < TOLERANCE
    );
    assert!(
        earth_central_angle(LOW_ORBIT, BODY_RADIUS, FRAC_PI_2)
            .expect("valid")
            .abs()
            < TOLERANCE
    );
    // The slant range is the altitude.
    // 斜距即高度。
    assert!(
        (slant_range(LOW_ORBIT, BODY_RADIUS, FRAC_PI_2).expect("valid") - 550.0).abs() < TOLERANCE
    );
}

#[test]
fn the_three_angles_are_complementary() {
    // elevation + nadir + central = 90 degrees, for every geometry.
    // 对每种几何，elevation + nadir + central = 90 度。
    for satellite_radius in [LOW_ORBIT, MEDIUM_ORBIT, GEOSTATIONARY] {
        for elevation_degrees in [0.0_f64, 5.0, 10.0, 30.0, 60.0, 89.0] {
            let elevation = elevation_degrees.to_radians();
            let nadir = nadir_angle(satellite_radius, BODY_RADIUS, elevation).expect("valid");
            let central =
                earth_central_angle(satellite_radius, BODY_RADIUS, elevation).expect("valid");
            let sum = elevation + nadir + central;
            assert!(
                (sum - FRAC_PI_2).abs() < 1e-9,
                "radius {satellite_radius}, elevation {elevation_degrees}: sum {sum}"
            );
        }
    }
}

#[test]
fn the_geometry_matches_a_hand_computation() {
    // A 550 km satellite at a ten degree mask. The nadir angle is
    // asin(Re cos(el) / Rs) = 65.0434 degrees, the central angle is
    // 14.9566 degrees, and the slant range is 1815.652 km.
    // 十度遮蔽下的 550 km 卫星。天底角为 asin(Re cos(el) / Rs) = 65.0434 度，
    // 中心角为 14.9566 度，斜距为 1815.652 km。
    let elevation = 10.0_f64.to_radians();
    assert!(
        (nadir_angle(LOW_ORBIT, BODY_RADIUS, elevation)
            .expect("valid")
            .to_degrees()
            - 65.043_364_875_042_71)
            .abs()
            < 1e-9
    );
    assert!(
        (earth_central_angle(LOW_ORBIT, BODY_RADIUS, elevation)
            .expect("valid")
            .to_degrees()
            - 14.956_635_124_957_29)
            .abs()
            < 1e-9
    );
    assert!(
        (slant_range(LOW_ORBIT, BODY_RADIUS, elevation).expect("valid") - 1_815.652_169_368_375_8)
            .abs()
            < 1e-9
    );
}

#[test]
fn a_higher_satellite_sees_further() {
    let elevation = 10.0_f64.to_radians();
    let low = earth_central_angle(LOW_ORBIT, BODY_RADIUS, elevation).expect("valid");
    let medium = earth_central_angle(MEDIUM_ORBIT, BODY_RADIUS, elevation).expect("valid");
    let high = earth_central_angle(GEOSTATIONARY, BODY_RADIUS, elevation).expect("valid");
    assert!(low < medium, "{low} against {medium}");
    assert!(medium < high, "{medium} against {high}");
}

#[test]
fn a_lower_elevation_mask_sees_further() {
    // Relaxing the mask lets the satellite be seen closer to the horizon,
    // which increases the central angle.
    // 放宽遮蔽使卫星能被看到至更接近地平，这增大中心角。
    let mut previous = 0.0;
    for elevation_degrees in [30.0_f64, 20.0, 10.0, 5.0, 0.0] {
        let central = earth_central_angle(LOW_ORBIT, BODY_RADIUS, elevation_degrees.to_radians())
            .expect("valid");
        assert!(
            central > previous,
            "elevation {elevation_degrees}: {central} against {previous}"
        );
        previous = central;
    }
}

#[test]
fn a_zero_degree_mask_is_the_horizon() {
    // At a zero degree mask the central angle is the horizon angle
    // acos(Re / Rs), which is where the line of sight is tangent to the
    // surface.
    // 零度遮蔽下中心角即地平角 acos(Re / Rs)，那里视线与地表相切。
    let central = earth_central_angle(LOW_ORBIT, BODY_RADIUS, 0.0).expect("valid");
    let horizon = (BODY_RADIUS / LOW_ORBIT).acos();
    assert!((central - horizon).abs() < 1e-9);
}

#[test]
fn the_slant_range_grows_as_the_elevation_falls() {
    // A satellite near the horizon is further away than one overhead.
    // 接近地平的卫星比过顶的更远。
    let overhead = slant_range(LOW_ORBIT, BODY_RADIUS, FRAC_PI_2).expect("valid");
    let low = slant_range(LOW_ORBIT, BODY_RADIUS, 5.0_f64.to_radians()).expect("valid");
    assert!(low > overhead, "{low} against {overhead}");
    assert!(overhead < 600.0);
    assert!(low > 1_000.0);
}

#[test]
fn the_elevation_inverts_the_central_angle() {
    for satellite_radius in [LOW_ORBIT, MEDIUM_ORBIT, GEOSTATIONARY] {
        for elevation_degrees in [0.0_f64, 10.0, 30.0, 70.0] {
            let elevation = elevation_degrees.to_radians();
            let central =
                earth_central_angle(satellite_radius, BODY_RADIUS, elevation).expect("valid");
            let recovered = elevation_from_central_angle(satellite_radius, BODY_RADIUS, central)
                .expect("valid");
            assert!(
                (recovered - elevation).abs() < 1e-9,
                "radius {satellite_radius}, elevation {elevation_degrees}: {recovered}"
            );
        }
    }
}

#[test]
fn the_elevation_is_a_right_angle_when_the_central_angle_is_zero() {
    let elevation = elevation_from_central_angle(LOW_ORBIT, BODY_RADIUS, 0.0).expect("valid");
    assert!((elevation - FRAC_PI_2).abs() < TOLERANCE);
}

#[test]
fn a_satellite_inside_the_body_is_rejected() {
    // A satellite below the surface has no geometry, so the functions
    // report nothing rather than a negative angle.
    // 地表以下的卫星没有几何，故这些函数报告空而非负角。
    assert_eq!(nadir_angle(BODY_RADIUS - 1.0, BODY_RADIUS, 0.0), None);
    assert_eq!(
        earth_central_angle(BODY_RADIUS - 1.0, BODY_RADIUS, 0.0),
        None
    );
    assert_eq!(slant_range(BODY_RADIUS - 1.0, BODY_RADIUS, 0.0), None);
}

#[test]
fn non_finite_arguments_are_rejected() {
    assert_eq!(nadir_angle(f64::NAN, BODY_RADIUS, 0.0), None);
    assert_eq!(nadir_angle(LOW_ORBIT, f64::NAN, 0.0), None);
    assert_eq!(nadir_angle(LOW_ORBIT, BODY_RADIUS, f64::NAN), None);
    assert_eq!(
        earth_central_angle(LOW_ORBIT, BODY_RADIUS, f64::INFINITY),
        None
    );
    assert_eq!(slant_range(LOW_ORBIT, 0.0, 0.0), None);
    assert_eq!(
        elevation_from_central_angle(LOW_ORBIT, BODY_RADIUS, f64::NAN),
        None
    );
}

#[test]
fn a_grazing_geometry_is_handled_at_the_horizon() {
    // At the horizon the nadir angle is the arc sine of the ratio of the
    // radii, which is exactly one when the satellite is at the surface and
    // less than one above it. The clamp keeps the arc sine defined.
    // 在地平处天底角是两半径之比的反正弦，卫星在地表时它恰为一，在其上方时小于一。
    // 该夹逼保持反正弦有定义。
    let nadir = nadir_angle(LOW_ORBIT, BODY_RADIUS, 0.0).expect("valid");
    assert!(nadir.is_finite());
    let expected = (BODY_RADIUS / LOW_ORBIT).asin();
    assert!((nadir - expected).abs() < 1e-9);
}
