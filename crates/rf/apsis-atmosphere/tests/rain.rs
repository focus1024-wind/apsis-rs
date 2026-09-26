//! Tests for rain attenuation.

use apsis_atmosphere::{
    polarisation_tilt, rain_attenuation, rain_coefficients, specific_rain_attenuation,
};
use core::f64::consts::FRAC_PI_2;

/// Horizontal polarisation.
const HORIZONTAL: f64 = 0.0;
/// Vertical polarisation.
const VERTICAL: f64 = FRAC_PI_2;

/// A mid latitude elevation of thirty degrees.
const ELEVATION: f64 = core::f64::consts::PI / 6.0;

#[test]
fn coefficients_are_available_across_the_tabulated_range() {
    for gigahertz in [1.0, 4.0, 12.0, 20.0, 30.0, 100.0, 500.0, 1_000.0] {
        let coefficients =
            rain_coefficients(gigahertz * 1.0e9, HORIZONTAL, ELEVATION).expect("valid");
        assert!(
            coefficients.0 > 0.0,
            "{gigahertz} GHz gave k {}",
            coefficients.0
        );
        assert!(
            coefficients.1 > 0.5 && coefficients.1 < 1.5,
            "{gigahertz} GHz gave alpha {}",
            coefficients.1
        );
    }
}

#[test]
fn coefficients_at_a_tabulated_frequency_match_the_table() {
    // At 12 GHz the table gives k = 0.0168 for horizontal polarisation,
    // with alpha_h = 1.200 and alpha_v = 1.000. The combined alpha depends
    // on the elevation through the `cos(2 elevation)` term, so at thirty
    // degrees it is the weighted mean of the two, which is 1.15.
    let (k, alpha) = rain_coefficients(12.0e9, HORIZONTAL, ELEVATION).expect("valid");
    assert!((k - 0.016_8).abs() < 1e-9, "k {k}");
    assert!((alpha - 1.15).abs() < 1e-9, "alpha {alpha}");

    // At the zenith the `cos(2 elevation)` factor is minus one, which
    // selects the vertical alpha instead, so the pair swaps behaviour.
    let (_, zenith_alpha) = rain_coefficients(12.0e9, HORIZONTAL, FRAC_PI_2).expect("valid");
    assert!((zenith_alpha - 1.0).abs() < 1e-9, "alpha {zenith_alpha}");
}

#[test]
fn horizontal_coefficients_exceed_vertical_at_mid_frequencies() {
    // At 20 GHz the table gives k_h = 0.0751 against k_v = 0.0691.
    let (horizontal, _) = rain_coefficients(20.0e9, HORIZONTAL, ELEVATION).expect("valid");
    let (vertical, _) = rain_coefficients(20.0e9, VERTICAL, ELEVATION).expect("valid");
    assert!(horizontal > vertical, "{horizontal} vs {vertical}");
}

#[test]
fn the_coefficients_are_interpolated_between_points() {
    // At 11 GHz the value lies between the 10 and 12 GHz entries in
    // logarithmic k, so it is the geometric mean of the two.
    let (k, _) = rain_coefficients(11.0e9, HORIZONTAL, ELEVATION).expect("valid");
    let expected = (0.008_87_f64 * 0.016_8).sqrt();
    assert!((k - expected).abs() < 1e-9, "k {k} expected {expected}");
}

#[test]
fn a_frequency_outside_the_table_is_rejected() {
    assert_eq!(rain_coefficients(0.5e9, HORIZONTAL, ELEVATION), None);
    assert_eq!(rain_coefficients(2_000.0e9, HORIZONTAL, ELEVATION), None);
    assert_eq!(rain_coefficients(f64::NAN, HORIZONTAL, ELEVATION), None);
    assert_eq!(rain_coefficients(12.0e9, f64::NAN, ELEVATION), None);
}

#[test]
fn specific_attenuation_follows_the_power_law() {
    let rain_rate = 20.0;
    let specific =
        specific_rain_attenuation(rain_rate, 12.0e9, HORIZONTAL, ELEVATION).expect("valid");
    // The value is k R^alpha with the coefficients for this frequency,
    // polarisation, and elevation.
    let (k, alpha) = rain_coefficients(12.0e9, HORIZONTAL, ELEVATION).expect("valid");
    assert!((specific - k * rain_rate.powf(alpha)).abs() < 1e-9);
    // For 20 mm/h at 12 GHz and thirty degrees this is about 0.53 dB/km.
    assert!((specific - 0.526_6).abs() < 0.001, "specific {specific}");
}

