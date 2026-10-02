//! The deep space branch of the propagator.
//!
//! Satellites with an orbital period of 225 minutes or more are propagated
//! with the SDP4 theory, which extends SGP4 with the luni-solar
//! perturbations and the resonance with the Earth's gravity field.
//!
//! The perturbations are split into three groups. Secular terms grow
//! linearly with time and are accumulated in the rate fields. Long period
//! terms vary with the position of the Sun and Moon and are applied by
//! [`dpper`]. Resonance terms arise when the orbital period is commensurate
//! with the Earth's rotation, either one revolution per day or two, and are
//! integrated numerically by [`dspace`] because no closed form exists.
//!
//! The coefficients are computed once at initialisation, since they depend
//! only on the epoch elements and the epoch time.
//!
//! Source: every expression follows Vallado et al., *Revisiting Spacetrack
//! Report #3*, AIAA 2006-6753, 2006, and the reference implementation
//! distributed with that paper. The perturbation amplitudes such as `zes`
//! and `znl`, the tesseral coefficients such as `q22`, and the resonance
//! coefficients are quoted to the precision the reference uses, because
//! reproducing its output requires the same rounding.
//!
//! 传播器的深空分支。
//!
//! 轨道周期为 225 分钟及以上的卫星用 SDP4 理论传播，它以日月摄动及与地球
//! 引力场的共振扩展 SGP4。
//!
//! 各摄动分为三组。长期项随时间线性增长，累积在各速率字段中。长周期项随
//! 太阳与月球的位置变化，由 [`dpper`] 施加。当轨道周期与地球自转成简单
//! 整数比（每天一圈或两圈）时出现共振项，由于没有闭式，由 [`dspace`] 数值
//! 积分。
//!
//! 各系数在初始化时计算一次，因为它们只依赖于历元根数与历元时间。
//!
//! 出处：每个表达式都遵循 Vallado 等，*Revisiting Spacetrack Report #3*，
//! AIAA 2006-6753，2006，以及随该论文分发的参考实现。诸如 `zes` 与 `znl`
//! 的摄动幅度、诸如 `q22` 的田谐系数，以及共振系数都取参考实现所用的精度，
//! 因为复现其输出需要相同的舍入。

use crate::satellite::{Satellite, TWO_PI};
use core::f64::consts::PI;

/// 太阳偏心率。
const ZES: f64 = 0.016_75;
/// 月球偏心率。
const ZEL: f64 = 0.054_90;
/// 太阳平运动系数，单位为弧度每分钟。
const ZNS: f64 = 1.194_59e-5;
/// 月球平运动系数，单位为弧度每分钟。
const ZNL: f64 = 1.583_521_8e-4;

/// The rate of rotation of the Earth used by the resonance integral, in
/// radians per minute.
///
/// 共振积分所用的地球自转速率，单位为每分钟弧度。
///
/// 该值为 `7.292 115 146 688 55e-5` 弧度每秒。
const RPTIM: f64 = 4.375_269_088_011_3e-3;

/// 共振积分的积分步长，单位为分钟。
const STEP_POSITIVE: f64 = 720.0;
/// 向后传播时共振积分的积分步长。
const STEP_NEGATIVE: f64 = -720.0;
/// 积分步长的平方，单位为分钟每二次方。
const STEP_SQUARED: f64 = 259_200.0;

/// The values returned by the first stage of the deep space
/// initialisation, which computes the luni-solar series.
///
/// The reference implementation names the two evaluations of the series with
/// a leading `s`: the values from the first pass, for the Moon, are stored
/// with a doubled letter, and the values from the second pass, for the Sun,
/// keep the single letter. The final coefficients combine one from each
/// group, so both are carried here.
///
/// 深空初始化第一阶段返回的值，该阶段计算日月级数。
///
/// 参考实现以首字母 `s` 命名对该级数的两次求值：第一遍（月球）的值用双写
/// 字母存放，第二遍（太阳）的值保留单字母。最终系数各取其一组合，故两者
/// 都在此保留。
#[derive(Debug, Clone, Copy)]
struct Lunisolar {
    /// 月球级数，二阶项。
    ss1: f64,
    /// 月球级数，二阶项。
    ss2: f64,
    /// 月球级数，二阶项。
    ss3: f64,
    /// 月球级数，二阶项。
    ss4: f64,
    /// 月球级数，二阶项。
    ss5: f64,
    /// 月球级数，二阶项。
    ss6: f64,
    /// 月球级数，二阶项。
    ss7: f64,
    /// 月球级数，径向项。
    sz1: f64,
    /// 月球级数，径向项。
    sz2: f64,
    /// 月球级数，径向项。
    sz3: f64,
    /// 月球级数，径向项。
    sz11: f64,
    /// 月球级数，径向项。
    sz12: f64,
    /// 月球级数，径向项。
    sz13: f64,
    /// 月球级数，径向项。
    sz21: f64,
    /// 月球级数，径向项。
    sz22: f64,
    /// 月球级数，径向项。
    sz23: f64,
    /// 月球级数，径向项。
    sz31: f64,
    /// 月球级数，径向项。
    sz32: f64,
    /// 月球级数，径向项。
    sz33: f64,
    /// 太阳级数，二阶项。
    s1: f64,
    /// 太阳级数，二阶项。
    s2: f64,
    /// 太阳级数，二阶项。
    s3: f64,
    /// 太阳级数，二阶项。
    s4: f64,
    /// 太阳级数，二阶项。
    s5: f64,
    /// 太阳级数，二阶项。
    s6: f64,
    /// 太阳级数，二阶项。
    s7: f64,
    /// 太阳级数，径向项。
    z1: f64,
    /// 太阳级数，径向项。
    z2: f64,
    /// 太阳级数，径向项。
    z3: f64,
    /// 太阳级数，径向项。
    z11: f64,
    /// 太阳级数，径向项。
    z12: f64,
    /// 太阳级数，径向项。
    z13: f64,
    /// 太阳级数，径向项。
    z21: f64,
    /// 太阳级数，径向项。
    z22: f64,
    /// 太阳级数，径向项。
    z23: f64,
    /// 太阳级数，径向项。
    z31: f64,
    /// 太阳级数，径向项。
    z32: f64,
    /// 太阳级数，径向项。
    z33: f64,
    /// 历元处月球平近点角，单位为弧度。
    zmol: f64,
    /// 历元处太阳平近点角，单位为弧度。
    zmos: f64,
    /// Eccentricity squared at the epoch.
    ///
    /// 历元处的偏心率平方。
    emsq: f64,
    /// Sine of the inclination at the epoch.
    ///
    /// 历元处倾角的正弦。
    sinim: f64,
    /// Cosine of the inclination at the epoch.
    ///
    /// 历元处倾角的余弦。
    cosim: f64,
}

