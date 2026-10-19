//! Tests for the ITU-R P.835 reference standard atmosphere.
//!
//! ITU-R P.835 参考标准大气的测试。

use apsis::apsis_atmosphere::{
    geopotential_height, radio_refractive_index, standard_pressure, standard_temperature,
    standard_water_vapour_density,
};

const TOLERANCE: f64 = 1e-9;

#[test]
fn the_geopotential_height_is_below_the_geometric_height() {
    // The conversion compresses the height because gravity weakens with
    // altitude, so the geopotential height is always the smaller value.
    // 该换算压缩高度，因为引力随高度减弱，故势高总是较小的值。
    for geometric in [0.0, 1.0, 11.0, 50.0, 100.0] {
        let geopotential = geopotential_height(geometric);
        assert!(geopotential <= geometric, "height {geometric}");
    }
    // At the surface the two coincide.
    // 在地表处二者重合。
    assert_eq!(geopotential_height(0.0), 0.0);
}

#[test]
fn the_surface_conditions_match_the_recommendation() {
    // The mean annual global reference atmosphere starts at 288.15 K and
    // 1013.25 hPa.
    // 年平均全球参考大气始于 288.15 K 与 1013.25 hPa。
    assert!((standard_temperature(0.0) - 288.15).abs() < TOLERANCE);
    assert!((standard_pressure(0.0) - 1_013.25).abs() < TOLERANCE);
    assert!((standard_water_vapour_density(0.0, 7.5) - 7.5).abs() < TOLERANCE);
}

#[test]
fn the_temperature_profile_has_the_tabulated_segments() {
    // The troposphere cools at 6.5 K per kilometer of geopotential height,
    // so the rate is expressed against that height, not the geometric one.
    // 对流层以每千米势高 6.5 K 冷却，故该速率是针对势高而非几何高度表达的。
    let expected = 288.15 - 6.5 * geopotential_height(1.0);
    assert!((standard_temperature(1.0) - expected).abs() < TOLERANCE);

    // The stratosphere is isothermal at 216.65 K between 11 and 20 km of
    // geopotential height. A geometric height of 15 km has a geopotential
    // height of about 14.97 km, which is inside that band.
    // 平流层在 11 至 20 km 势高之间等温于 216.65 K。15 km 的几何高度其势高约为
    // 14.97 km，落在该带内。
    let geopotential = geopotential_height(15.0);
    assert!(
        (11.0..=20.0).contains(&geopotential),
        "height {geopotential}"
    );
    assert!((standard_temperature(15.0) - 216.65).abs() < TOLERANCE);

    // Above 20 km geopotential the temperature rises at 1 K per kilometer.
    // A geometric height of 30 km reaches the middle of that segment.
    // 20 km 势高以上温度以每千米 1 K 上升。30 km 的几何高度到达该段中部。
    let geopotential = geopotential_height(30.0);
    assert!(
        (20.0..32.0).contains(&geopotential),
        "height {geopotential}"
    );
    let expected = 216.65 + (geopotential - 20.0);
    assert!((standard_temperature(30.0) - expected).abs() < TOLERANCE);

    // The temperature is continuous across each segment boundary. The
    // boundaries are in geopotential height, so they are converted to the
    // geometric heights at which they occur before sampling either side.
    // 温度在每个分段边界处连续。各边界以势高给出，故在两侧采样之前先换算为它们
    // 发生时对应的几何高度。
    for geopotential in [11.0_f64, 20.0, 32.0, 47.0, 51.0, 71.0] {
        let radius = 6_356.766;
        let geometric = radius * geopotential / (radius - geopotential);
        let below = standard_temperature(geometric - 0.001);
        let above = standard_temperature(geometric + 0.001);
        assert!(
            (below - above).abs() < 0.01,
            "boundary {geopotential}: {below} against {above}"
        );
    }
}

#[test]
fn the_temperature_is_continuous_across_every_segment() {
    for boundary in [11.0_f64, 20.0, 32.0, 47.0, 51.0, 71.0, 84.852] {
        // The boundaries are in geopotential height, so a geometric height
        // just either side is used.
        // 各边界以势高给出，故用恰好在其两侧的几何高度。
        let below = standard_temperature(boundary - 0.01);
        let above = standard_temperature(boundary + 0.01);
        assert!(
            (below - above).abs() < 0.2,
            "boundary {boundary}: {below} against {above}"
        );
    }
}

#[test]
fn the_pressure_falls_with_height() {
    let mut previous = f64::INFINITY;
    for height in [0.0, 1.0, 5.0, 11.0, 20.0, 32.0, 47.0, 51.0, 71.0, 84.852] {
        let pressure = standard_pressure(height);
        assert!(pressure < previous, "height {height} gave {pressure}");
        previous = pressure;
    }
}

#[test]
fn the_pressure_is_continuous_across_the_segments() {
    for boundary in [11.0_f64, 20.0, 32.0, 47.0, 51.0, 71.0, 84.852] {
        let below = standard_pressure(boundary - 0.01);
        let above = standard_pressure(boundary + 0.01);
        let relative = (below - above).abs() / below;
        assert!(
            relative < 0.01,
            "boundary {boundary}: {below} against {above}, relative {relative}"
        );
    }
}

