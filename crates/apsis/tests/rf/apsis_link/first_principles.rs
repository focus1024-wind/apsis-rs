//! Independent validation of the link budget against closed-form equations.
//!
//! Every expected value in this file is computed from the defining equations
//! with raw `f64` arithmetic and the SI constants, without calling any of the
//! budget's own helpers. A disagreement therefore points at the budget chain
//! rather than at a shared intermediate value.
//!
//! 链路预算对闭式方程的独立验证。
//!
//! 本文件中的每个期望值都由原始 `f64` 运算与 SI 常量按定义方程算出，不调用
//! 预算自身的任何辅助函数。故不一致指向预算链本身，而非共享的中间值。

use apsis::apsis_link::{
    Decibel, Gain, LinkBudget, LinkGeometry, NoiseTemperature, Power, gain_to_temperature,
    link_margin,
};

const SPEED_OF_LIGHT: f64 = 299_792_458.0;
const BOLTZMANN: f64 = 1.380_649e-23;
const TOLERANCE_DB: f64 = 1e-9;

fn budget(
    power_dbw: f64,
    tx_gain_dbi: f64,
    tx_loss_db: f64,
    rx_gain_dbi: f64,
    rx_loss_db: f64,
    temperature_k: f64,
    bandwidth_hz: f64,
    distance_m: f64,
    frequency_hz: f64,
) -> LinkBudget {
    LinkBudget {
        transmit_power: Power::from_dbw(power_dbw).expect("finite power"),
        transmit_gain: Gain::from_dbi(tx_gain_dbi).expect("finite gain"),
        transmit_loss: Decibel::from_db(tx_loss_db),
        receive_gain: Gain::from_dbi(rx_gain_dbi).expect("finite gain"),
        receive_loss: Decibel::from_db(rx_loss_db),
        system_temperature: NoiseTemperature::from_kelvin(temperature_k),
        bandwidth: bandwidth_hz,
        geometry: LinkGeometry {
            distance: distance_m,
            frequency: frequency_hz,
        },
    }
}

/// Returns path loss, EIRP, carrier power, C/N0, C/N, and Eb/N0 from the
/// defining equations.
///
/// 由定义方程返回路径损耗、EIRP、载波功率、C/N0、C/N 与 Eb/N0。
fn closed_form(budget: &LinkBudget, data_rate: f64) -> [f64; 6] {
    let wavelength = SPEED_OF_LIGHT / budget.geometry.frequency;
    let path_loss = 20.0 * (4.0 * core::f64::consts::PI * budget.geometry.distance / wavelength).log10();
    let eirp = 10.0 * budget.transmit_power.watts().log10()
        + budget.transmit_gain.dbi().expect("gain")
        - budget.transmit_loss.db();
    let carrier = eirp - path_loss + budget.receive_gain.dbi().expect("gain") - budget.receive_loss.db();
    let noise_density = 10.0 * (BOLTZMANN * budget.system_temperature.kelvin()).log10();
    let c_n0 = carrier - noise_density;
    let c_n = c_n0 - 10.0 * budget.bandwidth.log10();
    let eb_n0 = c_n0 - 10.0 * data_rate.log10();
    [path_loss, eirp, carrier, c_n0, c_n, eb_n0]
}

fn check(name: &str, budget: &LinkBudget, data_rate: f64) {
    let expected = closed_form(budget, data_rate);
    let actual = [
        budget.geometry.path_loss().expect("path").db(),
        budget.eirp().expect("eirp").dbw().expect("positive"),
        budget.received_power().expect("carrier").dbw().expect("positive"),
        budget.carrier_to_noise_density().expect("c/n0").db(),
        budget.carrier_to_noise().expect("c/n").db(),
        budget.energy_per_bit_over_noise(data_rate).expect("eb/n0").db(),
    ];
    let labels = ["path loss", "EIRP", "carrier", "C/N0", "C/N", "Eb/N0"];
    for index in 0..actual.len() {
        assert!(
            (actual[index] - expected[index]).abs() < TOLERANCE_DB,
            "{name}: {} differs by {} dB (actual {}, expected {})",
            labels[index],
            actual[index] - expected[index],
            actual[index],
            expected[index]
        );
    }
}

