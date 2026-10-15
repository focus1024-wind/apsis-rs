//! Tests for geodetic and Cartesian conversions.
//!
//! 大地与直角坐标换算的测试。

use apsis::apsis_geodesy::{Geodetic, WGS84, geodetic_from_cartesian};
use apsis::apsis_math::Vec3;

/// Tolerance for position comparisons, in meters.
///
/// 位置比较的容差，单位为米。
const POSITION_TOL: f64 = 1e-4;

/// Tolerance for latitude and longitude comparisons, in radians.
///
/// One microdegree is about 1.1e-8 radians, or roughly a centimeter on the
/// Earth's surface.
///
/// 纬度与经度比较的容差，单位为弧度。
///
/// 一微度约为 1.1e-8 弧度，或地表上大致一厘米。
const ANGLE_TOL: f64 = 1e-11;

#[test]
fn equatorial_point_lies_on_the_semi_major_axis() {
    let point = Geodetic::new(0.0, 0.0, 0.0).to_cartesian(WGS84);
    assert!((point.x - WGS84.semi_major_axis()).abs() < POSITION_TOL);
    assert!(point.y.abs() < POSITION_TOL);
    assert!(point.z.abs() < POSITION_TOL);
}

#[test]
fn pole_lies_on_the_semi_minor_axis() {
    let point = Geodetic::new(core::f64::consts::FRAC_PI_2, 0.0, 0.0).to_cartesian(WGS84);
    assert!(point.x.abs() < POSITION_TOL);
    assert!(point.y.abs() < POSITION_TOL);
    assert!((point.z - WGS84.semi_minor_axis()).abs() < POSITION_TOL);
}

#[test]
fn longitude_quarter_turn_places_the_point_on_the_second_axis() {
    let point = Geodetic::new(0.0, core::f64::consts::FRAC_PI_2, 0.0).to_cartesian(WGS84);
    assert!(point.x.abs() < POSITION_TOL);
    assert!((point.y - WGS84.semi_major_axis()).abs() < POSITION_TOL);
}

#[test]
fn height_moves_the_point_along_the_ellipsoid_normal() {
    let surface = Geodetic::new(0.6, 0.3, 0.0).to_cartesian(WGS84);
    let elevated = Geodetic::new(0.6, 0.3, 500.0).to_cartesian(WGS84);
    assert!((surface.distance(elevated) - 500.0).abs() < POSITION_TOL);
    // The normal at a non-polar latitude is not vertical, so the height
    // changes the horizontal components as well as the third one.
    // 非极地纬度处的法线并非竖直，故高度既改变第三分量也改变水平分量。
    assert!(elevated.x > surface.x);
}

#[test]
fn conversion_round_trips_over_a_global_grid() {
    for latitude_degrees in [-89.9_f64, -60.0, -45.0, 0.0, 30.0, 45.0, 60.0, 89.9] {
        for longitude_degrees in [-179.0_f64, -90.0, 0.0, 45.0, 90.0, 179.0] {
            for height in [-1_000.0_f64, 0.0, 1_000.0, 35_786_000.0] {
                let original = Geodetic::new(
                    latitude_degrees.to_radians(),
                    longitude_degrees.to_radians(),
                    height,
                );
                let cartesian = original.to_cartesian(WGS84);
                let recovered = geodetic_from_cartesian(cartesian, WGS84)
                    .expect("off-axis points have a longitude");
                assert!(
                    (recovered.latitude - original.latitude).abs() < ANGLE_TOL,
                    "latitude mismatch at {latitude_degrees}, {longitude_degrees}, {height}"
                );
                assert!(
                    (recovered.longitude - original.longitude).abs() < ANGLE_TOL,
                    "longitude mismatch at {latitude_degrees}, {longitude_degrees}, {height}"
                );
                assert!(
                    (recovered.height - original.height).abs() < POSITION_TOL,
                    "height mismatch at {latitude_degrees}, {longitude_degrees}, {height}: {} vs {}",
                    recovered.height,
                    original.height
                );
            }
        }
    }
}

#[test]
fn round_trip_preserves_the_cartesian_position() {
    let original = Geodetic::new(0.921, -1.234, 12_345.0).to_cartesian(WGS84);
    let recovered = geodetic_from_cartesian(original, WGS84)
        .expect("off-axis points have a longitude")
        .to_cartesian(WGS84);
    assert!(original.distance(recovered) < POSITION_TOL);
}

#[test]
fn polar_axis_has_no_defined_longitude() {
    let on_axis = Vec3::new(0.0, 0.0, WGS84.semi_minor_axis());
    assert_eq!(geodetic_from_cartesian(on_axis, WGS84), None);
    // The origin is also on the axis.
    // 原点也在该轴上。
    assert_eq!(geodetic_from_cartesian(Vec3::ZERO, WGS84), None);
}

#[test]
fn non_finite_positions_are_rejected() {
    let invalid = Vec3::new(f64::NAN, 0.0, 6_000_000.0);
    assert_eq!(geodetic_from_cartesian(invalid, WGS84), None);
    let infinite = Vec3::new(f64::INFINITY, 0.0, 0.0);
    assert_eq!(geodetic_from_cartesian(infinite, WGS84), None);
}

#[test]
fn known_point_matches_an_independent_computation() {
    // Latitude 37.5 degrees north, longitude 121.333333... degrees west, at
    // zero height, evaluated with the defining WGS 84 parameters in
    // extended precision.
    // 北纬 37.5 度、西经 121.333333... 度、零高度，用 WGS 84 定义参数以扩展精度
    // 求值。
    let point = Geodetic::new(
        37.5_f64.to_radians(),
        -121.333_333_333_333_33_f64.to_radians(),
        0.0,
    )
    .to_cartesian(WGS84);
    assert!(
        (point.x + 2_634_612.181_141_605_6).abs() < 1e-6,
        "x = {}",
        point.x
    );
    assert!(
        (point.y + 4_327_502.253_728_065).abs() < 1e-6,
        "y = {}",
        point.y
    );
    assert!(
        (point.z - 3_861_564.098_809_606_4).abs() < 1e-6,
        "z = {}",
        point.z
    );
}

#[test]
fn validity_is_reported() {
    assert!(Geodetic::new(0.5, 0.5, 100.0).is_valid());
    assert!(Geodetic::new(core::f64::consts::FRAC_PI_2, 0.0, 0.0).is_valid());
    assert!(!Geodetic::new(2.0, 0.0, 0.0).is_valid());
    assert!(!Geodetic::new(0.0, 4.0, 0.0).is_valid());
    assert!(!Geodetic::new(0.0, 0.0, f64::NAN).is_valid());
}
