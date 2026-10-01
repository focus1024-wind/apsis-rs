//! The fundamental arguments of the IAU 2000 nutation theory.
//!
//! The nutation series is a sum of periodic terms whose arguments are linear
//! functions of time. Each term is identified by integer multipliers applied
//! to these arguments, so the arguments must be evaluated in a fixed order.
//!
//! Two argument sets appear in the theory and they are not the same, which
//! is a detail that is easy to get wrong:
//!
//! - The **luni-solar** series uses the five Delaunay arguments in the order
//!   `l`, `l'`, `F`, `D`, `Om`: the mean anomaly of the Moon, the mean
//!   anomaly of the Sun, the mean argument of latitude of the Moon, the mean
//!   elongation of the Moon from the Sun, and the mean longitude of the
//!   Moon's ascending node.
//! - The **planetary** series uses `l`, `F`, `D`, `Om`, the mean longitudes
//!   of Mercury through Uranus, the mean longitude of Neptune, and the
//!   general accumulated precession in longitude.
//!
//! The reference evaluates `l'` and `D` with inline expressions while using
//! functions for the others, and the inline expressions differ in the last
//! digits from the functions of the same name in the older IERS 2003
//! conventions. The values here are the ones the series was fitted with, so
//! the inline forms are reproduced.
//!
//! All arguments are in radians and are reduced into `[0, 2 pi)`.
//!
//! Source: the argument definitions of the MHB2000 model as implemented in
//! `eraNut00a`, with the planetary longitudes from `eraFame03`,
//! `eraFave03`, `eraFae03`, `eraFama03`, `eraFaju03`, `eraFasa03` and
//! `eraFaur03`.

use core::f64::consts::PI;

/// Converts arcseconds to radians.
///
/// 把角秒转换为弧度。
const ARCSECOND_TO_RADIAN: f64 = PI / (180.0 * 3_600.0);

/// The number of arcs in a full turn, in arcseconds.
///
/// 一整周的角秒数。
const TURN_ARCSECONDS: f64 = 1_296_000.0;

/// Returns a value reduced into `[0, 2 pi)`.
///
/// 返回归约到 `[0, 2 pi)` 内的值。
fn anp(angle: f64) -> f64 {
    angle.rem_euclid(2.0 * PI)
}

/// Returns the mean anomaly of the Moon, in radians.
///
/// This is the Delaunay argument `l`, from the IERS 2003 expressions.
///
/// 返回月球的平近点角，单位为弧度。
///
/// 这是 Delaunay 参数 `l`，取自 IERS 2003 表达式。
pub fn mean_anomaly_moon(t: f64) -> f64 {
    let arcseconds = 485_868.249_036
        + t * (1_717_915_923.217_8 + t * (31.879_2 + t * (0.051_635 + t * (-0.000_244_70))));
    anp((arcseconds % TURN_ARCSECONDS) * ARCSECOND_TO_RADIAN)
}

/// Returns the mean anomaly of the Sun, in radians.
///
/// This is the Delaunay argument `l'`. The reference evaluates it inline
/// rather than through the general function, and the two differ slightly.
///
/// 返回太阳的平近点角，单位为弧度。
///
/// 这是 Delaunay 参数 `l'`。参考实现是内联求值而非经通用函数，二者略有
/// 差异。
pub fn mean_anomaly_sun(t: f64) -> f64 {
    let arcseconds = 1_287_104.793_05
        + t * (129_596_581.048_1 + t * (-0.553_2 + t * (0.000_136 + t * (-0.000_011_49))));
    anp((arcseconds % TURN_ARCSECONDS) * ARCSECOND_TO_RADIAN)
}

/// Returns the mean anomaly of the Sun from the IERS 2003 expressions.
///
/// This differs from [`mean_anomaly_sun`] in the last digits of the constant
/// and linear terms. The nutation series uses the inline form; the
/// complementary terms of the equation of the equinoxes use this one.
///
/// Source: as implemented by `eraFalp03`.
///
/// 返回取自 IERS 2003 表达式的太阳平近点角。
///
/// 它与 [`mean_anomaly_sun`] 在常数项与线性项的最后几位上不同。章动级数
/// 用内联形式；分点方程式的补充项用这一个。
///
/// 出处：按 `eraFalp03` 所实现。
pub fn mean_anomaly_sun_iers2003(t: f64) -> f64 {
    let arcseconds = 1_287_104.793_048
        + t * (129_596_581.048_1 + t * (-0.553_2 + t * (0.000_136 + t * (-0.000_011_49))));
    anp((arcseconds % TURN_ARCSECONDS) * ARCSECOND_TO_RADIAN)
}

