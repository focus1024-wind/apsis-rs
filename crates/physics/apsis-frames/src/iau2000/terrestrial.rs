//! The celestial-to-terrestrial transformation.
//!
//! This module composes the frame bias, precession, nutation, Earth rotation
//! angle, and polar motion into the rotation that carries a vector from the
//! inertial frame to the Earth-fixed frame.
//!
//! Two intermediate frames appear in the composition. The **celestial
//! intermediate frame** is the true equator and equinox of date; the
//! rotation from the inertial frame to it is the bias, precession, and
//! nutation. The **terrestrial intermediate frame** is that frame carried to
//! the Earth's prime meridian by the Earth rotation angle. The polar motion
//! is applied last, because it is the motion of the pole within the Earth
//! rather than of the Earth within space.
//!
//! The elementary rotations follow the convention of the reference: `rz`
//! rotates the frame about its third axis, and composing `a * b` applies `b`
//! first. That convention is what makes the products below read in the order
//! the rotations are physically applied.
//!
//! Source: the composition follows the IAU SOFA library and its ERFA
//! derivative. The bias and precession are `eraPmat06`, the nutation matrix
//! is `eraNumat`, the rotation angle is `eraEra00`, the sidereal time for
//! the equinox-based composition is `eraGmst06`, and the polar motion is
//! `eraPom00`, composed by `eraC2t00b`.

use super::nutation::nutation_2000a_with_2006_adjustment;
use super::precession::{frame_to_mean_of_date, mean_obliquity, rotation_z};
use apsis_math::Mat3;
use core::f64::consts::PI;

/// Converts arcseconds to radians.
///
/// 把角秒转换为弧度。
const ARCSECOND_TO_RADIAN: f64 = PI / (180.0 * 3_600.0);

/// Returns the bias-precession-nutation matrix for a time.
///
/// The matrix carries a vector from the celestial reference frame to the
/// true equator and equinox of date. It is built from the four precession
/// angles with the nutation added to the last two, which is the form the
/// reference uses.
///
/// Source: as implemented by `eraPnm06a` and `eraFw2m`.
///
/// 返回某时刻的偏差-岁差-章动矩阵。
///
/// 该矩阵把矢量从天球参考系带到该日的真赤道与真春分点。它由四个岁差角
/// 构成，其中最后两个角加上章动，这正是参考实现所用的形式。
///
/// 出处：按 `eraPnm06a` 与 `eraFw2m` 所实现。
pub fn bias_precession_nutation_matrix(t: f64) -> Mat3 {
    let angles = super::precession::precession_angles(t);
    let nutation = nutation_2000a_with_2006_adjustment(t);
    frame_to_mean_of_date(&super::precession::PrecessionAngles {
        gamma_bar: angles.gamma_bar,
        phi_bar: angles.phi_bar,
        psi_bar: angles.psi_bar + nutation.longitude,
        epsilon: angles.epsilon + nutation.obliquity,
    })
}

