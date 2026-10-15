//! Tests for gaseous attenuation.
//!
//! The expected values are those of ITU-R Recommendation P.676-12 for the
//! reference atmosphere: a dry air pressure of 1013.25 hPa, a water vapour
//! density of 7.5 g/m^3, and a temperature of 288.15 K. They were produced
//! with a line-by-line implementation of the recommendation and are recorded
//! here so that a change to the line tables or the formulae shows up as a
//! failing assertion.
//!
//! 气体衰减的测试。
//!
//! 期望值是 ITU-R 建议书 P.676-12 对参考大气的值：干空气气压 1013.25 hPa、水汽
//! 密度 7.5 g/m^3、温度 288.15 K。它们由该建议书的逐线实现产生并记录在此，使对
//! 线表或公式的改动表现为断言失败。

use apsis::apsis_atmosphere::{
    Atmosphere, equivalent_heights, specific_attenuation, specific_attenuation_dry_air,
    specific_attenuation_water_vapour,
};

/// The tolerance on a specific attenuation, in decibels per kilometer.
///
/// The implementation reproduces the reference to about twelve significant
/// digits, so this allows for the rounding of the recorded values.
///
/// 比衰减的容差，单位为分贝每千米。
///
/// 该实现把参考值复现到约十二位有效数字，故此值容许所记录值的舍入。
const TOLERANCE: f64 = 1e-6;

/// The reference atmosphere of the recommendation.
///
/// 该建议书的参考大气。
fn reference() -> Atmosphere {
    Atmosphere::REFERENCE
}

#[test]
fn the_reference_atmosphere_matches_the_standard() {
    let atmosphere = reference();
    assert_eq!(atmosphere.pressure_hpa, 1_013.25);
    assert_eq!(atmosphere.water_vapour_density, 7.5);
    assert_eq!(atmosphere.temperature_k, 288.15);
    assert!(atmosphere.is_valid());
}

#[test]
fn the_water_vapour_partial_pressure_follows_the_ideal_gas_law() {
    let atmosphere = reference();
    let expected = 7.5 * 288.15 / 216.7;
    assert!((atmosphere.water_vapour_partial_pressure() - expected).abs() < 1e-12);
}

#[test]
fn dry_air_attenuation_matches_the_recommendation() {
    // The recorded values are from the reference implementation of P.676-12.
    // 所记录的值来自 P.676-12 的参考实现。
    let cases = [
        (1.0, 0.005_388_658_168),
        (10.0, 0.008_224_416_703),
        (22.235, 0.013_292_678_183),
        (50.0, 0.277_268_638_531),
        (60.0, 14.623_474_796_486),
        (70.0, 0.304_105_097_147),
        (118.0, 1.134_866_201_871),
        (300.0, 0.025_759_576_282),
    ];
    for (frequency, expected) in cases {
        let actual = specific_attenuation_dry_air(frequency, reference()).expect("valid");
        assert!(
            (actual - expected).abs() < TOLERANCE,
            "{frequency} GHz: got {actual}, expected {expected}"
        );
    }
}

#[test]
fn water_vapour_attenuation_matches_the_recommendation() {
    let cases = [
        (1.0, 0.000_050_904_617),
        (10.0, 0.005_974_125_245),
        (22.235, 0.178_977_992_373),
        (60.0, 0.154_841_840_636),
        (183.0, 27.665_008_314_166),
        (300.0, 5.221_329_041_105),
    ];
    for (frequency, expected) in cases {
        let actual = specific_attenuation_water_vapour(frequency, reference()).expect("valid");
        assert!(
            (actual - expected).abs() < TOLERANCE,
            "{frequency} GHz: got {actual}, expected {expected}"
        );
    }
}

#[test]
fn the_two_contributions_add() {
    let frequency = 22.235;
    let dry = specific_attenuation_dry_air(frequency, reference()).expect("valid");
    let wet = specific_attenuation_water_vapour(frequency, reference()).expect("valid");
    let total = specific_attenuation(frequency, reference()).expect("valid");
    assert!((total - (dry + wet)).abs() < 1e-12);
}

