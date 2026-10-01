//! Tests for the IAU 2000/2006 Earth orientation models.
//!
//! The expected values are those of the IAU SOFA library and its ERFA
//! derivative for the same instants. They are recorded here so that a change
//! to a coefficient or a rotation convention shows up as a failing
//! assertion.
//!
//! IAU 2000/2006 地球定向模型的测试。
//!
//! 期望值是 IAU SOFA 库及其 ERFA 衍生物对相同瞬时的值。它们被记录在此，使对某个
//! 系数或转动约定的改动表现为断言失败。

use apsis_frames::iau2000::{
    PolarMotion, bias_precession_nutation_matrix, celestial_to_terrestrial_matrix,
    equation_of_the_equinoxes, equation_of_the_equinoxes_complementary_terms, general_precession,
    greenwich_apparent_sidereal_time, mean_obliquity, nutation_2000a,
    nutation_2000a_with_2006_adjustment, precession_angles,
};
use core::f64::consts::PI;

/// Converts arcseconds to radians.
///
/// 把角秒换算为弧度。
const ARCSECOND: f64 = PI / (180.0 * 3_600.0);

/// Returns the Terrestrial Time Julian date for a Modified Julian Date on
/// the UTC scale, using the offsets in force from 2017.
///
/// 使用自 2017 年起生效的偏移，返回 UTC 时标上某约化儒略日对应的地球时儒略日。
fn tt_julian_date(utc_mjd: f64) -> f64 {
    utc_mjd + 2_400_000.5 + 69.184 / 86_400.0
}

#[test]
fn the_mean_obliquity_at_j2000_matches_the_reference() {
    // The IAU 2006 obliquity at J2000.0 is 84381.406 arcseconds.
    // J2000.0 处的 IAU 2006 黄赤交角为 84381.406 角秒。
    let obliquity = mean_obliquity(0.0);
    assert!(
        (obliquity / ARCSECOND - 84_381.406).abs() < 1e-9,
        "{} arcseconds",
        obliquity / ARCSECOND
    );
}

#[test]
fn the_nutation_at_j2000_matches_the_reference() {
    // The IAU 2000A nutation at J2000.0, from ERFA.
    // J2000.0 处的 IAU 2000A 章动，取自 ERFA。
    let t = 0.0;
    let nutation = nutation_2000a(t);
    assert!(
        (nutation.longitude - -6.754422426417298e-05).abs() < 1e-16,
        "longitude {}",
        nutation.longitude
    );
    assert!(
        (nutation.obliquity - -2.7970831192374137e-05).abs() < 1e-16,
        "obliquity {}",
        nutation.obliquity
    );
}

#[test]
fn the_nutation_2006_adjustment_is_a_scaling() {
    // The IAU 2006 adjustment scales the IAU 2000A nutation rather than
    // adding an independent term, so the two must stay in proportion.
    // IAU 2006 调整是对 IAU 2000A 章动缩放而非加上独立项，故二者必须保持比例。
    for t in [0.0, 0.5, 1.0, 2.0] {
        let base = nutation_2000a(t);
        let adjusted = nutation_2000a_with_2006_adjustment(t);
        let expected_longitude = base.longitude * (1.0 + 0.469_7e-6 - 2.777_4e-6 * t);
        let expected_obliquity = base.obliquity * (1.0 - 2.777_4e-6 * t);
        assert!(
            (adjusted.longitude - expected_longitude).abs() < 1e-20,
            "t {t}: {} against {expected_longitude}",
            adjusted.longitude
        );
        assert!(
            (adjusted.obliquity - expected_obliquity).abs() < 1e-20,
            "t {t}: {} against {expected_obliquity}",
            adjusted.obliquity
        );
    }
}

#[test]
fn the_nutation_matches_the_reference_over_a_century() {
    // Recorded from ERFA at each date. The values are in radians.
    // 在每个日期从 ERFA 记录。各值以弧度计。
    let cases = [
        (40_587.0, 2.1896610891495316e-05, 4.070477585626008e-05),
        (51_544.5, -6.754419567422628e-05, -2.797091091919891e-05),
        (60_000.0, -4.496357815269203e-05, 3.753534415176348e-05),
    ];
    for (mjd, longitude, obliquity) in cases {
        let tt = tt_julian_date(mjd);
        let t = (tt - 2_451_545.0) / 36_525.0;
        let nutation = nutation_2000a(t);
        assert!(
            (nutation.longitude - longitude).abs() < 1e-14,
            "MJD {mjd}: longitude {} against {longitude}",
            nutation.longitude
        );
        assert!(
            (nutation.obliquity - obliquity).abs() < 1e-14,
            "MJD {mjd}: obliquity {} against {obliquity}",
            nutation.obliquity
        );
    }
}

