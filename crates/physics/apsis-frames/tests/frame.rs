//! Tests for the frame marker types.

use apsis_frames::{Ecef, Eci};
use apsis_math::Vec3;

#[test]
fn wrappers_preserve_the_vector() {
    let vector = Vec3::new(1.0, 2.0, 3.0);
    assert_eq!(Eci::new(vector).vector(), vector);
    assert_eq!(Ecef::new(vector).vector(), vector);
}

#[test]
fn wrappers_compare_by_value() {
    let vector = Vec3::new(-4.0, 5.5, 0.0);
    assert_eq!(Eci::new(vector), Eci(vector));
    assert_eq!(Ecef::new(vector), Ecef(vector));
    assert_ne!(Eci::new(vector), Eci::new(Vec3::ZERO));
}
