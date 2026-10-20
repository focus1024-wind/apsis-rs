//! Validation tests for sensor fields of view.
//!
//! 传感器视场的验证测试。

use apsis::apsis_math::Vec3;
use apsis::apsis_sensor::{ground_footprint_angle, observe_cone, observe_rectangular};

const TOLERANCE: f64 = 1e-9;

#[test]
fn a_target_on_the_boresight_is_visible() {
    let observation = observe_cone(
        Vec3::ZERO,
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(100.0, 0.0, 0.0),
        core::f64::consts::FRAC_PI_6,
        0.0,
        f64::INFINITY,
    )
    .expect("valid");
    assert!(observation.visible);
    assert!(observation.off_axis_rad.abs() < TOLERANCE);
    assert!((observation.range_km - 100.0).abs() < TOLERANCE);
}

#[test]
fn a_target_at_the_cone_edge_is_visible_and_beyond_it_is_not() {
    let angle = 10.0_f64.to_radians();
    let edge = Vec3::new(angle.cos(), angle.sin(), 0.0) * 100.0;
    let outside = Vec3::new(angle.cos(), (angle + 0.01).sin(), 0.0) * 100.0;
    let boresight = Vec3::new(1.0, 0.0, 0.0);
    assert!(
        observe_cone(Vec3::ZERO, boresight, edge, angle, 0.0, f64::INFINITY)
            .expect("valid")
            .visible
    );
    assert!(
        !observe_cone(Vec3::ZERO, boresight, outside, angle, 0.0, f64::INFINITY)
            .expect("valid")
            .visible
    );
}

#[test]
fn the_range_limits_are_inclusive() {
    let target = Vec3::new(100.0, 0.0, 0.0);
    assert!(
        !observe_cone(
            Vec3::ZERO,
            Vec3::X,
            target,
            core::f64::consts::FRAC_PI_6,
            0.0,
            50.0
        )
        .expect("valid")
        .visible
    );
    assert!(
        observe_cone(
            Vec3::ZERO,
            Vec3::X,
            target,
            core::f64::consts::FRAC_PI_6,
            100.0,
            100.0
        )
        .expect("valid")
        .visible
    );
}

#[test]
fn a_rectangular_field_uses_horizontal_and_vertical_angles() {
    let inside = Vec3::new(0.2, 0.1, 1.0);
    let outside_horizontal = Vec3::new(1.0, 0.0, 1.0);
    let observation = observe_rectangular(
        Vec3::ZERO,
        Vec3::Z,
        Vec3::Y,
        inside,
        15.0_f64.to_radians(),
        10.0_f64.to_radians(),
        0.0,
        f64::INFINITY,
    )
    .expect("valid");
    assert!(observation.visible, "{observation:?}");
    assert!(observation.horizontal_rad.unwrap().abs() < 15.0_f64.to_radians());
    assert!(observation.vertical_rad.unwrap().abs() < 10.0_f64.to_radians());
    assert!(
        !observe_rectangular(
            Vec3::ZERO,
            Vec3::Z,
            Vec3::Y,
            outside_horizontal,
            15.0_f64.to_radians(),
            10.0_f64.to_radians(),
            0.0,
            f64::INFINITY
        )
        .expect("valid")
        .visible
    );
}

#[test]
fn a_target_behind_the_rectangular_sensor_is_not_visible() {
    let observation = observe_rectangular(
        Vec3::ZERO,
        Vec3::Z,
        Vec3::Y,
        Vec3::new(0.0, 0.0, -1.0),
        30.0_f64.to_radians(),
        30.0_f64.to_radians(),
        0.0,
        f64::INFINITY,
    )
    .expect("valid");
    assert!(!observation.visible);
    assert!(!observation.within_field);
}

#[test]
fn the_cone_footprint_matches_the_sphere_intersection() {
    let angle = ground_footprint_angle(550.0, 5.0_f64.to_radians()).expect("valid");
    assert!(
        (angle.to_degrees() - 0.432_889_639_003_508_4).abs() < 1e-9,
        "{}",
        angle.to_degrees()
    );
    assert_eq!(ground_footprint_angle(550.0, 0.0).expect("valid"), 0.0);
    let horizon = (6_371.0 / 6_921.0_f64).acos();
    assert!(
        (ground_footprint_angle(550.0, 90.0_f64.to_radians()).expect("valid") - horizon).abs()
            < TOLERANCE
    );
}

#[test]
fn invalid_geometry_is_rejected() {
    assert!(observe_cone(Vec3::ZERO, Vec3::ZERO, Vec3::X, 0.1, 0.0, 1.0).is_err());
    assert!(observe_cone(Vec3::ZERO, Vec3::X, Vec3::ZERO, 0.1, 0.0, 1.0).is_err());
    assert!(
        observe_rectangular(Vec3::ZERO, Vec3::Z, Vec3::Z, Vec3::X, 0.1, 0.1, 0.0, 1.0).is_err()
    );
    assert!(ground_footprint_angle(-1.0, 0.1).is_err());
}