#[test]
fn the_precession_angles_at_j2000_match_the_reference() {
    // The P03 precession angles at J2000.0.
    // J2000.0 处的 P03 岁差角。
    let angles = precession_angles(0.0);
    assert!(
        (angles.gamma_bar - -0.052_928 * ARCSECOND).abs() < 1e-18,
        "gamma bar {}",
        angles.gamma_bar
    );
    assert!(
        (angles.phi_bar - 84_381.412_819 * ARCSECOND).abs() < 1e-15,
        "phi bar {}",
        angles.phi_bar
    );
    assert!(
        (angles.psi_bar - -0.041_775 * ARCSECOND).abs() < 1e-18,
        "psi bar {}",
        angles.psi_bar
    );
    assert!(
        (angles.epsilon - 84_381.406 * ARCSECOND).abs() < 1e-15,
        "epsilon {}",
        angles.epsilon
    );
}

#[test]
fn the_bias_precession_nutation_matrix_matches_the_reference() {
    // The GCRS-to-true-of-date matrix at J2000.0 on the Terrestrial Time
    // scale, from ERFA's `pnm06a`. The values are the first row.
    // 地球时时标上 J2000.0 处的 GCRS 到该日真坐标矩阵，取自 ERFA 的 `pnm06a`。
    // 各值是第一行。
    let tt = 51_544.5 + 2_400_000.5 + 69.184 / 86_400.0;
    let t = (tt - 2_451_545.0) / 36_525.0;
    let matrix = bias_precession_nutation_matrix(t);

    let expected = [
        [
            0.9999999977211409,
            6.189934767298694e-05,
            2.69478892058386e-05,
        ],
        [
            -6.1900102296513e-05,
            0.999999997692101,
            2.8003132867160385e-05,
        ],
        [
            -2.6946155767988307e-05,
            -2.8004800880443437e-05,
            0.9999999992448179,
        ],
    ];
    for (row, expected_row) in expected.iter().enumerate() {
        for (column, expected_value) in expected_row.iter().enumerate() {
            let value = matrix.get(row, column).expect("the indices are in range");
            assert!(
                (value - expected_value).abs() < 1e-14,
                "at ({row}, {column}): {value} against {expected_value}"
            );
        }
    }
}

#[test]
fn the_bias_precession_nutation_matrix_is_a_rotation() {
    for mjd in [40_587.0, 51_544.5, 60_000.0] {
        let tt = tt_julian_date(mjd);
        let t = (tt - 2_451_545.0) / 36_525.0;
        let matrix = bias_precession_nutation_matrix(t);
        assert!(matrix.is_rotation(1e-12), "MJD {mjd}");
        assert!(
            (matrix.determinant() - 1.0).abs() < 1e-12,
            "MJD {mjd}: determinant {}",
            matrix.determinant()
        );
    }
}

#[test]
fn the_celestial_to_terrestrial_matrix_is_a_rotation() {
    for mjd in [40_587.0, 51_544.5, 60_000.0] {
        let tt = tt_julian_date(mjd);
        let matrix = celestial_to_terrestrial_matrix(tt, tt, PolarMotion::NONE);
        assert!(matrix.is_rotation(1e-9), "MJD {mjd}");
        assert!(
            (matrix.determinant() - 1.0).abs() < 1e-9,
            "MJD {mjd}: determinant {}",
            matrix.determinant()
        );
    }
}

#[test]
fn the_celestial_to_terrestrial_matrix_matches_the_reference() {
    // The matrix at J2000.0 on the Terrestrial Time scale, from ERFA with
    // zero polar motion. Each entry is checked to a nanoradian.
    // 地球时时标上 J2000.0 处的矩阵，取自极移为零的 ERFA。每个元素检查到纳弧度。
    let tt = 51_544.5 + 2_400_000.5 + 69.184 / 86_400.0;
    let matrix = celestial_to_terrestrial_matrix(tt, tt, PolarMotion::NONE);

    // The first row, from ERFA's `c2t06a` composed with the equinox based
    // sidereal time.
    // 第一行，取自 ERFA 的 `c2t06a` 与基于春分点的恒星时之合成。
    let expected_first_row = [
        0.18651846184811946,
        -0.982451455739228,
        -2.248740188309595e-05,
    ];
    for (column, expected_value) in expected_first_row.iter().enumerate() {
        let value = matrix.get(0, column).expect("the index is in range");
        assert!(
            (value - expected_value).abs() < 1e-9,
            "at (0, {column}): {value} against {expected_value}"
        );
    }
}