/// Returns the complementary terms of the equation of the equinoxes.
///
/// The equation of the equinoxes has, besides the projection of the nutation
/// in longitude onto the true equator, a set of small periodic terms from
/// the same theory. They reach about twice the constant term of the
/// osculating nutation, which is a millisecond of time, so they matter for a
/// sidereal time that is quoted to that precision.
///
/// Source: the MHB2000 complementary terms, as implemented by `eraEect00`.
///
/// 返回分点方程式的补充项。
///
/// 分点方程式除黄经章动在真赤道上的投影外，还有来自同一理论的一组小
/// 周期项。它们达到密切章动常数项的两倍左右，即一毫秒的时间，故对以该
/// 精度给出的恒星时是重要的。
///
/// 出处：MHB2000 补充项，按 `eraEect00` 所实现。
pub fn equation_of_the_equinoxes_complementary_terms(t: f64) -> f64 {
    use super::eect_table::{CONSTANT_TERMS, LINEAR_TERMS};

    // The complementary terms use the IERS 2003 forms of two arguments,
    // which differ from the ones the nutation series was fitted with.
    // 补充项使用两个参数的 IERS 2003 形式，它们与章动级数所拟合的有所不同。
    let arguments = [
        super::fundamental::mean_anomaly_moon(t),
        super::fundamental::mean_anomaly_sun_iers2003(t),
        super::fundamental::mean_latitude_moon(t),
        super::fundamental::mean_elongation_moon_sun_iers2003(t),
        super::fundamental::mean_longitude_node(t),
        super::fundamental::mean_longitude_venus(t),
        super::fundamental::mean_longitude_earth(t),
        super::fundamental::general_precession(t),
    ];

    let mut constant = 0.0;
    for term in CONSTANT_TERMS.iter() {
        let argument: f64 = term
            .argument
            .iter()
            .zip(arguments.iter())
            .map(|(multiplier, value)| f64::from(*multiplier) * value)
            .sum();
        let (sine, cosine) = argument.sin_cos();
        constant += term.coefficients[0] * sine + term.coefficients[1] * cosine;
    }

    let mut linear = 0.0;
    for term in LINEAR_TERMS.iter() {
        let argument: f64 = term
            .argument
            .iter()
            .zip(arguments.iter())
            .map(|(multiplier, value)| f64::from(*multiplier) * value)
            .sum();
        let (sine, cosine) = argument.sin_cos();
        linear += term.coefficients[0] * sine + term.coefficients[1] * cosine;
    }

    // The tabulated coefficients are in arcseconds.
    // 所列系数以角秒计。
    (constant + linear * t) * ARCSECOND_TO_RADIAN
}

/// Returns the equation of the equinoxes for a time.
///
/// The equation of the equinoxes is the difference between the true and mean
/// equinoxes measured along the true equator. It is the quantity a sidereal
/// time must be corrected by to become the hour angle of the true equinox.
///
/// The value is the projection of the nutation in longitude onto the true
/// equator, `dpsi cos(eps)`, plus the complementary terms of the theory.
///
/// 返回某时刻的分点方程式。
///
/// 分点方程式是沿真赤道量得的真春分点与平春分点之差。它是恒星时须用以
/// 改正才能成为真春分点时角的量。
///
/// 该值是黄经章动在真赤道上的投影 `dpsi cos(eps)`，加上该理论的补充项。
pub fn equation_of_the_equinoxes(t: f64) -> f64 {
    let nutation = nutation_2000a_with_2006_adjustment(t);
    let obliquity = mean_obliquity(t);
    nutation.longitude * obliquity.cos() + equation_of_the_equinoxes_complementary_terms(t)
}

/// The polar motion of the Earth, as two angles and the rotation.
///
/// The polar motion is the motion of the rotation pole within the Earth. It
/// is measured daily and published by the International Earth Rotation and
/// Reference Systems Service, so it cannot be computed from a model and is
/// supplied by the caller.
///
/// 地球的极移，以两个角与转动表示。
///
/// 极移是自转极在地球内部的运动。它由国际地球自转与参考系服务每日测量
/// 并发布，故无法由模型算出，而由调用方提供。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PolarMotion {
    /// The displacement of the pole along the Greenwich meridian, in
    /// radians.
    ///
    /// 极点沿格林尼治子午线的位移，单位为弧度。
    pub x: f64,
    /// The displacement of the pole along the meridian ninety degrees west,
    /// in radians.
    ///
    /// 极点沿西经九十度子午线的位移，单位为弧度。
    pub y: f64,
    /// The rotation of the terrestrial frame about the pole, in radians.
    ///
    /// This is the terrestrial intermediate origin locator, which is small
    /// and is often taken as zero.
    ///
    /// 地球坐标系绕极点的转动，单位为弧度。
    ///
    /// 这是地球中间原点定位角，它很小，常取为零。
    pub s: f64,
}

impl PolarMotion {
    /// The zero polar motion, which is the approximation used when the
    /// published values are not available.
    ///
    /// The error from omitting the polar motion is under a tenth of an
    /// arcsecond, well below the nutation but above the precision of a
    /// modern transformation.
    ///
    /// 零极移，即无法获得已发布值时所采用的近似。
    ///
    /// 略去极移的误差不到十分之一角秒，远低于章动，但高于现代转换的
    /// 精度。
    pub const NONE: Self = Self {
        x: 0.0,
        y: 0.0,
        s: 0.0,
    };

