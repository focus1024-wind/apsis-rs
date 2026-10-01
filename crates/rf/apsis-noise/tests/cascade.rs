//! Tests for the noise cascade.
//!
//! 噪声级联的测试。

use apsis_noise::{
    NoiseTemperature, Stage, cascade_noise_figure_db, cascade_temperature,
    noise_figure_to_temperature, reference_temperature,
};
use apsis_rf::Gain;

const TOLERANCE: f64 = 1e-9;

/// A stage that adds the given temperature with the given gain in decibels.
///
/// 一级，以给定分贝增益加入给定温度。
fn stage(temperature: f64, gain_dbi: f64) -> Stage {
    Stage::new(
        NoiseTemperature::from_kelvin(temperature),
        Gain::from_dbi(gain_dbi).expect("finite gain"),
    )
}

#[test]
fn the_reference_temperature_helper_matches_the_constant() {
    assert_eq!(reference_temperature(), 290.0);
}

#[test]
fn a_single_stage_contributes_its_own_temperature() {
    let only = stage(100.0, 20.0);
    let total = cascade_temperature(&[only]).expect("valid stage");
    assert!((total.kelvin() - 100.0).abs() < TOLERANCE);
}

#[test]
fn an_empty_cascade_has_no_noise() {
    let total = cascade_temperature(&[]).expect("empty is valid");
    assert_eq!(total.kelvin(), 0.0);
}

#[test]
fn the_first_stage_dominates() {
    // A 100 K first stage with 20 dB of gain followed by a 1000 K second
    // stage contributes the second stage divided by 100.
    // 100 K 的第一级带 20 dB 增益，其后接 1000 K 的第二级，贡献第二级除以 100。
    let cascade = [stage(100.0, 20.0), stage(1_000.0, 20.0)];
    let total = cascade_temperature(&cascade).expect("valid stages");
    assert!((total.kelvin() - 110.0).abs() < 1e-6, "{}", total.kelvin());
}

#[test]
fn a_noiseless_stage_contributes_nothing() {
    let cascade = [
        Stage::ideal(Gain::from_dbi(20.0).expect("valid")),
        stage(50.0, 10.0),
    ];
    let total = cascade_temperature(&cascade).expect("valid stages");
    assert!((total.kelvin() - 0.5).abs() < 1e-9, "{}", total.kelvin());
}

#[test]
fn the_friis_formula_matches_by_hand() {
    // Three stages: 50 K at 20 dB, 100 K at 10 dB, 200 K at 0 dB.
    // 三级：50 K 配 20 dB、100 K 配 10 dB、200 K 配 0 dB。
    let cascade = [stage(50.0, 20.0), stage(100.0, 10.0), stage(200.0, 0.0)];
    let total = cascade_temperature(&cascade).expect("valid stages");
    let expected = 50.0 + 100.0 / 100.0 + 200.0 / (100.0 * 10.0);
    assert!((total.kelvin() - expected).abs() < 1e-9);
}

#[test]
fn a_lossy_component_ahead_of_the_receiver_raises_the_noise() {
    // A 3 dB loss is a gain of one half, and it contributes its own
    // temperature divided by the gain of the stages before it, which for a
    // loss is a division by a fraction and so an increase.
    // 3 dB 损耗是二分之一增益，它贡献自身温度除以其前各级的增益，而对损耗而言即除以
    // 一个分数，故是增大。
    let loss = Stage::new(
        NoiseTemperature::from_kelvin(0.0),
        Gain::from_dbi(-3.0).expect("finite"),
    );
    let receiver = stage(100.0, 30.0);
    let total = cascade_temperature(&[loss, receiver]).expect("valid stages");
    // The receiver contributes 100 / 0.5012 = 199.5 K.
    // 接收机贡献 100 / 0.5012 = 199.5 K。
    assert!(
        total.kelvin() > 199.0 && total.kelvin() < 200.0,
        "{}",
        total.kelvin()
    );
}

#[test]
fn the_cascade_figure_follows_from_the_cascade_temperature() {
    let cascade = [stage(50.0, 20.0), stage(100.0, 10.0)];
    let temperature = cascade_temperature(&cascade).expect("valid stages");
    let figure = cascade_noise_figure_db(&cascade).expect("valid stages");
    let expected = temperature.noise_figure_db().expect("valid");
    assert!((figure - expected).abs() < TOLERANCE);
}

#[test]
fn a_noiseless_cascade_has_zero_figure() {
    let cascade = [
        Stage::ideal(Gain::from_dbi(10.0).expect("valid")),
        Stage::ideal(Gain::from_dbi(10.0).expect("valid")),
    ];
    let figure = cascade_noise_figure_db(&cascade).expect("valid stages");
    assert!(figure.abs() < TOLERANCE, "figure {figure}");
}

#[test]
fn a_known_receiver_chain_gives_a_plausible_figure() {
    // A low noise amplifier at 1 dB and 30 dB gain followed by a mixer at
    // 10 dB and 0 dB gain. The cascade figure is close to the first stage
    // plus the second divided by the first stage's gain.
    // 1 dB、30 dB 增益的低噪声放大器，其后接 10 dB、0 dB 增益的混频器。级联系数接近
    // 第一级加上第二级除以第一级增益。
    let low_noise = Stage::from_noise_figure(1.0, 30.0).expect("valid");
    let mixer = Stage::from_noise_figure(10.0, 0.0).expect("valid");
    let figure = cascade_noise_figure_db(&[low_noise, mixer]).expect("valid stages");
    assert!(figure > 1.0 && figure < 1.1, "figure {figure}");
}

#[test]
fn a_single_stage_cascade_figure_is_its_own_figure() {
    let amplifier = Stage::from_noise_figure(2.5, 25.0).expect("valid");
    let figure = cascade_noise_figure_db(&[amplifier]).expect("valid");
    assert!((figure - 2.5).abs() < 1e-9, "figure {figure}");
}

#[test]
fn an_invalid_stage_is_rejected() {
    let bad_gain = Stage::new(NoiseTemperature::from_kelvin(100.0), Gain::from_ratio(-1.0));
    assert!(!bad_gain.is_valid());
    assert_eq!(cascade_temperature(&[bad_gain]), None);

    let bad_temperature = Stage::new(
        NoiseTemperature::from_kelvin(-1.0),
        Gain::from_dbi(10.0).expect("valid"),
    );
    assert!(!bad_temperature.is_valid());
    assert_eq!(cascade_temperature(&[bad_temperature]), None);
}

#[test]
fn the_conversion_helper_matches_the_method() {
    let via_function = noise_figure_to_temperature(3.0).expect("finite");
    let via_method = NoiseTemperature::from_noise_figure_db(3.0).expect("finite");
    assert_eq!(via_function, via_method);
}

#[test]
fn gain_ordering_matters() {
    // Placing the noisy stage first gives a worse cascade than placing it
    // second, which is why a low noise amplifier goes at the front.
    // 把噪声大的级放在最前给出的级联比放在第二差，这正是低噪声放大器置于前端的原因。
    let noisy = stage(500.0, 10.0);
    let quiet = stage(20.0, 20.0);
    let quiet_first = cascade_temperature(&[quiet, noisy]).expect("valid");
    let noisy_first = cascade_temperature(&[noisy, quiet]).expect("valid");
    assert!(
        noisy_first.kelvin() > quiet_first.kelvin(),
        "{} vs {}",
        noisy_first.kelvin(),
        quiet_first.kelvin()
    );
}
