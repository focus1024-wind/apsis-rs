//! Tests for the anomaly conversions.
//!
//! 近点角换算的测试。

use apsis::apsis_kepler::{
    eccentric_from_mean, eccentric_from_true, mean_from_eccentric, mean_from_true,
    true_from_eccentric, true_from_mean, wrap_to_pi,
};
use core::f64::consts::PI;

#[test]
fn wrap_places_angles_in_the_half_open_interval() {
    for turns in -3..=3 {
        for offset in [0.0, 0.25, 0.5, 0.75] {
            let angle = PI + (f64::from(turns) + offset) * 2.0 * PI;
            let wrapped = wrap_to_pi(angle);
            assert!(
                (-PI..PI).contains(&wrapped),
                "angle {angle} wrapped to {wrapped}"
            );
            // The result differs from the input by a whole number of turns.
            // 结果与输入相差整数圈。
            let turns_removed = (angle - wrapped) / (2.0 * PI);
            assert!((turns_removed - turns_removed.round()).abs() < 1e-9);
        }
    }
}

#[test]
fn wrap_keeps_values_already_in_range() {
    for angle in [-3.0, -1.0, 0.0, 1.0, 3.0] {
        assert!((wrap_to_pi(angle) - angle).abs() < 1e-12);
    }
}

#[test]
fn keplers_equation_holds_for_the_solution() {
    // M = E - e sin E must be satisfied by the returned eccentric anomaly.
    // 所返回的偏近点角必须满足 M = E - e sin E。
    for eccentricity in [0.0, 0.01, 0.1, 0.5, 0.9, 0.99] {
        for mean in [-3.0, -1.0, 0.0, 0.5, 1.5, 3.0] {
            let eccentric = eccentric_from_mean(mean, eccentricity);
            let recovered = mean_from_eccentric(eccentric, eccentricity);
            assert!(
                (recovered - mean).abs() < 1e-9,
                "e={eccentricity} M={mean} recovered={recovered}"
            );
        }
    }
}

#[test]
fn eccentric_and_true_anomaly_round_trip() {
    for eccentricity in [0.0, 0.05, 0.3, 0.7, 0.95] {
        for true_anomaly in [-3.0, -1.0, 0.0, 0.25, 1.0, 3.0] {
            let eccentric = eccentric_from_true(true_anomaly, eccentricity);
            let back = true_from_eccentric(eccentric, eccentricity);
            assert!(
                (back - true_anomaly).abs() < 1e-9,
                "e={eccentricity} nu={true_anomaly} back={back}"
            );
        }
    }
}

#[test]
fn mean_and_true_anomaly_round_trip() {
    for eccentricity in [0.0, 0.05, 0.3, 0.7, 0.95] {
        for true_anomaly in [-2.5, -0.5, 0.0, 0.75, 2.0] {
            let mean = mean_from_true(true_anomaly, eccentricity);
            let back = true_from_mean(mean, eccentricity);
            assert!(
                (back - true_anomaly).abs() < 1e-8,
                "e={eccentricity} nu={true_anomaly} back={back}"
            );
        }
    }
}

#[test]
fn a_circular_orbit_has_equal_anomalies() {
    // With zero eccentricity every anomaly is the same angle.
    // 偏心率为零时每种近点角都是同一个角。
    for angle in [-2.0, -0.5, 0.0, 1.0, 2.5] {
        assert!((eccentric_from_mean(angle, 0.0) - angle).abs() < 1e-12);
        assert!((true_from_eccentric(angle, 0.0) - angle).abs() < 1e-12);
        assert!((true_from_mean(angle, 0.0) - angle).abs() < 1e-12);
    }
}

#[test]
fn anomalies_at_perigee_are_zero() {
    for eccentricity in [0.0, 0.3, 0.8] {
        assert!(eccentric_from_mean(0.0, eccentricity).abs() < 1e-12);
        assert!(true_from_mean(0.0, eccentricity).abs() < 1e-12);
        assert!(mean_from_true(0.0, eccentricity).abs() < 1e-12);
    }
}

#[test]
fn true_anomaly_exceeds_mean_anomaly_before_apogee() {
    // For an eccentric orbit the satellite spends less time near perigee,
    // so the true anomaly runs ahead of the mean anomaly in the first half
    // of the orbit.
    // 对偏心轨道，卫星在近地点附近花的时间较少，故在轨道的上半段真近点角领先于平
    // 近点角。
    let eccentricity = 0.5;
    let mean = 1.0;
    let true_anomaly = true_from_mean(mean, eccentricity);
    assert!(true_anomaly > mean, "nu={true_anomaly} M={mean}");
}
