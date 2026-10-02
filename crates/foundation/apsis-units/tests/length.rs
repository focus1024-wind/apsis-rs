//! Tests for length quantities.
//!
//! 长度量的测试。

use apsis_core::{Quantity, Unit};
use apsis_units::dimension::Length;
use apsis_units::{Kilometers, Meters};

#[test]
fn stores_the_given_value() {
    assert_eq!(Meters::new(12.5).value(), 12.5);
    assert_eq!(Kilometers::new(-3.0).value(), -3.0);
}

#[test]
fn direct_conversions_are_consistent() {
    assert_eq!(Meters::new(2_500.0).to_kilometers().value(), 2.5);
    assert_eq!(Kilometers::new(2.5).to_meters().value(), 2_500.0);
}

#[test]
fn generic_conversion_matches_direct_conversion() {
    let distance = Meters::new(2_500.0);
    let via_trait: Kilometers = distance.to_unit();
    assert_eq!(via_trait, distance.to_kilometers());
}

#[test]
fn conversion_round_trips() {
    let original = Kilometers::new(42.195);
    let round_tripped: Kilometers = original.to_unit::<Meters>().to_unit();
    assert!((round_tripped.value() - original.value()).abs() < 1e-12);
}

#[test]
fn units_report_their_metadata() {
    fn assert_length<U: Unit<Dimension = Length>>() {}
    assert_eq!(Meters::symbol(), "m");
    assert_eq!(Kilometers::symbol(), "km");
    assert_eq!(Meters::factor_to_base(), 1.0);
    assert_eq!(Kilometers::factor_to_base(), 1_000.0);
    // Both units measure the same dimension, which is what makes the
    // generic conversion above type check.
    // 两种单位测量同一量纲，这正是使上面的泛型换算通过类型检查的原因。
    assert_length::<Meters>();
    assert_length::<Kilometers>();
}

#[test]
fn arithmetic_operates_on_the_stored_value() {
    let sum = Meters::new(1.5) + Meters::new(2.5);
    assert_eq!(sum.value(), 4.0);
    let difference = Meters::new(4.0) - Meters::new(1.0);
    assert_eq!(difference.value(), 3.0);
    let scaled = Meters::new(3.0) * 2.0;
    assert_eq!(scaled.value(), 6.0);
    let straddled = 2.0 * Meters::new(3.0);
    assert_eq!(straddled.value(), 6.0);
    let halved = Meters::new(3.0) / 2.0;
    assert_eq!(halved.value(), 1.5);
    let ratio = Meters::new(3.0) / Meters::new(1.5);
    assert_eq!(ratio, 2.0);
    assert_eq!((-Meters::new(1.0)).value(), -1.0);
}

#[test]
fn display_includes_the_symbol() {
    assert_eq!(Meters::new(1.0).to_string(), "1 m");
    assert_eq!(Kilometers::new(2.0).to_string(), "2 km");
}
