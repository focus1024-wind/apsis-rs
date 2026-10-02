//! The IAU 2006 precession model and the frame bias.
//!
//! Precession is the slow, secular motion of the Earth's rotation axis about
//! the pole of the ecliptic, caused mainly by the torque of the Moon and the
//! Sun on the equatorial bulge. The IAU 2006 model expresses it as a
//! rotation from the celestial reference frame to the mean equator and
//! equinox of date, built from four angles.
//!
//! The frame bias accounts for the small offset between the orientation of
//! the International Celestial Reference System and the dynamical mean
//! equator and equinox of J2000.0. It is a constant rotation, and omitting
//! it places a position about twenty milliarcseconds away, which is well
//! above the precision of a modern transformation.
//!
//! Source: the P03 precession theory of Capitaine, Wallace and Chapront,
//! adopted by the International Astronomical Union in 2006, as implemented
//! in the IAU SOFA library and its ERFA derivative, functions `eraPfw06`,
//! `eraPmat06`, `eraFw2m`, `eraBi00` and `eraObl06`.
//!
//! IAU 2006 岁差模型与坐标架偏差。
//!
//! 岁差是地球自转轴绕黄极缓慢的长期运动，主要由月球与太阳对赤道隆起的
//! 力矩造成。IAU 2006 模型把它表达为从天球参考系到该日平赤道与平春分点的
//! 转动，由四个角构成。
//!
//! 坐标架偏差计入了国际天球参考系的取向与 J2000.0 动力学平赤道与平春分点
//! 之间的小偏移。它是常值转动，略去它会把位置放偏约二十毫角秒，这远高于
//! 现代转换的精度。
//!
//! 出处：Capitaine、Wallace 与 Chapront 的 P03 岁差理论，2006 年由国际
//! 天文学联合会采纳，如 IAU SOFA 库及其 ERFA 衍生物所实现，函数
//! `eraPfw06`、`eraPmat06`、`eraFw2m`、`eraBi00` 与 `eraObl06`。

use apsis_math::Mat3;
use core::f64::consts::PI;

/// Converts arcseconds to radians.
///
/// 把角秒转换为弧度。
const ARCSECOND_TO_RADIAN: f64 = PI / (180.0 * 3_600.0);

/// The frame bias in right ascension, in radians.
///
/// The offset between the origin of the International Celestial Reference
/// System and the dynamical equinox of J2000.0. The value is
/// minus 0.0146 arcseconds.
///
/// 赤经方向的坐标架偏差，单位为弧度。
///
/// 国际天球参考系原点与 J2000.0 动力学春分点之间的偏移。该值为负
/// 0.0146 角秒。
pub const FRAME_BIAS_RIGHT_ASCENSION: f64 = -0.014_6 * ARCSECOND_TO_RADIAN;

/// The frame bias in declination, in radians.
///
/// The offset between the pole of the International Celestial Reference
/// System and the mean pole of J2000.0. The value is minus 0.0068192
/// arcseconds.
///
/// 赤纬方向的坐标架偏差，单位为弧度。
///
/// 国际天球参考系极与 J2000.0 平极之间的偏移。该值为负 0.0068192
/// 角秒。
pub const FRAME_BIAS_DECLINATION: f64 = -0.006_819_2 * ARCSECOND_TO_RADIAN;

/// The frame bias in longitude, in radians.
///
/// The offset between the two origins measured along the equator. The value
/// is minus 0.041775 arcseconds.
///
/// 黄经方向的坐标架偏差，单位为弧度。
///
/// 沿赤道量得的两个原点之间的偏移。该值为负 0.041775 角秒。
pub const FRAME_BIAS_LONGITUDE: f64 = -0.041_775 * ARCSECOND_TO_RADIAN;

