//! The IAU 2000A nutation model.
//!
//! Nutation is the periodic part of the motion of the Earth's rotation axis
//! about its mean position. It is caused mainly by the torque of the Moon
//! and the Sun on the equatorial bulge, with smaller contributions from the
//! planets. The model gives the offset of the true pole from the mean pole
//! as two angles: the nutation in longitude and the nutation in obliquity.
//!
//! The series has two parts whose arguments and coefficient groupings
//! differ. The luni-solar part has 678 terms built from the Delaunay
//! arguments, and its obliquity coefficient pairs the cosine with the
//! constant and time terms while the sine stands alone. The planetary part
//! has 687 terms built from the planetary longitudes, and there the sine
//! pairs with the constant and the cosine stands alone. Getting those two
//! groupings the same way round produces an error of arcseconds, so they are
//! kept separate here.
//!
//! Source: the MHB2000 model of Mathews, Herring and Buffett, adopted by the
//! International Astronomical Union in 2000, as implemented in the IAU SOFA
//! library and its ERFA derivative, functions `eraNut00a` and `eraNut06a`.
//!
//! IAU 2000A 章动模型。
//!
//! 章动是地球自转轴绕其平均位置的周期运动部分。它主要由月球与太阳对赤道
//! 隆起的力矩造成，行星的贡献较小。该模型把真极相对平极的偏移给出为两个
//! 角：黄经方向的章动与黄赤交角方向的章动。
//!
//! 该级数有两部分，其辐角与系数组都不同。日月部分有 678 项，由 Delaunay
//! 辐角构成，其黄赤交角系数把余弦与常数项及时间项配对，而正弦独自成立。
//! 行星部分有 687 项，由行星黄经构成，那里正弦与常数项配对而余弦独自成立。
//! 把这两种分组弄反会产生角秒级的误差，故此处将它们分开。
//!
//! 出处：Mathews、Herring 与 Buffett 的 MHB2000 模型，2000 年由国际天文
//! 学联合会采纳，如 IAU SOFA 库及其 ERFA 衍生物所实现，函数 `eraNut00a`
//! 与 `eraNut06a`。

use super::fundamental::{DelaunayArguments, PlanetaryArguments};
use super::nutation_table::{LUNI_SOLAR, PLANETARY};
use super::{PlanetaryTerm, SeriesTerm};
use core::f64::consts::PI;

/// Converts a coefficient in units of 0.1 microarcsecond to radians.
///
/// 把以 0.1 微角秒为单位的系数换算为弧度。
const UNIT_TO_RADIAN: f64 = 1.0e-7 * PI / (180.0 * 3_600.0);

/// Converts arcseconds to radians.
///
/// 把角秒转换为弧度。
const ARCSECOND_TO_RADIAN: f64 = PI / (180.0 * 3_600.0);

/// The nutation in longitude and obliquity, in radians.
///
/// 黄经与黄赤交角方向的章动，单位为弧度。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Nutation {
    /// The nutation in longitude, in radians.
    ///
    /// 黄经方向的章动，单位为弧度。
    pub longitude: f64,
    /// The nutation in obliquity, in radians.
    ///
    /// 黄赤交角方向的章动，单位为弧度。
    pub obliquity: f64,
}

impl Nutation {
    /// Creates a nutation from its two angles.
    ///
    /// 由其两个角创建章动。
    pub const fn new(longitude: f64, obliquity: f64) -> Self {
        Self {
            longitude,
            obliquity,
        }
    }

    /// Returns the nutation in longitude, in arcseconds.
    ///
    /// 返回黄经方向的章动，单位为角秒。
    pub fn longitude_arcseconds(self) -> f64 {
        self.longitude / ARCSECOND_TO_RADIAN
    }

    /// Returns the nutation in obliquity, in arcseconds.
    ///
    /// 返回黄赤交角方向的章动，单位为角秒。
    pub fn obliquity_arcseconds(self) -> f64 {
        self.obliquity / ARCSECOND_TO_RADIAN
    }

    /// Returns the sum of two nutations.
    ///
    /// 返回两个章动之和。
    pub const fn plus(self, other: Self) -> Self {
        Self::new(
            self.longitude + other.longitude,
            self.obliquity + other.obliquity,
        )
    }
}

