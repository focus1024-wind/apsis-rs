//! Tests for the ITU-R S.465 reference antenna pattern.
//!
//! ITU-R S.465 参考天线方向图的测试。

use apsis::apsis_antenna::{
    PEAK_GAIN_OFFSET_DB, diameter_over_wavelength_from_peak_gain, main_lobe_break_angle,
    reference_beam_width, reference_pattern_gain, reference_peak_gain,
};

const TOLERANCE: f64 = 1e-9;

#[test]
fn the_peak_gain_follows_the_recommendation() {
    // The recommendation gives 20 log10(D / lambda) + 7.7.
    // 该建议书给出 20 log10(D / lambda) + 7.7。
    for ratio in [10.0_f64, 50.0, 100.0, 200.0, 1_000.0] {
        let expected = 20.0 * ratio.log10() + PEAK_GAIN_OFFSET_DB;
        let actual = reference_peak_gain(ratio).expect("valid ratio");
        assert!((actual - expected).abs() < TOLERANCE, "ratio {ratio}");
    }
    // A ratio of one hundred gives 47.7 dBi.
    // 比值一百给出 47.7 dBi。
    assert!((reference_peak_gain(100.0).expect("valid") - 47.7).abs() < TOLERANCE);
}

#[test]
fn the_peak_gain_is_inverted_by_the_diameter_helper() {
    for ratio in [10.0, 100.0, 500.0] {
        let gain = reference_peak_gain(ratio).expect("valid");
        let recovered = diameter_over_wavelength_from_peak_gain(gain).expect("finite gain");
        assert!((recovered / ratio - 1.0).abs() < 1e-12, "ratio {ratio}");
    }
}

#[test]
fn the_pattern_at_boresight_is_the_peak_gain() {
    for ratio in [50.0, 100.0, 200.0] {
        let peak = reference_peak_gain(ratio).expect("valid");
        let boresight = reference_pattern_gain(ratio, 0.0).expect("valid");
        assert!((boresight - peak).abs() < TOLERANCE, "ratio {ratio}");
    }
}

#[test]
fn the_main_lobe_falls_as_the_square_of_the_angle() {
    // Inside the main lobe the attenuation is 2.5e-3 (D/lambda phi)^2.
    // 主瓣内衰减为 2.5e-3 (D/lambda phi)^2。
    let ratio = 100.0;
    let peak = reference_peak_gain(ratio).expect("valid");
    for angle_degrees in [0.05_f64, 0.1, 0.2, 0.3] {
        let gain = reference_pattern_gain(ratio, angle_degrees.to_radians()).expect("valid");
        let expected = peak - 2.5e-3 * (ratio * angle_degrees).powi(2);
        assert!(
            (gain - expected).abs() < TOLERANCE,
            "{angle_degrees} degrees: got {gain}, expected {expected}"
        );
    }
}

#[test]
fn the_pattern_matches_recorded_values() {
    // These are the values of the recommendation for a ratio of one hundred,
    // computed independently from its equations.
    // 这些是该建议书对比值一百的值，由其方程独立计算。
    let cases: [(f64, f64); 8] = [
        (0.1, 47.45),
        (0.5, 41.45),
        (1.0, 29.0),
        (5.0, 11.525_749_892),
        (20.0, -3.525_749_892),
        (36.0, -9.907_562_519),
        (48.0, -10.0),
        (90.0, -10.0),
    ];
    for (angle_degrees, expected) in cases {
        let gain = reference_pattern_gain(100.0, angle_degrees.to_radians()).expect("valid");
        assert!(
            (gain - expected).abs() < 1e-6,
            "{angle_degrees} degrees: got {gain}, expected {expected}"
        );
    }
}

#[test]
fn the_pattern_matches_recorded_values_for_a_small_aperture() {
    // A ratio of fifty uses the small aperture branch, with a head of 32 and
    // a floor angle of 48 degrees.
    // At exactly the floor angle the branch switches to the floor, so the
    // last value of the falling branch is approached from below.
    // 比值五十使用小孔径分支，head 为 32，底限角为 48 度。
    // 恰在底限角处该分支切换到取底限，故下降支的最后值是从下方趋近的。
    let cases: [(f64, f64); 6] = [
        (0.1, 41.616_900_087),
        (1.0, 35.429_400_087),
        (5.0, 14.525_749_892),
        (36.0, -6.907_562_519),
        (47.99, -10.028_768_748_310_533),
        (48.0, -10.0),
    ];
    for (angle_degrees, expected) in cases {
        let gain = reference_pattern_gain(50.0, angle_degrees.to_radians()).expect("valid");
        assert!(
            (gain - expected).abs() < 1e-6,
            "{angle_degrees} degrees: got {gain}, expected {expected}"
        );
    }
}

