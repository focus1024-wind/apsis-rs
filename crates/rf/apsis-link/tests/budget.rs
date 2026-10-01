//! Tests for the link budget.
//!
//! The expected values are computed by hand from the link equation and
//! recorded here so that a change to any part of the chain shows up as a
//! failing assertion rather than as a silently different budget.
//!
//! 链路预算的测试。
//!
//! 期望值由手工从链路方程计算并记录在此，使该链任何部分的改动都表现为断言失败，
//! 而非无声地不同的预算。

use apsis_link::{
    Decibel, Gain, LinkBudget, LinkGeometry, NoiseTemperature, Power, carrier_to_noise_density,
    gain_to_temperature, link_margin,
};

const TOLERANCE: f64 = 1e-9;

/// A representative Ku band downlink: a ten watt transmitter, a three meter
/// ground station, a 36 MHz transponder, and a 150 K system temperature.
///
/// 一个有代表性的 Ku 频段下行：十瓦发射机、三米地面站、36 MHz 转发器，以及
/// 150 K 的系统温度。
fn downlink() -> LinkBudget {
    LinkBudget {
        transmit_power: Power::from_dbw(10.0).expect("finite"),
        transmit_gain: Gain::from_dbi(30.0).expect("finite"),
        transmit_loss: Decibel::from_db(1.0),
        receive_gain: Gain::from_dbi(35.0).expect("finite"),
        receive_loss: Decibel::from_db(0.5),
        system_temperature: NoiseTemperature::from_kelvin(150.0),
        bandwidth: 36.0e6,
        geometry: LinkGeometry {
            distance: 38_000_000.0,
            frequency: 12.0e9,
        },
    }
}

#[test]
fn eirp_sums_the_power_gain_and_loss() {
    let budget = downlink();
    // 10 dBW + 30 dBi - 1 dB = 39 dBW.
    // 10 dBW + 30 dBi - 1 dB = 39 dBW。
    // 10 dBW + 30 dBi - 1 dB = 39 dBW。
    let eirp = budget.eirp().expect("valid budget");
    assert!((eirp.dbw().expect("positive") - 39.0).abs() < TOLERANCE);
}

#[test]
fn the_path_loss_matches_the_closed_form() {
    let budget = downlink();
    let loss = budget.geometry.path_loss().expect("valid geometry");
    // 4 pi d / lambda with d = 38 000 km and lambda = 0.02498 m gives
    // 205.627 dB.
    // 4 pi d / lambda，取 d = 38 000 km、lambda = 0.02498 m，给出 205.627 dB。
    assert!(
        (loss.db() - 205.627_080_075_172_1).abs() < 1e-9,
        "{}",
        loss.db()
    );
}

#[test]
fn the_carrier_power_matches_the_hand_computation() {
    let budget = downlink();
    let carrier = budget.received_power().expect("valid budget");
    // 39 dBW - 205.627 dB + 35 dBi - 0.5 dB = -132.127 dBW.
    // 39 dBW - 205.627 dB + 35 dBi - 0.5 dB = -132.127 dBW。
    // 39 dBW - 205.627 dB + 35 dBi - 0.5 dB = -132.127 dBW。
    assert!(
        (carrier.dbw().expect("positive") + 132.127_080_075_172_1).abs() < 1e-9,
        "{}",
        carrier.dbw().expect("positive")
    );
}

#[test]
fn the_noise_density_is_the_boltzmann_constant_times_temperature() {
    let budget = downlink();
    let density = budget.noise_density().expect("valid temperature");
    let expected = apsis_link::BOLTZMANN * 150.0;
    assert!((density - expected).abs() < 1e-40);
    // In decibels this is -206.838 dBW/Hz.
    // 以分贝表示为 -206.838 dBW/Hz。
    let db = 10.0 * density.log10();
    assert!((db + 206.838_254_582_660_85).abs() < 1e-9, "{db}");
}

#[test]
fn the_carrier_to_noise_density_matches_the_hand_computation() {
    let budget = downlink();
    let ratio = budget.carrier_to_noise_density().expect("valid budget");
    assert!(
        (ratio.db() - 74.711_174_507_488_75).abs() < 1e-9,
        "{}",
        ratio.db()
    );
}