#[test]
fn a_higher_rain_rate_gives_more_attenuation() {
    let light = specific_rain_attenuation(1.0, 12.0e9, HORIZONTAL, ELEVATION).expect("valid");
    let heavy = specific_rain_attenuation(50.0, 12.0e9, HORIZONTAL, ELEVATION).expect("valid");
    assert!(heavy > light);
    // The ratio follows the power law, so it is 50 raised to the same alpha
    // the coefficients give.
    let (_, alpha) = rain_coefficients(12.0e9, HORIZONTAL, ELEVATION).expect("valid");
    assert!((heavy / light - 50.0_f64.powf(alpha)).abs() < 1e-6);
}

#[test]
fn a_zero_rain_rate_gives_no_attenuation() {
    assert_eq!(
        specific_rain_attenuation(0.0, 12.0e9, HORIZONTAL, ELEVATION),
        Some(0.0)
    );
}

#[test]
fn a_negative_rain_rate_is_rejected() {
    assert_eq!(
        specific_rain_attenuation(-1.0, 12.0e9, HORIZONTAL, ELEVATION),
        None
    );
}

#[test]
fn total_attenuation_is_the_specific_value_times_the_effective_path() {
    let result = rain_attenuation(20.0, 12.0e9, HORIZONTAL, ELEVATION).expect("valid");
    assert!(
        (result.attenuation_db - result.specific_db_per_km * result.path_length_km).abs() < 1e-9
    );
    assert!(result.is_valid());
}

#[test]
fn the_effective_path_is_shorter_than_the_geometric_one() {
    // The reduction factor represents the fact that heavy rain does not fill
    // the whole slant path, so the effective length is below the geometric
    // length through a three kilometre layer.
    let result = rain_attenuation(20.0, 12.0e9, HORIZONTAL, ELEVATION).expect("valid");
    let geometric = 3.0 / ELEVATION.sin();
    assert!((geometric - 6.0).abs() < 1e-6);
    assert!(
        result.path_length_km < geometric,
        "{}",
        result.path_length_km
    );
    assert!(result.path_length_km > 0.0);
}

#[test]
fn attenuation_grows_as_the_elevation_falls() {
    let high = rain_attenuation(20.0, 12.0e9, HORIZONTAL, FRAC_PI_2).expect("valid");
    let low = rain_attenuation(20.0, 12.0e9, HORIZONTAL, 10.0_f64.to_radians()).expect("valid");
    assert!(low.attenuation_db > high.attenuation_db);
}

#[test]
fn a_heavy_rain_event_gives_a_significant_attenuation() {
    // A 20 mm/h rain cell at 12 GHz and thirty degrees elevation gives a few
    // decibels, which is the order of magnitude a Ku band link must budget.
    let result = rain_attenuation(20.0, 12.0e9, HORIZONTAL, ELEVATION).expect("valid");
    assert!(
        result.attenuation_db > 0.5 && result.attenuation_db < 5.0,
        "{} dB",
        result.attenuation_db
    );
}

#[test]
fn a_lower_frequency_suffers_less_rain_attenuation() {
    let c_band = rain_attenuation(20.0, 6.0e9, HORIZONTAL, ELEVATION).expect("valid");
    let ku_band = rain_attenuation(20.0, 12.0e9, HORIZONTAL, ELEVATION).expect("valid");
    assert!(ku_band.attenuation_db > c_band.attenuation_db);
}

#[test]
fn an_elevation_at_the_horizon_is_rejected() {
    assert_eq!(rain_attenuation(20.0, 12.0e9, HORIZONTAL, 0.0), None);
    assert_eq!(rain_attenuation(20.0, 12.0e9, HORIZONTAL, -0.1), None);
}

#[test]
fn polarisation_tilt_converts_degrees() {
    assert_eq!(polarisation_tilt(0.0), 0.0);
    assert!((polarisation_tilt(90.0) - FRAC_PI_2).abs() < 1e-12);
    assert!((polarisation_tilt(45.0) - FRAC_PI_2 / 2.0).abs() < 1e-12);
}
