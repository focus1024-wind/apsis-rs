//! Validation tests for the EOIR optics and radiometry.
//!
//! EOIR 光学与辐射的验证测试。

use apsis::apsis_eoir::{
    EoirSensor, SpectralBand, analyze, band_fraction, band_radiance, planck_radiance,
    total_radiance,
};

const TOLERANCE: f64 = 1e-9;

#[test]
fn the_planck_peak_follows_wien_displacement() {
    for temperature in [300.0_f64, 1_000.0, 5_800.0] {
        let expected = 2.897_771_955e-3 / temperature * 1e6;
        let low = expected * 0.5;
        let high = expected * 1.5;
        let mut best = low;
        let mut best_value = 0.0;
        for step in 0..20_000 {
            let wavelength = low + (high - low) * step as f64 / 19_999.0;
            let value = planck_radiance(wavelength, temperature).expect("valid");
            if value > best_value {
                best_value = value;
                best = wavelength;
            }
        }
        assert!(
            (best - expected).abs() / expected < 1e-3,
            "T {temperature}: peak {best} against {expected}"
        );
    }
}

#[test]
fn the_total_radiance_matches_stefan_boltzmann() {
    for temperature in [300.0_f64, 1_000.0] {
        let expected = 5.670_374_419e-8 * temperature.powi(4) / core::f64::consts::PI;
        assert!((total_radiance(temperature).expect("valid") - expected).abs() < TOLERANCE);
    }
}

#[test]
fn the_lwir_band_radiance_matches_the_reference_integration() {
    let value = band_radiance(SpectralBand::LWIR, 300.0).expect("valid");
    assert!((value - 38.500_423_933_347_74).abs() < 0.01, "{value}");
}

#[test]
fn the_mwir_band_radiance_matches_the_reference_integration() {
    let value = band_radiance(SpectralBand::MWIR, 300.0).expect("valid");
    assert!((value - 1.865_956_208_161_694_7).abs() < 0.001, "{value}");
}

#[test]
fn the_visible_band_is_a_negligible_fraction_at_room_temperature() {
    let fraction = band_fraction(SpectralBand::VISIBLE, 300.0).expect("valid");
    assert!(fraction < 1e-20, "{fraction}");
}

#[test]
fn the_optics_match_the_closed_form() {
    let sensor = EoirSensor::new(0.1, 0.5, 10e-6, 1024, 768).expect("valid");
    assert!((sensor.ifov_radians() - 2e-5).abs() < 1e-15);
    assert!((sensor.gsd_m(10_000.0).expect("valid") - 0.2).abs() < TOLERANCE);
    assert!((sensor.fov_degrees() - 1.173_376_553_091_570_7).abs() < 1e-12);
    assert!((sensor.diffraction_limit_radians(10.0).expect("valid") - 1.22e-4).abs() < 1e-15);
    assert!((sensor.airy_diameter_m(10.0).expect("valid") - 1.22e-4).abs() < 1e-15);
    let (width, height) = sensor.footprint_m(10_000.0).expect("valid");
    assert!((width - 204.8).abs() < TOLERANCE);
    assert!((height - 153.6).abs() < TOLERANCE);
}

#[test]
fn the_gsd_scales_with_range() {
    let sensor = EoirSensor::new(0.1, 0.5, 10e-6, 1024, 768).expect("valid");
    let near = sensor.gsd_m(5_000.0).expect("valid");
    let far = sensor.gsd_m(10_000.0).expect("valid");
    assert!((far - 2.0 * near).abs() < TOLERANCE);
}

#[test]
fn the_analysis_reports_target_pixels_and_max_range() {
    let sensor = EoirSensor::new(0.1, 0.5, 10e-6, 1024, 768).expect("valid");
    let result = analyze(
        sensor,
        SpectralBand::LWIR,
        10_000.0,
        310.0,
        290.0,
        0.95,
        2.0,
        3.0,
    )
    .expect("valid");
    assert!((result.gsd_m - 0.2).abs() < TOLERANCE);
    assert!((result.target_pixels - 10.0).abs() < TOLERANCE);
    assert!((result.max_range_m - 2.0 / (3.0 * 2e-5)).abs() < TOLERANCE);
    assert!(result.contrast_radiance > 0.0);
}

#[test]
fn invalid_inputs_are_rejected() {
    assert!(SpectralBand::new(0.7, 0.4).is_err());
    assert!(planck_radiance(0.0, 300.0).is_err());
    assert!(planck_radiance(10.0, 0.0).is_err());
    assert!(EoirSensor::new(0.0, 0.5, 10e-6, 100, 100).is_err());
    let sensor = EoirSensor::new(0.1, 0.5, 10e-6, 1024, 768).expect("valid");
    assert!(analyze(sensor, SpectralBand::LWIR, 0.0, 300.0, 290.0, 0.9, 2.0, 3.0).is_err());
}
