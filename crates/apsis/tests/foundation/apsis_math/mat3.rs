//! Tests for three by three matrices.
//!
//! 三阶矩阵的测试。

use apsis::apsis_math::{Mat3, Quat, Vec3};
use core::f64::consts::FRAC_PI_2;

const EPS: f64 = 1e-12;

fn assert_mat_close(lhs: Mat3, rhs: Mat3) {
    for row in 0..3 {
        for column in 0..3 {
            let (lhs, rhs) = (
                lhs.get(row, column).expect("indices are in range"),
                rhs.get(row, column).expect("indices are in range"),
            );
            assert!(
                (lhs - rhs).abs() < EPS,
                "at ({row}, {column}): {lhs} vs {rhs}"
            );
        }
    }
}

#[test]
fn constructors_place_entries_in_the_right_slots() {
    let m = Mat3::from_rows([1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]);
    assert_eq!(m.get(0, 0), Some(1.0));
    assert_eq!(m.get(0, 1), Some(2.0));
    assert_eq!(m.get(0, 2), Some(3.0));
    assert_eq!(m.get(1, 0), Some(4.0));
    assert_eq!(m.get(2, 2), Some(9.0));
    assert_eq!(m.get(3, 0), None);
    assert_eq!(m.get(0, 3), None);

    assert_eq!(m.row(1), Some(Vec3::new(4.0, 5.0, 6.0)));
    assert_eq!(m.column(1), Some(Vec3::new(2.0, 5.0, 8.0)));
    assert_eq!(m.row(3), None);
    assert_eq!(m.column(3), None);
}

#[test]
fn from_columns_matches_from_rows() {
    let c0 = Vec3::new(1.0, 4.0, 7.0);
    let c1 = Vec3::new(2.0, 5.0, 8.0);
    let c2 = Vec3::new(3.0, 6.0, 9.0);
    let expected = Mat3::from_rows([1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]);
    assert_eq!(Mat3::from_columns(c0, c1, c2), expected);
}

#[test]
fn identity_is_the_multiplicative_unit() {
    let m = Mat3::from_rotation_z(FRAC_PI_2);
    assert_eq!(Mat3::IDENTITY * m, m);
    assert_eq!(m * Mat3::IDENTITY, m);
    assert_eq!(Mat3::IDENTITY.determinant(), 1.0);
    assert_eq!(Mat3::default(), Mat3::ZERO);
}

#[test]
fn zero_annihilates() {
    assert_eq!(Mat3::ZERO * Vec3::new(1.0, 2.0, 3.0), Vec3::ZERO);
    assert_eq!(Mat3::ZERO * Mat3::IDENTITY, Mat3::ZERO);
    assert_eq!(Mat3::ZERO.determinant(), 0.0);
}

#[test]
fn transpose_swaps_rows_and_columns() {
    let m = Mat3::from_rows([1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]);
    let t = m.transpose();
    assert_eq!(t.row(0), m.column(0));
    assert_eq!(t.row(1), m.column(1));
    assert_eq!(t.row(2), m.column(2));
    assert_eq!(t.transpose(), m);
}

#[test]
fn determinant_matches_hand_computation() {
    let m = Mat3::from_rows([1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 10.0]);
    assert!((m.determinant() + 3.0).abs() < EPS);
    assert_eq!(
        Mat3::from_diagonal(Vec3::new(2.0, 3.0, 4.0)).determinant(),
        24.0
    );
}

#[test]
fn trace_sums_the_diagonal() {
    let m = Mat3::from_rows([1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]);
    assert_eq!(m.trace(), 15.0);
}

#[test]
fn matrix_vector_product_uses_rows() {
    let m = Mat3::from_rows([1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]);
    let v = Vec3::new(1.0, 1.0, 1.0);
    assert_eq!(m * v, Vec3::new(6.0, 15.0, 24.0));
}

#[test]
fn matrix_product_is_associative() {
    let a = Mat3::from_rows([1.0, 2.0, 0.0], [0.0, 1.0, 3.0], [1.0, 0.0, 1.0]);
    let b = Mat3::from_rows([0.0, 1.0, 2.0], [1.0, 0.0, 1.0], [2.0, 1.0, 0.0]);
    let c = Mat3::from_rows([1.0, 1.0, 1.0], [2.0, 2.0, 2.0], [3.0, 3.0, 3.0]);
    assert_eq!((a * b) * c, a * (b * c));
}

