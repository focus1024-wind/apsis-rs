//! Independent validation tests for the link budget wrapper.
//!
//! 链路预算包装的独立验证测试。

use apsis::apsis_link::{LinkParameters, link_budget};

const TOLERANCE: f64 = 1e-9;

fn ku() -> LinkParameters {
    LinkParameters {
        frequency_hz: 12.0e9,
        transmit_power_watts: 10.0,
        transmit_gain_dbi: 30.0,
        transmit_loss_db: 1.0,
        receive_gain_dbi: 35.0,
        receive_loss_db: 0.5,
        system_temperature_kelvin: 150.0,
        bandwidth_hz: 36.0e6,
        data_rate_bps: 10.0e6,
        required_ebno_db: 4.0,
    }
}

#[test]
fn the_ku_band_figures_match_the_closed_form() {
    let result = link_budget(38_000_000.0, &ku()).expect("valid link");
    assert!((result.path_loss_db - 205.627_080_075_172_1).abs() < TOLERANCE);
    assert!((result.eirp_dbw - 39.0).abs() < TOLERANCE);
    assert!((result.received_power_dbw + 132.127_080_075_172_1).abs() < TOLERANCE);
    assert!((result.carrier_to_noise_density_db - 74.711_174_507_488_75).abs() < TOLERANCE);
    assert!((result.carrier_to_noise_db + 0.851_850_500_184_127_7).abs() < TOLERANCE);
    assert!((result.ebno_db - 4.711_174_507_488_749).abs() < TOLERANCE);
    assert!((result.margin_db - 0.711_174_507_488_749).abs() < TOLERANCE);
    assert!(result.closes);
}

#[test]
fn the_gain_to_temperature_matches_the_definition() {
    let result = link_budget(38_000_000.0, &ku()).expect("valid link");
    let expected = 35.0 - 10.0 * 150.0f64.log10();
    assert!((result.gain_to_temperature_db - expected).abs() < TOLERANCE);
}

#[test]
fn ten_times_the_distance_adds_twenty_decibels_of_loss() {
    let near = link_budget(1_000_000.0, &ku()).expect("valid link");
    let far = link_budget(10_000_000.0, &ku()).expect("valid link");
    assert!((far.path_loss_db - near.path_loss_db - 20.0).abs() < TOLERANCE);
}

#[test]
fn ten_times_the_frequency_adds_twenty_decibels_of_loss() {
    let low = LinkParameters {
        frequency_hz: 1.0e9,
        ..ku()
    };
    let high = LinkParameters {
        frequency_hz: 10.0e9,
        ..ku()
    };
    let low = link_budget(1_000_000.0, &low).expect("valid link");
    let high = link_budget(1_000_000.0, &high).expect("valid link");
    assert!((high.path_loss_db - low.path_loss_db - 20.0).abs() < TOLERANCE);
}

#[test]
fn a_negative_margin_reports_an_open_link() {
    let parameters = LinkParameters {
        required_ebno_db: 10.0,
        ..ku()
    };
    let result = link_budget(38_000_000.0, &parameters).expect("valid link");
    assert!(!result.closes);
    assert!(result.margin_db < 0.0);
}

#[test]
fn invalid_inputs_are_rejected() {
    assert!(link_budget(0.0, &ku()).is_err());
    assert!(
        link_budget(
            1.0e6,
            &LinkParameters {
                frequency_hz: 0.0,
                ..ku()
            }
        )
        .is_err()
    );
    assert!(
        link_budget(
            1.0e6,
            &LinkParameters {
                transmit_power_watts: 0.0,
                ..ku()
            }
        )
        .is_err()
    );
    assert!(
        link_budget(
            1.0e6,
            &LinkParameters {
                system_temperature_kelvin: 0.0,
                ..ku()
            }
        )
        .is_err()
    );
    assert!(
        link_budget(
            1.0e6,
            &LinkParameters {
                bandwidth_hz: 0.0,
                ..ku()
            }
        )
        .is_err()
    );
    assert!(
        link_budget(
            1.0e6,
            &LinkParameters {
                data_rate_bps: 0.0,
                ..ku()
            }
        )
        .is_err()
    );
    assert!(
        link_budget(
            1.0e6,
            &LinkParameters {
                transmit_loss_db: -1.0,
                ..ku()
            }
        )
        .is_err()
    );
}
