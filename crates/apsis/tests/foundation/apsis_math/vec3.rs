//! Tests for three component vectors.
//!
//! 三分量矢量的测试。

use apsis::apsis_math::Vec3;

const EPS: f64 = 1e-12;

fn assert_close(lhs: Vec3, rhs: Vec3) {
    assert!((lhs - rhs).norm() < EPS, "left {lhs}, right {rhs}");
}

#[test]
fn constructors_set_the_components() {
    assert_eq!(
        Vec3::new(1.0, 2.0, 3.0),
        Vec3 {
            x: 1.0,
            y: 2.0,
            z: 3.0
        }
    );
    assert_eq!(Vec3::splat(2.0), Vec3::new(2.0, 2.0, 2.0));
    assert_eq!(Vec3::default(), Vec3::ZERO);
}

#[test]
fn get_selects_by_index_and_rejects_out_of_range() {
    let v = Vec3::new(1.0, 2.0, 3.0);
    assert_eq!(v.get(0), Some(1.0));
    assert_eq!(v.get(1), Some(2.0));
    assert_eq!(v.get(2), Some(3.0));
    assert_eq!(v.get(3), None);
}

#[test]
fn dot_product_is_symmetric() {
    let a = Vec3::new(1.0, 2.0, 3.0);
    let b = Vec3::new(-4.0, 5.0, 0.5);
    assert!((a.dot(b) - b.dot(a)).abs() < EPS);
    assert!((a.dot(b) - 7.5).abs() < EPS);
}

#[test]
fn cross_product_follows_the_right_hand_rule() {
    assert_eq!(Vec3::X.cross(Vec3::Y), Vec3::Z);
    assert_eq!(Vec3::Y.cross(Vec3::Z), Vec3::X);
    assert_eq!(Vec3::Z.cross(Vec3::X), Vec3::Y);
    // The cross product of parallel vectors vanishes.
    // 平行矢量的叉积为零。
    assert_eq!(Vec3::X.cross(Vec3::X), Vec3::ZERO);
}

#[test]
fn cross_product_is_orthogonal_to_its_inputs() {
    let a = Vec3::new(1.0, -2.0, 3.5);
    let b = Vec3::new(0.5, 4.0, -1.0);
    let c = a.cross(b);
    assert!(c.dot(a).abs() < EPS);
    assert!(c.dot(b).abs() < EPS);
}

#[test]
fn cross_product_is_antisymmetric() {
    let a = Vec3::new(1.0, 2.0, 3.0);
    let b = Vec3::new(4.0, 5.0, 6.0);
    assert_eq!(a.cross(b), -b.cross(a));
}

#[test]
fn norm_matches_the_pythagorean_length() {
    let v = Vec3::new(3.0, 4.0, 12.0);
    assert!((v.norm_squared() - 169.0).abs() < EPS);
    assert!((v.norm() - 13.0).abs() < EPS);
}

#[test]
fn normalize_produces_a_unit_vector() {
    let v = Vec3::new(-3.0, 0.0, 4.0);
    let unit = v.normalize().expect("non-zero vector has a direction");
    assert!((unit.norm() - 1.0).abs() < EPS);
    assert_close(unit, Vec3::new(-0.6, 0.0, 0.8));
}

#[test]
fn normalize_rejects_undefined_directions() {
    assert_eq!(Vec3::ZERO.normalize(), None);
    assert_eq!(Vec3::splat(f64::INFINITY).normalize(), None);
    assert_eq!(Vec3::splat(f64::NAN).normalize(), None);
}

#[test]
fn distance_measures_separation() {
    let a = Vec3::new(1.0, 1.0, 1.0);
    let b = Vec3::new(1.0, 1.0, 4.0);
    assert!((a.distance(b) - 3.0).abs() < EPS);
    assert!((a.distance_squared(b) - 9.0).abs() < EPS);
}

#[test]
fn angle_between_uses_the_right_hand_geometry() {
    let angle = Vec3::X
        .angle_between(Vec3::Y)
        .expect("both axes have unit length");
    assert!((angle - core::f64::consts::FRAC_PI_2).abs() < EPS);
    let opposite = Vec3::X
        .angle_between(-Vec3::X)
        .expect("both vectors have unit length");
    assert!((opposite - core::f64::consts::PI).abs() < EPS);
    let parallel = Vec3::X
        .angle_between(Vec3::X)
        .expect("both vectors have unit length");
    assert!(parallel.abs() < EPS);
}

#[test]
fn angle_between_rejects_undefined_directions() {
    assert_eq!(Vec3::ZERO.angle_between(Vec3::X), None);
    assert_eq!(Vec3::X.angle_between(Vec3::ZERO), None);
}

#[test]
fn lerp_hits_both_endpoints() {
    let a = Vec3::new(0.0, 0.0, 0.0);
    let b = Vec3::new(2.0, 4.0, 6.0);
    assert_eq!(a.lerp(b, 0.0), a);
    assert_eq!(a.lerp(b, 1.0), b);
    assert_eq!(a.lerp(b, 0.5), Vec3::new(1.0, 2.0, 3.0));
}

#[test]
fn component_reductions_report_extremes() {
    let v = Vec3::new(-2.0, 5.0, 1.0);
    assert!((v.max_component() - 5.0).abs() < EPS);
    assert!((v.min_component() + 2.0).abs() < EPS);
    assert_close(v.abs(), Vec3::new(2.0, 5.0, 1.0));
}

#[test]
fn finiteness_is_reported() {
    assert!(Vec3::new(1.0, 2.0, 3.0).is_finite());
    assert!(!Vec3::new(1.0, f64::NAN, 3.0).is_finite());
    assert!(!Vec3::splat(f64::INFINITY).is_finite());
}

#[test]
fn arithmetic_operators_match_component_wise_results() {
    let a = Vec3::new(1.0, 2.0, 3.0);
    let b = Vec3::new(4.0, 5.0, 6.0);
    assert_eq!(a + b, Vec3::new(5.0, 7.0, 9.0));
    assert_eq!(b - a, Vec3::new(3.0, 3.0, 3.0));
    assert_eq!(-a, Vec3::new(-1.0, -2.0, -3.0));
    assert_eq!(a * 2.0, Vec3::new(2.0, 4.0, 6.0));
    assert_eq!(2.0 * a, a * 2.0);
    assert_eq!(b / 2.0, Vec3::new(2.0, 2.5, 3.0));

    let mut acc = a;
    acc += b;
    assert_eq!(acc, a + b);
    acc -= b;
    assert_eq!(acc, a);
    acc *= 3.0;
    assert_eq!(acc, a * 3.0);
    acc /= 3.0;
    assert_eq!(acc, a);
}

#[test]
fn display_lists_the_components() {
    assert_eq!(Vec3::new(1.0, 2.5, -3.0).to_string(), "(1, 2.5, -3)");
}