/// Returns the mean elongation of the Moon from the Sun from the IERS 2003
/// expressions.
///
/// This differs from [`mean_elongation_moon_sun`] in the last digits of the
/// constant term. The nutation series uses the inline form; the
/// complementary terms of the equation of the equinoxes use this one.
///
/// Source: as implemented by `eraFad03`.
///
/// 返回取自 IERS 2003 表达式的月球相对太阳的平距角。
///
/// 它与 [`mean_elongation_moon_sun`] 在常数项的最后几位上不同。章动级数
/// 用内联形式；分点方程式的补充项用这一个。
///
/// 出处：按 `eraFad03` 所实现。
pub fn mean_elongation_moon_sun_iers2003(t: f64) -> f64 {
    let arcseconds = 1_072_260.703_692
        + t * (1_602_961_601.209_0 + t * (-6.370_6 + t * (0.006_593 + t * (-0.000_031_69))));
    anp((arcseconds % TURN_ARCSECONDS) * ARCSECOND_TO_RADIAN)
}

/// Returns the mean argument of latitude of the Moon, in radians.
///
/// This is the Delaunay argument `F`, the mean longitude of the Moon minus
/// the mean longitude of its ascending node.
///
/// 返回月球的平纬度参数，单位为弧度。
///
/// 这是 Delaunay 参数 `F`，即月球平黄经减去其升交点平黄经。
pub fn mean_latitude_moon(t: f64) -> f64 {
    let arcseconds = 335_779.526_232
        + t * (1_739_527_262.847_8 + t * (-12.751_2 + t * (-0.001_037 + t * (0.000_004_17))));
    anp((arcseconds % TURN_ARCSECONDS) * ARCSECOND_TO_RADIAN)
}

/// Returns the mean elongation of the Moon from the Sun, in radians.
///
/// This is the Delaunay argument `D`. The reference evaluates it inline.
///
/// 返回月球相对太阳的平距角，单位为弧度。
///
/// 这是 Delaunay 参数 `D`。参考实现是内联求值。
pub fn mean_elongation_moon_sun(t: f64) -> f64 {
    let arcseconds = 1_072_260.703_69
        + t * (1_602_961_601.209_0 + t * (-6.370_6 + t * (0.006_593 + t * (-0.000_031_69))));
    anp((arcseconds % TURN_ARCSECONDS) * ARCSECOND_TO_RADIAN)
}

/// Returns the mean longitude of the Moon's ascending node, in radians.
///
/// This is the Delaunay argument `Om`.
///
/// 返回月球升交点的平黄经，单位为弧度。
///
/// 这是 Delaunay 参数 `Om`。
pub fn mean_longitude_node(t: f64) -> f64 {
    let arcseconds = 450_160.398_036
        + t * (-6_962_890.543_1 + t * (7.472_2 + t * (0.007_702 + t * (-0.000_059_39))));
    anp((arcseconds % TURN_ARCSECONDS) * ARCSECOND_TO_RADIAN)
}

/// Returns the mean longitude of Mercury, in radians.
///
/// 返回水星的平黄经，单位为弧度。
pub fn mean_longitude_mercury(t: f64) -> f64 {
    anp(4.402_608_842 + 2_608.790_314_157_4 * t)
}

/// Returns the mean longitude of Venus, in radians.
///
/// 返回金星的平黄经，单位为弧度。
pub fn mean_longitude_venus(t: f64) -> f64 {
    anp(3.176_146_697 + 1_021.328_554_621_1 * t)
}

/// Returns the mean longitude of the Earth, in radians.
///
/// 返回地球的平黄经，单位为弧度。
pub fn mean_longitude_earth(t: f64) -> f64 {
    anp(1.753_470_314 + 628.307_584_999_1 * t)
}

/// Returns the mean longitude of Mars, in radians.
///
/// 返回火星的平黄经，单位为弧度。
pub fn mean_longitude_mars(t: f64) -> f64 {
    anp(6.203_480_913 + 334.061_242_670_0 * t)
}

/// Returns the mean longitude of Jupiter, in radians.
///
/// 返回木星的平黄经，单位为弧度。
pub fn mean_longitude_jupiter(t: f64) -> f64 {
    anp(0.599_546_497 + 52.969_096_264_1 * t)
}

/// Returns the mean longitude of Saturn, in radians.
///
/// 返回土星的平黄经，单位为弧度。
pub fn mean_longitude_saturn(t: f64) -> f64 {
    anp(0.874_016_757 + 21.329_910_496_0 * t)
}

/// Returns the mean longitude of Uranus, in radians.
///
/// 返回天王星的平黄经，单位为弧度。
pub fn mean_longitude_uranus(t: f64) -> f64 {
    anp(5.481_293_872 + 7.478_159_856_7 * t)
}

/// Returns the mean longitude of Neptune, in radians.
///
/// The reference evaluates this inline, from the MHB2000 model rather than
/// from the IERS 2003 expressions used for the other planets.
///
/// 返回海王星的平黄经，单位为弧度。
///
/// 参考实现是内联求值，取自 MHB2000 模型，而非用于其余行星的 IERS 2003
/// 表达式。
pub fn mean_longitude_neptune(t: f64) -> f64 {
    anp(5.321_159_000 + 3.812_777_400_0 * t)
}

/// Returns the general accumulated precession in longitude, in radians.
///
/// 返回黄经上的一般累积岁差，单位为弧度。
pub fn general_precession(t: f64) -> f64 {
    (0.024_381_750 + 0.000_005_386_91 * t) * t
}

