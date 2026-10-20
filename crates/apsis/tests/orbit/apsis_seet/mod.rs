//! Validation tests for the space environment models.
//!
//! 空间环境模型的验证测试。

use apsis::apsis_math::Vec3;
use apsis::apsis_seet::{
    EARTH_RADIUS_KM, dipole_field, eclipse_fraction, eclipse_fraction as fraction,
    equilibrium_temperature, is_in_eclipse, particle_fluence, sun_direction_unit,
};

const TOLERANCE: f64 = 1e-9;

#[test]
fn the_equatorial_surface_field_is_the_reference_value() {
    let field = dipole_field(Vec3::new(EARTH_RADIUS_KM, 0.0, 0.0)).expect("valid");
    assert!((field.magnitude_nt - 30_000.0).abs() < TOLERANCE);
    assert!((field.magnetic_latitude_rad).abs() < TOLERANCE);
    assert!((field.l_shell - 1.0).abs() < TOLERANCE);
    assert!(field.field_nt.z > 0.0, "the equatorial field points north");
}

#[test]
fn the_polar_surface_field_is_twice_the_equatorial_value() {
    let field = dipole_field(Vec3::new(0.0, 0.0, EARTH_RADIUS_KM)).expect("valid");
    assert!((field.magnitude_nt - 60_000.0).abs() < TOLERANCE);
    assert!(field.field_nt.z < 0.0, "the polar field points south");
    assert!(field.l_shell.is_infinite());
}

#[test]
fn the_field_falls_as_the_inverse_cube() {
    let inner = dipole_field(Vec3::new(EARTH_RADIUS_KM, 0.0, 0.0)).expect("valid");
    let outer = dipole_field(Vec3::new(2.0 * EARTH_RADIUS_KM, 0.0, 0.0)).expect("valid");
    assert!((outer.magnitude_nt - inner.magnitude_nt / 8.0).abs() < TOLERANCE);
    assert!((outer.l_shell - 2.0).abs() < TOLERANCE);
}

#[test]
fn the_l_shell_and_magnetic_latitude_follow_the_dipole() {
    let position = Vec3::new(4_000.0, 0.0, 4_000.0);
    let field = dipole_field(position).expect("valid");
    let latitude = field.magnetic_latitude_rad;
    assert!((latitude - core::f64::consts::FRAC_PI_4).abs() < TOLERANCE);
    let expected = position.norm() / EARTH_RADIUS_KM / latitude.cos().powi(2);
    assert!((field.l_shell - expected).abs() < TOLERANCE);
}

#[test]
fn the_solar_direction_matches_the_known_seasons() {
    let j2000 = sun_direction_unit(2_451_545.0).expect("valid");
    let right_ascension = j2000.y.atan2(j2000.x).to_degrees().rem_euclid(360.0);
    let declination = j2000.z.asin().to_degrees();
    assert!(
        (right_ascension - 281.286).abs() < 0.05,
        "{right_ascension}"
    );
    assert!((declination + 23.033).abs() < 0.05, "{declination}");

    let june = sun_direction_unit(2_451_717.0).expect("valid");
    assert!(
        june.z > 0.39,
        "June declination {}",
        june.z.asin().to_degrees()
    );
    let december = sun_direction_unit(2_451_898.0).expect("valid");
    assert!(december.z < -0.39);
    assert!((june.norm() - 1.0).abs() < TOLERANCE);
}

#[test]
fn eclipse_geometry_matches_the_cylinder() {
    let sun = Vec3::new(1.0, 0.0, 0.0);
    assert!(is_in_eclipse(Vec3::new(-7_000.0, 0.0, 0.0), sun).expect("valid"));
    assert!(!is_in_eclipse(Vec3::new(7_000.0, 0.0, 0.0), sun).expect("valid"));
    assert!(!is_in_eclipse(Vec3::new(-7_000.0, 7_000.0, 0.0), sun).expect("valid"));
    let positions = [
        Vec3::new(-7_000.0, 0.0, 0.0),
        Vec3::new(-7_000.0, 0.0, 0.0),
        Vec3::new(7_000.0, 0.0, 0.0),
        Vec3::new(0.0, 7_000.0, 0.0),
    ];
    assert!((fraction(&positions, sun).expect("valid") - 0.5).abs() < TOLERANCE);
}

#[test]
fn a_leo_orbit_spends_a_plausible_fraction_in_eclipse() {
    let mut positions = Vec::new();
    for index in 0..72 {
        let angle = core::f64::consts::TAU * index as f64 / 72.0;
        positions.push(Vec3::new(7_000.0 * angle.cos(), 7_000.0 * angle.sin(), 0.0));
    }
    let fraction = eclipse_fraction(&positions, Vec3::new(1.0, 0.0, 0.0)).expect("valid");
    assert!((0.3..0.45).contains(&fraction), "{fraction}");
}

#[test]
fn the_plate_temperature_matches_the_radiation_balance() {
    let temperature = equilibrium_temperature(1_361.0, 0.0, 0.0, 0.9, 0.9).expect("valid");
    let expected = (1_361.0 / 5.670_374_419e-8_f64).powf(0.25);
    assert!((temperature - expected).abs() < 1e-9);
    let cooler = equilibrium_temperature(1_361.0, 0.0, 0.0, 0.3, 0.9).expect("valid");
    assert!((cooler - 299.075_807_133_589_14).abs() < 1e-9);
}

#[test]
fn the_fluence_is_the_flux_times_the_duration() {
    assert!((particle_fluence(1.0e8, 3_600.0).expect("valid") - 3.6e11).abs() < 1.0);
}

#[test]
fn invalid_inputs_are_rejected() {
    assert!(dipole_field(Vec3::ZERO).is_err());
    assert!(sun_direction_unit(f64::NAN).is_err());
    assert!(equilibrium_temperature(-1.0, 0.0, 0.0, 0.9, 0.9).is_err());
    assert!(equilibrium_temperature(1_361.0, 2.0, 0.0, 0.9, 0.9).is_err());
    assert!(particle_fluence(-1.0, 10.0).is_err());
}
