//! Tests for antenna gain.
//!
//! 天线增益的测试。

use apsis::apsis_rf::{DIPOLE_OVER_ISOTROPIC_DB, Gain, HALF_WAVE_DIPOLE_RATIO};
use core::f64::consts::PI;

const TOLERANCE: f64 = 1e-9;

#[test]
fn named_gains_match_their_definitions() {
    assert_eq!(Gain::ISOTROPIC.ratio(), 1.0);
    assert_eq!(Gain::ISOTROPIC.dbi(), Some(0.0));
    assert!(Gain::ISOTROPIC.dbd().expect("positive") < 0.0);

    // The linear ratio and the decibel offset are derived from each other,
    // so the two must agree.
    // 线性比值与分贝偏移由彼此导出，故二者必须一致。
    assert!((Gain::HALF_WAVE_DIPOLE.ratio() - HALF_WAVE_DIPOLE_RATIO).abs() < 1e-15);
    assert!(
        (Gain::HALF_WAVE_DIPOLE.dbi().expect("positive") - DIPOLE_OVER_ISOTROPIC_DB).abs() < 1e-12
    );
    assert!(Gain::HALF_WAVE_DIPOLE.dbd().expect("positive").abs() < 1e-9);
}

#[test]
fn a_dipole_gain_is_zero_dbd_and_positive_dbi() {
    let gain = Gain::from_dbd(0.0).expect("finite reading");
    assert!((gain.ratio() - Gain::HALF_WAVE_DIPOLE.ratio()).abs() < 1e-12);
    assert!((gain.dbd().expect("positive")).abs() < 1e-12);
    assert!((gain.dbi().expect("positive") - DIPOLE_OVER_ISOTROPIC_DB).abs() < 1e-9);
}

#[test]
fn decibel_forms_round_trip() {
    for dbi in [-10.0, -3.0, 0.0, 3.0, 20.0, 45.0] {
        let gain = Gain::from_dbi(dbi).expect("finite reading");
        assert!(
            (gain.dbi().expect("positive") - dbi).abs() < 1e-9,
            "dbi {dbi}"
        );
    }
    for dbd in [-5.0, 0.0, 10.0, 20.0] {
        let gain = Gain::from_dbd(dbd).expect("finite reading");
        assert!(
            (gain.dbd().expect("positive") - dbd).abs() < 1e-9,
            "dbd {dbd}"
        );
    }
}

#[test]
fn dbd_and_dbi_differ_by_the_dipole_offset() {
    let gain = Gain::from_dbi(12.0).expect("finite reading");
    assert!(
        (gain.dbi().expect("positive") - gain.dbd().expect("positive") - DIPOLE_OVER_ISOTROPIC_DB)
            .abs()
            < 1e-9
    );
}

#[test]
fn non_finite_readings_are_rejected() {
    assert_eq!(Gain::from_dbi(f64::NAN), None);
    assert_eq!(Gain::from_dbi(f64::INFINITY), None);
    assert_eq!(Gain::from_dbd(f64::NAN), None);
}

#[test]
fn effective_aperture_follows_the_standard_relation() {
    // A gain of one has an aperture of lambda^2 / (4 pi).
    // 增益为一的孔径为 lambda^2 / (4 pi)。
    let gain = Gain::ISOTROPIC;
    let aperture = gain.effective_aperture(0.3).expect("valid wavelength");
    assert!((aperture - 0.3 * 0.3 / (4.0 * PI)).abs() < TOLERANCE);
}

#[test]
fn a_larger_wavelength_needs_a_larger_aperture() {
    let gain = Gain::from_dbi(30.0).expect("finite reading");
    let small = gain.effective_aperture(0.01).expect("valid");
    let large = gain.effective_aperture(0.1).expect("valid");
    assert!(large > small);
    // The aperture grows with the square of the wavelength.
    // 孔径随波长的平方增大。
    assert!((large / small - 100.0).abs() < 1e-6);
}

#[test]
fn aperture_and_gain_round_trip() {
    let gain = Gain::from_dbi(33.0).expect("finite reading");
    let wavelength = 0.025;
    let aperture = gain.effective_aperture(wavelength).expect("valid");
    let recovered = Gain::from_effective_aperture(aperture, wavelength).expect("valid");
    assert!((recovered.ratio() - gain.ratio()).abs() < 1e-6);
}

#[test]
fn invalid_aperture_arguments_are_rejected() {
    assert_eq!(Gain::ISOTROPIC.effective_aperture(0.0), None);
    assert_eq!(Gain::ISOTROPIC.effective_aperture(-1.0), None);
    assert_eq!(Gain::ISOTROPIC.effective_aperture(f64::NAN), None);
    assert_eq!(Gain::from_effective_aperture(1.0, 0.0), None);
    assert_eq!(Gain::from_effective_aperture(-1.0, 1.0), None);
}

#[test]
fn decibel_conversion_is_available() {
    let gain = Gain::from_dbi(10.0).expect("finite reading");
    let decibel = gain.as_decibel().expect("positive");
    assert!((decibel.db() - 10.0).abs() < 1e-9);
}

#[test]
fn default_is_isotropic() {
    assert_eq!(Gain::default(), Gain::ISOTROPIC);
}

#[test]
fn display_prefers_dbi() {
    assert_eq!(Gain::ISOTROPIC.to_string(), "0 dBi");
}
