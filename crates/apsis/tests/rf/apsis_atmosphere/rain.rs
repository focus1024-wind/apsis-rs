//! Tests for rain attenuation.
//!
//! The coefficient values are those of ITU-R Recommendation P.838-3,
//! produced with a reference implementation of the recommendation and
//! recorded here. The path results exercise the P.618-13 method.
//!
//! 降雨衰减的测试。
//!
//! 各系数值是 ITU-R 建议书 P.838-3 的值，由该建议书的参考实现产生并记录在此。
//! 路径结果检验 P.618-13 方法。

use apsis::apsis_atmosphere::{
    DEFAULT_RAIN_HEIGHT_KM, polarisation_tilt, rain_attenuation, rain_coefficients,
    specific_rain_attenuation,
};
use core::f64::consts::{FRAC_PI_2, PI};

/// Horizontal polarisation.
///
/// 水平极化。
const HORIZONTAL: f64 = 0.0;
/// Vertical polarisation.
///
/// 垂直极化。
const VERTICAL: f64 = FRAC_PI_2;
/// The zenith, where the two polarisations coincide.
///
/// 天顶，两种极化在那里重合。
const ZENITH: f64 = FRAC_PI_2;
/// A mid latitude elevation of thirty degrees.
///
/// 三十度的中纬度仰角。
const THIRTY_DEGREES: f64 = PI / 6.0;

/// The relative tolerance on the coefficient values.
///
/// 系数值的相对容差。
const TOLERANCE: f64 = 1e-9;

fn assert_relative(actual: f64, expected: f64, context: &str) {
    let relative = (actual - expected).abs() / expected.abs();
    assert!(
        relative < TOLERANCE,
        "{context}: got {actual}, expected {expected}, relative {relative}"
    );
}

#[test]
fn coefficients_match_the_recommendation_at_the_zenith() {
    // At the zenith the elevation term vanishes, so horizontal and vertical
    // polarisation give the same coefficients.
    // 天顶处仰角项消失，故水平与垂直极化给出相同的系数。
    let cases = [
        (12.0, 0.024_203_061_156_1, 1.151_599_196_3),
        (20.0, 0.093_876_937_766_6, 1.019_877_631_17),
        (30.0, 0.234_699_253_968, 0.931_114_875_787),
    ];
    for (frequency, k_expected, alpha_expected) in cases {
        for polarisation in [HORIZONTAL, VERTICAL] {
            let (k, alpha) =
                rain_coefficients(frequency * 1.0e9, polarisation, ZENITH).expect("valid");
            assert_relative(k, k_expected, &format!("{frequency} GHz k"));
            assert_relative(alpha, alpha_expected, &format!("{frequency} GHz alpha"));
        }
    }
}

#[test]
fn coefficients_match_the_recommendation_at_thirty_degrees() {
    let cases = [
        (12.0, 0.023_944_109_789_7, 1.174_670_745_2),
        (20.0, 0.092_201_236_241_3, 1.047_387_560_19),
        (30.0, 0.238_905_952_257, 0.944_198_048_818),
    ];
    for (frequency, k_expected, alpha_expected) in cases {
        let (k, alpha) =
            rain_coefficients(frequency * 1.0e9, HORIZONTAL, THIRTY_DEGREES).expect("valid");
        assert_relative(k, k_expected, &format!("{frequency} GHz k"));
        assert_relative(alpha, alpha_expected, &format!("{frequency} GHz alpha"));
    }
}

#[test]
fn horizontal_and_vertical_differ_off_the_zenith() {
    // Away from the zenith the polarisation matters. At 12 GHz and low
    // elevation the vertical coefficient is the larger of the two, and the
    // two converge as the elevation rises and the geometry becomes
    // symmetric.
    // 远离天顶时极化要紧。12 GHz 与低仰角下垂直系数是两者中较大者，随仰角升高、
    // 几何变得对称，二者趋于一致。
    let low_elevation = 10.0_f64.to_radians();
    let horizontal = rain_coefficients(12.0e9, HORIZONTAL, low_elevation).expect("valid");
    let vertical = rain_coefficients(12.0e9, VERTICAL, low_elevation).expect("valid");
    assert_relative(horizontal.0, 0.023_868_203_786_369_1, "horizontal k");
    assert_relative(vertical.0, 0.024_537_918_525_868_188, "vertical k");

    // At the zenith they coincide.
    // 天顶处二者重合。
    let zenith_horizontal = rain_coefficients(12.0e9, HORIZONTAL, ZENITH).expect("valid");
    let zenith_vertical = rain_coefficients(12.0e9, VERTICAL, ZENITH).expect("valid");
    assert_relative(zenith_horizontal.0, zenith_vertical.0, "zenith k");
}