/// The deep space coefficients of a satellite.
///
/// Every field is zero for a near-Earth orbit, so the structure can be
/// stored unconditionally and the near-Earth propagation step pays only the
/// cost of carrying the storage.
///
/// 卫星的深空系数。
///
/// 对近地轨道每个字段都为零，故该结构可无条件存储，而近地传播步骤只承担
/// 携带该存储的代价。
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct DeepSpace {
    /// 日月长周期项系数。
    pub(crate) e3: f64,
    /// 日月长周期项系数。
    pub(crate) ee2: f64,
    /// 日月长周期项系数。
    pub(crate) se2: f64,
    /// 日月长周期项系数。
    pub(crate) se3: f64,
    /// 日月长周期项系数。
    pub(crate) si2: f64,
    /// 日月长周期项系数。
    pub(crate) si3: f64,
    /// 日月长周期项系数。
    pub(crate) sl2: f64,
    /// 日月长周期项系数。
    pub(crate) sl3: f64,
    /// 日月长周期项系数。
    pub(crate) sl4: f64,
    /// 日月长周期项系数。
    pub(crate) sgh2: f64,
    /// 日月长周期项系数。
    pub(crate) sgh3: f64,
    /// 日月长周期项系数。
    pub(crate) sgh4: f64,
    /// 日月长周期项系数。
    pub(crate) sh2: f64,
    /// 日月长周期项系数。
    pub(crate) sh3: f64,
    /// 日月长周期项系数。
    pub(crate) xgh2: f64,
    /// 日月长周期项系数。
    pub(crate) xgh3: f64,
    /// 日月长周期项系数。
    pub(crate) xgh4: f64,
    /// 日月长周期项系数。
    pub(crate) xh2: f64,
    /// 日月长周期项系数。
    pub(crate) xh3: f64,
    /// 日月长周期项系数。
    pub(crate) xi2: f64,
    /// 日月长周期项系数。
    pub(crate) xi3: f64,
    /// 日月长周期项系数。
    pub(crate) xl2: f64,
    /// 日月长周期项系数。
    pub(crate) xl3: f64,
    /// 日月长周期项系数。
    pub(crate) xl4: f64,
    /// 历元处月球平近点角，单位为弧度。
    pub(crate) zmol: f64,
    /// 历元处太阳平近点角，单位为弧度。
    pub(crate) zmos: f64,
    /// 自历元沿用的长周期项。
    pub(crate) peo: f64,
    /// 自历元沿用的长周期项。
    pub(crate) pinco: f64,
    /// 自历元沿用的长周期项。
    pub(crate) plo: f64,
    /// 自历元沿用的长周期项。
    pub(crate) pgho: f64,
    /// 自历元沿用的长周期项。
    pub(crate) pho: f64,

    /// Resonance flag: zero for none, one for the synchronous resonance, two
    /// for the half-day resonance.
    ///
    /// 共振标志：零表示无共振，一表示同步共振，二表示半日共振。
    pub(crate) irez: i32,
    /// Secular rate of the eccentricity, per minute.
    ///
    /// 偏心率的长期变化率，每分钟。
    pub(crate) dedt: f64,
    /// Secular rate of the inclination, per minute.
    ///
    /// 倾角的长期变化率，每分钟。
    pub(crate) didt: f64,
    /// Secular rate of the mean anomaly, per minute.
    ///
    /// 平近点角的长期变化率，每分钟。
    pub(crate) dmdt: f64,
    /// Secular rate of the right ascension, per minute.
    ///
    /// 赤经的长期变化率，每分钟。
    pub(crate) dnodt: f64,
    /// Secular rate of the argument of perigee, per minute.
    ///
    /// 近地点幅角的长期变化率，每分钟。
    pub(crate) domdt: f64,
    /// 同步共振系数。
    pub(crate) del1: f64,
    /// 同步共振系数。
    pub(crate) del2: f64,
    /// 同步共振系数。
    pub(crate) del3: f64,
    /// 半日共振系数。
    pub(crate) d2201: f64,
    /// 半日共振系数。
    pub(crate) d2211: f64,
    /// 半日共振系数。
    pub(crate) d3210: f64,
    /// 半日共振系数。
    pub(crate) d3222: f64,
    /// 半日共振系数。
    pub(crate) d4410: f64,
    /// 半日共振系数。
    pub(crate) d4422: f64,
    /// 半日共振系数。
    pub(crate) d5220: f64,
    /// 半日共振系数。
    pub(crate) d5232: f64,
    /// 半日共振系数。
    pub(crate) d5421: f64,
    /// 半日共振系数。
    pub(crate) d5433: f64,
    /// Resonance angle rate.
    ///
    /// 共振角变化率。
    pub(crate) xfact: f64,
    /// Resonance angle at the epoch, in radians.
    ///
    /// 历元处的共振角，单位为弧度。
    pub(crate) xlamo: f64,
}

