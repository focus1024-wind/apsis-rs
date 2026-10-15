//! The True Equator Mean Equinox frame and its transformation to the
//! Earth-fixed frame.
//!
//! The SGP4 propagator produces its state in the True Equator Mean Equinox
//! (TEME) frame. TEME is the true equator of date, so its third axis is the
//! true celestial pole and it carries the nutation, but its first axis
//! points at the mean equinox rather than at the true equinox.
//!
//! Because the first axis is the mean equinox, the transformation to the
//! Earth-fixed frame uses Greenwich mean sidereal time, not the apparent
//! value. That is the detail that is easy to get wrong: subtracting the
//! equation of the equinoxes as well, as one would for a true-equinox frame,
//! rotates the result by about a thousandth of a degree, which is a
//! kilometre of ground position for a low Earth satellite.
//!
//! The time is therefore the UT1 Julian date, from which the mean sidereal
//! time follows, and the polar motion is applied last.
//!
//! Source: the relation between TEME and the Earth-fixed frame is given in
//! Vallado, D. A., *Fundamentals of Astrodynamics and Applications*, 4th
//! edition, microcosm Press, 2013, section 3.7. The sidereal time
//! expression is the one used by the SGP4 reference implementation, as
//! given in the same work and implemented by `eraGmst82`.
//!
//! 真赤道平春分点坐标系及其到地固系的变换。
//!
//! SGP4 传播器在真赤道平春分点（TEME）坐标系中产生其状态。TEME 是该日的
//! 真赤道，故其第三轴是真天极并携带章动，但它的第一轴指向平春分点而非真
//! 春分点。
//!
//! 因为第一轴是平春分点，到地固系的变换使用格林尼治平恒星时而非视值。这
//! 正是容易出错的细节：若像对真春分点坐标系那样再减去分点方程式，会把结果
//! 转动约千分之一度，对低轨卫星即一千米的地面位置。
//!
//! 因此时间是 UT1 儒略日，由它得出平恒星时，最后施加极移。
//!
//! 出处：TEME 与地固系之间的关系见 Vallado, D. A.，*Fundamentals of
//! Astrodynamics and Applications*，第 4 版，microcosm Press，2013，第 3.7
//! 节。恒星时表达式即 SGP4 参考实现所用者，见同一著作，由 `eraGmst82`
//! 实现。

use super::terrestrial::PolarMotion;
use crate::apsis_math::Mat3;
use core::f64::consts::PI;

/// The number of seconds in a day.
///
/// 一日的秒数。
const SECONDS_PER_DAY: f64 = 86_400.0;

/// Converts seconds of time to radians.
///
/// A full turn is 86 400 seconds of time, so one second of time is
/// `2 pi / 86400` radians. Sidereal time is quoted in seconds of time, not in
/// arcseconds, and the two differ by a factor of fifteen.
///
/// 把时间秒换算为弧度。
///
/// 一整圈是 86 400 时间秒，故一时间秒是 `2 pi / 86400` 弧度。恒星时以时间
/// 秒而非角秒给出，二者相差十五倍。
const SECONDS_OF_TIME_TO_RADIAN: f64 = 2.0 * PI / SECONDS_PER_DAY;

