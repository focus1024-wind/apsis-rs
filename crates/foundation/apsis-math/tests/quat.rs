//! Tests for unit quaternion rotations.

use apsis_math::{Mat3, Quat, Vec3};
use core::f64::consts::{FRAC_PI_2, PI};

const EPS: f64 = 1e-12;

fn assert_close(lhs: Vec3, rhs: Vec3) {
    assert!((lhs - rhs).norm() < EPS, "left {lhs}, right {rhs}");
}

fn assert_quat_close(lhs: Quat, rhs: Quat) {
    let difference = Quat::new(lhs.w - rhs.w, lhs.x - rhs.x, lhs.y - rhs.y, lhs.z - rhs.z);
    assert!(difference.norm() < EPS, "left {lhs}, right {rhs}");
}

#[test]
fn identity_rotates_nothing() {
    let v = Vec3::new(1.0, 2.0, 3.0);
    assert_close(Quat::IDENTITY.rotate(v), v);
    assert_eq!(Quat::IDENTITY.angle(), 0.0);
    assert_eq!(Quat::IDENTITY.axis(), None);
}

#[test]
fn axis_angle_round_trips() {
    let axis = Vec3::new(1.0, 2.0, -2.0)
        .normalize()
        .expect("non-zero axis");
    let q = Quat::from_axis_angle(axis, 0.7).expect("non-zero axis");
    assert!((q.angle() - 0.7).abs() < EPS);
    assert_close(q.axis().expect("rotation has an axis"), axis);
    assert!(q.is_normalized());
}

#[test]
fn axis_angle_rejects_undefined_axes() {
    assert_eq!(Quat::from_axis_angle(Vec3::ZERO, 1.0), None);
    assert_eq!(Quat::from_axis_angle(Vec3::splat(f64::NAN), 1.0), None);
}

#[test]
fn quarter_turn_about_z_maps_x_to_y() {
    let q = Quat::from_rotation_z(FRAC_PI_2);
    assert_close(q.rotate(Vec3::X), Vec3::Y);
    assert_close(q.rotate(Vec3::Y), -Vec3::X);
    assert_close(q.rotate(Vec3::Z), Vec3::Z);
}

#[test]
fn quarter_turn_about_x_maps_y_to_z() {
    let q = Quat::from_rotation_x(FRAC_PI_2);
    assert_close(q.rotate(Vec3::Y), Vec3::Z);
    assert_close(q.rotate(Vec3::Z), -Vec3::Y);
}

#[test]
fn quarter_turn_about_y_maps_z_to_x() {
    let q = Quat::from_rotation_y(FRAC_PI_2);
    assert_close(q.rotate(Vec3::Z), Vec3::X);
    assert_close(q.rotate(Vec3::X), -Vec3::Z);
}

#[test]
fn rotation_preserves_length() {
    let q = Quat::from_axis_angle(Vec3::new(1.0, -1.0, 0.5), 1.234).expect("non-zero axis");
    let v = Vec3::new(3.0, -4.0, 12.0);
    assert!((q.rotate(v).norm() - v.norm()).abs() < EPS);
}

#[test]
fn conjugate_undoes_the_rotation() {
    let q = Quat::from_axis_angle(Vec3::new(0.0, 1.0, 1.0), 0.9).expect("non-zero axis");
    let v = Vec3::new(1.0, -2.0, 0.5);
    assert_close(q.conjugate().rotate(q.rotate(v)), v);
}

#[test]
fn inverse_undoes_the_rotation() {
    let q = Quat::from_axis_angle(Vec3::new(-0.5, 1.0, 0.25), 2.0).expect("non-zero axis");
    let v = Vec3::new(2.0, 0.0, -1.0);
    let inverse = q.inverse().expect("unit quaternion is invertible");
    assert_close(inverse.rotate(q.rotate(v)), v);
}

#[test]
fn inverse_rejects_degenerate_quaternions() {
    assert_eq!(Quat::new(0.0, 0.0, 0.0, 0.0).inverse(), None);
    assert_eq!(Quat::new(f64::NAN, 0.0, 0.0, 0.0).inverse(), None);
}

#[test]
fn multiplication_composes_right_to_left() {
    let first = Quat::from_rotation_z(FRAC_PI_2);
    let second = Quat::from_rotation_x(FRAC_PI_2);
    let v = Vec3::new(1.0, 0.0, 0.0);
    // `second * first` applies `first` first, then `second`.
    assert_close((second * first).rotate(v), second.rotate(first.rotate(v)));
}

#[test]
fn identity_is_the_multiplicative_unit() {
    let q = Quat::from_rotation_y(0.4);
    assert_quat_close(q * Quat::IDENTITY, q);
    assert_quat_close(Quat::IDENTITY * q, q);
}