#[test]
fn the_two_branches_meet_at_the_break_angle() {
    // The main lobe and the sidelobe region must agree at the junction, or
    // the pattern would jump.
    // 主瓣与旁瓣区必须在交界处一致，否则方向图会跳变。
    for ratio in [50.0_f64, 100.0, 200.0, 500.0] {
        let peak = reference_peak_gain(ratio).expect("valid");
        let break_angle = main_lobe_break_angle(ratio).expect("valid");
        let (head, _) = if ratio >= 100.0 {
            (29.0, 36.0)
        } else {
            (32.0, 48.0)
        };

        let main_lobe = peak - 2.5e-3 * (ratio * break_angle).powi(2);
        let sidelobe = head - 25.0 * break_angle.log10();
        assert!(
            (main_lobe - sidelobe).abs() < 1e-6,
            "ratio {ratio}: {main_lobe} against {sidelobe}"
        );
    }
}

#[test]
fn the_break_angle_shrinks_as_the_aperture_grows() {
    // A larger antenna has a narrower main lobe, so the sidelobe region
    // begins sooner.
    // 更大的天线主瓣更窄，故旁瓣区开始得更早。
    let small = main_lobe_break_angle(50.0).expect("valid");
    let medium = main_lobe_break_angle(100.0).expect("valid");
    let large = main_lobe_break_angle(400.0).expect("valid");
    assert!(small > medium, "{small} against {medium}");
    assert!(medium > large, "{medium} against {large}");
}

#[test]
fn the_pattern_floors_at_minus_ten_decibels() {
    for ratio in [50.0, 100.0, 200.0] {
        let gain = reference_pattern_gain(ratio, 90.0_f64.to_radians()).expect("valid");
        assert!((gain + 10.0).abs() < TOLERANCE, "ratio {ratio}");
        let beyond = reference_pattern_gain(ratio, core::f64::consts::PI).expect("valid");
        assert!((beyond + 10.0).abs() < TOLERANCE, "ratio {ratio}");
    }
}

#[test]
fn the_pattern_never_rises_above_the_peak() {
    let ratio = 100.0;
    let peak = reference_peak_gain(ratio).expect("valid");
    for tenth in 0..=100 {
        let angle = f64::from(tenth) * 0.01;
        let gain = reference_pattern_gain(ratio, angle).expect("valid");
        assert!(gain <= peak + TOLERANCE, "angle {angle} gave {gain}");
    }
}

#[test]
fn the_pattern_falls_monotonically_across_the_main_lobe() {
    let ratio = 100.0;
    let mut previous = f64::INFINITY;
    for step in 0..=80 {
        let angle_degrees = f64::from(step) * 0.01;
        let gain = reference_pattern_gain(ratio, angle_degrees.to_radians()).expect("valid");
        assert!(gain <= previous + TOLERANCE, "angle {angle_degrees}");
        previous = gain;
    }
}

#[test]
fn the_beam_width_follows_from_the_main_lobe_law() {
    // Three decibels is reached at sqrt(3 / 2.5e-3) / (D/lambda) degrees
    // from boresight, so the full width is twice that.
    // 三分贝在距视轴 sqrt(3 / 2.5e-3) / (D/lambda) 度处达到，故全宽是它的两倍。
    let ratio = 100.0;
    let width = reference_beam_width(ratio).expect("valid");
    let half_degrees = (3.0_f64 / 2.5e-3).sqrt() / ratio;
    let expected = 2.0 * half_degrees.to_radians();
    assert!((width - expected).abs() < 1e-15);

    // The gain at half the beam width from boresight is three decibels down.
    // 距视轴半个波束宽度处的增益低三分贝。
    let gain = reference_pattern_gain(ratio, width / 2.0).expect("valid");
    let peak = reference_peak_gain(ratio).expect("valid");
    assert!(
        (peak - gain - 3.0).abs() < 1e-9,
        "peak - gain = {}",
        peak - gain
    );
}

#[test]
fn invalid_arguments_are_rejected() {
    assert_eq!(reference_peak_gain(0.0), None);
    assert_eq!(reference_peak_gain(-1.0), None);
    assert_eq!(reference_peak_gain(f64::NAN), None);
    assert_eq!(reference_pattern_gain(100.0, -0.1), None);
    assert_eq!(reference_pattern_gain(100.0, f64::NAN), None);
    assert_eq!(reference_pattern_gain(0.0, 0.1), None);
    assert_eq!(reference_beam_width(0.0), None);
    assert_eq!(diameter_over_wavelength_from_peak_gain(f64::NAN), None);
    assert_eq!(main_lobe_break_angle(0.0), None);
}