#[test]
fn the_equation_of_the_equinoxes_includes_the_complementary_terms() {
    // The complementary terms are a few thousandths of an arcsecond and must
    // not be dropped: their omission would show as a millisecond of
    // sidereal time.
    // 补充项是几千分之一角秒，不得省略：略去它们会表现为一毫秒的恒星时。
    let t = 0.0;
    let complementary = equation_of_the_equinoxes_complementary_terms(t);
    assert!(
        complementary.abs() > 1e-9,
        "the complementary terms are {complementary}"
    );
    // At J2000.0 the leading term is 2640.96 microarcseconds times the sine
    // of the Moon's ascending node.
    // 在 J2000.0 处首项是 2640.96 微角秒乘以月球升交点角的正弦。
    let expected = 1.021330096302465e-08;
    assert!(
        (complementary - expected).abs() < 1e-15,
        "{complementary} against {expected}"
    );
}

#[test]
fn the_general_precession_is_quadratic_in_time() {
    // The argument is (0.024381750 + 0.00000538691 t) t, so at small times
    // it grows linearly with the leading coefficient.
    // 该辐角为 (0.024381750 + 0.00000538691 t) t，故在小时间处它随首系数线性增长。
    for t in [0.1, 0.5, 1.0] {
        let value = general_precession(t);
        let expected = (0.024_381_750 + 0.000_005_386_91 * t) * t;
        assert!((value - expected).abs() < 1e-18, "t {t}");
    }
}

#[test]
fn sidereal_time_advances_by_about_a_turn_per_day() {
    // Greenwich apparent sidereal time must increase by a full turn plus a
    // small amount each day. The small amount is about 3.94 minutes of
    // time, which is 0.9856 degrees.
    // 格林尼治视恒星时每天必须增加一整圈再加上一小量。该小量约为 3.94 分钟时间，
    // 即 0.9856 度。
    let first = greenwich_apparent_sidereal_time(60_000.0, 60_000.0);
    let second = greenwich_apparent_sidereal_time(60_001.0, 60_001.0);
    let advance = (second - first).rem_euclid(2.0 * PI);
    let degrees = advance.to_degrees();
    assert!(
        (degrees - 0.985_6).abs() < 0.01,
        "advance {degrees} degrees"
    );
}

#[test]
fn zero_polar_motion_is_the_identity_rotation() {
    let matrix = PolarMotion::NONE.matrix();
    for row in 0..3 {
        for column in 0..3 {
            let value = matrix.get(row, column).expect("the indices are in range");
            let expected = if row == column { 1.0 } else { 0.0 };
            assert!((value - expected).abs() < 1e-15, "at ({row}, {column})");
        }
    }
    assert!(PolarMotion::NONE.is_valid());
}

#[test]
fn a_polar_motion_offsets_the_rotation() {
    // A polar motion of one arcsecond must tilt the matrix by about that
    // much, which is 4.85 microradians.
    // 一角秒的极移必须使矩阵倾斜约那么多，即 4.85 微弧度。
    let motion = PolarMotion::new(ARCSECOND, ARCSECOND, 0.0);
    let matrix = motion.matrix();
    assert!(matrix.is_rotation(1e-12));
    // The off-diagonal term is the sine of the displacement.
    // 非对角项是该位移的正弦。
    let value = matrix.get(0, 2).expect("the index is in range");
    assert!(
        (value - ARCSECOND.sin()).abs() < 1e-15,
        "at (0, 2): {value}"
    );
    assert!(motion.is_valid());
    assert!(!PolarMotion::new(f64::NAN, 0.0, 0.0).is_valid());
}

#[test]
fn the_polar_motion_shifts_the_earth_fixed_frame() {
    // Applying a polar motion must change the terrestrial position of a
    // fixed inertial direction.
    // 施加极移必须改变某固定惯性方向的地球位置。
    let tt = 60_000.0;
    let without = celestial_to_terrestrial_matrix(tt, tt, PolarMotion::NONE);
    let with = celestial_to_terrestrial_matrix(
        tt,
        tt,
        PolarMotion::new(0.1 * ARCSECOND, -0.2 * ARCSECOND, 0.0),
    );
    let difference = (without.transpose() * with).transpose() * with;
    // The two must differ, and the difference must be of the order of the
    // applied angles.
    // 二者必须不同，且差值必须与所施加的角度同量级。
    let mut largest = 0.0_f64;
    for row in 0..3 {
        for column in 0..3 {
            let a = without.get(row, column).expect("in range");
            let b = difference.get(row, column).expect("in range");
            largest = largest.max((a - b).abs());
        }
    }
    assert!(largest > 1e-8, "the largest difference is {largest}");
    assert!(largest < 1e-5, "the largest difference is {largest}");
}