/// The four angles that describe the precession from J2000.0 to a date.
///
/// They are applied in the order the reference uses: a rotation by
/// `gamma_bar` about the third axis, then by `phi_bar` about the first, then
/// by minus `psi_bar` about the third, then by minus `epsilon_a` about the
/// first.
///
/// 描述从 J2000.0 到某日岁差的四个角。
///
/// 它们按参考实现所用的次序施加：绕第三轴转 `gamma_bar`，再绕第一轴转
/// `phi_bar`，再绕第三轴转负的 `psi_bar`，再绕第一轴转负的
/// `epsilon_a`。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrecessionAngles {
    /// The precession of the equator along the ecliptic, in radians.
    ///
    /// 赤道沿黄道的岁差，单位为弧度。
    pub gamma_bar: f64,
    /// The inclination of the moving equator to the ecliptic, in radians.
    ///
    /// 运动赤道相对黄道的倾角，单位为弧度。
    pub phi_bar: f64,
    /// The accumulated precession in longitude, in radians.
    ///
    /// 黄经上的累积岁差，单位为弧度。
    pub psi_bar: f64,
    /// The mean obliquity at the date, in radians.
    ///
    /// 该日的平黄赤交角，单位为弧度。
    pub epsilon: f64,
}

/// Returns the mean obliquity of the ecliptic, in radians.
///
/// The polynomial is the IAU 2006 expression, evaluated for a time in Julian
/// centuries from J2000.0 on the Terrestrial Time scale. The constant term
/// is 84381.406 arcseconds, which is the obliquity at J2000.0.
///
/// Source: the IAU 2006 precession, as implemented by `eraObl06`.
///
/// 返回黄道的平黄赤交角，单位为弧度。
///
/// 该多项式是 IAU 2006 表达式，按地球时标、自 J2000.0 起的儒略世纪求值。
/// 常数项为 84381.406 角秒，即 J2000.0 处的黄赤交角。
///
/// 出处：IAU 2006 岁差，按 `eraObl06` 所实现。
#[must_use]
pub fn mean_obliquity(t: f64) -> f64 {
    let arcseconds = 84_381.406
        + t * (-46.836_769
            + t * (-0.000_183_1
                + t * (0.002_003_40 + t * (-0.000_000_576 + t * (-0.000_000_043_4)))));
    arcseconds * ARCSECOND_TO_RADIAN
}

/// Returns the four precession angles for a time.
///
/// The time is in Julian centuries from J2000.0 on the Terrestrial Time
/// scale.
///
/// Source: the P03 precession, as implemented by `eraPfw06`.
///
/// 返回某时刻的四个岁差角。
///
/// 时间以 J2000.0 起的儒略世纪计，位于地球时（TT）标度。
///
/// 出处：P03 岁差，按 `eraPfw06` 所实现。
#[must_use]
pub fn precession_angles(t: f64) -> PrecessionAngles {
    let gamma_bar = -0.052_928
        + t * (10.556_378
            + t * (0.493_204_4
                + t * (-0.000_312_38 + t * (-0.000_002_788 + t * (0.000_000_026_0)))));
    let phi_bar = 84_381.412_819
        + t * (-46.811_016
            + t * (0.051_126_8
                + t * (0.000_532_89 + t * (-0.000_000_440 + t * (-0.000_000_017_6)))));
    let psi_bar = -0.041_775
        + t * (5_038.481_484
            + t * (1.558_417_5
                + t * (-0.000_185_22 + t * (-0.000_026_452 + t * (-0.000_000_014_8)))));

    PrecessionAngles {
        gamma_bar: gamma_bar * ARCSECOND_TO_RADIAN,
        phi_bar: phi_bar * ARCSECOND_TO_RADIAN,
        psi_bar: psi_bar * ARCSECOND_TO_RADIAN,
        epsilon: mean_obliquity(t),
    }
}

/// Returns the frame bias rotation matrix.
///
/// The bias is a constant rotation from the International Celestial
/// Reference System to the dynamical mean equator and equinox of J2000.0.
/// Its three angles are far below a tenth of an arcsecond, but omitting it
/// would leave a systematic error that is larger than the precision of the
/// nutation series.
///
/// Source: as implemented by `eraBi00`, which gives the bias as three small
/// rotations rather than as a matrix.
///
/// 返回坐标架偏差旋转矩阵。
///
/// 该偏差是从国际天球参考系到 J2000.0 动力学平赤道与平春分点的常值
/// 转动。其三个角远小于十分之一角秒，但略去它会留下大于章动级数精度
/// 的系统误差。
///
/// 出处：按 `eraBi00` 所实现，它把偏差给出为三个小转动而非一个矩阵。
#[must_use]
pub fn frame_bias_matrix() -> Mat3 {
    // The reference builds the bias from three elementary rotations, and
    // the result is reproduced here in the same order.
    // 参考实现由三个基本转动构建该偏差，此处按同一次序再现结果。
    rotation_z(FRAME_BIAS_LONGITUDE)
        * rotation_x(FRAME_BIAS_DECLINATION)
        * rotation_y(FRAME_BIAS_RIGHT_ASCENSION)
}

