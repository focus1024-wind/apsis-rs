//! Tests for gaseous attenuation.

use apsis_atmosphere::{
    GaseousAttenuation, gaseous_attenuation, oxygen_zenith_attenuation,
    water_vapour_zenith_attenuation,
};
use core::f64::consts::{FRAC_PI_2, FRAC_PI_6};

const TOLERANCE: f64 = 1e-12;

#[test]
fn oxygen_attenuation_is_small_at_low_frequencies() {
    // Below about 10 GHz the oxygen contribution at the zenith is a small
    // fraction of a decibel.
    for gigahertz in [1.0, 2.0, 4.0, 6.0, 10.0] {
        let value = oxygen_zenith_attenuation(gigahertz * 1.0e9).expect("valid frequency");
        assert!(value < 0.2, "{gigahertz} GHz gave {value} dB");
        assert!(value >= 0.0);
    }
}

#[test]
fn oxygen_attenuation_peaks_near_sixty_gigahertz() {
    let peak = oxygen_zenith_attenuation(60.0e9).expect("valid");
    // The 60 GHz complex reaches about 15 dB through the whole atmosphere.
    assert!((peak - 15.0).abs() < 0.5, "peak {peak}");
    // A frequency on either side is lower.
    let below = oxygen_zenith_attenuation(50.0e9).expect("valid");
    let above = oxygen_zenith_attenuation(70.0e9).expect("valid");
    assert!(below < peak);
    assert!(above < peak);
}

#[test]
fn oxygen_attenuation_is_monotonic_through_the_complex() {
    // The value rises steadily from below the complex into its centre.
    let mut previous = 0.0;
    for gigahertz in [40.0, 45.0, 50.0, 55.0, 58.0, 59.0] {
        let value = oxygen_zenith_attenuation(gigahertz * 1.0e9).expect("valid");
        assert!(value > previous, "{gigahertz} GHz gave {value}");
        previous = value;
    }
}

#[test]
fn water_vapour_attenuation_peaks_near_twenty_two_gigahertz() {
    let peak = water_vapour_zenith_attenuation(22.235e9).expect("valid");
    assert!((peak - 0.20).abs() < 1e-9, "peak {peak}");
    let below = water_vapour_zenith_attenuation(15.0e9).expect("valid");
    let above = water_vapour_zenith_attenuation(30.0e9).expect("valid");
    assert!(below < peak);
    assert!(above < peak);
}

#[test]
fn water_vapour_attenuation_is_small_below_ten_gigahertz() {
    for gigahertz in [1.0, 4.0, 6.0, 10.0] {
        let value = water_vapour_zenith_attenuation(gigahertz * 1.0e9).expect("valid");
        assert!(value < 0.05, "{gigahertz} GHz gave {value} dB");
    }
}

#[test]
fn the_total_is_the_sum_of_its_parts() {
    let total = gaseous_attenuation(20.0e9).expect("valid");
    assert!((total.zenith_db() - (total.oxygen_db + total.water_vapour_db)).abs() < TOLERANCE);
}

#[test]
fn attenuation_grows_as_the_elevation_falls() {
    let gas = GaseousAttenuation::new(0.05, 0.15);
    let zenith = gas.at_elevation(FRAC_PI_2);
    let thirty = gas.at_elevation(FRAC_PI_6);
    // The zenith total is 0.2 dB and thirty degrees doubles it.
    assert!((zenith.db() - 0.2).abs() < TOLERANCE);
    assert!((thirty.db() - 0.4).abs() < 1e-9);
}

#[test]
fn attenuation_at_the_horizon_is_infinite() {
    let gas = GaseousAttenuation::new(0.05, 0.15);
    assert!(gas.at_elevation(0.0).db().is_infinite());
}

#[test]
fn a_zero_zenith_attenuation_gives_zero_at_every_elevation() {
    let none = GaseousAttenuation::new(0.0, 0.0);
    assert_eq!(none.at_elevation(FRAC_PI_6).db(), 0.0);
}

#[test]
fn validity_is_reported() {
    assert!(GaseousAttenuation::default().is_valid());
    assert!(GaseousAttenuation::new(0.05, 0.15).is_valid());
    assert!(!GaseousAttenuation::new(-0.1, 0.15).is_valid());
    assert!(!GaseousAttenuation::new(0.05, f64::NAN).is_valid());
}

#[test]
fn invalid_frequencies_are_rejected() {
    assert_eq!(oxygen_zenith_attenuation(0.0), None);
    assert_eq!(oxygen_zenith_attenuation(-1.0), None);
    assert_eq!(oxygen_zenith_attenuation(f64::NAN), None);
    assert_eq!(water_vapour_zenith_attenuation(0.0), None);
    assert_eq!(water_vapour_zenith_attenuation(f64::INFINITY), None);
    assert_eq!(gaseous_attenuation(0.0), None);
}