#[test]
fn circular_polarisation_lies_between_the_linear_cases() {
    // A tilt of forty five degrees is circular polarisation, whose
    // coefficients are the mean of the two linear cases.
    // 四十五度倾角是圆极化，其系数是两种线极化情形的均值。
    let horizontal = rain_coefficients(12.0e9, HORIZONTAL, THIRTY_DEGREES).expect("valid");
    let vertical = rain_coefficients(12.0e9, VERTICAL, THIRTY_DEGREES).expect("valid");
    let circular =
        rain_coefficients(12.0e9, polarisation_tilt(45.0), THIRTY_DEGREES).expect("valid");
    let mean = f64::midpoint(horizontal.0, vertical.0);
    assert_relative(circular.0, mean, "circular k");
}

#[test]
fn the_coefficients_are_smooth_across_the_band() {
    // The curve fits are continuous, so a small change in frequency gives a
    // small change in the coefficients.
    // 各曲线拟合是连续的，故频率的小变化给出系数的小变化。
    let mut previous = rain_coefficients(10.0e9, HORIZONTAL, ZENITH).expect("valid");
    for tenth in 101..=200 {
        let frequency = f64::from(tenth) * 0.1e9;
        let current = rain_coefficients(frequency, HORIZONTAL, ZENITH).expect("valid");
        assert!(
            (current.0 - previous.0).abs() / previous.0 < 0.05,
            "jump at {frequency} GHz"
        );
        previous = current;
    }
}

#[test]
fn a_frequency_outside_the_recommendation_is_rejected() {
    assert_eq!(rain_coefficients(0.5e9, HORIZONTAL, ZENITH), None);
    assert_eq!(rain_coefficients(2_000.0e9, HORIZONTAL, ZENITH), None);
    assert_eq!(rain_coefficients(f64::NAN, HORIZONTAL, ZENITH), None);
    assert_eq!(rain_coefficients(12.0e9, f64::NAN, ZENITH), None);
    assert_eq!(rain_coefficients(12.0e9, HORIZONTAL, f64::NAN), None);
}

#[test]
fn specific_attenuation_follows_the_power_law() {
    let rain_rate = 20.0;
    let frequency = 12.0e9;
    let specific =
        specific_rain_attenuation(rain_rate, frequency, HORIZONTAL, THIRTY_DEGREES).expect("valid");
    let (k, alpha) = rain_coefficients(frequency, HORIZONTAL, THIRTY_DEGREES).expect("valid");
    assert_relative(specific, k * rain_rate.powf(alpha), "specific attenuation");
    // At 12 GHz, thirty degrees elevation, and 20 mm/h this is 0.808 dB/km.
    // 在 12 GHz、三十度仰角与 20 mm/h 下这是 0.808 dB/km。
    assert!(
        (specific - 0.808_128_772_7).abs() < 1e-6,
        "specific {specific}"
    );
}

#[test]
fn a_higher_rain_rate_gives_more_attenuation() {
    let light = specific_rain_attenuation(1.0, 12.0e9, HORIZONTAL, THIRTY_DEGREES).expect("valid");
    let heavy = specific_rain_attenuation(50.0, 12.0e9, HORIZONTAL, THIRTY_DEGREES).expect("valid");
    assert!(heavy > light);
    let (_, alpha) = rain_coefficients(12.0e9, HORIZONTAL, THIRTY_DEGREES).expect("valid");
    assert_relative(heavy / light, 50.0_f64.powf(alpha), "rate ratio");
}

#[test]
fn a_zero_rain_rate_gives_no_attenuation() {
    assert_eq!(
        specific_rain_attenuation(0.0, 12.0e9, HORIZONTAL, THIRTY_DEGREES),
        Some(0.0)
    );
}