/// The five Delaunay arguments, in the order the luni-solar series uses.
///
/// 五个 Delaunay 参数，按日月级数所用的次序排列。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DelaunayArguments {
    /// The mean anomaly of the Moon.
    ///
    /// 月球的平近点角。
    ///
    /// 月球的平近点角。
    pub l: f64,
    /// The mean anomaly of the Sun.
    ///
    /// 太阳的平近点角。
    pub l_prime: f64,
    /// The mean argument of latitude of the Moon.
    ///
    /// 月球的平纬度参数。
    ///
    /// 月球的平纬度参数。
    pub f: f64,
    /// The mean elongation of the Moon from the Sun.
    ///
    /// 月球相对太阳的平距角。
    ///
    /// 月球相对太阳的平距角。
    pub d: f64,
    /// The mean longitude of the Moon's ascending node.
    ///
    /// 月球升交点的平黄经。
    ///
    /// 月球升交点的平黄经。
    pub omega: f64,
}

impl DelaunayArguments {
    /// Returns the five arguments for a time in Julian centuries.
    ///
    /// 返回某儒略世纪时刻的这五个参数。
    pub fn at(t: f64) -> Self {
        Self {
            l: mean_anomaly_moon(t),
            l_prime: mean_anomaly_sun(t),
            f: mean_latitude_moon(t),
            d: mean_elongation_moon_sun(t),
            omega: mean_longitude_node(t),
        }
    }

    /// Returns the arguments as an array in series order.
    ///
    /// 按级数次序把各参数作为数组返回。
    ///
    /// 按级数次序把各参数作为数组返回。
    pub const fn as_array(self) -> [f64; 5] {
        [self.l, self.l_prime, self.f, self.d, self.omega]
    }
}

/// The thirteen arguments of the planetary series, in series order.
///
/// The order is `l`, `F`, `D`, `Om`, Mercury through Uranus, Neptune, and the
/// general accumulated precession. Note that `l'` does not appear; the
/// planetary series uses the Moon's mean anomaly, not the Sun's.
///
/// 行星级数的十三个参数，按级数次序排列。
///
/// 次序为 `l`、`F`、`D`、`Om`、水星到天王星、海王星，以及一般累积岁差。
/// 注意 `l'` 不出现；行星级数用的是月球的平近点角，而非太阳的。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlanetaryArguments {
    /// The mean anomaly of the Moon.
    ///
    /// 月球的平近点角。
    pub l: f64,
    /// The mean argument of latitude of the Moon.
    ///
    /// 月球的平纬度参数。
    pub f: f64,
    /// The mean elongation of the Moon from the Sun.
    ///
    /// 月球相对太阳的平距角。
    pub d: f64,
    /// The mean longitude of the Moon's ascending node.
    ///
    /// 月球升交点的平黄经。
    pub omega: f64,
    /// The mean longitude of Mercury.
    ///
    /// 水星的平黄经。
    pub mercury: f64,
    /// The mean longitude of Venus.
    ///
    /// 金星的平黄经。
    pub venus: f64,
    /// The mean longitude of the Earth.
    ///
    /// 地球的平黄经。
    pub earth: f64,
    /// The mean longitude of Mars.
    ///
    /// 火星的平黄经。
    pub mars: f64,
    /// The mean longitude of Jupiter.
    ///
    /// 木星的平黄经。
    pub jupiter: f64,
    /// The mean longitude of Saturn.
    ///
    /// 土星的平黄经。
    pub saturn: f64,
    /// The mean longitude of Uranus.
    ///
    /// 天王星的平黄经。
    pub uranus: f64,
    /// The mean longitude of Neptune.
    ///
    /// 海王星的平黄经。
    pub neptune: f64,
    /// The general accumulated precession in longitude.
    ///
    /// 黄经上的一般累积岁差。
    pub precession: f64,
}

impl PlanetaryArguments {
    /// Returns the arguments for a time in Julian centuries.
    ///
    /// 返回某儒略世纪时刻的各参数。
    pub fn at(t: f64) -> Self {
        Self {
            l: mean_anomaly_moon(t),
            f: mean_latitude_moon(t),
            d: mean_elongation_moon_sun(t),
            omega: mean_longitude_node(t),
            mercury: mean_longitude_mercury(t),
            venus: mean_longitude_venus(t),
            earth: mean_longitude_earth(t),
            mars: mean_longitude_mars(t),
            jupiter: mean_longitude_jupiter(t),
            saturn: mean_longitude_saturn(t),
            uranus: mean_longitude_uranus(t),
            neptune: mean_longitude_neptune(t),
            precession: general_precession(t),
        }
    }

    /// Returns the arguments as an array in series order.
    ///
    /// 按级数次序把各参数作为数组返回。
    pub const fn as_array(self) -> [f64; 13] {
        [
            self.l,
            self.f,
            self.d,
            self.omega,
            self.mercury,
            self.venus,
            self.earth,
            self.mars,
            self.jupiter,
            self.saturn,
            self.uranus,
            self.neptune,
            self.precession,
        ]
    }
}