#[test]
fn the_carrier_to_noise_matches_the_hand_computation() {
    let budget = downlink();
    let ratio = budget.carrier_to_noise().expect("valid budget");
    assert!(
        (ratio.db() + 0.851_850_500_184_127_7).abs() < 1e-9,
        "{}",
        ratio.db()
    );
}

#[test]
fn the_energy_per_bit_matches_the_hand_computation() {
    let budget = downlink();
    let ratio = budget
        .energy_per_bit_over_noise(10.0e6)
        .expect("valid budget");
    assert!(
        (ratio.db() - 4.711_174_507_488_749).abs() < 1e-9,
        "{}",
        ratio.db()
    );
}

#[test]
fn the_carrier_to_noise_density_helper_matches_the_method() {
    let budget = downlink();
    assert_eq!(
        carrier_to_noise_density(&budget),
        budget.carrier_to_noise_density()
    );
}

#[test]
fn a_wider_bandwidth_lowers_the_carrier_to_noise() {
    let narrow = LinkBudget {
        bandwidth: 1.0e6,
        ..downlink()
    };
    let wide = LinkBudget {
        bandwidth: 10.0e6,
        ..downlink()
    };
    let narrow_ratio = narrow.carrier_to_noise().expect("valid");
    let wide_ratio = wide.carrier_to_noise().expect("valid");
    // A factor of ten in bandwidth is ten decibels.
    // 带宽十倍即十分贝。
    assert!((narrow_ratio.db() - wide_ratio.db() - 10.0).abs() < 1e-9);
    // The carrier to noise density ratio does not depend on bandwidth.
    // 载噪密度比不依赖于带宽。
    assert_eq!(
        narrow.carrier_to_noise_density(),
        wide.carrier_to_noise_density()
    );
}

#[test]
fn a_higher_data_rate_lowers_the_energy_per_bit() {
    let budget = downlink();
    let slow = budget.energy_per_bit_over_noise(1.0e6).expect("valid");
    let fast = budget.energy_per_bit_over_noise(10.0e6).expect("valid");
    assert!((slow.db() - fast.db() - 10.0).abs() < 1e-9);
}

#[test]
fn added_noise_raises_the_system_temperature() {
    let budget = downlink();
    let warmed = budget
        .with_added_noise(NoiseTemperature::from_kelvin(50.0))
        .expect("valid temperature");
    assert_eq!(warmed.system_temperature.kelvin(), 200.0);
    // The carrier to noise density ratio falls by 10 log10(200 / 150).
    // 载噪密度比下降 10 log10(200 / 150)。
    let before = budget.carrier_to_noise_density().expect("valid");
    let after = warmed.carrier_to_noise_density().expect("valid");
    let expected_drop = 10.0 * (200.0_f64 / 150.0).log10();
    assert!((before.db() - after.db() - expected_drop).abs() < 1e-9);
}

#[test]
fn the_carrier_wavelength_matches_the_frequency() {
    let budget = downlink();
    let lambda = budget.carrier_wavelength().expect("valid frequency");
    assert!((lambda - 299_792_458.0 / 12.0e9).abs() < 1e-12);
}

#[test]
fn validity_is_reported() {
    assert!(downlink().is_valid());

    let bad_geometry = LinkBudget {
        geometry: LinkGeometry {
            distance: 0.0,
            frequency: 12.0e9,
        },
        ..downlink()
    };
    assert!(!bad_geometry.is_valid());
    assert_eq!(bad_geometry.eirp(), None);
    assert_eq!(bad_geometry.received_power(), None);

    let bad_temperature = LinkBudget {
        system_temperature: NoiseTemperature::from_kelvin(-1.0),
        ..downlink()
    };
    assert!(!bad_temperature.is_valid());
    assert_eq!(bad_temperature.noise_density(), None);
    assert_eq!(bad_temperature.carrier_to_noise_density(), None);

    let bad_bandwidth = LinkBudget {
        bandwidth: -1.0,
        ..downlink()
    };
    assert!(!bad_bandwidth.is_valid());
    assert_eq!(bad_bandwidth.carrier_to_noise(), None);
}

