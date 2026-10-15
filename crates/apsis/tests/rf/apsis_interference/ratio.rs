//! Tests for carrier to interference computation.
//!
//! 载干比计算的测试。

use apsis::apsis_interference::{
    BOLTZMANN, Decibel, Interferer, Power, carrier_to_interference, carrier_to_interference_single,
    carrier_to_noise_plus_interference, combine, equivalent_noise_temperature,
    interference_to_noise_ratio, summarise, total_interference,
};

const TOLERANCE: f64 = 1e-9;

/// A wanted carrier at a convenient level.
///
/// 一个处于便利电平的有用载波。
fn carrier() -> Power {
    Power::from_dbw(-130.0).expect("finite")
}

/// An interferer at the given level relative to the carrier.
///
/// 一个相对载波处于给定电平的干扰源。
fn interferer_at(decibels_below: f64) -> Interferer {
    Interferer::new(Power::from_dbw(-130.0 - decibels_below).expect("finite"))
}

#[test]
fn a_single_interferer_gives_the_expected_ratio() {
    // An interferer ten decibels below the carrier gives a ratio of 10 dB.
    // 比载波低十分贝的干扰源给出 10 dB 的比。
    let ratio = carrier_to_interference_single(carrier(), interferer_at(10.0)).expect("valid");
    assert!((ratio.db() - 10.0).abs() < TOLERANCE, "{}", ratio.db());
}

#[test]
fn an_equal_interferer_gives_zero_decibels() {
    let ratio = carrier_to_interference_single(carrier(), interferer_at(0.0)).expect("valid");
    assert!(ratio.db().abs() < TOLERANCE);
}

#[test]
fn an_interferer_above_the_carrier_gives_a_negative_ratio() {
    let ratio = carrier_to_interference_single(carrier(), interferer_at(-5.0)).expect("valid");
    assert!((ratio.db() + 5.0).abs() < TOLERANCE);
}

#[test]
fn interfering_powers_add() {
    // Two equal interferers give twice the power of one, which is three
    // decibels above one alone.
    // 两个相等干扰源给出的功率是单个的两倍，即比单个高三分贝。
    let one = total_interference(&[interferer_at(10.0)]).expect("valid");
    let two = total_interference(&[interferer_at(10.0), interferer_at(10.0)]).expect("valid");
    assert!(
        (two.dbw().expect("positive") - one.dbw().expect("positive") - 3.010_299_956_639_812).abs()
            < 1e-9
    );
}

#[test]
fn an_empty_interferer_list_has_no_power() {
    let total = total_interference(&[]).expect("empty is valid");
    assert_eq!(total.watts(), 0.0);
    // With no interference there is no ratio to compute.
    // 没有干扰时没有可比。
    assert_eq!(carrier_to_interference(carrier(), &[]), None);
}

#[test]
fn the_ratio_combines_multiple_interferers() {
    // One interferer at 10 dB below gives 10 dB. Adding a second at the
    // same level halves the ratio, which is three decibels less.
    // 一个低 10 dB 的干扰源给出 10 dB。再加一个相同电平的使该比减半，即少三分贝。
    let single = carrier_to_interference(carrier(), &[interferer_at(10.0)]).expect("valid");
    let pair = carrier_to_interference(carrier(), &[interferer_at(10.0), interferer_at(10.0)])
        .expect("valid");
    assert!(
        (single.db() - pair.db() - 3.010_299_956_639_812).abs() < 1e-9,
        "{} vs {}",
        single.db(),
        pair.db()
    );
}

#[test]
fn a_dominant_interferer_hides_the_others() {
    // An interferer twenty decibels weaker than the carrier contributes
    // almost nothing to the total, so it barely changes the ratio.
    // 比载波弱二十分贝的干扰源对总量几乎无贡献，故几乎不改变该比。
    let strong = carrier_to_interference(carrier(), &[interferer_at(10.0)]).expect("valid");
    let with_weak = carrier_to_interference(carrier(), &[interferer_at(10.0), interferer_at(30.0)])
        .expect("valid");
    assert!(with_weak.db() < strong.db());
    assert!(
        strong.db() - with_weak.db() < 0.05,
        "change was {}",
        strong.db() - with_weak.db()
    );
}