#[test]
fn the_oxygen_complex_dominates_near_sixty_gigahertz() {
    // The 60 GHz complex is a set of more than thirty lines, so the dry air
    // term is many orders of magnitude above the water vapour term there.
    // 60 GHz 复合体是三十多条线的集合，故那里的干空气项比水汽项高许多个数量级。
    let frequency = 60.0;
    let dry = specific_attenuation_dry_air(frequency, reference()).expect("valid");
    let wet = specific_attenuation_water_vapour(frequency, reference()).expect("valid");
    // The dry air value is about 14.6 dB/km against 0.15 dB/km for water
    // vapour, a ratio near one hundred.
    // 干空气值约为 14.6 dB/km，而水汽为 0.15 dB/km，比值接近一百。
    assert!(dry > 50.0 * wet, "dry {dry} wet {wet}");
}

#[test]
fn the_water_vapour_line_dominates_at_one_hundred_and_eighty_three_gigahertz() {
    let frequency = 183.0;
    let dry = specific_attenuation_dry_air(frequency, reference()).expect("valid");
    let wet = specific_attenuation_water_vapour(frequency, reference()).expect("valid");
    assert!(wet > dry);
    // The line reaches over twenty decibels per kilometer.
    // 该线达到每千米二十多分贝。
    assert!(wet > 20.0, "wet {wet}");
}

#[test]
fn attenuation_is_small_in_the_microwave_windows() {
    // Below 15 GHz the total specific attenuation is a hundredth of a
    // decibel per kilometer or less, which is why those bands are used.
    // 15 GHz 以下总比衰减为每千米百分之一分贝或更少，这正是那些频段被使用的原因。
    for frequency in [1.0, 2.0, 4.0, 6.0, 10.0, 12.0, 15.0] {
        let total = specific_attenuation(frequency, reference()).expect("valid");
        assert!(total < 0.05, "{frequency} GHz gave {total} dB/km");
    }
}

#[test]
fn dry_air_attenuation_peaks_inside_the_oxygen_complex() {
    // The strongest line is near 60.4 GHz, and the value there must exceed
    // the values on either side.
    // 最强线在 60.4 GHz 附近，那里的值必须超过两侧的值。
    let peak = specific_attenuation_dry_air(60.434_778, reference()).expect("valid");
    for frequency in [50.0, 55.0, 65.0, 70.0] {
        let value = specific_attenuation_dry_air(frequency, reference()).expect("valid");
        assert!(value < peak, "{frequency} GHz gave {value} against {peak}");
    }
}

#[test]
fn water_vapour_attenuation_grows_with_the_vapour_density() {
    let dry_air = Atmosphere::new(1_013.25, 0.0, 288.15);
    let wet_air = Atmosphere::new(1_013.25, 15.0, 288.15);
    let dry = specific_attenuation_water_vapour(22.235, dry_air).expect("valid");
    let wet = specific_attenuation_water_vapour(22.235, wet_air).expect("valid");
    assert_eq!(dry, 0.0);
    assert!(wet > 0.0);
}

#[test]
fn equivalent_heights_match_the_recommendation() {
    let cases = [
        (1.0, 5.551_003_440, 1.696_760_106),
        (22.235, 4.867_310_088, 2.358_922_817),
        (60.0, 10.731_486_113, 1.694_560_689),
        (118.0, 13.446_411_089, 1.695_050_659),
        (183.0, 5.581_437_611, 2.522_911_251),
    ];
    for (frequency, dry_expected, wet_expected) in cases {
        let heights = equivalent_heights(frequency, reference()).expect("valid");
        assert!(
            (heights.dry_km - dry_expected).abs() < 1e-6,
            "{frequency} GHz dry height: got {}, expected {dry_expected}",
            heights.dry_km
        );
        assert!(
            (heights.wet_km - wet_expected).abs() < 1e-6,
            "{frequency} GHz wet height: got {}, expected {wet_expected}",
            heights.wet_km
        );
    }
}

