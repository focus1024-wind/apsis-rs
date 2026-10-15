//! Tests for the physical and astronomical constants.
//!
//! 物理与天文常量的测试。

use apsis::apsis_core::constant::*;

#[test]
fn defining_constants_are_exact() {
    assert_eq!(SPEED_OF_LIGHT, 299_792_458.0);
    assert_eq!(BOLTZMANN, 1.380_649e-23);
    assert_eq!(PLANCK, 6.626_070_15e-34);
    assert_eq!(ELEMENTARY_CHARGE, 1.602_176_634e-19);
    assert_eq!(ASTRONOMICAL_UNIT, 149_597_870_700.0);
    assert_eq!(STANDARD_GRAVITY, 9.806_65);
}

#[test]
fn time_constants_are_consistent() {
    assert_eq!(HOUR, MINUTE * 60.0);
    assert_eq!(DAY, HOUR * 24.0);
    assert_eq!(JULIAN_YEAR, DAY * 365.25);
    assert_eq!(JULIAN_CENTURY, JULIAN_YEAR * 100.0);
}

#[test]
fn j2000_minus_unix_epoch_is_the_expected_span() {
    // The two epochs are 10 957 days apart.
    // 这两个历元相隔 10 957 天。
    assert_eq!(JULIAN_DATE_J2000 - JULIAN_DATE_UNIX_EPOCH, 10_957.5);
    assert_eq!(SECONDS_J2000_TO_UNIX, 10_957.5 * DAY);
    assert_eq!(SECONDS_J2000_TO_UNIX, 946_728_000.0);
}

#[test]
fn earth_flattening_matches_the_inverse_value() {
    assert!((1.0 / EARTH_FLATTENING - 298.257_223_563).abs() < 1e-9);
}

#[test]
fn celsius_offset_is_exact() {
    assert_eq!(ZERO_CELSIUS_IN_KELVIN, 273.15);
}
