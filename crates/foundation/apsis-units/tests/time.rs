//! Tests for the duration quantity.
//!
//! 时长量的测试。

use apsis_core::Unit;
use apsis_units::Seconds;
use apsis_units::dimension::Time;

#[test]
fn stores_the_given_value() {
    assert_eq!(Seconds::new(0.5).value(), 0.5);
}

#[test]
fn unit_metadata_is_available() {
    assert_eq!(Seconds::symbol(), "s");
    assert_eq!(Seconds::factor_to_base(), 1.0);
    fn assert_time<U: Unit<Dimension = Time>>() {}
    assert_time::<Seconds>();
}

#[test]
fn arithmetic_operates_on_the_stored_value() {
    assert_eq!((Seconds::new(1.0) + Seconds::new(2.0)).value(), 3.0);
    assert_eq!((Seconds::new(3.0) - Seconds::new(2.0)).value(), 1.0);
    assert_eq!((Seconds::new(3.0) * 2.0).value(), 6.0);
    assert_eq!((Seconds::new(3.0) / 3.0).value(), 1.0);
    assert_eq!(Seconds::new(6.0) / Seconds::new(3.0), 2.0);
}

#[test]
fn display_includes_the_symbol() {
    assert_eq!(Seconds::new(1.5).to_string(), "1.5 s");
}
