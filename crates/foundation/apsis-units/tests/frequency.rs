//! Tests for frequency quantities.
//!
//! 频率量的测试。

use apsis_core::Unit;
use apsis_units::Hertz;
use apsis_units::dimension::Frequency;

#[test]
fn stores_the_given_value() {
    assert_eq!(Hertz::new(1_575.42).value(), 1_575.42);
}

#[test]
fn period_is_the_reciprocal() {
    assert!((Hertz::new(2.0).period() - 0.5).abs() < 1e-12);
    assert!(Hertz::new(0.0).period().is_infinite());
}

#[test]
fn unit_metadata_is_available() {
    fn assert_frequency<U: Unit<Dimension = Frequency>>() {}
    assert_eq!(Hertz::symbol(), "Hz");
    assert_eq!(Hertz::factor_to_base(), 1.0);
    assert_frequency::<Hertz>();
}

#[test]
fn display_includes_the_symbol() {
    assert_eq!(Hertz::new(1_000.0).to_string(), "1000 Hz");
}