#[test]
fn a_negative_rain_rate_is_rejected() {
    assert_eq!(
        specific_rain_attenuation(-1.0, 12.0e9, HORIZONTAL, THIRTY_DEGREES),
        None
    );
    assert_eq!(
        rain_attenuation(
            -1.0,
            12.0e9,
            HORIZONTAL,
            THIRTY_DEGREES,
            0.0,
            DEFAULT_RAIN_HEIGHT_KM,
            45.0
        ),
        None
    );
}

#[test]
fn total_attenuation_is_the_specific_value_times_the_effective_path() {
    let result = rain_attenuation(
        20.0,
        12.0e9,
        HORIZONTAL,
        THIRTY_DEGREES,
        0.0,
        DEFAULT_RAIN_HEIGHT_KM,
        45.0,
    )
    .expect("valid");
    assert_relative(
        result.attenuation_db,
        result.specific_db_per_km * result.path_length_km,
        "total attenuation",
    );
    assert!(result.is_valid());
}

#[test]
fn the_reduction_factors_are_physical() {
    // The horizontal reduction factor accounts for rain not filling the
    // whole horizontal extent of the path, so it is at most one. The
    // vertical adjustment is the ratio of two path estimates and can exceed
    // one when the path crosses the rain layer at a shallow angle, which is
    // the case Recommendation P.618 allows for.
    // 水平缩减因子计入降雨并未填满路径的整个水平范围，故它至多为一。垂直调整是两个
    // 路径估计之比，可在路径以浅角穿越雨层时超过一，这正是建议书 P.618 所容许的
    // 情形。
    let result = rain_attenuation(
        20.0,
        12.0e9,
        HORIZONTAL,
        THIRTY_DEGREES,
        0.0,
        DEFAULT_RAIN_HEIGHT_KM,
        45.0,
    )
    .expect("valid");
    assert!(
        result.horizontal_reduction > 0.0 && result.horizontal_reduction <= 1.0,
        "horizontal reduction {}",
        result.horizontal_reduction
    );
    assert!(result.vertical_adjustment > 0.0, "vertical adjustment");
}

#[test]
fn attenuation_grows_as_the_elevation_falls() {
    let high = rain_attenuation(
        20.0,
        12.0e9,
        HORIZONTAL,
        ZENITH,
        0.0,
        DEFAULT_RAIN_HEIGHT_KM,
        45.0,
    )
    .expect("valid");
    let low = rain_attenuation(
        20.0,
        12.0e9,
        HORIZONTAL,
        10.0_f64.to_radians(),
        0.0,
        DEFAULT_RAIN_HEIGHT_KM,
        45.0,
    )
    .expect("valid");
    assert!(low.attenuation_db > high.attenuation_db);
}

#[test]
fn a_heavy_rain_event_gives_a_significant_attenuation() {
    // A 20 mm/h rain cell at 12 GHz and thirty degrees elevation gives a few
    // decibels, which is the order of magnitude a Ku band link must budget.
    // 12 GHz、三十度仰角下 20 mm/h 的雨胞给出几分贝，这是 Ku 频段链路必须预算的量
    // 级。
    let result = rain_attenuation(
        20.0,
        12.0e9,
        HORIZONTAL,
        THIRTY_DEGREES,
        0.0,
        DEFAULT_RAIN_HEIGHT_KM,
        45.0,
    )
    .expect("valid");
    assert!(
        result.attenuation_db > 1.0 && result.attenuation_db < 6.0,
        "{} dB",
        result.attenuation_db
    );
}

#[test]
fn a_lower_frequency_suffers_less_rain_attenuation() {
    let c_band = rain_attenuation(
        20.0,
        6.0e9,
        HORIZONTAL,
        THIRTY_DEGREES,
        0.0,
        DEFAULT_RAIN_HEIGHT_KM,
        45.0,
    )
    .expect("valid");
    let ku_band = rain_attenuation(
        20.0,
        12.0e9,
        HORIZONTAL,
        THIRTY_DEGREES,
        0.0,
        DEFAULT_RAIN_HEIGHT_KM,
        45.0,
    )
    .expect("valid");
    assert!(ku_band.attenuation_db > c_band.attenuation_db);
}