/// Returns the rotation from the celestial reference frame to the mean
/// equator and equinox of date.
///
/// This is the bias-precession matrix. Nutation is applied on top of it to
/// reach the true equator and equinox of date.
///
/// The time is in Julian centuries from J2000.0 on the Terrestrial Time
/// scale.
///
/// Source: as implemented by `eraPmat06`.
///
/// 返回从天球参考系到该日平赤道与平春分点的转动。
///
/// 这是偏差-岁差矩阵。章动施加于其上，以到达该日的真赤道与真春分点。
///
/// 时间以 J2000.0 起的儒略世纪计，位于地球时（TT）标度。
///
/// 出处：按 `eraPmat06` 所实现。
#[must_use]
pub fn bias_precession_matrix(t: f64) -> Mat3 {
    let angles = precession_angles(t);
    frame_to_mean_of_date(&angles)
}

/// Returns the rotation from the celestial reference frame to the mean
/// equator and equinox of date, given precomputed precession angles.
///
/// Source: as implemented by `eraFw2m`, which applies the four elementary
/// rotations in sequence. The reference helper composes each new rotation
/// on the left of the accumulator, so the product here is written with the
/// last rotation applied first, in the same order.
///
/// 在给定预先算出的岁差角时，返回从天球参考系到该日平赤道与平春分点的
/// 转动。
///
/// 出处：按 `eraFw2m` 所实现，它依次施加四个基本转动。参考辅助函数把
/// 每个新转动合成到累加器的左侧，故此处的乘积按最后施加的转动写在最前，
/// 次序一致。
#[must_use]
pub fn frame_to_mean_of_date(angles: &PrecessionAngles) -> Mat3 {
    rotation_x(-angles.epsilon)
        * rotation_z(-angles.psi_bar)
        * rotation_x(angles.phi_bar)
        * rotation_z(angles.gamma_bar)
}

/// Returns a rotation of `angle` about the first axis, in radians.
///
/// The convention matches the reference helpers `eraRx` and `eraRz`: the
/// rotation is applied to the frame rather than to a vector, so the matrix
/// is the transpose of the one that would rotate a vector by the same angle.
/// Composing `a * b` therefore applies `b` first, and the products in this
/// module read in the order the rotations are physically applied.
///
/// 返回绕第一轴转 `angle` 的旋转，单位为弧度。
///
/// 该约定与参考辅助函数 `eraRx` 与 `eraRz` 一致：转动施加于坐标系而非
/// 矢量，故该矩阵是把矢量转同一角度的矩阵的转置。因此合成 `a * b` 时先
/// 施加 `b`，本模块中的各乘积按各转动实际施加的次序阅读。
pub(crate) fn rotation_x(angle: f64) -> Mat3 {
    let (sine, cosine) = angle.sin_cos();
    Mat3::from_rows([1.0, 0.0, 0.0], [0.0, cosine, sine], [0.0, -sine, cosine])
}

/// Returns a rotation of `angle` about the second axis, in radians.
///
/// The sign convention matches the other axis rotations in this module.
///
/// 返回绕第二轴转 `angle` 的旋转，单位为弧度。
///
/// 符号约定与本模块中其余各轴转动一致。
fn rotation_y(angle: f64) -> Mat3 {
    let (sine, cosine) = angle.sin_cos();
    Mat3::from_rows([cosine, 0.0, -sine], [0.0, 1.0, 0.0], [sine, 0.0, cosine])
}

/// Returns a rotation of `angle` about the third axis, in radians.
///
/// The sign convention matches the other axis rotations in this module.
///
/// 返回绕第三轴转 `angle` 的旋转，单位为弧度。
///
/// 符号约定与本模块中其余各轴转动一致。
#[must_use]
pub fn rotation_z(angle: f64) -> Mat3 {
    let (sine, cosine) = angle.sin_cos();
    Mat3::from_rows([cosine, sine, 0.0], [-sine, cosine, 0.0], [0.0, 0.0, 1.0])
}