impl DeepSpace {
    /// The all zero value used by near-Earth orbits.
    ///
    /// 近地轨道所用的全零取值。
    pub(crate) const ZERO: Self = Self {
        e3: 0.0,
        ee2: 0.0,
        se2: 0.0,
        se3: 0.0,
        si2: 0.0,
        si3: 0.0,
        sl2: 0.0,
        sl3: 0.0,
        sl4: 0.0,
        sgh2: 0.0,
        sgh3: 0.0,
        sgh4: 0.0,
        sh2: 0.0,
        sh3: 0.0,
        xgh2: 0.0,
        xgh3: 0.0,
        xgh4: 0.0,
        xh2: 0.0,
        xh3: 0.0,
        xi2: 0.0,
        xi3: 0.0,
        xl2: 0.0,
        xl3: 0.0,
        xl4: 0.0,
        zmol: 0.0,
        zmos: 0.0,
        peo: 0.0,
        pinco: 0.0,
        plo: 0.0,
        pgho: 0.0,
        pho: 0.0,
        irez: 0,
        dedt: 0.0,
        didt: 0.0,
        dmdt: 0.0,
        dnodt: 0.0,
        domdt: 0.0,
        del1: 0.0,
        del2: 0.0,
        del3: 0.0,
        d2201: 0.0,
        d2211: 0.0,
        d3210: 0.0,
        d3222: 0.0,
        d4410: 0.0,
        d4422: 0.0,
        d5220: 0.0,
        d5232: 0.0,
        d5421: 0.0,
        d5433: 0.0,
        xfact: 0.0,
        xlamo: 0.0,
    };
}

/// The result of applying the long period periodic perturbations.
///
/// 施加长周期项后的结果。
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PerturbedElements {
    /// 偏心率，无量纲。
    pub(crate) eccentricity: f64,
    /// 倾角，单位为弧度。
    pub(crate) inclination: f64,
    /// 升交点赤经，单位为弧度。
    pub(crate) node: f64,
    /// 近地点幅角，单位为弧度。
    pub(crate) argument: f64,
    /// 平近点角，单位为弧度。
    pub(crate) mean_anomaly: f64,
}

/// The mean elements after the deep space secular and resonance terms.
///
/// 施加深空长期项与共振项之后的平均根数。
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct DeepSpaceState {
    /// 偏心率，无量纲。
    pub(crate) eccentricity: f64,
    /// 近地点幅角，单位为弧度。
    pub(crate) argument: f64,
    /// 倾角，单位为弧度。
    pub(crate) inclination: f64,
    /// 平近点角，单位为弧度。
    pub(crate) mean_anomaly: f64,
    /// 升交点赤经，单位为弧度。
    pub(crate) node: f64,
    /// Mean motion, in radians per minute.
    ///
    /// 平运动，单位为弧度每分钟。
    pub(crate) mean_motion: f64,
    /// The angle of the resonance integrator at the propagated time.
    ///
    /// 传播时刻共振积分器的角度。
    pub(crate) atime: f64,
    /// The resonance angle at the propagated time, in radians.
    ///
    /// 传播时刻的共振角，单位为弧度。
    pub(crate) xli: f64,
    /// The resonance mean motion at the propagated time.
    ///
    /// 传播时刻的共振平运动。
    pub(crate) xni: f64,
}

/// Reduces a value into `[0, 2 pi)` by taking the positive remainder.
///
/// 取正余数把某个值归约到 `[0, 2 pi)`。
fn remainder_positive(value: f64) -> f64 {
    value.rem_euclid(TWO_PI)
}

/// Reduces a value keeping the sign of the dividend.
///
/// The reference uses this convention for the right ascension, where a
/// negative result is meaningful, and it differs from a positive remainder
/// for negative inputs.
///
/// 归约某个值并保留被除数的符号。
///
/// 参考实现对赤经采用此约定，因为负结果在此有意义；对负输入，它与取正余数
/// 的结果不同。
fn remainder_signed(value: f64) -> f64 {
    if value >= 0.0 {
        value % TWO_PI
    } else {
        -((-value) % TWO_PI)
    }
}

/// Computes the deep space coefficients for a satellite.
///
/// 计算某卫星的深空系数。
pub(crate) fn initialise(satellite: &Satellite) -> DeepSpace {
    let lunisolar = lunisolar_coefficients(satellite);
    let mut deep = DeepSpace::ZERO;

    deep.zmol = lunisolar.zmol;
    deep.zmos = lunisolar.zmos;
    deep.se2 = 2.0 * lunisolar.ss1 * lunisolar.ss6;
    deep.se3 = 2.0 * lunisolar.ss1 * lunisolar.ss7;
    deep.si2 = 2.0 * lunisolar.ss2 * lunisolar.sz12;
    deep.si3 = 2.0 * lunisolar.ss2 * (lunisolar.sz13 - lunisolar.sz11);
    deep.sl2 = -2.0 * lunisolar.ss3 * lunisolar.sz2;
    deep.sl3 = -2.0 * lunisolar.ss3 * (lunisolar.sz3 - lunisolar.sz1);
    deep.sl4 = -2.0 * lunisolar.ss3 * (-21.0 - 9.0 * lunisolar.emsq) * ZES;
    deep.sgh2 = 2.0 * lunisolar.ss4 * lunisolar.sz32;
    deep.sgh3 = 2.0 * lunisolar.ss4 * (lunisolar.sz33 - lunisolar.sz31);
    deep.sgh4 = -18.0 * lunisolar.ss4 * ZES;
    deep.sh2 = -2.0 * lunisolar.ss2 * lunisolar.sz22;
    deep.sh3 = -2.0 * lunisolar.ss2 * (lunisolar.sz23 - lunisolar.sz21);
    deep.ee2 = 2.0 * lunisolar.s1 * lunisolar.s6;
    deep.e3 = 2.0 * lunisolar.s1 * lunisolar.s7;
    deep.xi2 = 2.0 * lunisolar.s2 * lunisolar.z12;
    deep.xi3 = 2.0 * lunisolar.s2 * (lunisolar.z13 - lunisolar.z11);
    deep.xl2 = -2.0 * lunisolar.s3 * lunisolar.z2;
    deep.xl3 = -2.0 * lunisolar.s3 * (lunisolar.z3 - lunisolar.z1);
    deep.xl4 = -2.0 * lunisolar.s3 * (-21.0 - 9.0 * lunisolar.emsq) * ZEL;
    deep.xgh2 = 2.0 * lunisolar.s4 * lunisolar.z32;
    deep.xgh3 = 2.0 * lunisolar.s4 * (lunisolar.z33 - lunisolar.z31);
    deep.xgh4 = -18.0 * lunisolar.s4 * ZEL;
    deep.xh2 = -2.0 * lunisolar.s2 * lunisolar.z22;
    deep.xh3 = -2.0 * lunisolar.s2 * (lunisolar.z23 - lunisolar.z21);

    initialise_rates(satellite, &lunisolar, &mut deep);
    initialise_resonance(satellite, &lunisolar, &mut deep);

    deep
}