#[test]
fn invalid_inputs_are_rejected() {
    let zero_carrier = Power::from_watts(0.0);
    assert_eq!(
        carrier_to_interference(zero_carrier, &[interferer_at(10.0)]),
        None
    );
    let negative = Interferer::new(Power::from_watts(-1.0));
    assert_eq!(carrier_to_interference(carrier(), &[negative]), None);
    assert_eq!(total_interference(&[negative]), None);
    // A zero interferer gives an infinite ratio, which has no finite
    // decibel value here.
    // 零干扰源给出无穷的比，此处没有有限的分贝值。
    let zero_interferer = Interferer::new(Power::from_watts(0.0));
    assert_eq!(carrier_to_interference(carrier(), &[zero_interferer]), None);
}

#[test]
fn combining_two_equal_ratios_lowers_the_result_by_three_decibels() {
    let first = Decibel::from_db(20.0);
    let second = Decibel::from_db(20.0);
    let combined = combine(first, second).expect("valid");
    assert!(
        (combined.db() - 16.989_700_043_360_188).abs() < 1e-9,
        "{}",
        combined.db()
    );
}

#[test]
fn combining_a_small_ratio_with_a_large_one_gives_the_small_one() {
    // A carrier to noise ratio of 30 dB alongside a carrier to interference
    // ratio of 5 dB is dominated by the interference.
    // 30 dB 的载噪比与 5 dB 的载干比并存时，由干扰主导。
    let combined = combine(Decibel::from_db(30.0), Decibel::from_db(5.0)).expect("valid");
    assert!(combined.db() < 5.0);
    assert!(
        (combined.db() - 4.986_288_071_673_166).abs() < 1e-9,
        "{}",
        combined.db()
    );
}

#[test]
fn combining_is_symmetric() {
    let first = Decibel::from_db(17.0);
    let second = Decibel::from_db(9.0);
    let forward = combine(first, second).expect("valid");
    let backward = combine(second, first).expect("valid");
    assert!((forward.db() - backward.db()).abs() < TOLERANCE);
}

#[test]
fn the_same_helper_is_available_for_noise_plus_interference() {
    let noise = Decibel::from_db(30.0);
    let interference = Decibel::from_db(5.0);
    let direct = combine(noise, interference).expect("valid");
    let named = carrier_to_noise_plus_interference(noise, interference).expect("valid");
    assert!((direct.db() - named.db()).abs() < TOLERANCE);
}

#[test]
fn invalid_ratios_are_rejected_when_combining() {
    assert_eq!(
        combine(Decibel::from_db(f64::NAN), Decibel::from_db(1.0)),
        None
    );
    assert_eq!(
        combine(Decibel::from_db(1.0), Decibel::from_db(f64::INFINITY)),
        None
    );
}

#[test]
fn the_interference_to_noise_ratio_compares_two_powers() {
    let interference = Power::from_dbw(-140.0).expect("finite");
    let noise = Power::from_dbw(-130.0).expect("finite");
    let ratio = interference_to_noise_ratio(interference, noise).expect("valid");
    // The interference is ten decibels below the noise.
    // 干扰比噪声低十分贝。
    assert!((ratio.db() + 10.0).abs() < TOLERANCE);
}

#[test]
fn invalid_powers_are_rejected_in_the_interference_to_noise_ratio() {
    let noise = Power::from_dbw(-130.0).expect("finite");
    assert_eq!(
        interference_to_noise_ratio(Power::from_watts(0.0), noise),
        None
    );
    assert_eq!(
        interference_to_noise_ratio(Power::from_watts(-1.0), noise),
        None
    );
    assert_eq!(
        interference_to_noise_ratio(
            Power::from_dbw(-140.0).expect("finite"),
            Power::from_watts(0.0)
        ),
        None
    );
}

