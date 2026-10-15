//! Tests for power levels.
//!
//! 功率电平的测试。

use apsis::apsis_rf::{Decibel, Power};

const TOLERANCE: f64 = 1e-12;

#[test]
fn reference_powers_match_their_definitions() {
    assert_eq!(Power::WATT.watts(), 1.0);
    assert_eq!(Power::MILLIWATT.watts(), 1.0e-3);
    assert_eq!(Power::WATT.dbw(), Some(0.0));
    assert_eq!(Power::MILLIWATT.dbm(), Some(0.0));
    // One watt is thirty decibel milliwatts.
    // 一瓦是三十毫瓦分贝。
    assert!((Power::WATT.dbm().expect("positive") - 30.0).abs() < TOLERANCE);
}

#[test]
fn decibel_forms_round_trip() {
    for dbw in [-150.0, -100.0, -20.0, 0.0, 10.0, 53.0] {
        let power = Power::from_dbw(dbw).expect("finite reading");
        assert!(
            (power.dbw().expect("positive") - dbw).abs() < 1e-9,
            "dbw {dbw}"
        );
    }
    for dbm in [-120.0, -60.0, 0.0, 30.0, 60.0] {
        let power = Power::from_dbm(dbm).expect("finite reading");
        assert!(
            (power.dbm().expect("positive") - dbm).abs() < 1e-9,
            "dbm {dbm}"
        );
    }
}

#[test]
fn dbw_and_dbm_differ_by_thirty() {
    let power = Power::from_watts(2.0);
    assert!((power.dbm().expect("positive") - power.dbw().expect("positive") - 30.0).abs() < 1e-9);
}

#[test]
fn known_power_levels_convert_correctly() {
    // Ten watts is ten decibel watts and forty decibel milliwatts.
    // 十瓦是十分贝瓦与四十毫瓦分贝。
    let ten_watts = Power::from_watts(10.0);
    assert!((ten_watts.dbw().expect("positive") - 10.0).abs() < 1e-12);
    assert!((ten_watts.dbm().expect("positive") - 40.0).abs() < 1e-12);

    // One milliwatt is one milliwatt.
    // 一毫瓦即一毫瓦。
    assert!((Power::from_milliwatts(1.0).watts() - 1.0e-3).abs() < 1e-18);
    assert!((Power::from_milliwatts(1.0).milliwatts() - 1.0).abs() < 1e-12);
}

#[test]
fn zero_and_negative_power_have_no_decibel_form() {
    assert_eq!(Power::from_watts(0.0).dbw(), None);
    assert_eq!(Power::from_watts(-1.0).dbw(), None);
    assert_eq!(Power::from_watts(0.0).dbm(), None);
}

#[test]
fn non_finite_readings_are_rejected() {
    assert_eq!(Power::from_dbw(f64::NAN), None);
    assert_eq!(Power::from_dbw(f64::INFINITY), None);
    assert_eq!(Power::from_dbm(f64::NAN), None);
    assert_eq!(Power::from_dbm(f64::NEG_INFINITY), None);
}

#[test]
fn a_ratio_between_powers_is_a_decibel() {
    let high = Power::from_watts(100.0);
    let low = Power::from_watts(1.0);
    let ratio = high.ratio_in_decibels(low).expect("both are positive");
    assert!((ratio.db() - 20.0).abs() < 1e-12);

    // The reverse ratio is the negative.
    // 反向之比是负数。
    let inverse = low.ratio_in_decibels(high).expect("both are positive");
    assert!((inverse.db() + 20.0).abs() < 1e-12);
}

#[test]
fn a_ratio_is_not_defined_against_zero_power() {
    let power = Power::from_watts(1.0);
    assert_eq!(power.ratio_in_decibels(Power::from_watts(0.0)), None);
    assert_eq!(Power::from_watts(0.0).ratio_in_decibels(power), None);
}

#[test]
fn scaling_by_a_gain_multiplies_the_power() {
    let power = Power::from_dbw(0.0).expect("finite reading");
    let amplified = power
        .scaled(Decibel::from_db(10.0))
        .expect("result is finite");
    assert!((amplified.dbw().expect("positive") - 10.0).abs() < 1e-9);
    assert!((amplified.watts() - 10.0).abs() < 1e-9);
}

#[test]
fn scaling_zero_stays_zero() {
    let zero = Power::from_watts(0.0);
    let scaled = zero.scaled(Decibel::from_db(20.0)).expect("zero is finite");
    assert_eq!(scaled.watts(), 0.0);
}

#[test]
fn validity_is_reported() {
    assert!(Power::from_watts(1.0).is_valid());
    assert!(Power::from_watts(0.0).is_valid());
    assert!(!Power::from_watts(-1.0).is_valid());
    assert!(!Power::from_watts(f64::NAN).is_valid());
}

#[test]
fn display_prefers_decibel_watts() {
    assert_eq!(Power::from_watts(1.0).to_string(), "0 dBW");
    // A zero power has no decibel form, so the linear value is shown.
    // 零功率没有分贝形式，故显示线性值。
    assert_eq!(Power::from_watts(0.0).to_string(), "0 W");
}
