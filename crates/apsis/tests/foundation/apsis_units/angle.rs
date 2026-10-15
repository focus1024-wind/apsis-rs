//! Tests for plane angle quantities.
//!
//! 平面角量的测试。

use apsis::apsis_core::{Quantity, Unit};
use apsis::apsis_units::dimension::Angle;
use apsis::apsis_units::{Degrees, Radians};
use core::f64::consts::PI;

#[test]
fn direct_conversions_are_consistent() {
    assert!((Degrees::new(180.0).to_radians().value() - PI).abs() < 1e-12);
    assert!((Radians::new(PI).to_degrees().value() - 180.0).abs() < 1e-12);
    assert!((Degrees::new(90.0).to_radians().value() - PI / 2.0).abs() < 1e-12);
}

#[test]
fn generic_conversion_matches_direct_conversion() {
    let angle = Degrees::new(45.0);
    let via_trait: Radians = angle.to_unit();
    assert!((via_trait.value() - angle.to_radians().value()).abs() < 1e-12);
}

#[test]
fn conversion_round_trips() {
    let original = Degrees::new(137.5);
    let round_tripped: Degrees = original.to_unit::<Radians>().to_unit();
    assert!((round_tripped.value() - original.value()).abs() < 1e-12);
}

#[test]
fn wrap_places_angles_in_the_canonical_range() {
    for turns in -3..=3 {
        let angle = Radians::new(PI + f64::from(turns) * 2.0 * PI);
        let wrapped = angle.wrap();
        assert!(wrapped.value() >= -PI);
        assert!(wrapped.value() < PI);
        // Wrapping shifts by whole turns, so the angle is preserved
        // modulo two pi.
        // 环绕按整周平移，故该角在模 2 pi 的意义下保持。
        let difference = angle.value() - wrapped.value();
        assert!((difference / (2.0 * PI)).fract().abs() < 1e-12);
    }
    assert!((Radians::new(0.0).wrap().value()).abs() < 1e-12);
    assert!((Radians::new(PI).wrap().value() + PI).abs() < 1e-12);
}

#[test]
fn unit_metadata_is_available() {
    fn assert_angle<U: Unit<Dimension = Angle>>() {}
    assert_eq!(Radians::symbol(), "rad");
    assert_eq!(Degrees::symbol(), "deg");
    assert_eq!(Radians::factor_to_base(), 1.0);
    assert!((Degrees::factor_to_base() - PI / 180.0).abs() < 1e-15);
    assert_angle::<Radians>();
    assert_angle::<Degrees>();
}

#[test]
fn display_includes_the_symbol() {
    assert_eq!(Radians::new(1.0).to_string(), "1 rad");
    assert_eq!(Degrees::new(90.0).to_string(), "90 deg");
}