#[test]
fn the_nutation_reaches_the_expected_magnitude() {
    // The nutation in longitude reaches about seventeen arcseconds and the
    // nutation in obliquity about nine, so over a range of dates the
    // largest value found must be in that range.
    // 黄经章动达约十七角秒，黄赤交角章动约九角秒，故在一段日期上所能找到的最大值
    // 必须落在该范围内。
    let mut largest_longitude = 0.0_f64;
    let mut largest_obliquity = 0.0_f64;
    for step in 0..400 {
        // A span of about fifty years in steps of a fortnight.
        // 约五十年、步长为两周的跨度。
        let t = f64::from(step) * 0.125;
        let nutation = nutation_2000a(t);
        largest_longitude = largest_longitude.max(nutation.longitude.abs());
        largest_obliquity = largest_obliquity.max(nutation.obliquity.abs());
    }
    let longitude_arcseconds = largest_longitude / ARCSECOND;
    let obliquity_arcseconds = largest_obliquity / ARCSECOND;
    assert!(
        (19.0..20.0).contains(&longitude_arcseconds),
        "longitude {longitude_arcseconds} arcseconds"
    );
    assert!(
        (8.0..10.0).contains(&obliquity_arcseconds),
        "obliquity {obliquity_arcseconds} arcseconds"
    );
}

#[test]
fn the_nutation_is_periodic() {
    // The dominant terms have periods of about eighteen years and half a
    // year, so the value must return close to its initial value after
    // eighteen years and must oscillate in between.
    // 主导项的周期约为十八年与半年，故该值必须在十八年后回到接近其初值，并在其间
    // 振荡。
    let initial = nutation_2000a(0.0).longitude;
    let after_eighteen_years = nutation_2000a(0.18).longitude;
    assert!(
        (after_eighteen_years - initial).abs() < 1.5e-05,
        "{} against {initial}",
        after_eighteen_years
    );

    // The sign must change at some point within that span.
    // 符号必须在该跨度内某处改变。
    let mut seen_positive = false;
    let mut seen_negative = false;
    for step in 0..200 {
        let t = f64::from(step) * 0.001;
        let value = nutation_2000a(t).longitude;
        seen_positive |= value > 0.0;
        seen_negative |= value < 0.0;
    }
    assert!(seen_positive && seen_negative);
}

#[test]
fn the_equation_of_the_equinoxes_follows_the_nutation() {
    // The equation of the equinoxes is dominated by the nutation in
    // longitude projected onto the true equator, so it shares its sign.
    // 分点方程式主要由投影到真赤道上的黄经章动主导，故它与后者同号。
    for t in [0.02, 0.05, 0.1, 0.2] {
        let nutation = nutation_2000a_with_2006_adjustment(t);
        let equation = equation_of_the_equinoxes(t);
        assert!(
            nutation.longitude.signum() == equation.signum(),
            "t {t}: {} against {}",
            nutation.longitude,
            equation
        );
    }
}

#[test]
fn a_matrix_at_two_epochs_separated_by_a_century_differs_by_the_precession() {
    // The precession over a century is about 1.4 degrees, so the two
    // matrices must differ by an angle of that order.
    // 一世纪的岁差约为 1.4 度，故两个矩阵必须相差该量级的角。
    let first = bias_precession_nutation_matrix(0.0);
    let second = bias_precession_nutation_matrix(1.0);
    let difference = first.transpose() * second;
    // The trace of a rotation matrix is 1 + 2 cos(angle), so the angle
    // follows from it.
    // 转动矩阵的迹为 1 + 2 cos(angle)，故该角由它得出。
    let cosine = ((difference.trace() - 1.0) / 2.0).clamp(-1.0, 1.0);
    let degrees = cosine.acos().to_degrees();
    assert!(
        (1.0..2.0).contains(&degrees),
        "the rotation angle is {degrees} degrees"
    );
}

#[test]
fn the_1982_sidereal_time_matches_the_reference() {
    // Greenwich mean sidereal time from the 1982 expression, which is the
    // one the SGP4 theory was fitted against. The values are from ERFA.
    // 1982 表达式给出的格林尼治平恒星时，即 SGP4 理论所拟合者。各值取自 ERFA。
    let cases = [
        (51_544.5, 4.894_961_212_823_059),
        (60_476.999_999_810_5, 4.620_858_198_832_686),
        (40_587.0, 1.749_337_177_344_97),
    ];
    for (mjd, expected) in cases {
        let actual = apsis_frames::iau2000::greenwich_mean_sidereal_time_1982(2_400_000.5, mjd);
        assert!(
            (actual - expected).abs() < 1e-15,
            "MJD {mjd}: {actual} against {expected}"
        );
    }
}

