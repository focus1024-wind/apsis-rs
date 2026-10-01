//! Tests for the reference ellipsoid.
//!
//! 参考椭球的测试。

use apsis_geodesy::{Ellipsoid, WGS84};

#[test]
fn defining_parameters_match_the_standard() {
    assert_eq!(WGS84.semi_major_axis(), 6_378_137.0);
    assert_eq!(WGS84.inverse_flattening(), 298.257_223_563);
    assert!((WGS84.flattening() - 1.0 / 298.257_223_563).abs() < 1e-15);
}

#[test]
fn derived_axes_match_the_published_values() {
    // The WGS 84 semi-minor axis is 6 356 752.314 245 m.
    // WGS 84 的半短轴为 6 356 752.314 245 m。
    assert!((WGS84.semi_minor_axis() - 6_356_752.314_245).abs() < 1e-6);
    // The first eccentricity is 0.081 819 190 842 622.
    // 第一偏心率为 0.081 819 190 842 622。
    assert!((WGS84.eccentricity() - 0.081_819_190_842_622).abs() < 1e-12);
}

#[test]
fn eccentricity_is_consistent_with_the_flattening() {
    let flattening = WGS84.flattening();
    let expected = 2.0 * flattening - flattening * flattening;
    assert!((WGS84.eccentricity_squared() - expected).abs() < 1e-18);
}

#[test]
fn second_eccentricity_relates_to_the_first() {
    let first = WGS84.eccentricity_squared();
    let second = WGS84.second_eccentricity_squared();
    assert!((second - first / (1.0 - first)).abs() < 1e-18);
    // Derived from the defining parameters this is 0.006 739 496 742 276.
    // 由定义参数导出，该值为 0.006 739 496 742 276。
    assert!((second - 0.006_739_496_742_276_435).abs() < 1e-15);
}

#[test]
fn prime_vertical_radius_spans_the_expected_range() {
    // At the equator it equals the semi-major axis.
    // 在赤道处它等于半长轴。
    assert!((WGS84.prime_vertical_radius(0.0) - WGS84.semi_major_axis()).abs() < 1e-9);
    // At the pole it equals the polar radius of curvature, a / sqrt(1 - e^2),
    // which is 6 399 593.625 9 m.
    // 在极点处它等于极曲率半径 a / sqrt(1 - e^2)，即 6 399 593.625 9 m。
    let pole = WGS84.prime_vertical_radius(core::f64::consts::FRAC_PI_2);
    assert!((pole - WGS84.polar_radius_of_curvature()).abs() < 1e-9);
    assert!((pole - 6_399_593.625_9).abs() < 1e-3);
}

#[test]
fn meridian_radius_spans_the_expected_range() {
    // At the equator it is a (1 - e^2), which is 6 335 439.327 m.
    // 在赤道处它是 a (1 - e^2)，即 6 335 439.327 m。
    let equator = WGS84.meridian_radius(0.0);
    let expected = WGS84.semi_major_axis() * (1.0 - WGS84.eccentricity_squared());
    assert!((equator - expected).abs() < 1e-6);
    assert!((equator - 6_335_439.327_1).abs() < 1e-3);
    // At the pole it is a / sqrt(1 - e^2).
    // 在极点处它是 a / sqrt(1 - e^2)。
    let pole = WGS84.meridian_radius(core::f64::consts::FRAC_PI_2);
    assert!((pole - WGS84.polar_radius_of_curvature()).abs() < 1e-6);
}

#[test]
fn geocentric_latitude_is_zero_at_the_equator_and_pole() {
    assert!(WGS84.geocentric_latitude(0.0).abs() < 1e-12);
    let pole = WGS84.geocentric_latitude(core::f64::consts::FRAC_PI_2);
    assert!((pole - core::f64::consts::FRAC_PI_2).abs() < 1e-12);
}

#[test]
fn geocentric_latitude_is_smaller_than_geodetic_away_from_the_axes() {
    let geodetic = 45.0_f64.to_radians();
    let geocentric = WGS84.geocentric_latitude(geodetic);
    assert!(geocentric < geodetic);
    // The maximum separation is about 0.19 degrees at 45 degrees.
    // 最大差约在 45 度处为 0.19 度。
    let difference = (geodetic - geocentric).to_degrees();
    assert!((difference - 0.1924).abs() < 0.001, "got {difference}");
}

#[test]
fn a_sphere_has_no_flattening_or_eccentricity() {
    let sphere = Ellipsoid::new(1_000.0, f64::INFINITY);
    assert_eq!(sphere.flattening(), 0.0);
    assert_eq!(sphere.eccentricity_squared(), 0.0);
    assert!((sphere.semi_minor_axis() - 1_000.0).abs() < 1e-12);
    assert!((sphere.prime_vertical_radius(0.7) - 1_000.0).abs() < 1e-12);
    assert!((sphere.meridian_radius(0.7) - 1_000.0).abs() < 1e-12);
}