/// Evaluates one term of the luni-solar series.
///
/// The five multipliers are applied to the Delaunay arguments. The longitude
/// is the sum of a time dependent sine, a constant sine, and a cosine. The
/// obliquity is the sum of a time dependent cosine, a constant cosine, and a
/// sine.
///
/// 求日月级数的一项。
///
/// 五个乘数施加于 Delaunay 参数。黄经是一个随时间变化的正弦、一个常数
/// 正弦与一个余弦之和。黄赤交角是一个随时间变化的余弦、一个常数余弦与
/// 一个正弦之和。
fn lunisolar_term(term: &SeriesTerm, arguments: &DelaunayArguments, t: f64) -> Nutation {
    let multipliers = &term.argument;
    let c = &term.coefficients;
    let argument = f64::from(multipliers[0]) * arguments.l
        + f64::from(multipliers[1]) * arguments.l_prime
        + f64::from(multipliers[2]) * arguments.f
        + f64::from(multipliers[3]) * arguments.d
        + f64::from(multipliers[4]) * arguments.omega;
    let (sine, cosine) = argument.sin_cos();

    // The order is the one the reference uses: sp, spt, cp, ce, cet, se.
    // 次序即参考实现所用者：sp、spt、cp、ce、cet、se。
    Nutation::new(
        (c[0] + c[1] * t) * sine + c[2] * cosine,
        (c[3] + c[4] * t) * cosine + c[5] * sine,
    )
}

/// Evaluates one term of the planetary series.
///
/// The thirteen multipliers are applied to the planetary arguments. The
/// longitude is a sine plus a cosine, and the obliquity is a sine plus a
/// cosine, with no time dependence in either.
///
/// 求行星级数的一项。
///
/// 十三个乘数施加于行星参数。黄经是一个正弦加一个余弦，黄赤交角也是一个
/// 正弦加一个余弦，二者都不含时间依赖。
fn planetary_term(term: &PlanetaryTerm, arguments: &PlanetaryArguments) -> Nutation {
    let multipliers = &term.argument;
    let c = &term.coefficients;
    let values = arguments.as_array();
    let mut argument = 0.0;
    for index in 0..13 {
        argument += f64::from(multipliers[index]) * values[index];
    }
    let (sine, cosine) = argument.sin_cos();

    // The order is the one the reference uses: sp, cp, se, ce.
    // 次序即参考实现所用者：sp、cp、se、ce。
    Nutation::new(c[0] * sine + c[1] * cosine, c[2] * sine + c[3] * cosine)
}

/// Returns the nutation in longitude and obliquity for a time.
///
/// The time is in Julian centuries from J2000.0 on the Terrestrial Time
///
/// 时间以 J2000.0 起的儒略世纪计，位于地球时（TT）标度
/// scale. This is the IAU 2000A model, which includes the planetary terms.
///
/// Source: IAU 2000A, as implemented by `eraNut00a`.
pub fn nutation_2000a(t: f64) -> Nutation {
    let delaunay = DelaunayArguments::at(t);
    let planetary = PlanetaryArguments::at(t);

    let mut sum = Nutation::default();
    for term in LUNI_SOLAR.iter() {
        sum = sum.plus(lunisolar_term(term, &delaunay, t));
    }
    for term in PLANETARY.iter() {
        sum = sum.plus(planetary_term(term, &planetary));
    }

    Nutation::new(
        sum.longitude * UNIT_TO_RADIAN,
        sum.obliquity * UNIT_TO_RADIAN,
    )
}

/// Returns the nutation with the IAU 2006 precession adjustments applied.
///
/// The IAU 2000A model was built on the IAU 1976 precession, and the IAU
/// 2006 precession changed the precession rates. The adjustment brings the
/// nutation into consistency with the newer precession, which is the
/// combination used for the modern celestial-to-terrestrial transformation.
///
/// Source: IAU 2006, as implemented by `eraNut06a`, which applies the
/// expressions of Wallace and Capitaine (2006), equations 5.
///
/// 返回施加 IAU 2006 岁差调整后的章动。
///
/// IAU 2000A 模型建立在 IAU 1976 岁差之上，而 IAU 2006 岁差改变了岁差
/// 速率。该调整使章动与较新的岁差一致，这一组合正是现代天球到地球转换
/// 所用的。
///
/// 出处：IAU 2006，按 `eraNut06a` 所实现，它施加 Wallace 与
/// Capitaine（2006）式 5 的表达式。
pub fn nutation_2000a_with_2006_adjustment(t: f64) -> Nutation {
    let nutation = nutation_2000a(t);
    // The adjustment scales the nutation itself rather than adding an
    // independent term, which is why it is written as a factor.
    // 该调整是对章动本身缩放，而非加上一个独立项，故它被写成一个因子。
    let fj2 = -2.777_4e-6 * t;
    Nutation::new(
        nutation.longitude * (1.0 + 0.469_7e-6 + fj2),
        nutation.obliquity * (1.0 + fj2),
    )
}