#[test]
fn the_1982_sidereal_time_is_a_full_turn_per_day() {
    // Sidereal time advances by one turn plus the excess of the sidereal
    // day over the solar day, which is 0.9856 degrees per day.
    // 恒星时推进一整圈再加上恒星日超出太阳日的部分，即每天 0.9856 度。
    let first = apsis_frames::iau2000::greenwich_mean_sidereal_time_1982(2_400_000.5, 60_000.0);
    let second = apsis_frames::iau2000::greenwich_mean_sidereal_time_1982(2_400_000.5, 60_001.0);
    let advance = (second - first).rem_euclid(2.0 * PI);
    assert!(
        (advance.to_degrees() - 0.985_6).abs() < 0.001,
        "advance {} degrees",
        advance.to_degrees()
    );
}

#[test]
fn the_teme_transformation_to_earth_fixed_matches_the_reference() {
    use apsis_frames::iau2000::{PolarMotion, teme_to_terrestrial_matrix};
    use apsis_math::Vec3;

    // The reference values are ERFA's `gmst82` applied as a third axis
    // rotation with the identity polar motion.
    // 参考值是把 ERFA 的 `gmst82` 作为第三轴转动、配以单位极移施加所得。
    let cases = [
        (
            60_481.5,
            [7_000.0, 1_000.0, 2_000.0],
            [1_098.716_242_792, -6_985.185_940_104, 2_000.0],
        ),
        (
            51_544.5,
            [7_000.0, -3_000.0, 1_000.0],
            [4_221.057_369_867, 6_338.980_571_061, 1_000.0],
        ),
    ];
    for (mjd, position, expected) in cases {
        let matrix = teme_to_terrestrial_matrix(2_400_000.5, mjd, PolarMotion::NONE);
        let result = matrix * Vec3::new(position[0], position[1], position[2]);
        let expected = Vec3::new(expected[0], expected[1], expected[2]);
        assert!(
            (result - expected).norm() < 1e-6,
            "MJD {mjd}: {result} against {expected}"
        );
    }
}

#[test]
fn the_teme_transformation_preserves_length() {
    use apsis_frames::iau2000::{PolarMotion, teme_to_terrestrial_matrix};
    use apsis_math::Vec3;

    let matrix = teme_to_terrestrial_matrix(2_400_000.5, 60_000.0, PolarMotion::NONE);
    assert!(matrix.is_rotation(1e-12));
    let vector = Vec3::new(7_000.0, -1_000.0, 500.0);
    assert!(
        ((matrix * vector).norm() - vector.norm()).abs() < 1e-9,
        "the length changed"
    );
}

#[test]
fn the_teme_frame_uses_the_mean_equinox_not_the_true_one() {
    // The distinction matters. TEME's first axis is the mean equinox, so it
    // pairs with a mean sidereal time. Substituting the apparent value, or
    // the modern mean value, rotates the result by a fraction of a degree,
    // which is many kilometres of ground position.
    // 该区别要紧。TEME 的第一轴是平春分点，故它与平恒星时配对。换成视值或现代平值
    // 会把结果转动零点几度，即许多千米的地面位置。
    use apsis_frames::iau2000::{
        greenwich_apparent_sidereal_time, greenwich_mean_sidereal_time,
        greenwich_mean_sidereal_time_1982,
    };

    let mjd = 60_481.5;
    let tt = mjd + 2_400_000.5 + 69.184 / 86_400.0;
    let sidereal_1982 = greenwich_mean_sidereal_time_1982(2_400_000.5, mjd);
    let apparent = greenwich_apparent_sidereal_time(tt, tt);
    let modern_mean = greenwich_mean_sidereal_time(tt, tt);

    // The apparent value includes the equation of the equinoxes, which is a
    // few thousandths of a degree.
    // 视值包含分点方程式，即千分之几度。
    let equation = (apparent - modern_mean).abs().to_degrees();
    assert!(
        equation > 1e-4 && equation < 1e-2,
        "the equation of the equinoxes is {equation} degrees"
    );

    // The 1982 expression, which TEME is fitted against, differs from the
    // modern mean value by a fraction of a degree, because the precession
    // model changed between them.
    // TEME 所拟合的 1982 表达式与现代平值相差零点几度，因为二者之间的岁差模型
    // 改变了。
    let secular = (sidereal_1982 - modern_mean).abs().to_degrees();
    assert!(
        secular > 0.1 && secular < 1.0,
        "the difference is {secular} degrees"
    );
}