/// Evaluates the luni-solar series at the epoch.
///
/// The series is evaluated twice with different orientations, once aligned
/// with the Moon and once with the Sun. Both sets of results are needed
/// because the final coefficients pair the two.
///
/// 在历元处求值日月级数。
///
/// 该级数以两种不同的取向各求值一次，一次与月球对齐，一次与太阳对齐。
/// 两组结果都需要，因为最终系数把二者配对使用。
fn lunisolar_coefficients(satellite: &Satellite) -> Lunisolar {
    const C1SS: f64 = 2.986_479_7e-6;
    const C1L: f64 = 4.796_806_5e-7;

    let ep = satellite.ecco;
    let argpp = satellite.argpo;
    let inclp = satellite.inclo;
    let nodep = satellite.nodeo;
    let np = satellite.no_unkozai;

    // Days counted from the start of 1900. The reference adds 18 261.5 to a
    // day count from 1950, which is the same origin.
    // 自 1900 年初起计的天数。参考实现在自 1950 年的日计数上加 18 261.5，
    // 二者原点相同。
    let day = satellite.epoch_mjd - 15_019.5;

    let emsq = ep * ep;
    let betasq = 1.0 - emsq;
    let rtemsq = betasq.sqrt();
    let snodm = nodep.sin();
    let cnodm = nodep.cos();
    let sinomm = argpp.sin();
    let cosomm = argpp.cos();
    let sinim = inclp.sin();
    let cosim = inclp.cos();

    let xnodce = remainder_positive(4.523_602_0 - 9.242_202_9e-4 * day);
    let stem = xnodce.sin();
    let ctem = xnodce.cos();
    let zcosil = 0.913_751_64 - 0.035_680_96 * ctem;
    let zsinil = (1.0 - zcosil * zcosil).sqrt();
    let zsinhl = 0.089_683_511 * stem / zsinil;
    let zcoshl = (1.0 - zsinhl * zsinhl).sqrt();
    let gam = 5.835_151_4 + 0.001_944_368_0 * day;
    let zx = 0.397_854_16 * stem / zsinil;
    let zy = zcoshl * ctem + 0.917_448_67 * zsinhl * stem;
    let zx = gam + zx.atan2(zy) - xnodce;
    let zcosgl = zx.cos();
    let zsingl = zx.sin();

    let mut zcosg = 0.194_590_5;
    let mut zsing = -0.980_884_58;
    let mut zcosi = 0.917_448_67;
    let mut zsini = 0.397_854_16;
    let mut zcosh = cnodm;
    let mut zsinh = snodm;
    let mut cc = C1SS;
    let xnoi = 1.0 / np;

    // Values from the first pass, for the Moon.
    // 第一遍的值，对应月球。
    let (mut ss1, mut ss2, mut ss3, mut ss4, mut ss5, mut ss6, mut ss7) =
        (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    let (
        mut sz1,
        mut sz2,
        mut sz3,
        mut sz11,
        mut sz12,
        mut sz13,
        mut sz21,
        mut sz22,
        mut sz23,
        mut sz31,
        mut sz32,
        mut sz33,
    ) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);

    // Values from the second pass, for the Sun.
    // 第二遍的值，对应太阳。
    let (mut s1, mut s2, mut s3, mut s4, mut s5, mut s6, mut s7) =
        (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    let (
        mut z1,
        mut z2,
        mut z3,
        mut z11,
        mut z12,
        mut z13,
        mut z21,
        mut z22,
        mut z23,
        mut z31,
        mut z32,
        mut z33,
    ) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);

    for pass in 0..2 {
        let a1 = zcosg * zcosh + zsing * zcosi * zsinh;
        let a3 = -zsing * zcosh + zcosg * zcosi * zsinh;
        let a7 = -zcosg * zsinh + zsing * zcosi * zcosh;
        let a8 = zsing * zsini;
        let a9 = zsing * zsinh + zcosg * zcosi * zcosh;
        let a10 = zcosg * zsini;
        let a2 = cosim * a7 + sinim * a8;
        let a4 = cosim * a9 + sinim * a10;
        let a5 = -sinim * a7 + cosim * a8;
        let a6 = -sinim * a9 + cosim * a10;

        let x1 = a1 * cosomm + a2 * sinomm;
        let x2 = a3 * cosomm + a4 * sinomm;
        let x3 = -a1 * sinomm + a2 * cosomm;
        let x4 = -a3 * sinomm + a4 * cosomm;
        let x5 = a5 * sinomm;
        let x6 = a6 * sinomm;
        let x7 = a5 * cosomm;
        let x8 = a6 * cosomm;

        let z31p = 12.0 * x1 * x1 - 3.0 * x3 * x3;
        let z32p = 24.0 * x1 * x2 - 6.0 * x3 * x4;
        let z33p = 12.0 * x2 * x2 - 3.0 * x4 * x4;
        let mut z1p = 3.0 * (a1 * a1 + a2 * a2) + z31p * emsq;
        let mut z2p = 6.0 * (a1 * a3 + a2 * a4) + z32p * emsq;
        let mut z3p = 3.0 * (a3 * a3 + a4 * a4) + z33p * emsq;
        let z11p = -6.0 * a1 * a5 + emsq * (-24.0 * x1 * x7 - 6.0 * x3 * x5);
        let z12p = -6.0 * (a1 * a6 + a3 * a5)
            + emsq * (-24.0 * (x2 * x7 + x1 * x8) - 6.0 * (x3 * x6 + x4 * x5));
        let z13p = -6.0 * a3 * a6 + emsq * (-24.0 * x2 * x8 - 6.0 * x4 * x6);
        let z21p = 6.0 * a2 * a5 + emsq * (24.0 * x1 * x5 - 6.0 * x3 * x7);
        let z22p = 6.0 * (a4 * a5 + a2 * a6)
            + emsq * (24.0 * (x2 * x5 + x1 * x6) - 6.0 * (x4 * x7 + x3 * x8));
        let z23p = 6.0 * a4 * a6 + emsq * (24.0 * x2 * x6 - 6.0 * x4 * x8);
        // The reference doubles each of these and adds the sub-expression
        // multiplied by `1 - e^2`.
        // 参考实现把各项乘以二，再加上该子表达式与 `1 - e^2` 的乘积。
        z1p = z1p + z1p + betasq * z31p;
        z2p = z2p + z2p + betasq * z32p;
        z3p = z3p + z3p + betasq * z33p;

        let s3p = cc * xnoi;
        let s2p = -0.5 * s3p / rtemsq;
        let s4p = s3p * rtemsq;
        let s1p = -15.0 * ep * s4p;
        let s5p = x1 * x3 + x2 * x4;
        let s6p = x2 * x3 + x1 * x4;
        let s7p = x2 * x4 - x1 * x3;

        if pass == 0 {
            ss1 = s1p;
            ss2 = s2p;
            ss3 = s3p;
            ss4 = s4p;
            ss5 = s5p;
            ss6 = s6p;
            ss7 = s7p;
            sz1 = z1p;
            sz2 = z2p;
            sz3 = z3p;
            sz11 = z11p;
            sz12 = z12p;
            sz13 = z13p;
            sz21 = z21p;
            sz22 = z22p;
            sz23 = z23p;
            sz31 = z31p;
            sz32 = z32p;
            sz33 = z33p;

            zcosg = zcosgl;
            zsing = zsingl;
            zcosi = zcosil;
            zsini = zsinil;
            zcosh = zcoshl * cnodm + zsinhl * snodm;
            zsinh = snodm * zcoshl - cnodm * zsinhl;
            cc = C1L;
        } else {
            s1 = s1p;
            s2 = s2p;
            s3 = s3p;
            s4 = s4p;
            s5 = s5p;
            s6 = s6p;
            s7 = s7p;
            z1 = z1p;
            z2 = z2p;
            z3 = z3p;
            z11 = z11p;
            z12 = z12p;
            z13 = z13p;
            z21 = z21p;
            z22 = z22p;
            z23 = z23p;
            z31 = z31p;
            z32 = z32p;
            z33 = z33p;
        }
    }

    Lunisolar {
        ss1,
        ss2,
        ss3,
        ss4,
        ss5,
        ss6,
        ss7,
        sz1,
        sz2,
        sz3,
        sz11,
        sz12,
        sz13,
        sz21,
        sz22,
        sz23,
        sz31,
        sz32,
        sz33,
        s1,
        s2,
        s3,
        s4,
        s5,
        s6,
        s7,
        z1,
        z2,
        z3,
        z11,
        z12,
        z13,
        z21,
        z22,
        z23,
        z31,
        z32,
        z33,
        zmol: remainder_positive(4.719_967_2 + 0.229_971_50 * day - gam),
        zmos: remainder_positive(6.256_583_7 + 0.017_201_977 * day),
        emsq,
        sinim,
        cosim,
    }
}