#[test]
fn a_station_above_the_rain_layer_has_no_rain_path() {
    let result = rain_attenuation(
        20.0,
        12.0e9,
        HORIZONTAL,
        THIRTY_DEGREES,
        4.0,
        DEFAULT_RAIN_HEIGHT_KM,
        45.0,
    )
    .expect("valid");
    assert_eq!(result.attenuation_db, 0.0);
    assert_eq!(result.path_length_km, 0.0);
}

#[test]
fn an_elevation_at_the_horizon_is_rejected() {
    assert_eq!(
        rain_attenuation(
            20.0,
            12.0e9,
            HORIZONTAL,
            0.0,
            0.0,
            DEFAULT_RAIN_HEIGHT_KM,
            45.0
        ),
        None
    );
    assert_eq!(
        rain_attenuation(
            20.0,
            12.0e9,
            HORIZONTAL,
            -0.1,
            0.0,
            DEFAULT_RAIN_HEIGHT_KM,
            45.0
        ),
        None
    );
}

#[test]
fn the_latitude_enters_through_the_vertical_adjustment() {
    // Below thirty six degrees the latitude term is nonzero, so the same
    // geometry gives a different result at two latitudes.
    // 三十六度以下纬度项非零，故同一几何在两个纬度处给出不同结果。
    let temperate = rain_attenuation(
        20.0,
        12.0e9,
        HORIZONTAL,
        THIRTY_DEGREES,
        0.0,
        DEFAULT_RAIN_HEIGHT_KM,
        45.0,
    )
    .expect("valid");
    let tropical = rain_attenuation(
        20.0,
        12.0e9,
        HORIZONTAL,
        THIRTY_DEGREES,
        0.0,
        DEFAULT_RAIN_HEIGHT_KM,
        10.0,
    )
    .expect("valid");
    assert!(
        (temperate.vertical_adjustment - tropical.vertical_adjustment).abs() > 1e-6,
        "{} vs {}",
        temperate.vertical_adjustment,
        tropical.vertical_adjustment
    );
}

#[test]
fn non_finite_geometry_is_rejected() {
    assert_eq!(
        rain_attenuation(
            20.0,
            12.0e9,
            HORIZONTAL,
            THIRTY_DEGREES,
            f64::NAN,
            3.0,
            45.0
        ),
        None
    );
    assert_eq!(
        rain_attenuation(20.0, 12.0e9, HORIZONTAL, THIRTY_DEGREES, 0.0, 3.0, f64::NAN),
        None
    );
}

#[test]
fn polarisation_tilt_converts_degrees() {
    assert_eq!(polarisation_tilt(0.0), 0.0);
    assert_relative(polarisation_tilt(90.0), FRAC_PI_2, "ninety degrees");
    assert_relative(
        polarisation_tilt(45.0),
        FRAC_PI_2 / 2.0,
        "forty five degrees",
    );
}

#[test]
fn the_integrated_path_uses_the_p839_rain_height() {
    use apsis::apsis_atmosphere::{rain_attenuation_at, rain_height};
    let latitude = 45.0;
    let longitude = 0.0;
    let integrated = rain_attenuation_at(
        20.0,
        12.0e9,
        HORIZONTAL,
        THIRTY_DEGREES,
        0.0,
        latitude,
        longitude,
    )
    .expect("valid");
    let explicit = rain_attenuation(
        20.0,
        12.0e9,
        HORIZONTAL,
        THIRTY_DEGREES,
        0.0,
        rain_height(latitude, longitude).expect("valid coordinates"),
        latitude,
    )
    .expect("valid");
    assert!((integrated.attenuation_db - explicit.attenuation_db).abs() < 1e-12);
}

#[test]
fn the_integrated_path_rejects_unusable_coordinates() {
    use apsis::apsis_atmosphere::rain_attenuation_at;
    assert_eq!(
        rain_attenuation_at(20.0, 12.0e9, HORIZONTAL, THIRTY_DEGREES, 0.0, 91.0, 0.0),
        None
    );
    assert_eq!(
        rain_attenuation_at(20.0, 12.0e9, HORIZONTAL, THIRTY_DEGREES, 0.0, 0.0, f64::NAN),
        None
    );
}
