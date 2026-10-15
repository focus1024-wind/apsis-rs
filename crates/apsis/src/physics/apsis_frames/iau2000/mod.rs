//! The IAU 2000/2006 Earth orientation models.
//!
//! The transformation from an inertial frame to an Earth-fixed frame is a
//! sequence of rotations, each from a different model:
//!
//! 1. The **frame bias**, a constant rotation from the International
//!    Celestial Reference System to the dynamical mean equator and equinox
//!    of J2000.0.
//! 2. The **precession**, the secular motion of the pole, from the P03
//!    theory adopted in 2006.
//! 3. The **nutation**, the periodic motion of the pole, from the MHB2000
//!    series adopted in 2000.
//! 4. The **Earth rotation angle**, which carries the frame from the
//!    celestial to the terrestrial meridian.
//! 5. The **polar motion**, the small motion of the pole within the Earth,
//!    which a caller supplies because it is published as a daily measured
//!    quantity.
//!
//! The composition of the first three is the celestial-to-intermediate
//! matrix, and the composition with the rotation angle and the polar motion
//! is the celestial-to-terrestrial matrix. For satellite work the polar
//! motion is often omitted, because it is under a tenth of an arcsecond,
//! but the nutation and precession are not: the nutation reaches seventeen
//! arcseconds, which is half a kilometer of ground position for a low Earth
//! satellite.
//!
//! All times are Julian centuries from J2000.0 on the Terrestrial Time
//! scale, unless a function says otherwise. The angles a caller supplies to
//! the polar motion are in radians.
//!
//! Source: the models and the order of the rotations follow the IAU SOFA
//! library and its ERFA derivative. The specific functions are named in each
//! module.
//!
//! IAU 2000/2006 地球定向模型。
//!
//! 从惯性系到地固系的变换是一串转动，每一个都来自不同的模型：
//!
//! 1. **坐标架偏差**，从国际天球参考系到 J2000.0 动力学平赤道与平春分点
//!    的常值转动。
//! 2. **岁差**，极的长期运动，来自 2006 年采纳的 P03 理论。
//! 3. **章动**，极的周期运动，来自 2000 年采纳的 MHB2000 级数。
//! 4. **地球自转角**，把坐标系从天球子午线带到地球子午线。
//! 5. **极移**，极在地球内部的小运动，由调用方提供，因为它作为每日测量的
//!    量发布。
//!
//! 前三者的合成是天球到中间坐标系的矩阵，再与自转角及极移合成则是天球到
//! 地球坐标系的矩阵。对卫星工作，极移常被略去，因为它不到十分之一角秒，
//! 但章动与岁差不能略去：章动达十七角秒，对低轨卫星即半千米的地面位置。
//!
//! 除非某函数另有说明，所有时间都是自 J2000.0 起、在地球时时标上的儒略
//! 世纪。调用方提供给极移的各角以弧度计。
//!
//! 出处：各模型与转动的次序遵循 IAU SOFA 库及其 ERFA 衍生物。各具体函数在
//! 各模块中指名。

mod eect_table;
mod fundamental;
mod nutation;
mod nutation_table;
mod precession;
mod teme;
mod terrestrial;

pub use fundamental::{
    DelaunayArguments, PlanetaryArguments, general_precession, mean_anomaly_moon, mean_anomaly_sun,
    mean_anomaly_sun_iers2003, mean_elongation_moon_sun, mean_elongation_moon_sun_iers2003,
    mean_latitude_moon, mean_longitude_earth, mean_longitude_jupiter, mean_longitude_mars,
    mean_longitude_mercury, mean_longitude_neptune, mean_longitude_node, mean_longitude_saturn,
    mean_longitude_uranus, mean_longitude_venus,
};
pub use nutation::{Nutation, nutation_2000a, nutation_2000a_with_2006_adjustment};
pub use precession::{
    FRAME_BIAS_DECLINATION, FRAME_BIAS_LONGITUDE, FRAME_BIAS_RIGHT_ASCENSION, PrecessionAngles,
    bias_precession_matrix, frame_bias_matrix, frame_to_mean_of_date, mean_obliquity,
    precession_angles, rotation_z,
};
pub use teme::{greenwich_mean_sidereal_time_1982, teme_to_terrestrial_matrix};
pub use terrestrial::{
    PolarMotion, bias_precession_nutation_matrix, celestial_to_terrestrial_matrix,
    earth_rotation_angle, equation_of_the_equinoxes, equation_of_the_equinoxes_complementary_terms,
    greenwich_apparent_sidereal_time, greenwich_mean_sidereal_time,
    intermediate_to_terrestrial_matrix,
};

