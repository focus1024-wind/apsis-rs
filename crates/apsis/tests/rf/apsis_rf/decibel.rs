//! Tests for the decibel scale.
//!
//! 分贝尺度的测试。

use apsis::apsis_rf::Decibel;

const TOLERANCE: f64 = 1e-12;

#[test]
fn unity_and_doubling_have_the_expected_readings() {
    assert_eq!(Decibel::UNITY.db(), 0.0);
    assert!((Decibel::UNITY.ratio() - 1.0).abs() < TOLERANCE);
    // Three decibels is a ratio of 1.9953, not exactly two, which is the
    // reason the doubling constant is an approximation in practice.
    // 三分贝是 1.9953 的比而非恰好二，这正是加倍常量在实践中是近似的原因。
    assert!((Decibel::DOUBLING.db() - 3.010_299_956_639_812).abs() < TOLERANCE);
    assert!((Decibel::DOUBLING.ratio() - 2.0).abs() < TOLERANCE);
}

#[test]
fn ratio_and_reading_round_trip() {
    for reading in [-30.0, -10.0, -3.0, 0.0, 1.5, 10.0, 30.0, 100.0] {
        let value = Decibel::from_db(reading);
        let ratio = value.ratio();
        let recovered = Decibel::from_ratio(ratio).expect("ratio is positive");
        assert!((recovered.db() - reading).abs() < 1e-9, "reading {reading}");
    }
}

#[test]
fn a_ratio_of_ten_is_ten_decibels() {
    assert!((Decibel::from_ratio(10.0).expect("positive").db() - 10.0).abs() < TOLERANCE);
    assert!((Decibel::from_ratio(100.0).expect("positive").db() - 20.0).abs() < TOLERANCE);
    assert!((Decibel::from_ratio(0.001).expect("positive").db() + 30.0).abs() < TOLERANCE);
}

#[test]
fn amplitude_ratios_use_twenty_per_decade() {
    // A voltage ratio of ten is twenty decibels, because power is
    // proportional to the square of the amplitude.
    // 电压比十即二十分贝，因为功率与幅度的平方成正比。
    let value = Decibel::from_amplitude_ratio(10.0).expect("positive");
    assert!((value.db() - 20.0).abs() < TOLERANCE);
    assert!((value.amplitude_ratio() - 10.0).abs() < 1e-9);
}

#[test]
fn non_positive_ratios_are_rejected() {
    assert_eq!(Decibel::from_ratio(0.0), None);
    assert_eq!(Decibel::from_ratio(-1.0), None);
    assert_eq!(Decibel::from_ratio(f64::NAN), None);
    assert_eq!(Decibel::from_ratio(f64::INFINITY), None);
    assert_eq!(Decibel::from_amplitude_ratio(0.0), None);
}

#[test]
fn addition_multiplies_the_ratio() {
    // The exact doubling constant added to itself quadruples the ratio.
    // 精确的加倍常量自加使其比变为四倍。
    let total = Decibel::DOUBLING + Decibel::DOUBLING;
    assert!((total.ratio() - 4.0).abs() < 1e-9);

    // Two ten decibel gains make a gain of one hundred.
    // 两个十分贝增益合成一百倍增益。
    let decades = Decibel::from_db(10.0) + Decibel::from_db(10.0);
    assert!((decades.db() - 20.0).abs() < TOLERANCE);
    assert!((decades.ratio() - 100.0).abs() < 1e-9);
}

#[test]
fn subtraction_divides_the_ratio() {
    let net = Decibel::from_db(10.0) - Decibel::from_db(3.0);
    assert!((net.db() - 7.0).abs() < TOLERANCE);
    let ratio = Decibel::from_db(10.0).ratio() / Decibel::from_db(3.0).ratio();
    assert!((net.ratio() - ratio).abs() < 1e-9);
}

#[test]
fn negation_inverts_the_ratio() {
    let value = Decibel::from_db(7.5);
    assert!((-value).db() + value.db() < TOLERANCE);
    assert!(((-value).ratio() * value.ratio() - 1.0).abs() < 1e-12);
    assert_eq!((-value).db(), value.negated().db());
}

#[test]
fn finiteness_is_reported() {
    assert!(Decibel::from_db(1.0).is_finite());
    assert!(!Decibel::from_db(f64::INFINITY).is_finite());
    assert!(!Decibel::from_db(f64::NAN).is_finite());
}

#[test]
fn display_includes_the_unit() {
    assert_eq!(Decibel::from_db(3.5).to_string(), "3.5 dB");
}

#[test]
fn default_is_unity() {
    assert_eq!(Decibel::default(), Decibel::UNITY);
}