/// Computes the secular rates of the mean elements.
///
/// 计算平均根数的长期变化率。
fn initialise_rates(satellite: &Satellite, ls: &Lunisolar, deep: &mut DeepSpace) {
    // The reference sets `shs` and `shll` to zero for inclinations within
    // about 3 degrees of the equator or the pole, where the division by the
    // sine of the inclination is ill conditioned.
    // 倾角在赤道或极点附近约 3 度以内时，参考实现把 `shs` 与 `shll` 置零，
    // 因为那里除以倾角正弦是病态的。
    let inclined = (5.235_987_7e-2..=PI - 5.235_987_7e-2).contains(&satellite.inclo);

    let ses = ls.ss1 * ZNS * ls.ss5;
    let sis = ls.ss2 * ZNS * (ls.sz11 + ls.sz13);
    let sls = -ZNS * ls.ss3 * (ls.sz1 + ls.sz3 - 14.0 - 6.0 * ls.emsq);
    let sghs = ls.ss4 * ZNS * (ls.sz31 + ls.sz33 - 6.0);
    let mut shs = -ZNS * ls.ss2 * (ls.sz21 + ls.sz23);
    if !inclined {
        shs = 0.0;
    }
    if ls.sinim != 0.0 {
        shs /= ls.sinim;
    }
    let sgs = sghs - ls.cosim * shs;

    deep.dedt = ses + ls.s1 * ZNL * ls.s5;
    deep.didt = sis + ls.s2 * ZNL * (ls.z11 + ls.z13);
    deep.dmdt = sls - ZNL * ls.s3 * (ls.z1 + ls.z3 - 14.0 - 6.0 * ls.emsq);

    let sghl = ls.s4 * ZNL * (ls.z31 + ls.z33 - 6.0);
    let mut shll = -ZNL * ls.s2 * (ls.z21 + ls.z23);
    if !inclined {
        shll = 0.0;
    }
    let mut domdt = sgs + sghl;
    let mut dnodt = shs;
    if ls.sinim != 0.0 {
        domdt -= ls.cosim / ls.sinim * shll;
        dnodt += shll / ls.sinim;
    }
    deep.domdt = domdt;
    deep.dnodt = dnodt;
}

