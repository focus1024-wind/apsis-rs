//! Tests for noise temperature and thermal noise power.
//!
//! 噪声温度与热噪声功率的测试。

use apsis::apsis_core::constant::BOLTZMANN;
use apsis::apsis_noise::{NoiseTemperature, REFERENCE_TEMPERATURE_K, thermal_noise_power};

const TOLERANCE: f64 = 1e-12;

#[test]
fn the_reference_temperature_is_290_kelvin() {
    assert_eq!(REFERENCE_TEMPERATURE_K, 290.0);
    assert_eq!(NoiseTemperature::REFERENCE.kelvin(), 290.0);
}

#[test]
fn a_zero_temperature_has_unit_noise_factor() {
    let quiet = NoiseTemperature::from_kelvin(0.0);
    assert_eq!(quiet.noise_factor(), 1.0);
    assert_eq!(quiet.noise_figure_db(), Some(0.0));
}

#[test]
fn the_reference_temperature_is_a_noise_figure_of_three_decibels() {
    // A noise figure is defined as the ratio of the total noise to the
    // noise of a source at the reference temperature, so a stage whose
    // excess noise equals the reference temperature has a factor of two,
    // which is 3.01 dB.
    // 噪声系数定义为总噪声与参考温度下源的噪声之比，故超额噪声等于参考温度的级其
    // 因子为二，即 3.01 dB。
    let value = NoiseTemperature::from_kelvin(REFERENCE_TEMPERATURE_K);
    assert!((value.noise_factor() - 2.0).abs() < TOLERANCE);
    assert!((value.noise_figure_db().expect("valid") - 3.010_299_956_639_812).abs() < 1e-12);
}

#[test]
fn noise_figure_and_temperature_round_trip() {
    for figure in [0.0, 0.5, 1.0, 3.0, 10.0, 20.0] {
        let temperature = NoiseTemperature::from_noise_figure_db(figure).expect("finite figure");
        let recovered = temperature.noise_figure_db().expect("valid temperature");
        assert!((recovered - figure).abs() < 1e-9, "figure {figure}");
    }
}

#[test]
fn noise_factor_and_temperature_round_trip() {
    for factor in [1.0, 1.5, 2.0, 10.0, 100.0] {
        let temperature = NoiseTemperature::from_noise_factor(factor).expect("valid factor");
        assert!(
            (temperature.noise_factor() - factor).abs() < 1e-9,
            "factor {factor}"
        );
    }
}

#[test]
fn a_known_noise_figure_gives_a_known_temperature() {
    // A noise figure of 1 dB is a factor of 1.2589, so the temperature is
    // 290 * 0.2589, about 75.1 K.
    // 1 dB 的噪声系数其因子为 1.2589，故温度为 290 * 0.2589，约 75.1 K。
    let temperature = NoiseTemperature::from_noise_figure_db(1.0).expect("finite");
    assert!(
        (temperature.kelvin() - 75.09).abs() < 0.01,
        "{}",
        temperature.kelvin()
    );
}

#[test]
fn invalid_inputs_are_rejected() {
    assert_eq!(NoiseTemperature::from_noise_figure_db(f64::NAN), None);
    assert_eq!(NoiseTemperature::from_noise_figure_db(f64::INFINITY), None);
    assert_eq!(NoiseTemperature::from_noise_factor(0.5), None);
    assert_eq!(NoiseTemperature::from_noise_factor(f64::NAN), None);
}

#[test]
fn temperatures_add() {
    let first = NoiseTemperature::from_kelvin(50.0);
    let second = NoiseTemperature::from_kelvin(25.0);
    assert_eq!((first + second).kelvin(), 75.0);
    assert_eq!(first.plus(second).kelvin(), 75.0);
}

#[test]
fn validity_is_reported() {
    assert!(NoiseTemperature::from_kelvin(0.0).is_valid());
    assert!(NoiseTemperature::from_kelvin(290.0).is_valid());
    assert!(!NoiseTemperature::from_kelvin(-1.0).is_valid());
    assert!(!NoiseTemperature::from_kelvin(f64::NAN).is_valid());
}

#[test]
fn thermal_noise_power_follows_the_boltzmann_relation() {
    let temperature = NoiseTemperature::from_kelvin(290.0);
    let bandwidth = 1.0e6;
    let power = thermal_noise_power(temperature, bandwidth).expect("valid");
    let expected = BOLTZMANN * 290.0 * bandwidth;
    assert!((power.watts() - expected).abs() < 1e-30);
}

#[test]
fn a_one_hertz_bandwidth_at_290_kelvin_is_minus_204_dbw() {
    // k T B with T = 290 K and B = 1 Hz is 4.0e-21 W, which is -203.98 dBW.
    // k T B，取 T = 290 K、B = 1 Hz，为 4.0e-21 W，即 -203.98 dBW。
    let power = thermal_noise_power(NoiseTemperature::from_kelvin(290.0), 1.0).expect("valid");
    assert!(
        (power.dbw().expect("positive") + 203.98).abs() < 0.01,
        "{} dBW",
        power.dbw().expect("positive")
    );
}

#[test]
fn a_thirty_six_megahertz_transponder_is_about_minus_128_dbw() {
    // k T B with T = 290 K and B = 36 MHz is 1.44e-13 W, which is
    // -128.41 dBW. A one hertz bandwidth at the same temperature is
    // -203.98 dBW, and 36 MHz is 75.56 dB above one hertz.
    // k T B，取 T = 290 K、B = 36 MHz，为 1.44e-13 W，即 -128.41 dBW。同一温度下
    // 一赫兹带宽为 -203.98 dBW，而 36 MHz 比一赫兹高 75.56 dB。
    let power = thermal_noise_power(NoiseTemperature::from_kelvin(290.0), 36.0e6).expect("valid");
    let dbw = power.dbw().expect("positive");
    assert!((dbw + 128.41).abs() < 0.01, "{dbw} dBW");
}

#[test]
fn noise_power_scales_with_temperature_and_bandwidth() {
    let one_hertz = thermal_noise_power(NoiseTemperature::from_kelvin(100.0), 1.0).expect("valid");
    let hundred_hertz =
        thermal_noise_power(NoiseTemperature::from_kelvin(100.0), 100.0).expect("valid");
    let double_temperature =
        thermal_noise_power(NoiseTemperature::from_kelvin(200.0), 1.0).expect("valid");
    // Each doubling of either factor adds three decibels.
    // 任一因子每加倍一次加三分贝。
    assert!(
        (hundred_hertz.dbw().expect("positive") - one_hertz.dbw().expect("positive") - 20.0).abs()
            < 1e-9
    );
    assert!(
        (double_temperature.dbw().expect("positive")
            - one_hertz.dbw().expect("positive")
            - 3.010_299_956_639_812)
            .abs()
            < 1e-9
    );
}

#[test]
fn a_zero_bandwidth_produces_no_noise() {
    assert_eq!(
        thermal_noise_power(NoiseTemperature::from_kelvin(290.0), 0.0),
        None
    );
}

#[test]
fn invalid_arguments_are_rejected() {
    assert_eq!(
        thermal_noise_power(NoiseTemperature::from_kelvin(-1.0), 1.0e6),
        None
    );
    assert_eq!(
        thermal_noise_power(NoiseTemperature::from_kelvin(290.0), -1.0),
        None
    );
    assert_eq!(
        thermal_noise_power(NoiseTemperature::from_kelvin(290.0), f64::NAN),
        None
    );
}

#[test]
fn display_includes_the_unit() {
    assert_eq!(NoiseTemperature::from_kelvin(290.0).to_string(), "290 K");
}