#[test]
fn multiplication_matches_matrix_composition() {
    let a = Quat::from_axis_angle(Vec3::new(1.0, 0.5, -0.25), 0.8).expect("non-zero axis");
    let b = Quat::from_axis_angle(Vec3::new(-0.5, 1.0, 0.75), 1.6).expect("non-zero axis");
    let composed = a * b;
    let v = Vec3::new(1.0, 2.0, 3.0);
    let via_matrices = Mat3::from_quat(a) * (Mat3::from_quat(b) * v);
    assert_close(composed.rotate(v), via_matrices);
}

#[test]
fn normalize_produces_unit_length() {
    let q = Quat::new(2.0, 0.0, 0.0, 0.0);
    let unit = q.normalize().expect("non-zero quaternion");
    assert!(unit.is_normalized());
    assert!((unit.w - 1.0).abs() < EPS);
}

#[test]
fn normalize_rejects_degenerate_quaternions() {
    assert_eq!(Quat::new(0.0, 0.0, 0.0, 0.0).normalize(), None);
    assert_eq!(Quat::new(f64::NAN, 0.0, 0.0, 0.0).normalize(), None);
}

#[test]
fn from_mat3_recovers_the_original_rotation() {
    let q = Quat::from_axis_angle(Vec3::new(0.3, -0.4, 0.5), 1.1).expect("non-zero axis");
    let recovered = Quat::from_mat3(Mat3::from_quat(q)).expect("rotation matrix is accepted");
    // `q` and `-q` are the same rotation, so compare through their effect.
    let v = Vec3::new(1.0, -2.0, 3.0);
    assert_close(recovered.rotate(v), q.rotate(v));
}

#[test]
fn from_mat3_covers_every_branch() {
    // These angles exercise the trace branch and each of the three
    // largest-diagonal branches.
    for (axis, angle) in [
        (Vec3::new(1.0, 0.0, 0.0), 0.2),
        (Vec3::new(1.0, 0.0, 0.0), PI - 0.1),
        (Vec3::new(0.0, 1.0, 0.0), PI - 0.1),
        (Vec3::new(0.0, 0.0, 1.0), PI - 0.1),
    ] {
        let q = Quat::from_axis_angle(axis, angle).expect("non-zero axis");
        let recovered = Quat::from_mat3(Mat3::from_quat(q)).expect("rotation matrix is accepted");
        let v = Vec3::new(0.5, 1.5, -2.5);
        assert_close(recovered.rotate(v), q.rotate(v));
    }
}

#[test]
fn from_mat3_rejects_non_rotations() {
    let scaled = Mat3::from_diagonal(Vec3::splat(2.0));
    assert_eq!(Quat::from_mat3(scaled), None);
    assert_eq!(Quat::from_mat3(Mat3::ZERO), None);
}

#[test]
fn slerp_walks_the_short_arc_uniformly() {
    let start = Quat::IDENTITY;
    let end = Quat::from_rotation_z(PI);
    let half = start.slerp(end, 0.5);
    assert!((half.angle() - FRAC_PI_2).abs() < 1e-9);
    assert_close(half.rotate(Vec3::X), Vec3::Y);
}

#[test]
fn slerp_selects_the_short_arc_when_endpoints_are_negated() {
    let start = Quat::IDENTITY;
    let end = Quat::new(-1.0, 0.0, 0.0, 0.0);
    // The two quaternions describe the same rotation, so any
    // interpolation between them must stay at the identity.
    let middle = start.slerp(end, 0.5);
    assert_close(middle.rotate(Vec3::X), Vec3::X);
}

#[test]
fn slerp_hits_both_endpoints() {
    let start = Quat::from_rotation_x(0.3);
    let end = Quat::from_rotation_y(1.2);
    assert_quat_close(start.slerp(end, 0.0), start);
    assert_quat_close(start.slerp(end, 1.0), end);
}

#[test]
fn slerp_handles_nearly_parallel_inputs() {
    let start = Quat::IDENTITY;
    let end = Quat::from_rotation_z(1e-9);
    let middle = start.slerp(end, 0.5);
    assert!(middle.is_normalized());
}

#[test]
fn display_lists_the_components() {
    assert_eq!(Quat::new(1.0, 0.0, 0.0, 0.0).to_string(), "(1, 0, 0, 0)");
}

#[test]
fn finiteness_is_reported() {
    assert!(Quat::IDENTITY.is_finite());
    assert!(!Quat::new(f64::INFINITY, 0.0, 0.0, 0.0).is_finite());
}

#[test]
fn named_turns_behave_as_documented() {
    assert_close(Quat::quarter_turn_x().rotate(Vec3::Y), Vec3::Z);
    assert_close(Quat::half_turn_z().rotate(Vec3::X), -Vec3::X);
    assert!(Quat::quarter_turn_x().is_normalized());
    assert!(Quat::half_turn_z().is_normalized());
}