/// Computes the resonance coefficients.
///
/// 计算共振系数。
fn initialise_resonance(satellite: &Satellite, ls: &Lunisolar, deep: &mut DeepSpace) {
    const Q22: f64 = 1.789_167_9e-6;
    const Q31: f64 = 2.146_074_8e-6;
    const Q33: f64 = 2.212_301_5e-7;
    const ROOT22: f64 = 1.789_167_9e-6;
    const ROOT32: f64 = 3.739_379_2e-7;
    const ROOT44: f64 = 7.363_695_3e-9;
    const ROOT52: f64 = 1.142_863_9e-7;
    const ROOT54: f64 = 2.176_580_3e-9;
    const X2O3: f64 = 2.0 / 3.0;

    let nm = satellite.no_unkozai;
    let em = satellite.ecco;
    let emsq = ls.emsq;
    let sinim = ls.sinim;
    let cosim = ls.cosim;
    let mo = satellite.mo;
    let nodeo = satellite.nodeo;
    let argpo = satellite.argpo;

    // The synchronous resonance applies near one revolution per sidereal
    // day and the half-day resonance near two, with a further restriction on
    // the eccentricity.
    // 同步共振适用于每恒星日约一圈处，半日共振适用于约两圈处，并对偏心率
    // 另有约束。
    let mut irez = 0;
    if 0.003_490_658_5 < nm && nm < 0.005_235_987_7 {
        irez = 1;
    }
    if (8.26e-3..=9.24e-3).contains(&nm) && em >= 0.5 {
        irez = 2;
    }
    deep.irez = irez;
    if irez == 0 {
        return;
    }

    let aonv = (nm / satellite.xke()).powf(X2O3);
    let theta = remainder_positive(satellite.gsto);

    if irez == 2 {
        // Half-day resonance. The eccentricity series is evaluated at the
        // eccentricity from the element set, not the value carried through
        // the secular rates.
        // 半日共振。偏心率级数按根数集给出的偏心率求值，而非按经长期速率
        // 传递后的值。
        let cosisq = cosim * cosim;
        let emo = em;
        let em = satellite.ecco;
        let emsqo = emsq;
        let emsq = em * em;
        let eoc = em * emsq;

        let g201 = -0.306 - (em - 0.64) * 0.440;
        let (g211, g310, g322, g410, g422, g520) = if em <= 0.65 {
            (
                3.616 - 13.2470 * em + 16.2900 * emsq,
                -19.302 + 117.3900 * em - 228.4190 * emsq + 156.5910 * eoc,
                -18.9068 + 109.7927 * em - 214.6334 * emsq + 146.5816 * eoc,
                -41.122 + 242.6940 * em - 471.0940 * emsq + 313.9530 * eoc,
                -146.407 + 841.8800 * em - 1629.014 * emsq + 1083.4350 * eoc,
                -532.114 + 3017.977 * em - 5740.032 * emsq + 3708.2760 * eoc,
            )
        } else {
            let g520 = if em > 0.715 {
                -5149.66 + 29936.92 * em - 54087.36 * emsq + 31324.56 * eoc
            } else {
                1464.74 - 4664.75 * em + 3763.64 * emsq
            };
            (
                -72.099 + 331.819 * em - 508.738 * emsq + 266.724 * eoc,
                -346.844 + 1582.851 * em - 2415.925 * emsq + 1246.113 * eoc,
                -342.585 + 1554.908 * em - 2366.899 * emsq + 1215.972 * eoc,
                -1052.797 + 4758.686 * em - 7193.992 * emsq + 3651.957 * eoc,
                -3581.690 + 16178.110 * em - 24462.770 * emsq + 12422.520 * eoc,
                g520,
            )
        };
        let (g533, g521, g532) = if em < 0.7 {
            (
                -919.227_70 + 4988.6100 * em - 9064.7700 * emsq + 5542.21 * eoc,
                -822.710_72 + 4568.6173 * em - 8491.4146 * emsq + 5337.524 * eoc,
                -853.666_00 + 4690.2500 * em - 8624.7700 * emsq + 5341.4 * eoc,
            )
        } else {
            (
                -37995.780 + 161616.52 * em - 229838.20 * emsq + 109377.94 * eoc,
                -51752.104 + 218913.95 * em - 309468.16 * emsq + 146349.42 * eoc,
                -40023.880 + 170470.89 * em - 242699.48 * emsq + 115605.82 * eoc,
            )
        };

        let sini2 = sinim * sinim;
        let f220 = 0.75 * (1.0 + 2.0 * cosim + cosisq);
        let f221 = 1.5 * sini2;
        let f321 = 1.875 * sinim * (1.0 - 2.0 * cosim - 3.0 * cosisq);
        let f322 = -1.875 * sinim * (1.0 + 2.0 * cosim - 3.0 * cosisq);
        let f441 = 35.0 * sini2 * f220;
        let f442 = 39.375_0 * sini2 * sini2;
        let f522 = 9.843_75
            * sinim
            * (sini2 * (1.0 - 2.0 * cosim - 5.0 * cosisq)
                + 0.333_333_33 * (-2.0 + 4.0 * cosim + 6.0 * cosisq));
        let f523 = sinim
            * (4.921_875_12 * sini2 * (-2.0 - 4.0 * cosim + 10.0 * cosisq)
                + 6.562_500_12 * (1.0 + 2.0 * cosim - 3.0 * cosisq));
        let f542 = 29.531_25
            * sinim
            * (2.0 - 8.0 * cosim + cosisq * (-12.0 + 8.0 * cosim + 10.0 * cosisq));
        let f543 = 29.531_25
            * sinim
            * (-2.0 - 8.0 * cosim + cosisq * (12.0 + 8.0 * cosim - 10.0 * cosisq));

        let xno2 = nm * nm;
        let ainv2 = aonv * aonv;
        let mut temp1 = 3.0 * xno2 * ainv2;
        let mut temp = temp1 * ROOT22;
        deep.d2201 = temp * f220 * g201;
        deep.d2211 = temp * f221 * g211;
        temp1 *= aonv;
        temp = temp1 * ROOT32;
        deep.d3210 = temp * f321 * g310;
        deep.d3222 = temp * f322 * g322;
        temp1 *= aonv;
        temp = 2.0 * temp1 * ROOT44;
        deep.d4410 = temp * f441 * g410;
        deep.d4422 = temp * f442 * g422;
        temp1 *= aonv;
        temp = temp1 * ROOT52;
        deep.d5220 = temp * f522 * g520;
        deep.d5232 = temp * f523 * g532;
        temp = 2.0 * temp1 * ROOT54;
        deep.d5421 = temp * f542 * g521;
        deep.d5433 = temp * f543 * g533;

        deep.xlamo = remainder_positive(mo + nodeo + nodeo - theta - theta);
        deep.xfact =
            satellite.mdot + deep.dmdt + 2.0 * (satellite.nodedot + deep.dnodt - RPTIM) - nm;

        let _ = (emo, emsqo);
    } else {
        // Synchronous resonance.
        // 同步共振。
        let g200 = 1.0 + emsq * (-2.5 + 0.8125 * emsq);
        let g310 = 1.0 + 2.0 * emsq;
        let g300 = 1.0 + emsq * (-6.0 + 6.609_37 * emsq);
        let f220 = 0.75 * (1.0 + cosim) * (1.0 + cosim);
        let f311 = 0.9375 * sinim * sinim * (1.0 + 3.0 * cosim) - 0.75 * (1.0 + cosim);
        let f330 = 1.0 + cosim;
        let f330 = 1.875 * f330 * f330 * f330;
        let mut del1 = 3.0 * nm * nm * aonv * aonv;
        let del2 = 2.0 * del1 * f220 * g200 * Q22;
        let del3 = 3.0 * del1 * f330 * g300 * Q33 * aonv;
        del1 = del1 * f311 * g310 * Q31 * aonv;

        deep.del1 = del1;
        deep.del2 = del2;
        deep.del3 = del3;
        deep.xlamo = remainder_positive(mo + nodeo + argpo - theta);
        deep.xfact = satellite.mdot + satellite.argpdot + satellite.nodedot - RPTIM
            + deep.dmdt
            + deep.domdt
            + deep.dnodt
            - nm;
    }
}