#[test]
fn ku_band_downlink_matches_the_closed_form() {
    let budget = budget(10.0, 30.0, 1.0, 35.0, 0.5, 150.0, 36.0e6, 38_000_000.0, 12.0e9);
    check("Ku downlink", &budget, 10.0e6);
}

#[test]
fn uhf_telemetry_matches_the_closed_form() {
    let budget = budget(7.0, 3.0, 2.0, 0.0, 0.5, 500.0, 25.0e3, 1_000_000.0, 400.0e6);
    check("UHF telemetry", &budget, 9_600.0);
}

#[test]
fn deep_space_x_band_matches_the_closed_form() {
    let budget = budget(13.0, 40.0, 1.0, 60.0, 0.2, 20.0, 1.0e6, 1.0e9, 8.4e9);
    check("X-band deep space", &budget, 1.0e6);
}

#[test]
fn every_decade_of_distance_adds_twenty_decibels_of_loss() {
    let near = budget(10.0, 30.0, 0.0, 35.0, 0.0, 150.0, 1.0e6, 1_000_000.0, 12.0e9);
    let far = budget(10.0, 30.0, 0.0, 35.0, 0.0, 150.0, 1.0e6, 10_000_000.0, 12.0e9);
    let near_loss = near.geometry.path_loss().expect("path").db();
    let far_loss = far.geometry.path_loss().expect("path").db();
    assert!((far_loss - near_loss - 20.0).abs() < TOLERANCE_DB);
}

#[test]
fn every_decade_of_frequency_adds_twenty_decibels_of_loss() {
    let low = budget(10.0, 30.0, 0.0, 35.0, 0.0, 150.0, 1.0e6, 1_000_000.0, 1.0e9);
    let high = budget(10.0, 30.0, 0.0, 35.0, 0.0, 150.0, 1.0e6, 1_000_000.0, 10.0e9);
    let low_loss = low.geometry.path_loss().expect("path").db();
    let high_loss = high.geometry.path_loss().expect("path").db();
    assert!((high_loss - low_loss - 20.0).abs() < TOLERANCE_DB);
}

#[test]
fn decibel_conversions_agree_with_their_definitions() {
    assert!((Power::from_dbw(10.0).expect("finite").watts() - 10.0).abs() < 1e-12);
    assert!((Power::from_dbm(30.0).expect("finite").watts() - 1.0).abs() < 1e-12);
    assert!((Gain::from_dbi(30.0).expect("finite").ratio() - 1000.0).abs() < 1e-9);
    assert!((Decibel::from_db(3.0).ratio() - 1.995_262_314_968_875).abs() < 1e-12);
}

#[test]
fn the_gain_to_temperature_matches_its_definition() {
    let gain = Gain::from_dbi(35.0).expect("finite");
    let temperature = NoiseTemperature::from_kelvin(150.0);
    let ratio = gain_to_temperature(gain, temperature).expect("valid");
    let expected = 10f64.powf(35.0 / 10.0) / 150.0;
    assert!((ratio - expected).abs() < 1e-9);
    let in_db = 10.0 * ratio.log10();
    assert!((in_db - (35.0 - 10.0 * 150.0f64.log10())).abs() < 1e-9);
}

#[test]
fn a_zero_margin_closes_and_a_negative_one_does_not() {
    let exact = link_margin(Decibel::from_db(70.0), Decibel::from_db(70.0));
    assert!(exact.closes());
    assert_eq!(exact.margin().db(), 0.0);
    let short = link_margin(Decibel::from_db(69.9), Decibel::from_db(70.0));
    assert!(!short.closes());
}