#[test]
fn matrix_product_preserves_the_determinant_product() {
    let a = Mat3::from_rows([1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 10.0]);
    let b = Mat3::from_rows([2.0, 0.0, 1.0], [1.0, 3.0, 2.0], [0.0, 1.0, 4.0]);
    let expected = a.determinant() * b.determinant();
    assert!(((a * b).determinant() - expected).abs() < 1e-9);
}

#[test]
fn scalar_product_scales_every_entry() {
    let m = Mat3::from_rows([1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]);
    let scaled = m * 2.0;
    assert_eq!(scaled.get(0, 0), Some(2.0));
    assert_eq!(scaled.get(2, 2), Some(18.0));
    assert_eq!(2.0 * m, scaled);
}

#[test]
fn rotation_matrices_satisfy_the_rotation_predicate() {
    let m = Mat3::from_rotation_z(FRAC_PI_2);
    assert!(m.is_rotation(1e-12));
    assert!((m.determinant() - 1.0).abs() < EPS);
    assert_mat_close(m * m.transpose(), Mat3::IDENTITY);
}

#[test]
fn is_rotation_rejects_non_rotations() {
    assert!(!Mat3::ZERO.is_rotation(1e-9));
    assert!(!Mat3::from_diagonal(Vec3::splat(2.0)).is_rotation(1e-9));
    // A reflection has determinant minus one.
    // 反射的行列式为负一。
    assert!(!Mat3::from_diagonal(Vec3::new(1.0, 1.0, -1.0)).is_rotation(1e-9));
}

#[test]
fn axis_angle_matches_the_quaternion_rotation() {
    let axis = Vec3::new(0.5, -2.0, 1.0);
    let m = Mat3::from_axis_angle(axis, 0.9).expect("non-zero axis");
    let q = Quat::from_axis_angle(axis, 0.9).expect("non-zero axis");
    let v = Vec3::new(1.0, -2.0, 3.0);
    assert!((m * v - q.rotate(v)).norm() < EPS);
    assert_eq!(Mat3::from_axis_angle(Vec3::ZERO, 1.0), None);
}

#[test]
fn rotation_maps_axes_as_expected() {
    let m = Mat3::from_rotation_z(FRAC_PI_2);
    assert!((m * Vec3::X - Vec3::Y).norm() < EPS);
    assert!((m * Vec3::Y + Vec3::X).norm() < EPS);
    assert!((m * Vec3::Z - Vec3::Z).norm() < EPS);
}

#[test]
fn per_axis_constructors_agree_with_the_axis_angle_form() {
    let angle = 0.6;
    let about_x = Mat3::from_rotation_x(angle);
    assert_eq!(
        about_x,
        Mat3::from_axis_angle(Vec3::X, angle).expect("unit axis")
    );
    assert_eq!(about_x * Vec3::X, Vec3::X);

    let about_y = Mat3::from_rotation_y(angle);
    assert_eq!(
        about_y,
        Mat3::from_axis_angle(Vec3::Y, angle).expect("unit axis")
    );
    assert_eq!(about_y * Vec3::Y, Vec3::Y);

    let about_z = Mat3::from_rotation_z(angle);
    assert_eq!(
        about_z,
        Mat3::from_axis_angle(Vec3::Z, angle).expect("unit axis")
    );
    assert_eq!(about_z * Vec3::Z, Vec3::Z);
    assert!((about_z.determinant() - 1.0).abs() < EPS);
}

#[test]
fn finiteness_is_reported() {
    assert!(Mat3::IDENTITY.is_finite());
    let mut m = Mat3::IDENTITY;
    m.m01 = f64::NAN;
    assert!(!m.is_finite());
}

#[test]
fn display_uses_row_major_groups() {
    let m = Mat3::from_diagonal(Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(m.to_string(), "[1, 0, 0; 0, 2, 0; 0, 0, 3]");
}

#[test]
fn from_quat_is_consistent_with_quaternion_rotation() {
    let q = Quat::from_axis_angle(Vec3::new(1.0, 1.0, 0.0), 1.3).expect("non-zero axis");
    let m = Mat3::from_quat(q);
    let v = Vec3::new(2.0, -1.0, 0.5);
    assert!((m * v - q.rotate(v)).norm() < EPS);
    assert!(m.is_rotation(1e-12));
}