/// Applies the deep space secular and resonance terms to the mean elements.
///
/// 把深空长期项与共振项施加到平均根数上。
#[allow(clippy::too_many_arguments)]
pub(crate) fn dspace(
    satellite: &Satellite,
    minutes: f64,
    eccentricity: f64,
    argument: f64,
    inclination: f64,
    mean_anomaly: f64,
    node: f64,
    mean_motion: f64,
) -> DeepSpaceState {
    let deep = &satellite.ds;
    let no = satellite.no_unkozai;

    let em = eccentricity + deep.dedt * minutes;
    let inclm = inclination + deep.didt * minutes;
    let argpm = argument + deep.domdt * minutes;
    let nodem = node + deep.dnodt * minutes;
    let mut mm = mean_anomaly + deep.dmdt * minutes;
    let mut nm = mean_motion;

    let theta = remainder_positive(satellite.gsto + minutes * RPTIM);

    // The resonance is integrated in fixed 720 minute steps counted from
    // the epoch. Starting from the epoch on every call rather than carrying
    // the state between calls keeps propagation free of hidden state, and
    // the result is the same because the step boundaries do not depend on
    // the requested time.
    // 共振以自历元起算的固定 720 分钟步长积分。每次调用都从历元重新开始而
    // 不在调用之间传递状态，使传播不含隐藏状态；结果相同，因为步长边界与
    // 所请求的时刻无关。
    let mut atime = 0.0;
    let mut xli = deep.xlamo;
    let mut xni = no;

    if deep.irez != 0 {
        let delta_step = if minutes > 0.0 {
            STEP_POSITIVE
        } else {
            STEP_NEGATIVE
        };

        let mut xndt;
        let mut xnddt;
        let mut xldot;
        loop {
            let (derivative, derivative_derivative) = resonance_derivatives(
                deep,
                satellite.argpo,
                satellite.argpdot,
                atime,
                xli,
                xni,
                deep.irez,
            );
            xndt = derivative;
            xldot = xni + deep.xfact;
            xnddt = derivative_derivative * xldot;

            if (minutes - atime).abs() >= STEP_POSITIVE {
                xli += xldot * delta_step + xndt * STEP_SQUARED;
                xni += xndt * delta_step + xnddt * STEP_SQUARED;
                atime += delta_step;
            } else {
                break;
            }
        }

        let ft = minutes - atime;
        nm = xni + xndt * ft + xnddt * ft * ft * 0.5;
        let xl = xli + xldot * ft + xndt * ft * ft * 0.5;
        if deep.irez == 1 {
            mm = xl - nodem - argpm + theta;
        } else {
            mm = xl - 2.0 * nodem + 2.0 * theta;
        }
        nm = no + (nm - no);
    }

    DeepSpaceState {
        eccentricity: em,
        argument: argpm,
        inclination: inclm,
        mean_anomaly: mm,
        node: nodem,
        mean_motion: nm,
        atime,
        xli,
        xni,
    }
}

/// Returns the first and second derivatives of the resonance angle.
///
/// The two resonance cases use different series. The synchronous case,
/// `irez == 1`, uses three terms that depend only on the angle, while the
/// half-day case uses ten terms that also depend on the argument of perigee.
///
/// 返回共振角的一阶与二阶导数。
///
/// 两种共振情形使用不同的级数。同步情形（`irez == 1`）使用只依赖该角的
/// 三项，而半日情形使用同时依赖近地点幅角的十项。
fn resonance_derivatives(
    deep: &DeepSpace,
    argpo: f64,
    argpdot: f64,
    atime: f64,
    xli: f64,
    _xni: f64,
    irez: i32,
) -> (f64, f64) {
    const FASX2: f64 = 0.131_309_08;
    const FASX4: f64 = 2.884_319_8;
    const FASX6: f64 = 0.374_480_87;
    const G22: f64 = 5.768_639_6;
    const G32: f64 = 0.952_408_98;
    const G44: f64 = 1.801_499_8;
    const G52: f64 = 1.050_833_0;
    const G54: f64 = 4.410_889_8;

    if irez == 2 {
        let xomi = argpo + argpdot * atime;
        let x2omi = xomi + xomi;
        let x2li = xli + xli;
        let xndt = deep.d2201 * (x2omi + xli - G22).sin()
            + deep.d2211 * (xli - G22).sin()
            + deep.d3210 * (xomi + xli - G32).sin()
            + deep.d3222 * (-xomi + xli - G32).sin()
            + deep.d4410 * (x2omi + x2li - G44).sin()
            + deep.d4422 * (x2li - G44).sin()
            + deep.d5220 * (xomi + xli - G52).sin()
            + deep.d5232 * (-xomi + xli - G52).sin()
            + deep.d5421 * (xomi + x2li - G54).sin()
            + deep.d5433 * (-xomi + x2li - G54).sin();
        let xnddt = deep.d2201 * (x2omi + xli - G22).cos()
            + deep.d2211 * (xli - G22).cos()
            + deep.d3210 * (xomi + xli - G32).cos()
            + deep.d3222 * (-xomi + xli - G32).cos()
            + deep.d5220 * (xomi + xli - G52).cos()
            + deep.d5232 * (-xomi + xli - G52).cos()
            + 2.0
                * (deep.d4410 * (x2omi + x2li - G44).cos()
                    + deep.d4422 * (x2li - G44).cos()
                    + deep.d5421 * (xomi + x2li - G54).cos()
                    + deep.d5433 * (-xomi + x2li - G54).cos());
        (xndt, xnddt)
    } else {
        let xndt = deep.del1 * (xli - FASX2).sin()
            + deep.del2 * (2.0 * (xli - FASX4)).sin()
            + deep.del3 * (3.0 * (xli - FASX6)).sin();
        let xnddt = deep.del1 * (xli - FASX2).cos()
            + 2.0 * deep.del2 * (2.0 * (xli - FASX4)).cos()
            + 3.0 * deep.del3 * (3.0 * (xli - FASX6)).cos();
        (xndt, xnddt)
    }
}