#[test]
fn the_pressure_at_the_tropopause_matches_the_reference() {
    // The tropopause is at 11 km of geopotential height, which is a
    // geometric height of about 11.02 km. The pressure there is 226.32 hPa.
    // 对流层顶在 11 km 势高处，即约 11.02 km 的几何高度。那里的气压是 226.32 hPa。
    let geometric = 11.019_068;
    assert!((geopotential_height(geometric) - 11.0).abs() < 1e-5);
    let pressure = standard_pressure(geometric);
    assert!((pressure - 226.32).abs() < 0.01, "pressure {pressure}");
}

#[test]
fn the_water_vapour_profile_is_exponential() {
    // The scale height is two kilometers, so the density halves about every
    // 1.386 km.
    // 标高为两千米，故密度大约每 1.386 km 减半。
    let surface = 7.5;
    let at_two = standard_water_vapour_density(2.0, surface);
    assert!((at_two - surface / core::f64::consts::E).abs() < 1e-9);
    // The profile scales with the surface value.
    // 该廓线随地表值缩放。
    let doubled = standard_water_vapour_density(2.0, 15.0);
    assert!((doubled / at_two - 2.0).abs() < 1e-12);
}

#[test]
fn the_radio_refractive_index_at_the_surface_is_about_one() {
    // A refractivity of about three hundred N units gives an index about
    // 1.0003.
    // 约三百 N 单位的折射率给出约 1.0003 的折射指数。
    let index = radio_refractive_index(1_013.25, 10.0, 288.15);
    assert!(index > 1.0 && index < 1.001, "index {index}");
    assert!((index - 1.000_315).abs() < 1e-5, "index {index}");
}

#[test]
fn the_radio_refractive_index_grows_with_humidity() {
    let dry = radio_refractive_index(1_013.25, 0.0, 288.15);
    let moist = radio_refractive_index(1_013.25, 20.0, 288.15);
    assert!(moist > dry);
}

#[test]
fn a_zero_temperature_is_handled_without_dividing_by_zero() {
    // The expression divides by the temperature, so a zero would give an
    // infinite value. The function returns the index of vacuum instead.
    // 该表达式除以温度，故零会给出无穷值。该函数改而返回真空的折射指数。
    assert_eq!(radio_refractive_index(1_013.25, 10.0, 0.0), 1.0);
}

/// The specific gas constant of dry air, in J/(kg K).
///
/// 干空气比气体常数，单位为 J/(kg K)。
const R_DRY_AIR: f64 = 287.052_874;

#[test]
fn the_troposphere_density_matches_the_barometric_formula() {
    // The density is pressure over the gas constant times temperature. The
    // pressure is re-derived from the barometric formula at the geopotential
    // height the P.835 profile actually uses, so the check is independent of
    // the pressure function.
    // 密度为气压除以气体常数与温度之积。气压由气压公式在该高度对应的势高处重新
    // 导出，故校验独立于气压函数。
    for geometric_km in [0.0_f64, 1.0, 5.0, 10.0] {
        let geopotential = geopotential_height(geometric_km);
        let temperature = 288.15 - 6.5 * geopotential;
        let pressure_hpa = 1_013.25 * (288.15 / temperature).powf(-34.163_2 / 6.5);
        let expected = pressure_hpa * 100.0 / (R_DRY_AIR * temperature);
        let actual = standard_pressure(geometric_km) * 100.0
            / (R_DRY_AIR * standard_temperature(geometric_km));
        assert!(
            (actual - expected).abs() < 1e-6,
            "height {geometric_km}: {actual} against {expected}"
        );
    }
}

#[test]
fn the_stratosphere_density_matches_the_isothermal_formula() {
    // Between 11 and 20 km of geopotential height the temperature is
    // constant, so the pressure falls exponentially with the scale height
    // R T / g.
    // 在势高 11 至 20 km 之间温度恒定，故气压以标高 R T / g 指数下降。
    for geometric_km in [12.0_f64, 15.0, 18.0] {
        let geopotential = geopotential_height(geometric_km);
        let temperature = 216.65;
        let pressure_hpa = 226.322_6
            * (-34.163_2 * (geopotential - 11.0) / temperature).exp();
        let expected = pressure_hpa * 100.0 / (R_DRY_AIR * temperature);
        let actual = standard_pressure(geometric_km) * 100.0
            / (R_DRY_AIR * standard_temperature(geometric_km));
        assert!(
            (actual - expected).abs() < 1e-6,
            "height {geometric_km}: {actual} against {expected}"
        );
    }
}

#[test]
fn the_sea_level_density_is_the_isa_reference_value() {
    let density = standard_pressure(0.0) * 100.0 / (R_DRY_AIR * standard_temperature(0.0));
    assert!((density - 1.2250).abs() < 1e-4, "{density}");
}
