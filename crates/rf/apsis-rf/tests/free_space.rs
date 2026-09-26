//! Tests for free space propagation.

use apsis_core::constant::SPEED_OF_LIGHT;
use apsis_rf::{Gain, free_space_path_loss, free_space_received_ratio, wavelength};

const TOLERANCE: f64 = 1e-9;

#[test]
fn wavelength_is_the_speed_of_light_over_frequency() {
    assert!((wavelength(SPEED_OF_LIGHT).expect("valid") - 1.0).abs() < TOLERANCE);
    // A frequency of 300 MHz has a wavelength of about one meter.
    assert!((wavelength(300.0e6).expect("valid") - 0.999_308).abs() < 1e-5);
    // The same relation holds for a microwave frequency.
    assert!((wavelength(12.0e9).expect("valid") - 0.024_982_7).abs() < 1e-6);
}

#[test]
fn wavelength_rejects_invalid_frequencies() {
    assert_eq!(wavelength(0.0), None);
    assert_eq!(wavelength(-1.0), None);
    assert_eq!(wavelength(f64::NAN), None);
    assert_eq!(wavelength(f64::INFINITY), None);
}

#[test]
fn path_loss_matches_the_closed_form() {
    // Attenuation over one meter at one meter wavelength: the ratio is
    // (4 pi)^2, about 21.98 dB.
    let loss = free_space_path_loss(1.0, SPEED_OF_LIGHT).expect("valid arguments");
    let expected = 10.0 * (4.0 * core::f64::consts::PI).powi(2).log10();
    assert!((loss.db() - expected).abs() < 1e-9, "loss {}", loss.db());
    assert!((loss.db() - 21.984).abs() < 0.001);
}

#[test]
fn path_loss_matches_the_engineering_form() {
    // The form used in link budgets, with distance in kilometers and
    // frequency in megahertz,
    //
    //   L = 32.44 + 20 log10(d_km) + 20 log10(f_MHz)
    //
    // has a constant that follows from the speed of light rounded to
    // 299 792 458 m/s, and agrees with the expression used here to the
    // precision of that constant.
    let distance_km = 1_000.0;
    let frequency_mhz = 12_000.0;
    let loss = free_space_path_loss(distance_km * 1_000.0, frequency_mhz * 1.0e6)
        .expect("valid arguments");
    let engineering = 32.44 + 20.0 * distance_km.log10() + 20.0 * frequency_mhz.log10();
    assert!(
        (loss.db() - engineering).abs() < 0.01,
        "loss {} engineering {}",
        loss.db(),
        engineering
    );
}

#[test]
fn a_larger_distance_increases_the_loss_by_six_decibels_per_doubling() {
    let near = free_space_path_loss(1_000.0, 1.0e9).expect("valid");
    let far = free_space_path_loss(2_000.0, 1.0e9).expect("valid");
    assert!((far.db() - near.db() - 6.020_6).abs() < 1e-3);
}

#[test]
fn a_higher_frequency_increases_the_loss() {
    let low = free_space_path_loss(1_000.0, 1.0e9).expect("valid");
    let high = free_space_path_loss(1_000.0, 2.0e9).expect("valid");
    assert!(high.db() > low.db());
    assert!((high.db() - low.db() - 6.020_6).abs() < 1e-3);
}

#[test]
fn a_zero_or_negative_distance_is_rejected() {
    // A zero distance is outside the far field range where the expression
    // applies, and would give an infinite loss in decibels.
    assert_eq!(free_space_path_loss(0.0, 1.0e9), None);
    assert_eq!(free_space_path_loss(-1.0, 1.0e9), None);
}

#[test]
fn invalid_arguments_are_rejected() {
    assert_eq!(free_space_path_loss(-1.0, 1.0e9), None);
    assert_eq!(free_space_path_loss(f64::NAN, 1.0e9), None);
    assert_eq!(free_space_path_loss(1_000.0, 0.0), None);
    assert_eq!(free_space_path_loss(1_000.0, -1.0), None);
}

#[test]
fn received_ratio_combines_gains_and_loss() {
    let distance = 1_000_000.0;
    let frequency = 12.0e9;
    let transmitter = Gain::from_dbi(30.0).expect("valid");
    let receiver = Gain::from_dbi(40.0).expect("valid");
    let ratio =
        free_space_received_ratio(distance, frequency, transmitter, receiver).expect("valid");
    let loss = free_space_path_loss(distance, frequency).expect("valid");
    assert!((ratio.db() - (30.0 + 40.0 - loss.db())).abs() < 1e-9);
}

#[test]
fn a_large_receive_gain_can_exceed_the_spreading_loss() {
    // Two dishes of 40 dBi at ten meters and 12 GHz. The spreading loss
    // there is about 74 dB, so the 80 dB of combined gain leaves a positive
    // ratio. This is the near field of a large antenna pair and is the
    // reason a link budget cannot be read without also checking the
    // distance regime.
    let ratio = free_space_received_ratio(
        10.0,
        12.0e9,
        Gain::from_dbi(40.0).expect("valid"),
        Gain::from_dbi(40.0).expect("valid"),
    )
    .expect("valid");
    assert!(ratio.db() > 0.0, "ratio {}", ratio.db());
    assert!((ratio.db() - 5.969).abs() < 0.01, "ratio {}", ratio.db());
}

#[test]
fn invalid_received_ratio_arguments_are_rejected() {
    assert_eq!(
        free_space_received_ratio(-1.0, 1.0e9, Gain::ISOTROPIC, Gain::ISOTROPIC),
        None
    );
    assert_eq!(
        free_space_received_ratio(1_000.0, 0.0, Gain::ISOTROPIC, Gain::ISOTROPIC),
        None
    );
}
