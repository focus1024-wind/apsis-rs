//! The ITU-R S.465 reference antenna pattern.
//!
//! Recommendation S.465 defines a reference pattern for an Earth station
//! antenna, used to assess interference between satellite networks. The
//! pattern has three regions: a main lobe that falls as the square of the
//! off-axis angle, a sidelobe region that falls as the logarithm of the
//! angle, and a floor.
//!
//! The pattern is selected only by the ratio of the antenna diameter to the
//! wavelength. The recommendation gives one pattern for antennas with
//! `D / lambda` of at least 100 and another for smaller antennas, whose main
//! lobe is broader and whose sidelobe region therefore begins later.
//!
//! The main lobe expression is what distinguishes this pattern from the
//! generic reference pattern of the Radio Regulations Appendix 8, which
//! models the sidelobe region only and takes the main lobe width as an
//! input. The two share the shape of the sidelobe region.
//!
//! Source: ITU-R Recommendation S.465-6, *Reference earth-station radiation
//! pattern for use in coordination and interference assessment in the
//! frequency range from 2 to about 30 GHz*.
//!
//! ITU-R S.465 参考天线方向图。
//!
//! 建议书 S.465 为地球站天线定义了一个参考方向图，用于评估卫星网络之间的
//! 干扰。该方向图有三个区：随偏轴角平方下降的主瓣、随角度对数下降的旁瓣
//! 区，以及一个底限。
//!
//! 该方向图仅由天线口径与波长之比选择。该建议书为 `D / lambda` 至少为 100
//! 的天线给出一个方向图，为更小的天线给出另一个，后者主瓣更宽，故其旁瓣区
//! 开始得更晚。
//!
//! 主瓣表达式正是本方向图与《无线电规则》附录 8 通用参考方向图相区别之处，
//! 后者只建模旁瓣区并把主瓣宽度作为输入。二者共享旁瓣区的形状。
//!
//! 出处：ITU-R 建议书 S.465-6，*Reference earth-station radiation pattern
//! for use in coordination and interference assessment in the frequency range
//! from 2 to about 30 GHz*。

use apsis_rf::Decibel;
use core::f64::consts::PI;

/// The peak gain offset of the reference pattern, in decibels.
///
/// The recommendation gives the peak gain as `20 log10(D / lambda) + 7.7`,
/// which corresponds to an aperture efficiency near 0.6. It is slightly
/// below what a well adjusted dish reaches, because the reference pattern
/// represents a typical rather than an optimal antenna.
///
/// 参考方向图的峰值增益偏移，单位为分贝。
///
/// 建议书把峰值增益给出为 `20 log10(D / lambda) + 7.7`，对应约 0.6 的
/// 孔径效率。它略低于调准良好的天线所达到者，因为参考方向图代表的是
/// 典型天线而非最优天线。
pub const PEAK_GAIN_OFFSET_DB: f64 = 7.7;

/// The diameter to wavelength ratio at which the pattern changes form.
///
/// A larger antenna has a narrower main lobe, so its sidelobe region begins
/// earlier and reaches the floor earlier.
///
/// 方向图改变形式处的口径与波长之比。
///
/// 更大的天线主瓣更窄，故其旁瓣区开始得更早、到达底限也更早。
const LARGE_APERTURE_RATIO: f64 = 100.0;

/// The constant and floor angle of the sidelobe region.
///
/// The constant is the value of `G + 25 log10(phi_deg)` in the sidelobe
/// region, and the floor angle is the angle at which the region reaches its
/// floor.
///
/// 旁瓣区的常数与底限角。
///
/// 该常数是旁瓣区内 `G + 25 log10(phi_deg)` 的值，底限角是该区到达其
/// 底限的角度。
fn sidelobe_branch(diameter_over_wavelength: f64) -> (f64, f64) {
    if diameter_over_wavelength >= LARGE_APERTURE_RATIO {
        (29.0, 36.0)
    } else {
        (32.0, 48.0)
    }
}

/// Returns the peak gain of the reference pattern, in decibels relative to an
/// isotropic radiator.
///
/// The expression is `20 log10(D / lambda) + 7.7`.
///
/// Returns `None` when the ratio is not positive and finite.
///
/// 比值非正或非有限时返回 `None`。
pub fn reference_peak_gain(diameter_over_wavelength: f64) -> Option<f64> {
    if !diameter_over_wavelength.is_finite() || diameter_over_wavelength <= 0.0 {
        return None;
    }
    Some(20.0 * diameter_over_wavelength.log10() + PEAK_GAIN_OFFSET_DB)
}

/// Returns the angle at which the main lobe meets the sidelobe region, in
/// degrees.
///
/// The two expressions intersect at this angle, and there is no closed form,
/// so the value is found by bisection. The main lobe falls as the square of
/// the angle while the sidelobe region falls as its logarithm, so the
/// difference has a single zero in the interval searched, which makes
/// bisection reliable.
///
/// The result lies between zero and the floor angle of the sidelobe region.
///
/// 返回主瓣与旁瓣区相接的角度，单位为度。
///
/// 两个表达式在该角度处相交，且没有闭式解，故该值以二分法求得。主瓣按
/// 角度的平方下降，而旁瓣区按其对数下降，故二者之差在所搜索区间内只有
/// 一个零点，这使二分法可靠。
///
/// 结果落在零与旁瓣区底限角之间。
pub fn main_lobe_break_angle(diameter_over_wavelength: f64) -> Option<f64> {
    let peak = reference_peak_gain(diameter_over_wavelength)?;
    let (head, floor_angle) = sidelobe_branch(diameter_over_wavelength);

    let difference = |angle_degrees: f64| {
        let main_lobe = peak - 2.5e-3 * (diameter_over_wavelength * angle_degrees).powi(2);
        let sidelobe = head - 25.0 * angle_degrees.log10();
        main_lobe - sidelobe
    };

    // The main lobe dominates at very small angles and the sidelobe
    // expression dominates at the floor angle, so the zero lies between.
    // 在很小的角度处主瓣占主导，在底限角处旁瓣表达式占主导，故零点位于
    // 二者之间。
    let mut low = 1.0e-9;
    let mut high = floor_angle;
    if difference(high) > 0.0 {
        return Some(high);
    }
    for _ in 0..100 {
        let middle = 0.5 * (low + high);
        if difference(middle) > 0.0 {
            low = middle;
        } else {
            high = middle;
        }
    }
    Some(0.5 * (low + high))
}

