//! Tests for off-axis gain patterns.

use apsis_antenna::{
    ElementPattern, ParabolicAperture, Pattern, floor_angle, mask_gain, off_axis_attenuation,
};
use apsis_rf::Gain;
use core::f64::consts::PI;

const TOLERANCE: f64 = 1e-9;

#[test]
fn boresight_gain_is_the_peak() {
    let peak = Gain::from_dbi(30.0).expect("valid");
    let beam = 0.01;
    let attenuation = off_axis_attenuation(peak, beam, 0.0, 32.0).expect("valid arguments");
    assert!((attenuation.db() - 30.0).abs() < TOLERANCE);
}

#[test]
fn the_half_power_angle_gives_three_decibels() {
    // At half the beam width the off-axis angle is half of the half power
    // beam width, and the squared angle law gives 12 * (0.5)^2 = 3 dB.
    let peak = Gain::from_dbi(30.0).expect("valid");
    let beam = 0.01;
    let attenuation = off_axis_attenuation(peak, beam, beam / 2.0, 32.0).expect("valid arguments");
    assert!(
        (attenuation.db() - 27.0).abs() < 1e-9,
        "{}",
        attenuation.db()
    );
}

#[test]
fn the_attenuation_grows_with_the_square_of_the_angle() {
    let peak = Gain::from_dbi(40.0).expect("valid");
    let beam = 0.02;
    // Both angles are chosen to stay on the squared angle law, well inside
    // the angle at which the pattern reaches its floor.
    let first = off_axis_attenuation(peak, beam, 0.02, 60.0).expect("valid");
    let second = off_axis_attenuation(peak, beam, 0.04, 60.0).expect("valid");
    // At the beam width the attenuation is 12 dB; at twice the beam width
    // it is 48 dB. The rise of 36 dB is what a quadrupling of the
    // attenuation ratio means in decibels.
    assert!((first.db() - (40.0 - 12.0)).abs() < 1e-9);
    assert!((second.db() - (40.0 - 48.0)).abs() < 1e-9);
    let rise = first.db() - second.db();
    assert!((rise - 36.0).abs() < 1e-9, "rise {rise}");
}

#[test]
fn the_pattern_floors_at_the_sidelobe_level() {
    let peak = Gain::from_dbi(40.0).expect("valid");
    let beam = 0.02;
    let floor = 32.0;
    // Far off axis the gain is the peak minus the floor.
    let far = off_axis_attenuation(peak, beam, PI, floor).expect("valid");
    assert!((far.db() - (40.0 - floor)).abs() < TOLERANCE);

    // The floor is reached at the angle the helper reports.
    let angle = floor_angle(beam, floor).expect("valid");
    let at_floor = off_axis_attenuation(peak, beam, angle, floor).expect("valid");
    assert!((at_floor.db() - (40.0 - floor)).abs() < 1e-9);
    // Just inside that angle the gain is above the floor.
    let inside = off_axis_attenuation(peak, beam, angle * 0.99, floor).expect("valid");
    assert!(inside.db() > at_floor.db());
}

#[test]
fn invalid_arguments_are_rejected() {
    let peak = Gain::from_dbi(30.0).expect("valid");
    assert_eq!(off_axis_attenuation(peak, 0.0, 0.01, 32.0), None);
    assert_eq!(off_axis_attenuation(peak, -0.01, 0.01, 32.0), None);
    assert_eq!(off_axis_attenuation(peak, 0.01, 0.01, 0.0), None);
    assert_eq!(off_axis_attenuation(peak, 0.01, f64::NAN, 32.0), None);
    assert_eq!(floor_angle(0.0, 32.0), None);
    assert_eq!(floor_angle(0.01, 0.0), None);
}

#[test]
fn the_itu_mask_uses_the_large_aperture_branch() {
    // For D/lambda >= 100 the pattern is 29 - 25 log10(theta) between one
    // and 36 degrees and -10 beyond.
    let at_ten_degrees = mask_gain(1_000.0, 10.0_f64.to_radians()).expect("valid");
    assert!((at_ten_degrees - (29.0 - 25.0 * 10.0_f64.log10())).abs() < 1e-9);
    assert!((at_ten_degrees - 4.0).abs() < 1e-9);

    // The falling branch ends at 36 degrees, where the expression is close
    // to but not exactly -10 dB. The standard accepts that small step and
    // then holds the value at -10 dB.
    let at_break = mask_gain(1_000.0, 36.0_f64.to_radians()).expect("valid");
    assert!(
        (at_break - (29.0 - 25.0 * 36.0_f64.log10())).abs() < 1e-9,
        "at break {at_break}"
    );
    assert!((at_break + 10.0).abs() < 0.1, "at break {at_break}");

    let beyond = mask_gain(1_000.0, 90.0_f64.to_radians()).expect("valid");
    assert!((beyond + 10.0).abs() < TOLERANCE);
}

