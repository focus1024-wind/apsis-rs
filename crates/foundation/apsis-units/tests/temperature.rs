//! Tests for thermodynamic temperature quantities.

use apsis_core::Unit;
use apsis_core::constant::ZERO_CELSIUS_IN_KELVIN;
use apsis_units::Kelvin;
use apsis_units::dimension::Temperature;

#[test]
fn stores_the_given_value() {
    assert_eq!(Kelvin::new(290.0).value(), 290.0);
}

#[test]
fn celsius_conversion_uses_the_exact_offset() {
    let freezing = Kelvin::from_celsius(0.0);
    assert_eq!(freezing.value(), ZERO_CELSIUS_IN_KELVIN);
    assert_eq!(freezing.to_celsius(), 0.0);
    let boiling = Kelvin::from_celsius(100.0);
    assert!((boiling.to_celsius() - 100.0).abs() < 1e-12);
}

#[test]
fn celsius_conversion_round_trips() {
    let original = Kelvin::new(273.15 + 21.5);
    assert!((Kelvin::from_celsius(original.to_celsius()).value() - original.value()).abs() < 1e-12);
}

#[test]
fn unit_metadata_is_available() {
    assert_eq!(Kelvin::symbol(), "K");
    assert_eq!(Kelvin::factor_to_base(), 1.0);
    fn assert_temperature<U: Unit<Dimension = Temperature>>() {}
    assert_temperature::<Kelvin>();
}

#[test]
fn display_includes_the_symbol() {
    assert_eq!(Kelvin::new(300.0).to_string(), "300 K");
}