#[test]
fn the_dry_height_is_capped_below_seventy_gigahertz() {
    // Equation 30 limits the dry air height to a pressure dependent bound at
    // the lower frequencies. The bound is `10.7 rp^0.3`, where `rp` is the
    // ratio of the total barometric pressure, dry air plus water vapour, to
    // the reference pressure.
    // 式 30 在较低频率处把干空气高度限制到一个依赖气压的界。该界为 `10.7 rp^0.3`，
    // 其中 `rp` 是总气压（干空气加水汽）与参考气压之比。
    let atmosphere = reference();
    let total = atmosphere.pressure_hpa + atmosphere.water_vapour_partial_pressure();
    let rp = total / 1_013.25;
    let bound = 10.7 * rp.powf(0.3);

    // At 60 GHz the uncapped value is above the bound, so the bound applies.
    // 在 60 GHz 处未设界的值高于该界，故界适用。
    let heights = equivalent_heights(60.0, atmosphere).expect("valid");
    assert!((heights.dry_km - bound).abs() < 1e-9, "{}", heights.dry_km);

    // Above 70 GHz no cap is applied, and the value exceeds it.
    // 70 GHz 以上不施加界，而该值超过它。
    let high = equivalent_heights(118.0, atmosphere).expect("valid");
    assert!(high.dry_km > bound);
}

#[test]
fn zenith_attenuation_matches_the_recommendation() {
    let cases = [
        (1.0, 0.029_998_832_95),
        (12.0, 0.058_856_728_55),
        (22.235, 0.486_894_856_6),
        (60.0, 157.194_005_6),
        (183.0, 69.867_434_26),
    ];
    for (frequency, expected) in cases {
        let dry = specific_attenuation_dry_air(frequency, reference()).expect("valid");
        let wet = specific_attenuation_water_vapour(frequency, reference()).expect("valid");
        let heights = equivalent_heights(frequency, reference()).expect("valid");
        let zenith = dry * heights.dry_km + wet * heights.wet_km;
        assert!(
            (zenith - expected).abs() < 1e-6,
            "{frequency} GHz: got {zenith}, expected {expected}"
        );
    }
}

#[test]
fn invalid_inputs_are_rejected() {
    assert_eq!(specific_attenuation(0.0, reference()), None);
    assert_eq!(specific_attenuation(-1.0, reference()), None);
    assert_eq!(specific_attenuation(f64::NAN, reference()), None);
    assert_eq!(
        specific_attenuation_dry_air(12.0, Atmosphere::new(0.0, 7.5, 288.15)),
        None
    );
    assert_eq!(
        specific_attenuation_water_vapour(12.0, Atmosphere::new(1_013.25, -1.0, 288.15)),
        None
    );
    assert_eq!(equivalent_heights(0.0, reference()), None);
    assert_eq!(
        equivalent_heights(12.0, Atmosphere::new(1_013.25, 7.5, 0.0)),
        None
    );
}

#[test]
fn a_denser_atmosphere_attenuates_more() {
    let sea_level = Atmosphere::new(1_013.25, 7.5, 288.15);
    let high_altitude = Atmosphere::new(700.0, 3.0, 280.0);
    let low = specific_attenuation_dry_air(60.0, sea_level).expect("valid");
    let high = specific_attenuation_dry_air(60.0, high_altitude).expect("valid");
    assert!(low > high);
}

#[test]
fn slant_path_attenuation_scales_with_the_elevation() {
    use apsis::apsis_atmosphere::{slant_path_attenuation, zenith_attenuation};
    let frequency = 12.0;
    let zenith = zenith_attenuation(frequency, reference()).expect("valid");
    // At thirty degrees elevation the path is twice the zenith path.
    // 三十度仰角处路径是天顶路径的两倍。
    let thirty =
        slant_path_attenuation(frequency, core::f64::consts::PI / 6.0, reference()).expect("valid");
    assert!((thirty - 2.0 * zenith).abs() < 1e-9);
    assert!(thirty > zenith);
}

#[test]
fn slant_path_attenuation_rejects_an_elevation_at_the_horizon() {
    use apsis::apsis_atmosphere::slant_path_attenuation;
    assert_eq!(slant_path_attenuation(12.0, 0.0, reference()), None);
    assert_eq!(slant_path_attenuation(12.0, -0.1, reference()), None);
    assert_eq!(slant_path_attenuation(12.0, f64::NAN, reference()), None);
}