#[test]
fn an_invalid_data_rate_is_rejected() {
    let budget = downlink();
    assert_eq!(budget.energy_per_bit_over_noise(0.0), None);
    assert_eq!(budget.energy_per_bit_over_noise(-1.0), None);
    assert_eq!(budget.energy_per_bit_over_noise(f64::NAN), None);
}

#[test]
fn a_margin_reports_whether_the_link_closes() {
    let achieved = Decibel::from_db(75.0);
    let required = Decibel::from_db(70.0);
    let margin = link_margin(achieved, required);
    assert!((margin.margin().db() - 5.0).abs() < TOLERANCE);
    assert!(margin.closes());

    // A link that exactly meets the requirement closes with zero margin.
    // 恰好满足要求的链路以零余量闭合。
    let exact = link_margin(required, required);
    assert_eq!(exact.margin().db(), 0.0);
    assert!(exact.closes());

    // A link below the requirement does not close.
    // 低于要求的链路不闭合。
    let short = link_margin(Decibel::from_db(65.0), required);
    assert!(!short.closes());
    assert!((short.margin().db() + 5.0).abs() < TOLERANCE);
}

#[test]
fn the_gain_to_temperature_ratio_uses_reciprocal_kelvin() {
    let gain = Gain::from_dbi(35.0).expect("finite");
    let temperature = NoiseTemperature::from_kelvin(150.0);
    let ratio = gain_to_temperature(gain, temperature).expect("valid");
    let expected = 10f64.powf(3.5) / 150.0;
    assert!((ratio - expected).abs() < 1e-9);

    // In decibels this is 35 - 10 log10(150), which is 13.239087 dB/K.
    // 以分贝表示为 35 - 10 log10(150)，即 13.239087 dB/K。
    let db = 10.0 * ratio.log10();
    assert!((db - 13.239_087_409_443_187).abs() < 1e-9, "{db}");
}

#[test]
fn a_hotter_receiver_lowers_the_gain_to_temperature_ratio() {
    let gain = Gain::from_dbi(35.0).expect("finite");
    let cold = gain_to_temperature(gain, NoiseTemperature::from_kelvin(100.0)).expect("valid");
    let hot = gain_to_temperature(gain, NoiseTemperature::from_kelvin(200.0)).expect("valid");
    assert!(cold > hot);
    // Doubling the temperature halves the ratio, which is three decibels.
    // 温度加倍使该比减半，即三分贝。
    assert!((10.0 * (cold / hot).log10() - 3.010_299_956_639_812).abs() < 1e-9);
}

#[test]
fn a_zero_temperature_has_no_gain_to_temperature_ratio() {
    let gain = Gain::from_dbi(35.0).expect("finite");
    assert_eq!(
        gain_to_temperature(gain, NoiseTemperature::from_kelvin(0.0)),
        None
    );
    assert_eq!(
        gain_to_temperature(gain, NoiseTemperature::from_kelvin(-1.0)),
        None
    );
}

#[test]
fn a_budget_can_be_used_with_a_computed_antenna_pattern() {
    // The receive gain of a three meter dish at 12 GHz, rounded to a single
    // decibel, is used here to confirm the budget accepts a gain derived
    // from the antenna layer.
    // 此处使用 12 GHz 下三米天线的接收增益（舍入到一分贝），以确认预算接受由天线层
    // 导出的增益。
    use apsis_link::ParabolicAperture;
    let dish = ParabolicAperture {
        diameter: 3.0,
        efficiency: 0.65,
    };
    let wavelength = 0.025;
    let gain = dish.peak_gain(wavelength).expect("valid dish");
    let budget = LinkBudget {
        receive_gain: gain,
        geometry: LinkGeometry {
            distance: 38_000_000.0,
            frequency: 12.0e9,
        },
        ..downlink()
    };
    assert!(budget.is_valid());
    assert!(budget.carrier_to_noise_density().is_some());
}
