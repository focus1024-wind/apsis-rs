//! Tests for the atmospheric path geometry.
//!
//! 大气路径几何的测试。

use apsis::apsis_atmosphere::{air_mass, elevation_from_zenith, zenith_to_elevation};
use core::f64::consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_6};

const TOLERANCE: f64 = 1e-12;

#[test]
fn the_zenith_has_unit_air_mass() {
    assert!((air_mass(FRAC_PI_2) - 1.0).abs() < TOLERANCE);
    assert!((air_mass(zenith_to_elevation(0.0)) - 1.0).abs() < TOLERANCE);
}

#[test]
fn the_air_mass_is_the_cosecant_of_the_elevation() {
    for elevation in [FRAC_PI_6, FRAC_PI_4, FRAC_PI_2, 0.2, 1.0, 1.4] {
        let expected = 1.0 / elevation.sin();
        assert!(
            (air_mass(elevation) - expected).abs() < 1e-9,
            "elevation {elevation}"
        );
    }
}

#[test]
fn known_elevations_give_known_air_masses() {
    // Thirty degrees is a cosecant of two.
    // 三十度的余割为二。
    assert!((air_mass(FRAC_PI_6) - 2.0).abs() < 1e-9);
    // Forty five degrees is the square root of two.
    // 四十五度是根号二。
    assert!((air_mass(FRAC_PI_4) - 2.0_f64.sqrt()).abs() < 1e-9);
    // Ten degrees is about 5.76.
    // 十度约为 5.76。
    assert!((air_mass(10.0_f64.to_radians()) - 5.758_8).abs() < 1e-3);
}

#[test]
fn a_higher_elevation_has_a_smaller_air_mass() {
    let low = air_mass(5.0_f64.to_radians());
    let high = air_mass(80.0_f64.to_radians());
    assert!(low > high);
}

#[test]
fn the_air_mass_is_infinite_at_or_below_the_horizon() {
    assert!(air_mass(0.0).is_infinite());
    assert!(air_mass(-0.1).is_infinite());
}

#[test]
fn elevation_and_zenith_angle_are_complementary() {
    for angle in [0.0, FRAC_PI_6, FRAC_PI_4, 1.0, FRAC_PI_2] {
        let elevation = elevation_from_zenith(angle);
        assert!((elevation + angle - FRAC_PI_2).abs() < TOLERANCE);
        assert!((zenith_to_elevation(elevation) - angle).abs() < TOLERANCE);
    }
}

#[test]
fn the_conversions_are_involutions() {
    for angle in [0.0, 0.3, 1.0, 1.5, FRAC_PI_2] {
        assert!((zenith_to_elevation(elevation_from_zenith(angle)) - angle).abs() < TOLERANCE);
        assert!((elevation_from_zenith(zenith_to_elevation(angle)) - angle).abs() < TOLERANCE);
    }
}