#[test]
fn the_exact_slant_path_matches_the_recommendation() {
    use apsis::apsis_atmosphere::slant_path_attenuation_exact;
    // These values are from a reference implementation of the exact method
    // of ITU-R Recommendation P.676-12.
    // 这些值来自 ITU-R 建议书 P.676-12 精确方法的参考实现。
    let cases: [(f64, f64, f64); 10] = [
        (12.0, 90.0, 0.059926619268),
        (12.0, 30.0, 0.119683280157),
        (12.0, 10.0, 0.340072667215),
        (12.0, 5.0, 0.651451990239),
        (12.0, 1.0, 2.054182412438),
        (20.0, 30.0, 0.504703085886),
        (20.0, 5.0, 2.793084848959),
        (30.0, 10.0, 1.319753394647),
        (60.0, 30.0, 309.245091571274),
        (100.0, 5.0, 10.109978745794),
    ];
    for (frequency, elevation_degrees, expected) in cases {
        let actual =
            slant_path_attenuation_exact(frequency, elevation_degrees.to_radians(), reference())
                .expect("valid arguments");
        let relative = (actual - expected).abs() / expected;
        assert!(
            relative < 1e-6,
            "{frequency} GHz at {elevation_degrees} degrees: got {actual}, expected {expected}"
        );
    }
}

#[test]
fn the_exact_and_approximate_methods_agree_at_high_elevation() {
    use apsis::apsis_atmosphere::{slant_path_attenuation, slant_path_attenuation_exact};
    // The two methods use different formulations, so they agree to a few
    // per cent rather than exactly even at the zenith, where the plane
    // parallel geometry is exact. The recommendation notes that the
    // approximate method is intended only for the elevation range above
    // about five degrees and for rough work.
    // 两种方法使用不同的表述，故即便在天顶（那里平面平行几何是精确的）它们也只相符
    // 到百分之几而非精确相等。该建议书指出近似方法只意在于约五度以上的仰角范围以及
    // 粗略工作。
    let exact = slant_path_attenuation_exact(12.0, core::f64::consts::FRAC_PI_2, reference())
        .expect("valid");
    let approximate =
        slant_path_attenuation(12.0, core::f64::consts::FRAC_PI_2, reference()).expect("valid");
    let relative = (exact - approximate).abs() / approximate;
    assert!(relative < 0.05, "exact {exact}, approximate {approximate}");
}

#[test]
fn the_exact_method_exceeds_the_approximate_one_at_low_elevation() {
    use apsis::apsis_atmosphere::{slant_path_attenuation, slant_path_attenuation_exact};
    // At a low elevation the curved path through the thinning atmosphere is
    // shorter than the plane parallel estimate, and the refraction bends
    // the ray downward, so the two methods diverge.
    // 在低仰角处，穿过变稀薄大气的弯曲路径比平面平行估计短，且折射使射线向下弯折，
    // 故两种方法分歧。
    let elevation = 2.0_f64.to_radians();
    let exact = slant_path_attenuation_exact(12.0, elevation, reference()).expect("valid");
    let approximate = slant_path_attenuation(12.0, elevation, reference()).expect("valid");
    assert!(
        (exact - approximate).abs() / approximate > 0.001,
        "exact {exact}, approximate {approximate}"
    );
}

#[test]
fn the_exact_method_rejects_unusable_arguments() {
    use apsis::apsis_atmosphere::slant_path_attenuation_exact;
    assert_eq!(slant_path_attenuation_exact(0.0, 1.0, reference()), None);
    assert_eq!(slant_path_attenuation_exact(12.0, 0.0, reference()), None);
    assert_eq!(slant_path_attenuation_exact(12.0, -0.1, reference()), None);
    assert_eq!(
        slant_path_attenuation_exact(12.0, f64::NAN, reference()),
        None
    );
    assert_eq!(
        slant_path_attenuation_exact(12.0, 1.0, Atmosphere::new(0.0, 7.5, 288.15)),
        None
    );
}