    /// Creates a polar motion from its three angles.
    ///
    /// 由其三个角创建极移。
    pub const fn new(x: f64, y: f64, s: f64) -> Self {
        Self { x, y, s }
    }

    /// Returns `true` when the angles are usable.
    ///
    /// 当各角可用时返回 `true`。
    pub fn is_valid(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.s.is_finite()
    }

    /// Returns the rotation matrix of the polar motion.
    ///
    /// The rotation is a turn about the third axis by the locator, then
    /// about the second by minus the `x` displacement, then about the first
    /// by minus the `y` displacement. The signs are the ones the reference
    /// uses, which carry the terrestrial frame into alignment with the
    /// celestial pole.
    ///
    /// Source: as implemented by `eraPom00`, which applies the rotations in
    /// sequence by composing each on the left of the accumulator.
    ///
    /// 返回极移的旋转矩阵。
    ///
    /// 该转动是绕第三轴转定位角，再绕第二轴转负的 `x` 位移，再绕第一轴
    /// 转负的 `y` 位移。符号是参考实现所用者，它们把地球坐标系带到与
    /// 天极对齐。
    ///
    /// 出处：按 `eraPom00` 所实现，它依次施加各转动，每次都把新转动合成
    /// 到累加器的左侧。
    pub fn matrix(self) -> Mat3 {
        rotation_x(-self.y) * rotation_y(-self.x) * rotation_z(self.s)
    }
}

/// Returns the intermediate-to-terrestrial matrix for a time.
///
/// The intermediate frame is the true equator and equinox of date, and the
/// terrestrial frame is the Earth-fixed frame at the same date. This uses
/// the equinox-based approach, in which the rotation from one to the other
/// is Greenwich apparent sidereal time.
///
/// The time arguments are the UT1 and TT Julian dates.
///
/// Source: the composition is the one `eraC2t06a` produces for the equinox
/// based chain, with the polar motion applied separately by the caller of
/// the full transformation.
///
/// 返回某时刻的中间坐标系到地球坐标系的矩阵。
///
/// 中间坐标系是该日的真赤道与真春分点，地球坐标系是同一日的地固系。这里
/// 用基于春分点的方法，其中从一个到另一个的转动是格林尼治视恒星时。
///
/// 时间参数为 UT1 与 TT 儒略日。
///
/// 出处：该合成即 `eraC2t06a` 对基于春分点的链条所产生者，极移由完整
/// 变换的调用方单独施加。
pub fn intermediate_to_terrestrial_matrix(ut1_julian_date: f64, tt_julian_date: f64) -> Mat3 {
    let apparent_sidereal = greenwich_apparent_sidereal_time(ut1_julian_date, tt_julian_date);
    rotation_z(apparent_sidereal)
}

/// Returns Greenwich apparent sidereal time for a UT1 and a TT Julian date.
///
/// This is the mean sidereal time plus the equation of the equinoxes, so it
/// is the hour angle of the true equinox.
///
/// Source: as implemented by `eraGst06a`.
///
/// 返回某 UT1 与 TT 儒略日的格林尼治视恒星时。
///
/// 它是平恒星时加分点方程式，故即真春分点的时角。
///
/// 出处：按 `eraGst06a` 所实现。
pub fn greenwich_apparent_sidereal_time(ut1_julian_date: f64, tt_julian_date: f64) -> f64 {
    let t = (tt_julian_date - 2_451_545.0) / 36_525.0;
    let mean = greenwich_mean_sidereal_time(ut1_julian_date, tt_julian_date);
    (mean + equation_of_the_equinoxes(t)).rem_euclid(2.0 * PI)
}

/// Returns Greenwich mean sidereal time for a UT1 and a TT Julian date.
///
/// The relation is the Earth rotation angle plus the accumulated precession
/// in right ascension, which is the quantity that relates the equinox to the
/// non-rotating origin.
///
/// Source: as implemented by `eraGmst06`.
///
/// 返回某 UT1 与 TT 儒略日的格林尼治平恒星时。
///
/// 关系为地球自转角加上赤经上的累积岁差，后者是联系春分点与非旋转原点
/// 的量。
///
/// 出处：按 `eraGmst06` 所实现。
pub fn greenwich_mean_sidereal_time(ut1_julian_date: f64, tt_julian_date: f64) -> f64 {
    let t = (tt_julian_date - 2_451_545.0) / 36_525.0;
    // The accumulated precession in right ascension, in arcseconds.
    // 赤经上的累积岁差，单位为角秒。
    let arcseconds = 0.014_506
        + t * (4_612.156_534
            + t * (1.391_581_7
                + t * (-0.000_000_44 + t * (-0.000_029_956 + t * (-0.000_000_036_8)))));
    (earth_rotation_angle(ut1_julian_date) + arcseconds * ARCSECOND_TO_RADIAN).rem_euclid(2.0 * PI)
}