#[test]
fn the_equivalent_noise_temperature_follows_the_boltzmann_relation() {
    let interference = Power::from_dbw(-140.0).expect("finite");
    let bandwidth = 36.0e6;
    let temperature = equivalent_noise_temperature(interference, bandwidth).expect("valid");
    let expected = interference.watts() / (BOLTZMANN * bandwidth);
    assert!((temperature - expected).abs() < 1e-9);
    // The interference is 1e-14 W, so the temperature is about 20 K.
    // 干扰为 1e-14 W，故温度约为 20 K。
    assert!(
        temperature > 15.0 && temperature < 25.0,
        "temperature {temperature}"
    );
}

#[test]
fn the_equivalent_temperature_grows_with_the_interfering_power() {
    let low = equivalent_noise_temperature(Power::from_dbw(-150.0).expect("finite"), 1.0e6)
        .expect("valid");
    let high = equivalent_noise_temperature(Power::from_dbw(-140.0).expect("finite"), 1.0e6)
        .expect("valid");
    assert!(high > low);
    // Ten decibels of power is ten times the temperature.
    // 十分贝的功率是温度的十倍。
    assert!((high / low - 10.0).abs() < 1e-6);
}

#[test]
fn the_equivalent_temperature_falls_with_bandwidth() {
    let narrow = equivalent_noise_temperature(Power::from_dbw(-140.0).expect("finite"), 1.0e6)
        .expect("valid");
    let wide = equivalent_noise_temperature(Power::from_dbw(-140.0).expect("finite"), 10.0e6)
        .expect("valid");
    assert!(narrow > wide);
    assert!((narrow / wide - 10.0).abs() < 1e-6);
}

#[test]
fn invalid_arguments_are_rejected_for_the_temperature() {
    let interference = Power::from_dbw(-140.0).expect("finite");
    assert_eq!(
        equivalent_noise_temperature(Power::from_watts(0.0), 1.0e6),
        None
    );
    assert_eq!(equivalent_noise_temperature(interference, 0.0), None);
    assert_eq!(equivalent_noise_temperature(interference, -1.0), None);
    assert_eq!(equivalent_noise_temperature(interference, f64::NAN), None);
}

#[test]
fn a_summary_reports_the_carrier_and_the_interference() {
    let summary = summarise(
        carrier(),
        &[interferer_at(10.0)],
        Some(Power::from_dbw(-140.0).expect("finite")),
    )
    .expect("valid");
    assert_eq!(summary.carrier, carrier());
    assert!(summary.interference.watts() > 0.0);
    assert!((summary.carrier_to_interference.db() - 10.0).abs() < TOLERANCE);
    // The combined ratio is below both the carrier to noise and the carrier
    // to interference ratio.
    // 合并之比低于载噪比与载干比二者。
    let combined = summary
        .carrier_to_noise_plus_interference
        .expect("noise was given");
    assert!(combined.db() < summary.carrier_to_interference.db());
}

#[test]
fn a_summary_without_noise_omits_the_combined_ratio() {
    let summary = summarise(carrier(), &[interferer_at(10.0)], None).expect("valid");
    assert_eq!(summary.carrier_to_noise_plus_interference, None);
}

#[test]
fn the_summary_reports_the_interference_to_carrier_ratio() {
    let summary = summarise(carrier(), &[interferer_at(10.0)], None).expect("valid");
    let interference_to_carrier = summary.interference_to_carrier();
    assert!((interference_to_carrier.db() + 10.0).abs() < TOLERANCE);
}

#[test]
fn a_summary_rejects_invalid_inputs() {
    let invalid = Interferer::new(Power::from_watts(-1.0));
    assert_eq!(summarise(carrier(), &[invalid], None), None);
    assert_eq!(summarise(carrier(), &[], None), None);
    assert_eq!(
        summarise(
            carrier(),
            &[interferer_at(10.0)],
            Some(Power::from_watts(0.0))
        ),
        None
    );
}
