//! Tests for the unit and quantity marker traits.

use apsis_core::{Quantity, Unit};
use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Length;

#[derive(Debug, Clone, Copy)]
struct Meter;

#[derive(Debug, Clone, Copy)]
struct Kilometer;

impl fmt::Display for Meter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("m")
    }
}

impl fmt::Display for Kilometer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("km")
    }
}

impl Unit for Meter {
    type Dimension = Length;

    fn factor_to_base() -> f64 {
        1.0
    }

    fn symbol() -> &'static str {
        "m"
    }
}

impl Unit for Kilometer {
    type Dimension = Length;

    fn factor_to_base() -> f64 {
        1_000.0
    }

    fn symbol() -> &'static str {
        "km"
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Meters(f64);

#[derive(Debug, Clone, Copy, PartialEq)]
struct Kilometers(f64);

impl Quantity for Meters {
    type U = Meter;

    fn value(self) -> f64 {
        self.0
    }

    fn from_value(value: f64) -> Self {
        Self(value)
    }
}

impl Quantity for Kilometers {
    type U = Kilometer;

    fn value(self) -> f64 {
        self.0
    }

    fn from_value(value: f64) -> Self {
        Self(value)
    }
}

#[test]
fn value_round_trips() {
    let distance = Meters::from_value(1_500.0);
    assert_eq!(distance.value(), 1_500.0);
}

#[test]
fn converts_between_units_of_one_dimension() {
    let distance = Meters::from_value(2_500.0);
    let in_kilometers: Kilometers = distance.to_unit();
    assert!((in_kilometers.value() - 2.5).abs() < 1e-12);
}

#[test]
fn conversion_round_trips() {
    let original = Kilometers::from_value(7.25);
    let round_tripped: Kilometers = original.to_unit::<Meters>().to_unit();
    assert!((round_tripped.value() - 7.25).abs() < 1e-12);
}

#[test]
fn unit_metadata_is_available() {
    assert_eq!(<Meter as Unit>::symbol(), "m");
    assert_eq!(<Kilometer as Unit>::factor_to_base(), 1_000.0);
}
