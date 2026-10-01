//! Initialisation of the propagator from a two-line element set.
//!
//! The element set gives mean elements in degrees, revolutions per day, and
//! a drag term. The propagator works in radians, radians per minute, and
//! Earth radii, so the initialisation converts the inputs and precomputes
//! every coefficient that depends only on the elements.
//!
//! The quantity `no_unkozai` is the mean motion after the correction that
//! removes the effect of the Kozai mean motion convention used in the
//! element set. The correction follows from the second zonal harmonic and
//! is solved by a fixed point iteration, because the corrected value appears
//! on both sides of the relation.
//!
//! Source: the correction and the coefficient definitions follow Vallado et
//! al., *Revisiting Spacetrack Report #3*, AIAA 2006-6753, 2006.
//!
//! 由两行根数集初始化传播器。
//!
//! 根数集给出以度、每天圈数计的平根数与一个阻力项。传播器以弧度、每分钟
//! 弧度与地球半径工作，故初始化换算各输入并预计算只依赖于根数的每个系数。
//!
//! 量 `no_unkozai` 是去掉根数集所用 Kozai 平运动约定影响后的修正平运动。
//! 该修正由二阶带谐项得出，并用不动点迭代求解，因为修正后的值出现在关系的
//! 两侧。
//!
//! 出处：该修正与各系数定义遵循 Vallado 等，*Revisiting Spacetrack
//! Report #3*，AIAA 2006-6753，2006。

use crate::error::{PropagationError, PropagationErrorKind};
use crate::model::GravityModel;
use apsis_tle::TwoLineElement;
use core::f64::consts::PI;

/// The number of minutes in a day.
///
/// 一天中的分钟数。
const MINUTES_PER_DAY: f64 = 1_440.0;

/// The number of radians in a revolution.
///
/// 一周中的弧度数。
const TWO_PI: f64 = 2.0 * PI;

/// The period, in minutes, at or above which the deep space branch is used.
///
/// The theory states 225 minutes. Satellites below that period are handled
/// by the near-Earth branch, which is the case for all low Earth orbits.
///
/// 使用深空分支的周期阈值，单位为分钟。
///
/// 理论给出 225 分钟。周期短于该值的卫星由近地分支处理，所有低地球轨道皆属此类。
const DEEP_SPACE_PERIOD_MINUTES: f64 = 225.0;

/// The elements of a satellite in the units the propagator uses.
///
/// Angles are in radians and mean motion is in radians per minute. Helpers
/// convert a [`TwoLineElement`] into this form.
///
/// 卫星根数在传播器所用单位下的形式。
///
/// 角度用弧度，平运动用弧度每分钟。辅助函数把 [`TwoLineElement`] 转为该形式。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Elements {
    /// Epoch of the elements, expressed as a Modified Julian Date on the
    /// scale the element set was written in.
    ///
    /// 根数历元，以根数集书写所用标度上的修正儒略日（Modified Julian Date）
    /// 表示。
    pub epoch_mjd: f64,
    /// `BSTAR` drag term, in inverse Earth radii.
    ///
    /// `BSTAR` 阻力项，单位为地球半径的倒数。
    pub bstar: f64,
    /// Mean motion, in radians per minute.
    ///
    /// 平运动，单位为弧度每分钟。
    pub mean_motion: f64,
    /// Eccentricity, dimensionless.
    ///
    /// 偏心率，无量纲。
    ///
    /// 偏心率，无量纲。
    pub eccentricity: f64,
    /// Inclination, in radians.
    ///
    /// 倾角，单位为弧度。
    ///
    /// 倾角，单位为弧度。
    pub inclination: f64,
    /// Right ascension of the ascending node, in radians.
    ///
    /// 升交点赤经，单位为弧度。
    ///
    /// 升交点赤经，单位为弧度。
    pub right_ascension: f64,
    /// Argument of perigee, in radians.
    ///
    /// 近地点幅角，单位为弧度。
    ///
    /// 近地点幅角，单位为弧度。
    pub argument_of_perigee: f64,
    /// Mean anomaly, in radians.
    ///
    /// 平近点角，单位为弧度。
    ///
    /// 平近点角，单位为弧度。
    pub mean_anomaly: f64,
}

impl Elements {
    /// Converts a parsed element set into the units the propagator uses.
    ///
    /// The epoch is taken as the UTC Modified Julian Date exactly as written
    /// in the element set. The element set epoch is a UTC calendar instant,
    /// but the propagator treats it as a uniform time origin, which is the
    /// convention the theory was fitted under. Converting it to a uniform
    /// scale first would shift the epoch by the leap second offset, about
    /// 33 seconds, and that shift propagates into the solar and lunar series
    /// of the deep space branch.
    ///
    /// 把已解析的根数集转换为传播器所用单位。
    ///
    /// 历元直接取根数集中原样写下的 UTC 修正儒略日（Modified Julian Date）。
    /// 根数集历元是 UTC 日历瞬时，但传播器把它当作均匀时间原点，这正是理论
    /// 拟合所依据的约定。若先换算到均匀标度，历元会偏移一个闰秒量（约 33
    /// 秒），而该偏移会传播进深空分支的日月级数。
    pub fn from_tle(element: &TwoLineElement) -> Option<Self> {
        let epoch_mjd = element.epoch.mjd(apsis_time::TimeScale::Utc)?;
        Some(Self {
            epoch_mjd,
            bstar: element.bstar,
            mean_motion: element.mean_motion * TWO_PI / MINUTES_PER_DAY,
            eccentricity: element.eccentricity,
            inclination: element.inclination.to_radians(),
            right_ascension: element.right_ascension.to_radians(),
            argument_of_perigee: element.argument_of_perigee.to_radians(),
            mean_anomaly: element.mean_anomaly.to_radians(),
        })
    }
}

