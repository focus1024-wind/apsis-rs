//! Tests for the Earth Rotation Angle and the frame rotations.
//!
//! 地球自转角与坐标架转动的测试。

use apsis::apsis_frames::{
    ERA_RATE_TURNS_PER_DAY, EarthRotationAngle, rotation_ecef_to_eci, rotation_eci_to_ecef,
    ut1_days_from_j2000,
};
use apsis::apsis_math::Vec3;
use core::f64::consts::PI;

const EPS: f64 = 1e-12;

#[test]
fn rate_matches_the_published_value() {
    // 1.002 737 811 911 354 6 turns per day is 360.985 612 288 088 degrees
    // per day.
    // 每天 1.002 737 811 911 354 6 周即每天 360.985 612 288 088 度。
    assert!((ERA_RATE_TURNS_PER_DAY - 1.002_737_811_911_354_6).abs() < 1e-15);
    let degrees = ERA_RATE_TURNS_PER_DAY * 360.0;
    assert!(
        (degrees - 360.985_612_288_088).abs() < 1e-9,
        "got {degrees}"
    );
}

#[test]
fn angle_advances_by_the_rate_each_day() {
    let first = EarthRotationAngle::from_ut1_days(0.0);
    let second = EarthRotationAngle::from_ut1_days(1.0);
    assert!((second.turns() - first.turns() - ERA_RATE_TURNS_PER_DAY).abs() < 1e-15);
    // The angle advances a little under one degree per day more than a
    // full turn.
    // 该角每天推进一整圈再多略小于一度。
    let advance_degrees = (second.turns() - first.turns()) * 360.0 - 360.0;
    assert!((advance_degrees - 0.985_612_288_088).abs() < 1e-9);
}

#[test]
fn epoch_angle_matches_the_published_constant() {
    // At J2000.0 the angle is 0.779 057 273 2640 turns, about 280.46
    // degrees.
    // 在 J2000.0 处该角为 0.779 057 273 2640 周，约 280.46 度。
    let epoch = EarthRotationAngle::from_ut1_days(0.0);
    assert!((epoch.turns() - 0.779_057_273_264).abs() < 1e-15);
    let degrees = epoch.turns_positive() * 360.0;
    assert!((degrees - 280.460_618_375).abs() < 1e-9, "got {degrees}");
}

#[test]
fn julian_date_and_day_count_agree() {
    // J2000.0 is Julian Date 2451545.0.
    // J2000.0 是儒略日 2451545.0。
    assert_eq!(ut1_days_from_j2000(2_451_545.0), 0.0);
    assert_eq!(ut1_days_from_j2000(2_451_546.0), 1.0);
    let via_date = EarthRotationAngle::from_ut1_julian_date(2_451_546.0);
    let via_days = EarthRotationAngle::from_ut1_days(1.0);
    assert!((via_date.turns() - via_days.turns()).abs() < 1e-15);
}

#[test]
fn radians_and_turns_agree() {
    let angle = EarthRotationAngle::from_ut1_days(3.25);
    assert!((angle.radians() - angle.turns() * 2.0 * PI).abs() < 1e-12);
    let from_radians = EarthRotationAngle::from_radians(1.25);
    assert!((from_radians.turns() - 1.25 / (2.0 * PI)).abs() < 1e-15);
}

#[test]
fn positive_variants_reduce_to_the_principal_interval() {
    for days in [-5.0, -1.0, 0.0, 1.0, 7.5, 100.0] {
        let angle = EarthRotationAngle::from_ut1_days(days);
        let turns = angle.turns_positive();
        assert!((0.0..1.0).contains(&turns), "turns out of range: {turns}");
        // Reduction removes whole turns, so the angle is unchanged modulo
        // one turn.
        // 归约去掉整周，故该角在模一周的意义下不变。
        let difference = angle.turns() - turns;
        assert!((difference - difference.round()).abs() < 1e-9);

        let radians = angle.radians_positive();
        assert!((0.0..2.0 * PI).contains(&radians));
    }
}