/// Returns the Earth rotation angle for a UT1 Julian date.
///
/// This is the linear expression of the International Astronomical Union,
/// which is exact to the precision the angle is known.
///
/// Source: IERS Conventions (2010), equation 8.32.
///
/// 返回某 UT1 儒略日的地球自转角。
///
/// 这是国际天文学联合会的线性表达式，它在该角已知的精度内是精确的。
///
/// 出处：IERS Conventions（2010），式 8.32。
pub fn earth_rotation_angle(ut1_julian_date: f64) -> f64 {
    let days = ut1_julian_date - 2_451_545.0;
    let turns = 0.779_057_273_264_0 + 1.002_737_811_911_354_6 * days;
    (turns.rem_euclid(1.0)) * 2.0 * PI
}

/// Returns the celestial-to-terrestrial matrix for a time.
///
/// This composes the whole chain: the frame bias, the precession, the
/// nutation, the Earth rotation, and the polar motion.
///
/// The time arguments are the UT1 and TT Julian dates, and the polar motion
/// is supplied by the caller because it is measured rather than modelled.
///
/// Source: the composition is the one `eraC2t06a` produces.
///
/// 返回某时刻的天球到地球坐标系矩阵。
///
/// 它合成整条链：坐标架偏差、岁差、章动、地球自转与极移。
///
/// 时间参数为 UT1 与 TT 儒略日，极移由调用方提供，因为它是实测而非建模
/// 的。
///
/// 出处：该合成即 `eraC2t06a` 所产生者。
pub fn celestial_to_terrestrial_matrix(
    ut1_julian_date: f64,
    tt_julian_date: f64,
    polar_motion: PolarMotion,
) -> Mat3 {
    let t = (tt_julian_date - 2_451_545.0) / 36_525.0;
    let bias_precession_nutation = bias_precession_nutation_matrix(t);
    let sidereal = greenwich_apparent_sidereal_time(ut1_julian_date, tt_julian_date);
    // The composition is the one `eraC2tcio` produces: the bias, precession,
    // and nutation first, then the Earth rotation, then the polar motion,
    // with each new rotation composed on the left of the accumulator.
    // 该合成即 `eraC2tcio` 所产生者：先是偏差、岁差与章动，然后是地球
    // 自转，再是极移，每个新转动都合成到累加器的左侧。
    polar_motion.matrix() * rotation_z(sidereal) * bias_precession_nutation
}

/// Returns a rotation of `angle` about the first axis, in radians.
///
/// The convention matches the reference helper `eraRx`: the rotation is
/// applied to the frame, so the matrix is the transpose of the one that
/// would rotate a vector by the same angle.
///
/// 返回绕第一轴转 `angle` 的旋转，单位为弧度。
///
/// 该约定与参考辅助函数 `eraRx` 一致：转动施加于坐标系，故该矩阵是把
/// 矢量转同一角度的矩阵的转置。
fn rotation_x(angle: f64) -> Mat3 {
    let (sine, cosine) = angle.sin_cos();
    Mat3::from_rows([1.0, 0.0, 0.0], [0.0, cosine, sine], [0.0, -sine, cosine])
}

/// Returns a rotation of `angle` about the second axis, in radians.
///
/// The convention is the one of [`rotation_x`].
///
/// 返回绕第二轴转 `angle` 的旋转，单位为弧度。
///
/// 约定与 [`rotation_x`] 相同。
fn rotation_y(angle: f64) -> Mat3 {
    let (sine, cosine) = angle.sin_cos();
    Mat3::from_rows([cosine, 0.0, -sine], [0.0, 1.0, 0.0], [sine, 0.0, cosine])
}