/// One term of the luni-solar nutation series.
///
/// The five multipliers are applied to the Delaunay arguments in the order
/// `l`, `l'`, `F`, `D`, `Om`. The six coefficients are the longitude sine,
/// its time derivative, the longitude cosine, the obliquity cosine, its time
/// derivative, and the obliquity sine, in units of 0.1 microarcsecond.
///
/// 日月章动级数的一项。
///
/// 五个乘数按 `l`、`l'`、`F`、`D`、`Om` 的次序施加于 Delaunay 参数。六个
/// 系数依次是黄经正弦、其时间导数、黄经余弦、黄赤交角余弦、其时间导数，
/// 以及黄赤交角正弦，单位为 0.1 微角秒。
#[derive(Debug, Clone, Copy)]
pub struct SeriesTerm {
    /// The integer multipliers of the five Delaunay arguments.
    ///
    /// 五个 Delaunay 参数的整数乘数。
    pub argument: [i32; 5],
    /// The six coefficients of the term, in the order the reference lists
    /// them.
    ///
    /// 该项的六个系数，按参考实现列出的次序。
    pub coefficients: [f64; 6],
}

/// One term of the complementary terms of the equation of the equinoxes.
///
/// The eight multipliers apply to the arguments `l`, `l'`, `F`, `D`, `Om`,
/// the mean longitude of Venus, the mean longitude of the Earth, and the
/// general accumulated precession. The two coefficients are the sine and
/// cosine amplitudes, in arcseconds.
///
/// 分点方程式补充项的一项。
///
/// 八个乘数施加于参数 `l`、`l'`、`F`、`D`、`Om`、金星的平黄经、地球的平
/// 黄经，以及一般累积岁差。两个系数是正弦与余弦幅度，单位为角秒。
#[derive(Debug, Clone, Copy)]
pub struct ComplementaryTerm {
    /// The integer multipliers of the eight arguments.
    ///
    /// 八个参数的整数乘数。
    pub argument: [i32; 8],
    /// The sine and cosine coefficients.
    ///
    /// 正弦与余弦系数。
    pub coefficients: [f64; 2],
}

/// One term of the planetary nutation series.
///
/// The thirteen multipliers are applied to the planetary arguments in the
/// order `l`, `F`, `D`, `Om`, Mercury through Uranus, Neptune, and the
/// general accumulated precession. The four coefficients are the longitude
/// sine, the longitude cosine, the obliquity sine, and the obliquity cosine,
/// in units of 0.1 microarcsecond.
///
/// 行星章动级数的一项。
///
/// 十三个乘数按 `l`、`F`、`D`、`Om`、水星到天王星、海王星，以及一般累积
/// 岁差的次序施加于行星参数。四个系数依次是黄经正弦、黄经余弦、黄赤交角
/// 正弦与黄赤交角余弦，单位为 0.1 微角秒。
#[derive(Debug, Clone, Copy)]
pub struct PlanetaryTerm {
    /// The integer multipliers of the thirteen planetary arguments.
    ///
    /// 十三个行星参数的整数乘数。
    pub argument: [i32; 13],
    /// The four coefficients of the term, in the order the reference lists
    /// them.
    ///
    /// 该项的四个系数，按参考实现列出的次序。
    pub coefficients: [f64; 4],
}