/// Applies the Kozai to Brouwer mean motion correction.
///
/// The mean motion in an element set is a Kozai mean motion, which the
/// theory must convert to a Brouwer mean motion before use. The relation is
///
/// ```text
/// n0 = n / (1 + 1.5 k2 (3 cos^2 i - 1) / (a^2 sqrt(1 - e^2)) / a^2)
/// ```
///
/// written in the form used here as a fixed point iteration, because `a`
/// depends on `n0` through Kepler's third law. The iteration converges from
/// the unmodified mean motion in a few steps for every orbit that appears in
/// an element set.
/// Applies the Kozai to Brouwer mean motion correction.
///
/// The mean motion in an element set is a Kozai mean motion, which the
/// theory must convert to a Brouwer mean motion before use. Writing `no` for
/// the element set value and `xke` for the Kepler constant, the reference
/// implementation evaluates
///
/// ```text
/// ak   = (xke / no)^(2/3)
/// d1   = 0.75 j2 (3 cos^2 i - 1) / (sqrt(1 - e^2) (1 - e^2))
/// del  = d1 / ak^2
/// adel = ak (1 - del^2 - del (1/3 + 134 del^2 / 81))
/// del  = d1 / adel^2
/// n0   = no / (1 + del)
/// ```
///
/// The second evaluation of `del` uses `adel` rather than `ak`, which is
/// what makes the correction consistent with the semi-major axis implied by
/// the corrected mean motion.
///
/// Source: Vallado et al., *Revisiting Spacetrack Report #3*, AIAA
/// 2006-6753, 2006.
///
/// 施加从 Kozai 平运动到 Brouwer 平运动的修正。
///
/// 根数集中的平运动是 Kozai 平运动，理论须先把它换算为 Brouwer 平运动方能
/// 使用。以 `no` 表示根数集给出的值、`xke` 表示开普勒常数，参考实现按下式
/// 求值：
///
/// ```text
/// ak   = (xke / no)^(2/3)
/// d1   = 0.75 j2 (3 cos^2 i - 1) / (sqrt(1 - e^2) (1 - e^2))
/// del  = d1 / ak^2
/// adel = ak (1 - del^2 - del (1/3 + 134 del^2 / 81))
/// del  = d1 / adel^2
/// n0   = no / (1 + del)
/// ```
///
/// 第二次求 `del` 用的是 `adel` 而非 `ak`，这正是该修正与修正后平运动所隐含
/// 的半长轴相一致的原因。
///
/// 出处：Vallado 等，*Revisiting Spacetrack Report #3*，AIAA 2006-6753，2006。
pub fn unkozai_mean_motion(
    mean_motion: f64,
    eccentricity: f64,
    inclination: f64,
    model: GravityModel,
) -> f64 {
    let xke = model.xke();
    let j2 = model.j2;
    let cosio = inclination.cos();
    let cosio2 = cosio * cosio;
    let omeosq = 1.0 - eccentricity * eccentricity;
    let rteosq = omeosq.sqrt();

    let ak = (xke / mean_motion).powf(2.0 / 3.0);
    let d1 = 0.75 * j2 * (3.0 * cosio2 - 1.0) / (rteosq * omeosq);
    let mut delta = d1 / (ak * ak);
    let adel = ak * (1.0 - delta * delta - delta * (1.0 / 3.0 + 134.0 * delta * delta / 81.0));
    delta = d1 / (adel * adel);
    mean_motion / (1.0 + delta)
}

/// Validates the mean elements before initialisation.
///
/// Returns an error when the orbit cannot be propagated, for example when
/// the eccentricity does not describe a closed orbit or the mean motion is
/// not positive.
///
/// 在初始化前校验平均根数。
///
/// 当轨道不可传播时返回错误，例如偏心率不描述闭合轨道，或平运动不为正。
pub fn validate(elements: &Elements) -> Result<(), PropagationError> {
    if !elements.mean_motion.is_finite() || elements.mean_motion <= 0.0 {
        return Err(PropagationError::new(PropagationErrorKind::InvalidElements)
            .with_detail(format!("mean motion is {}", elements.mean_motion)));
    }
    if !elements.eccentricity.is_finite()
        || elements.eccentricity < 0.0
        || elements.eccentricity >= 1.0
    {
        return Err(PropagationError::new(PropagationErrorKind::InvalidElements)
            .with_detail(format!("eccentricity is {}", elements.eccentricity)));
    }
    for (name, value) in [
        ("inclination", elements.inclination),
        ("right ascension", elements.right_ascension),
        ("argument of perigee", elements.argument_of_perigee),
        ("mean anomaly", elements.mean_anomaly),
    ] {
        if !value.is_finite() {
            return Err(PropagationError::new(PropagationErrorKind::InvalidElements)
                .with_detail(format!("{name} is {value}")));
        }
    }
    if !elements.bstar.is_finite() {
        return Err(PropagationError::new(PropagationErrorKind::InvalidElements)
            .with_detail(format!("bstar is {}", elements.bstar)));
    }
    Ok(())
}

/// Returns `true` when the elements describe a deep space orbit.
///
/// The test is on the orbital period, which follows from the mean motion.
///
/// 当这些根数描述一条深空轨道时返回 `true`。
///
/// 判据是轨道周期，它由平运动推出。
pub fn is_deep_space(mean_motion: f64) -> bool {
    TWO_PI / mean_motion >= DEEP_SPACE_PERIOD_MINUTES
}
