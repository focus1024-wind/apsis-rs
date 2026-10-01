//! Tests for the parabolic aperture model.
//!
//! 抛物面孔径模型的测试。

use apsis_antenna::{ParabolicAperture, beam_width};
use core::f64::consts::PI;

/// A three meter dish with a typical aperture efficiency.
///
/// 一面三米天线，具有典型孔径效率。
const DISH: ParabolicAperture = ParabolicAperture {
    diameter: 3.0,
    efficiency: 0.65,
};

#[test]
fn aperture_area_follows_from_the_diameter() {
    let expected = PI * 1.5 * 1.5;
    assert!((DISH.area() - expected).abs() < 1e-12);
    assert!((DISH.effective_area() - expected * 0.65).abs() < 1e-12);
}

#[test]
fn peak_gain_matches_the_closed_form() {
    // At 12 GHz the wavelength is 0.02498 m, and the gain is
    // eta (pi D / lambda)^2.
    // 12 GHz 下波长为 0.02498 m，增益为 eta (pi D / lambda)^2。
    let wavelength = 0.024_982_710_7;
    let gain = DISH.peak_gain(wavelength).expect("valid");
    let expected = 0.65 * (PI * 3.0 / wavelength).powi(2);
    assert!((gain.ratio() - expected).abs() < 1e-6);
    // The result is about 49.66 dBi.
    // 结果约为 49.66 dBi。
    assert!(
        (gain.dbi().expect("positive") - 49.661_764_937_398_814).abs() < 1e-9,
        "dBi {}",
        gain.dbi().expect("positive")
    );
}

#[test]
fn a_larger_dish_has_more_gain() {
    let small = ParabolicAperture {
        diameter: 1.0,
        efficiency: 0.65,
    };
    let large = ParabolicAperture {
        diameter: 2.0,
        efficiency: 0.65,
    };
    let wavelength = 0.025;
    let small_gain = small.peak_gain(wavelength).expect("valid");
    let large_gain = large.peak_gain(wavelength).expect("valid");
    // Doubling the diameter quadruples the gain, which is six decibels.
    // 直径加倍使增益变为四倍，即六分贝。
    assert!(
        (large_gain.dbi().expect("positive") - small_gain.dbi().expect("positive") - 6.020_6).abs()
            < 1e-6
    );
}

#[test]
fn a_higher_aperture_efficiency_adds_gain() {
    let low = ParabolicAperture {
        diameter: 3.0,
        efficiency: 0.5,
    };
    let high = ParabolicAperture {
        diameter: 3.0,
        efficiency: 0.7,
    };
    let wavelength = 0.025;
    assert!(
        high.peak_gain(wavelength).expect("valid").ratio()
            > low.peak_gain(wavelength).expect("valid").ratio()
    );
}

#[test]
fn the_beam_width_follows_the_wavelength_over_diameter() {
    let wavelength = 0.025;
    let uniform = DISH.uniform_beam_width(wavelength).expect("valid");
    let tapered = DISH.half_power_beam_width(wavelength).expect("valid");
    assert!((uniform - 1.02 * wavelength / 3.0).abs() < 1e-12);
    // A tapered illumination widens the beam.
    // 锥削照射使波束变宽。
    assert!(tapered > uniform);
    assert!((tapered - 1.15 * wavelength / 3.0).abs() < 1e-12);
}

#[test]
fn the_uniform_beam_width_matches_a_known_value() {
    // A 3 m dish at 12 GHz has a half power beam width near 0.0085 radians,
    // which is about 0.49 degrees.
    // 12 GHz 下的 3 m 天线半功率波束宽度约为 0.0085 弧度，即约 0.49 度。
    let width = DISH.uniform_beam_width(0.024_982_710_7).expect("valid");
    assert!((width - 0.008_494).abs() < 1e-5, "width {width}");
    assert!((width.to_degrees() - 0.486_7).abs() < 0.001);
}

#[test]
fn beam_width_helper_validates_its_arguments() {
    assert!(beam_width(3.0, 0.025, 1.02).is_some());
    assert_eq!(beam_width(0.0, 0.025, 1.02), None);
    assert_eq!(beam_width(-1.0, 0.025, 1.02), None);
    assert_eq!(beam_width(3.0, 0.0, 1.02), None);
    assert_eq!(beam_width(3.0, -0.025, 1.02), None);
    assert_eq!(beam_width(3.0, 0.025, 0.0), None);
    assert_eq!(beam_width(f64::NAN, 0.025, 1.02), None);
    assert_eq!(beam_width(3.0, f64::INFINITY, 1.02), None);
}

#[test]
fn an_invalid_aperture_produces_no_result() {
    let zero_diameter = ParabolicAperture {
        diameter: 0.0,
        efficiency: 0.65,
    };
    assert!(!zero_diameter.is_valid());
    assert_eq!(zero_diameter.peak_gain(0.025), None);
    assert_eq!(zero_diameter.half_power_beam_width(0.025), None);

    let bad_efficiency = ParabolicAperture {
        diameter: 3.0,
        efficiency: 1.5,
    };
    assert!(!bad_efficiency.is_valid());
    assert_eq!(bad_efficiency.peak_gain(0.025), None);

    let zero_efficiency = ParabolicAperture {
        diameter: 3.0,
        efficiency: 0.0,
    };
    assert!(!zero_efficiency.is_valid());

    let non_finite = ParabolicAperture {
        diameter: f64::NAN,
        efficiency: 0.65,
    };
    assert!(!non_finite.is_valid());
}

#[test]
fn the_aperture_and_gain_conversions_are_consistent() {
    // The peak gain and the effective area must describe the same antenna.
    // 峰值增益与有效面积必须描述同一面天线。
    let wavelength = 0.025;
    let gain = DISH.peak_gain(wavelength).expect("valid");
    let aperture = gain.effective_aperture(wavelength).expect("valid");
    assert!((aperture - DISH.effective_area()).abs() < 1e-9);
}