#[test]
fn rotation_is_orthonormal_with_unit_determinant() {
    let rotation = rotation_eci_to_ecef(EarthRotationAngle::from_ut1_days(12.5));
    assert!(rotation.is_rotation(1e-12));
    assert!((rotation.determinant() - 1.0).abs() < EPS);
}

#[test]
fn a_positive_angle_drifts_inertial_directions_westward() {
    // The Earth turns eastward, so a direction fixed in inertial space
    // drifts westward when read in the Earth-fixed frame. In the
    // Earth-fixed frame that is toward negative second axis.
    // 地球向东转，故在惯性空间中固定的方向在地固系中读数时向西漂移。在地固系中
    // 即朝负第二轴方向。
    let angle = EarthRotationAngle::from_radians(0.25);
    let rotation = rotation_eci_to_ecef(angle);
    let rotated = rotation * Vec3::X;
    assert!(rotated.y < 0.0, "expected negative y, got {}", rotated.y);
    assert!((rotated.norm() - 1.0).abs() < EPS);
}

#[test]
fn opposite_rotations_cancel() {
    let angle = EarthRotationAngle::from_ut1_days(4.75);
    let forward = rotation_eci_to_ecef(angle);
    let backward = rotation_ecef_to_eci(angle);
    let identity = forward * backward;
    assert!(identity.is_rotation(1e-12));
    // Applying one after the other returns the original vector.
    // 先后施加二者返回原矢量。
    let vector = Vec3::new(1.5, -2.5, 3.5);
    assert!((backward * (forward * vector) - vector).norm() < EPS);
}

#[test]
fn earth_rotation_turns_the_third_axis_into_itself() {
    let angle = EarthRotationAngle::from_ut1_days(2.0);
    let rotation = rotation_eci_to_ecef(angle);
    assert!((rotation * Vec3::Z - Vec3::Z).norm() < EPS);
}

#[test]
fn rotation_carries_the_second_axis_toward_the_first() {
    // The companion to the westward drift test: the second axis of the
    // inertial frame maps toward the first axis of the Earth-fixed frame.
    // 向西漂移测试的对应项：惯性系的第二轴映射向地固系的第一轴。
    let angle = EarthRotationAngle::from_radians(0.25);
    let rotation = rotation_eci_to_ecef(angle);
    let rotated = rotation * Vec3::Y;
    assert!(rotated.x > 0.0, "expected positive x, got {}", rotated.x);
    assert!((rotated.norm() - 1.0).abs() < EPS);
}

#[test]
fn a_full_turn_returns_to_the_same_rotation() {
    // The rotation depends on the angle modulo one turn, so angles that
    // differ by whole turns give the same matrix.
    // 该转动依模一周的角而定，故相差整周的角给出相同的矩阵。
    let first = rotation_eci_to_ecef(EarthRotationAngle::from_turns(0.25));
    let second = rotation_eci_to_ecef(EarthRotationAngle::from_turns(2.25));
    let difference = first * second.transpose();
    assert!(difference.is_rotation(1e-12));
    let vector = Vec3::new(1.0, 2.0, 3.0);
    assert!((first * vector - second * vector).norm() < EPS);
}

#[test]
fn identity_angle_leaves_inertial_vectors_unchanged() {
    // An angle of zero, constructed directly rather than from the epoch
    // constant, is the identity rotation.
    // 直接构建的（而非由历元常量得来的）零角是恒等转动。
    let rotation = rotation_eci_to_ecef(EarthRotationAngle::from_turns(0.0));
    let vector = Vec3::new(1.0, 2.0, 3.0);
    assert!((rotation * vector - vector).norm() < EPS);
    assert!(rotation.is_rotation(1e-12));
    assert!((rotation.determinant() - 1.0).abs() < EPS);
}