/// Applies the deep space long period periodic perturbations.
///
/// The terms are evaluated on the solar and lunar mean anomalies at the
/// requested time, except during initialisation, where they are evaluated at
/// the epoch and stored as the reference values that later evaluations
/// subtract.
///
/// 施加深空长周期项。
///
/// 各项在所请求时刻的太阳与月球平近点角上求值；初始化阶段除外，那是在历元
/// 处求值并存为参考基准，供此后的求值相减。
#[allow(clippy::too_many_arguments)]
pub(crate) fn dpper(
    satellite: &Satellite,
    minutes: f64,
    initialised: bool,
    eccentricity: f64,
    inclination: f64,
    node: f64,
    argument: f64,
    mean_anomaly: f64,
) -> PerturbedElements {
    let deep = &satellite.ds;

    let mut ep = eccentricity;
    let mut inclp = inclination;
    let mut nodep = node;
    let mut argpp = argument;
    let mut mp = mean_anomaly;

    let mut zm = deep.zmos + ZNS * minutes;
    if initialised {
        zm = deep.zmos;
    }
    let mut zf = zm + 2.0 * ZES * zm.sin();
    let sinzf = zf.sin();
    let f2 = 0.5 * sinzf * sinzf - 0.25;
    let f3 = -0.5 * sinzf * zf.cos();
    let ses = deep.se2 * f2 + deep.se3 * f3;
    let sis = deep.si2 * f2 + deep.si3 * f3;
    let sls = deep.sl2 * f2 + deep.sl3 * f3 + deep.sl4 * sinzf;
    let sghs = deep.sgh2 * f2 + deep.sgh3 * f3 + deep.sgh4 * sinzf;
    let shs = deep.sh2 * f2 + deep.sh3 * f3;

    zm = deep.zmol + ZNL * minutes;
    if initialised {
        zm = deep.zmol;
    }
    zf = zm + 2.0 * ZEL * zm.sin();
    let sinzf = zf.sin();
    let f2 = 0.5 * sinzf * sinzf - 0.25;
    let f3 = -0.5 * sinzf * zf.cos();
    let sel = deep.ee2 * f2 + deep.e3 * f3;
    let sil = deep.xi2 * f2 + deep.xi3 * f3;
    let sll = deep.xl2 * f2 + deep.xl3 * f3 + deep.xl4 * sinzf;
    let sghl = deep.xgh2 * f2 + deep.xgh3 * f3 + deep.xgh4 * sinzf;
    let shll = deep.xh2 * f2 + deep.xh3 * f3;

    let mut pe = ses + sel;
    let mut pinc = sis + sil;
    let mut pl = sls + sll;
    let mut pgh = sghs + sghl;
    let mut ph = shs + shll;

    if !initialised {
        pe -= deep.peo;
        pinc -= deep.pinco;
        pl -= deep.plo;
        pgh -= deep.pgho;
        ph -= deep.pho;
        inclp += pinc;
        ep += pe;
        let sinip = inclp.sin();
        let cosip = inclp.cos();

        // The Lyddane modification is used at low inclination, where the
        // node and the argument of perigee are individually ill conditioned.
        // The threshold of 0.2 radians is about 11.5 degrees. The reference
        // applies it against the perturbed inclination.
        // Lyddane 修正在低倾角处使用，那里交点与近地点幅角各自病态。阈值
        // 0.2 弧度约为 11.5 度。参考实现以受扰动后的倾角作判据。
        if inclp >= 0.2 {
            ph /= sinip;
            pgh -= cosip * ph;
            argpp += pgh;
            nodep += ph;
            mp += pl;
        } else {
            let sinop = nodep.sin();
            let cosop = nodep.cos();
            let mut alfdp = sinip * sinop;
            let mut betdp = sinip * cosop;
            let dalf = ph * cosop + pinc * cosip * sinop;
            let dbet = -ph * sinop + pinc * cosip * cosop;
            alfdp += dalf;
            betdp += dbet;
            nodep = remainder_signed(nodep);
            let xls = mp + argpp + pl + pgh + (cosip - pinc * sinip) * nodep;
            let xnoh = nodep;
            nodep = alfdp.atan2(betdp);
            if (xnoh - nodep).abs() > PI {
                if nodep < xnoh {
                    nodep += TWO_PI;
                } else {
                    nodep -= TWO_PI;
                }
            }
            mp += pl;
            argpp = xls - mp - cosip * nodep;
        }
    }

    PerturbedElements {
        eccentricity: ep,
        inclination: inclp,
        node: nodep,
        argument: argpp,
        mean_anomaly: mp,
    }
}