/// Returns Greenwich mean sidereal time from the 1982 expression, in
/// radians.
///
/// The expression is a polynomial in the number of Julian centuries from
/// J2000.0 plus the fraction of the day in seconds of time. It is the
/// sidereal time the SGP4 theory was fitted against, which is why it is used
/// here rather than the modern expression: the element sets are consistent
/// with it.
///
/// The Julian date is taken in two parts, a whole number of days and a
/// fraction, because a single `f64` near 2 461 000 has a resolution of about
/// 40 microseconds and the sidereal time is wanted to better than that. The
/// caller that has only one value may pass the whole part and the fraction
/// separately, or pass a value twice: `(jd, 0.0)`, `(2400000.5, mjd)`, and
/// `(jd - 0.5, 0.5)` are all accepted, but the split with the smaller whole
/// part preserves more of the fraction.
///
/// Source: as implemented by `eraGmst82`.
///
/// 返回 1982 表达式给出的格林尼治平恒星时，单位为弧度。
///
/// 该表达式是自 J2000.0 起儒略世纪数加上当日以时间秒计的零头所构成的多项
/// 式。它是 SGP4 理论所拟合的恒星时，故此处用它而非现代表达式：根数集与
/// 它一致。
///
/// 儒略日按两部分取，一为整数天数、一为零头，因为接近 2 461 000 的单个
/// `f64` 分辨率约为 40 微秒，而所要求的恒星时优于该值。只有一个值的调用
/// 方可以分别传入整数部分与零头，或把同一个值传两次：`(jd, 0.0)`、
/// `(2400000.5, mjd)` 与 `(jd - 0.5, 0.5)` 都可接受，但整数部分更小的
/// 拆分能保留更多零头。
#[must_use]
pub fn greenwich_mean_sidereal_time_1982(day: f64, fraction: f64) -> f64 {
    // The constant absorbs the half day between the start of the Julian
    // date and the start of the day.
    // 该常数吸收了儒略日起点与当日起点之间的半日。
    let a = 24_110.548_41 - SECONDS_PER_DAY / 2.0;
    let b = 8_640_184.812_866;
    let c = 0.093_104;
    let d = -6.2e-6;

    // The reference orders the two parts so that the smaller is the first,
    // which is the arrangement that preserves the most precision.
    // 参考实现把两部分排序使较小者在前，这是最能保留精度的安排。
    let (first, second) = if day < fraction {
        (day, fraction)
    } else {
        (fraction, day)
    };

    let t = (first + (second - 2_451_545.0)) / 36_525.0;
    let seconds_of_time = SECONDS_PER_DAY * (first.rem_euclid(1.0) + second.rem_euclid(1.0));
    let seconds = (a + (b + (c + d * t) * t) * t) + seconds_of_time;
    (seconds * SECONDS_OF_TIME_TO_RADIAN).rem_euclid(2.0 * PI)
}

/// Returns the rotation from the True Equator Mean Equinox frame to the
/// Earth-fixed frame.
///
/// The argument is the UT1 Julian date, because the rotation is the Earth's
/// rotation, and the polar motion is supplied by the caller because it is
/// measured rather than modelled.
///
/// This is the transformation to apply to the position and velocity that
/// `apsis_sgp4` produces.
///
/// 返回从真赤道平春分点坐标系到地固系的转动。
///
/// 参数是 UT1 儒略日，因为该转动即地球自转；极移由调用方提供，因为它是
/// 实测而非建模的。
///
/// 这是施加于 `apsis_sgp4` 所产生的位置与速度的变换。
#[must_use]
pub fn teme_to_terrestrial_matrix(
    ut1_day: f64,
    ut1_fraction: f64,
    polar_motion: PolarMotion,
) -> Mat3 {
    let sidereal = greenwich_mean_sidereal_time_1982(ut1_day, ut1_fraction);
    // The composition applies the Earth rotation first and the polar motion
    // last, with each new rotation on the left of the accumulator.
    // 该合成先施加地球自转、最后施加极移，每个新转动都在累加器的左侧。
    polar_motion.matrix() * rotation_z(sidereal)
}

/// Returns a rotation of `angle` about the third axis, in radians.
///
/// The convention is the one the reference helpers use: the rotation is
/// applied to the frame, so composing `a * b` applies `b` first.
///
/// 返回绕第三轴转 `angle` 的旋转，单位为弧度。
///
/// 约定即各参考辅助函数所用者：转动施加于坐标系，故合成 `a * b` 时先施加
/// `b`。
fn rotation_z(angle: f64) -> Mat3 {
    super::precession::rotation_z(angle)
}