#[test]
fn the_itu_mask_uses_the_small_aperture_branch() {
    // For D/lambda < 100 the pattern starts at 32 and breaks at 48 degrees.
    let at_ten_degrees = mask_gain(50.0, 10.0_f64.to_radians()).expect("valid");
    assert!((at_ten_degrees - (32.0 - 25.0 * 10.0_f64.log10())).abs() < 1e-9);

    // The falling branch ends at 48 degrees, again close to -10 dB.
    let at_break = mask_gain(50.0, 48.0_f64.to_radians()).expect("valid");
    assert!(
        (at_break - (32.0 - 25.0 * 48.0_f64.log10())).abs() < 1e-9,
        "at break {at_break}"
    );
    assert!((at_break + 10.0).abs() < 0.1, "at break {at_break}");

    let beyond = mask_gain(50.0, 120.0_f64.to_radians()).expect("valid");
    assert!((beyond + 10.0).abs() < TOLERANCE);
}

#[test]
fn the_itu_mask_is_capped_inside_one_degree() {
    // Below one degree the pattern does not continue to rise without bound.
    let peak_limit = 32.0 - 25.0 * 1.0_f64.log10();
    for angle_degrees in [0.0_f64, 0.01, 0.5, 0.99] {
        let value = mask_gain(50.0, angle_degrees.to_radians()).expect("valid");
        assert!(
            (value - peak_limit).abs() < 1e-9,
            "angle {angle_degrees} gave {value}"
        );
    }
}

#[test]
fn the_itu_mask_rejects_invalid_arguments() {
    assert_eq!(mask_gain(0.0, 0.1), None);
    assert_eq!(mask_gain(-1.0, 0.1), None);
    assert_eq!(mask_gain(1_000.0, -0.1), None);
    assert_eq!(mask_gain(f64::NAN, 0.1), None);
    assert_eq!(mask_gain(1_000.0, f64::NAN), None);
}

#[test]
fn an_element_pattern_reproduces_the_model() {
    let pattern = ElementPattern::new(Gain::from_dbi(35.0).expect("valid"), 0.02);
    assert!(pattern.is_valid());
    let gain = pattern.gain_at(0.01).expect("valid");
    // 0.01 is half the beam width, so the attenuation is 3 dB.
    assert!((gain.dbi().expect("positive") - 32.0).abs() < 1e-9);
}

#[test]
fn an_element_pattern_floors_far_off_axis() {
    let pattern = ElementPattern::new(Gain::from_dbi(35.0).expect("valid"), 0.02).with_floor(30.0);
    let gain = pattern.gain_at(PI).expect("valid");
    assert!((gain.dbi().expect("positive") - 5.0).abs() < 1e-9);
}

#[test]
fn a_pattern_can_be_built_from_a_dish() {
    let dish = ParabolicAperture {
        diameter: 3.0,
        efficiency: 0.65,
    };
    let wavelength = 0.025;
    let pattern = ElementPattern::from_parabolic(dish, wavelength).expect("valid");
    assert!(pattern.is_valid());
    assert!(
        (pattern.peak.dbi().expect("positive")
            - dish
                .peak_gain(wavelength)
                .expect("valid")
                .dbi()
                .expect("positive"))
        .abs()
            < 1e-9
    );
    assert!(
        (pattern.beam_width - dish.half_power_beam_width(wavelength).expect("valid")).abs() < 1e-12
    );
}

#[test]
fn an_invalid_dish_produces_no_pattern() {
    let dish = ParabolicAperture {
        diameter: 0.0,
        efficiency: 0.65,
    };
    assert_eq!(ElementPattern::from_parabolic(dish, 0.025), None);
}

#[test]
fn the_pattern_enum_derives_the_same_values() {
    let dish = ParabolicAperture {
        diameter: 3.0,
        efficiency: 0.65,
    };
    let wavelength = 0.025;
    let from_geometry = Pattern::Parabolic {
        diameter: 3.0,
        efficiency: 0.65,
    };
    assert!(
        (from_geometry.peak_gain(wavelength).expect("valid").ratio()
            - dish.peak_gain(wavelength).expect("valid").ratio())
        .abs()
            < 1e-9
    );
    assert!(
        (from_geometry.beam_width(wavelength).expect("valid")
            - dish.half_power_beam_width(wavelength).expect("valid"))
        .abs()
            < 1e-12
    );

    let direct = Pattern::Direct {
        peak: Gain::from_dbi(30.0).expect("valid"),
        beam_width: 0.05,
    };
    // The wavelength is ignored for a directly specified pattern.
    assert!(
        (direct
            .peak_gain(0.025)
            .expect("valid")
            .dbi()
            .expect("positive")
            - 30.0)
            .abs()
            < 1e-9
    );
    assert!((direct.beam_width(0.025).expect("valid") - 0.05).abs() < 1e-12);
}

#[test]
fn an_invalid_direct_pattern_produces_no_beam_width() {
    let direct = Pattern::Direct {
        peak: Gain::from_dbi(30.0).expect("valid"),
        beam_width: -1.0,
    };
    assert_eq!(direct.beam_width(0.025), None);
}

#[test]
fn the_default_pattern_is_effectively_isotropic() {
    let pattern = ElementPattern::default();
    assert!(pattern.is_valid());
    // With a floor at the smallest positive number the attenuation is
    // negligible in every direction.
    let gain = pattern.gain_at(1.0).expect("valid");
    assert!(gain.dbi().expect("positive").abs() < 1e-6);
}