/// Returns the gain of the S.465 reference pattern at an off-axis angle.
///
/// The angle is in radians. The three regions are
///
/// ```text
/// G = G_max - 2.5e-3 (D/lambda phi_deg)^2   for 0 <= phi < phi_break
/// G = head - 25 log10(phi_deg)              for phi_break <= phi < phi_floor
/// G = -10                                   for phi_floor <= phi <= 180
/// ```
///
/// where `phi_break` is the angle returned by [`main_lobe_break_angle`], the
/// head is 29 for a large aperture and 32 for a small one, and the floor
/// angle is 36 or 48 degrees accordingly.
///
/// The falling sidelobe expression does not reach exactly minus ten decibels
/// at the floor angle, so the pattern has a step of a few hundredths of a
/// decibel there. This is a property of the recommendation, which defines
/// the regions separately, and it is reproduced here rather than smoothed
/// over.
///
/// Returns `None` when the ratio or the angle is not usable.
///
/// 返回 S.465 参考方向图在偏轴某角度处的增益。
///
/// 角度以弧度计。三个区是
///
/// ```text
/// G = G_max - 2.5e-3 (D/lambda phi_deg)^2   for 0 <= phi < phi_break
/// G = head - 25 log10(phi_deg)              for phi_break <= phi < phi_floor
/// G = -10                                   for phi_floor <= phi <= 180
/// ```
///
/// 其中 `phi_break` 是 [`main_lobe_break_angle`] 返回的角度，head 对大
/// 孔径为 29、对小孔径为 32，底限角相应地是 36 或 48 度。
///
/// 下降的旁瓣表达式在底限角处并不精确达到负十分贝，故方向图在那里有
/// 几百分之一分贝的台阶。这是建议书的性质，它把各区分开定义；此处照实
/// 再现而非加以平滑。
///
/// 比值或角度不可用时返回 `None`。
pub fn reference_pattern_gain(diameter_over_wavelength: f64, off_axis_angle: f64) -> Option<f64> {
    if !off_axis_angle.is_finite() || off_axis_angle < 0.0 {
        return None;
    }
    let peak = reference_peak_gain(diameter_over_wavelength)?;
    let break_angle = main_lobe_break_angle(diameter_over_wavelength)?;
    let (head, floor_angle) = sidelobe_branch(diameter_over_wavelength);
    let angle_degrees = off_axis_angle.to_degrees();

    if angle_degrees < break_angle {
        Some(peak - 2.5e-3 * (diameter_over_wavelength * angle_degrees).powi(2))
    } else if angle_degrees <= floor_angle {
        Some(head - 25.0 * angle_degrees.log10())
    } else {
        Some(-10.0)
    }
}
/// Returns the gain of the S.465 reference pattern as a decibel value.
///
/// Returns `None` when the ratio or the angle is not usable.
///
/// 返回 S.465 参考方向图的增益，以分贝值表示。
///
/// 比值或角度不可用时返回 `None`。
pub fn reference_pattern_decibel(
    diameter_over_wavelength: f64,
    off_axis_angle: f64,
) -> Option<Decibel> {
    let gain = reference_pattern_gain(diameter_over_wavelength, off_axis_angle)?;
    Decibel::from_db(gain)
        .is_finite()
        .then_some(Decibel::from_db(gain))
}

/// Returns the diameter to wavelength ratio of an antenna with a given peak
/// gain.
///
/// This inverts the peak gain expression, which is useful when a station is
/// specified by its gain rather than its size.
///
/// Returns `None` when the gain is not finite.
///
/// 返回某峰值增益天线的口径与波长之比。
///
/// 它反解峰值增益表达式，当某站以其增益而非尺寸规定时有用。
///
/// 增益非有限时返回 `None`。
pub fn diameter_over_wavelength_from_peak_gain(peak_gain_dbi: f64) -> Option<f64> {
    if !peak_gain_dbi.is_finite() {
        return None;
    }
    Some(10f64.powf((peak_gain_dbi - PEAK_GAIN_OFFSET_DB) / 20.0))
}

/// Returns the half power beam width of the reference pattern main lobe, in
/// radians.
///
/// The main lobe falls to three decibels at
/// `phi = sqrt(3 / 2.5e-3) / (D / lambda)` degrees from boresight, so the
/// full width between the two three decibel points is twice that value.
///
/// Returns `None` when the ratio is not positive and finite.
///
/// 比值非正或非有限时返回 `None`。
pub fn reference_beam_width(diameter_over_wavelength: f64) -> Option<f64> {
    if !diameter_over_wavelength.is_finite() || diameter_over_wavelength <= 0.0 {
        return None;
    }
    let half_angle_degrees = (3.0_f64 / 2.5e-3).sqrt() / diameter_over_wavelength;
    Some(2.0 * half_angle_degrees * PI / 180.0)
}
