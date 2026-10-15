//! The satellite record and its initialisation from mean elements.
//!
//! 卫星记录及其由平根数的初始化。

use crate::apsis_sgp4::error::{PropagationError, PropagationErrorKind};
use crate::apsis_sgp4::init::{self, Elements};
use crate::apsis_sgp4::model::GravityModel;
use crate::apsis_sgp4::state::State;
use crate::apsis_tle::TwoLineElement;
use core::f64::consts::PI;

/// The number of radians in a revolution.
///
/// 一周中的弧度数。
pub(crate) const TWO_PI: f64 = 2.0 * PI;

/// The number of minutes in a day.
///
/// 一天中的分钟数。
pub(crate) const MINUTES_PER_DAY: f64 = 1_440.0;

/// Whether the near-Earth or the deep space branch applies.
///
/// 适用近地分支还是深空分支。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Method {
    /// Period below 225 minutes.
    ///
    /// 周期短于 225 分钟。
    NearEarth,
    /// Period at or above 225 minutes.
    ///
    /// 周期不短于 225 分钟。
    DeepSpace,
}

/// A satellite prepared for propagation.
///
/// Construct one with [`Satellite::from_element`], then call
/// [`Satellite::propagate_minutes`] or [`Satellite::propagate_at`] for each
/// instant of interest. The initialisation work, including the mean motion
/// correction and every coefficient that depends only on the elements, is
/// done once at construction.
///
/// 为传播而准备好的卫星。
///
/// 用 [`Satellite::from_element`] 构造一个，然后对每个关心的瞬时调用
/// [`Satellite::propagate_minutes`] 或 [`Satellite::propagate_at`]。初始化工作
/// ——包括平运动修正以及所有只依赖根数的系数——在构造时完成一次。
#[derive(Debug, Clone)]
pub struct Satellite {
    /// The model this satellite was initialised against.
    ///
    /// 该卫星初始化时所用的模型。
    pub(crate) model: GravityModel,
    /// Corrected mean motion, in radians per minute.
    ///
    /// 修正后的平运动，单位为弧度每分钟。
    pub(crate) no_unkozai: f64,
    /// Eccentricity.
    ///
    /// 偏心率。
    pub(crate) ecco: f64,
    /// Inclination, in radians.
    ///
    /// 倾角，单位为弧度。
    pub(crate) inclo: f64,
    /// Right ascension of the ascending node, in radians.
    ///
    /// 升交点赤经，单位为弧度。
    pub(crate) nodeo: f64,
    /// Argument of perigee, in radians.
    ///
    /// 近地点幅角，单位为弧度。
    pub(crate) argpo: f64,
    /// Mean anomaly, in radians.
    ///
    /// 平近点角，单位为弧度。
    pub(crate) mo: f64,
    /// `BSTAR` drag term, in inverse Earth radii.
    ///
    /// `BSTAR` 阻力项，单位为地球半径的倒数。
    pub(crate) bstar: f64,
    /// Epoch as a Modified Julian Date on the TAI scale.
    ///
    /// 历元，为 TAI 标度上的修正儒略日（Modified Julian Date）。
    pub(crate) epoch_mjd: f64,
    /// Which branch of the theory applies.
    ///
    /// 适用理论的哪一分支。
    pub(crate) method: Method,
    /// Set when the short period terms are omitted.
    ///
    /// 省略短周期项时置位。
    pub(crate) isimp: bool,

    /// `3 cos^2 i - 1`.
    ///
    /// `3 cos^2 i - 1`。
    pub(crate) con41: f64,
    /// `1 - cos^2 i`.
    ///
    /// `1 - cos^2 i`。
    pub(crate) x1mth2: f64,
    /// `7 cos^2 i - 1`.
    ///
    /// `7 cos^2 i - 1`。
    pub(crate) x7thm1: f64,
    /// `-0.5 j3 / j2 sin i`.
    ///
    /// `-0.5 j3 / j2 sin i`。
    pub(crate) aycof: f64,
    /// The `j3` contribution to the argument of latitude.
    ///
    /// `j3` 对纬度幅角的贡献。
    pub(crate) xlcof: f64,

    /// First drag coefficient.
    ///
    /// 第一阻力系数。
    pub(crate) cc1: f64,
    /// Fourth drag coefficient.
    ///
    /// 第四阻力系数。
    pub(crate) cc4: f64,
    /// Fifth drag coefficient.
    ///
    /// 第五阻力系数。
    pub(crate) cc5: f64,
    /// Second order drag coefficient.
    ///
    /// 二阶阻力系数。
    pub(crate) d2: f64,
    /// Third order drag coefficient.
    ///
    /// 三阶阻力系数。
    pub(crate) d3: f64,
    /// Fourth order drag coefficient.
    ///
    /// 四阶阻力系数。
    pub(crate) d4: f64,
    /// `(1 + eta cos M)^3` at epoch.
    ///
    /// 历元处的 `(1 + eta cos M)^3`。
    pub(crate) delmo: f64,
    /// `eta`.
    ///
    /// `eta`。
    pub(crate) eta: f64,
    /// Contribution of drag to the argument of perigee.
    ///
    /// 阻力对近地点幅角的贡献。
    pub(crate) omgcof: f64,
    /// `sin M` at epoch.
    ///
    /// 历元处的 `sin M`。
    pub(crate) sinmao: f64,
    /// Quadratic coefficient of the time polynomial.
    ///
    /// 时间多项式的二次项系数。
    pub(crate) t2cof: f64,
    /// Cubic coefficient of the time polynomial.
    ///
    /// 时间多项式的三次项系数。
    pub(crate) t3cof: f64,
    /// Quartic coefficient of the time polynomial.
    ///
    /// 时间多项式的四次项系数。
    pub(crate) t4cof: f64,
    /// Quintic coefficient of the time polynomial.
    ///
    /// 时间多项式的五次项系数。
    pub(crate) t5cof: f64,
    /// Contribution of drag to the mean anomaly.
    ///
    /// 阻力对平近点角的贡献。
    pub(crate) xmcof: f64,

    /// Rate of change of the mean anomaly, in radians per minute.
    ///
    /// 平近点角的变化率，单位为弧度每分钟。
    pub(crate) mdot: f64,
    /// Rate of change of the argument of perigee, in radians per minute.
    ///
    /// 近地点幅角的变化率，单位为弧度每分钟。
    pub(crate) argpdot: f64,
    /// Rate of change of the right ascension, in radians per minute.
    ///
    /// 赤经的变化率，单位为弧度每分钟。
    pub(crate) nodedot: f64,
    /// Quadratic term of the right ascension.
    ///
    /// 赤经的二次项。
    pub(crate) nodecf: f64,

    /// Greenwich sidereal time at the epoch, in radians.
    ///
    /// 历元处的格林尼治恒星时，单位为弧度。
    pub(crate) gsto: f64,

    /// Deep space resonance coefficients, all zero for near-Earth orbits.
    ///
    /// 深空共振系数，近地轨道全为零。
    pub(crate) ds: crate::apsis_sgp4::deep_space::DeepSpace,
}

impl Satellite {
    /// Initialises a satellite from a parsed element set.
    ///
    /// Returns `None` when the mean elements do not describe an orbit the
    /// model can propagate, for example when the eccentricity is at least
    /// one. A caller that needs the reason should use
    /// [`Satellite::try_from_element`].
    ///
    /// 由已解析的根数集初始化卫星。
    ///
    /// 当平均根数不描述模型可传播的轨道时返回 `None`，例如偏心率不小于一
    /// 的情形。需要知道原因的调用方应使用 [`Satellite::try_from_element`]。
    #[must_use]
    pub fn from_element(element: &TwoLineElement, model: GravityModel) -> Option<Self> {
        Self::try_from_element(element, model).ok()
    }

    /// Initialises a satellite from a parsed element set.
    ///
    /// Returns an error describing which element is not propagatable when
    /// the elements are rejected.
    ///
    /// 由已解析的根数集初始化卫星。
    ///
    /// 当根数被拒绝时，返回一个说明哪个根数不可传播的错误。
    pub fn try_from_element(
        element: &TwoLineElement,
        model: GravityModel,
    ) -> Result<Self, PropagationError> {
        let elements = Elements::from_tle(element).ok_or_else(|| {
            PropagationError::new(PropagationErrorKind::InvalidElements)
                .with_detail("element set epoch is outside the leap second table")
        })?;
        Self::try_from_elements(&elements, model)
    }

    /// Initialises a satellite from mean elements already in model units.
    ///
    /// 由已按模型单位表示的平均根数初始化卫星。
    pub fn try_from_elements(
        elements: &Elements,
        model: GravityModel,
    ) -> Result<Self, PropagationError> {
        init::validate(elements)?;
        Ok(Self::initialise(elements, model))
    }

    /// Returns the epoch of the elements as a Modified Julian Date on the
    /// TAI scale.
    ///
    /// 返回根数历元，为 TAI 标度上的修正儒略日（Modified Julian Date）。
    #[must_use]
    pub fn epoch_mjd(&self) -> f64 {
        self.epoch_mjd
    }

    /// Returns `true` when the deep space branch applies.
    ///
    /// 当深空分支适用时返回 `true`。
    #[must_use]
    pub fn is_deep_space(&self) -> bool {
        self.method == Method::DeepSpace
    }

    /// Returns the number of minutes from the element set epoch to the
    /// given instant.
    ///
    /// The instant is given as a Modified Julian Date on the TAI scale, the
    /// same scale as [`Satellite::epoch_mjd`].
    ///
    /// 返回自根数集历元到给定瞬时的分钟数。
    ///
    /// 该瞬时以 TAI 标度上的修正儒略日（Modified Julian Date）给出，与
    /// [`Satellite::epoch_mjd`] 同一标度。
    #[must_use]
    pub fn minutes_since_epoch(&self, tai_mjd: f64) -> f64 {
        (tai_mjd - self.epoch_mjd) * MINUTES_PER_DAY
    }

    /// Returns the state of the satellite at a given instant.
    ///
    /// The instant is a Modified Julian Date on the TAI scale.
    ///
    /// 返回卫星在某瞬时的状态。
    ///
    /// 该瞬时是 TAI 时标上的约化儒略日。
    pub fn propagate_at(&self, tai_mjd: f64) -> Result<State, PropagationError> {
        self.propagate_minutes(self.minutes_since_epoch(tai_mjd))
    }

    /// Returns the state of the satellite at a number of minutes from the
    /// element set epoch.
    ///
    /// Negative values propagate backwards. The result is in the True
    /// Equator Mean Equinox frame, with position in kilometers and velocity
    /// in kilometers per second.
    ///
    /// Returns [`PropagationErrorKind::Decayed`](crate::apsis_sgp4::PropagationErrorKind::Decayed)
    /// when the propagated radius falls below one Earth radius, which the
    /// model treats as the satellite having re-entered.
    ///
    /// 返回卫星距根数集历元若干分钟处的状态。
    ///
    /// 负值向后传播。结果在真赤道平春分点坐标系中，位置以千米计、速度以千米
    /// 每秒计。
    ///
    /// 当传播所得半径低于一个地球半径时返回
    /// [`PropagationErrorKind::Decayed`](crate::apsis_sgp4::PropagationErrorKind::Decayed)，
    /// 模型将其视为卫星已经再入。
    pub fn propagate_minutes(&self, minutes: f64) -> Result<State, PropagationError> {
        crate::apsis_sgp4::propagate::propagate(self, minutes)
    }

    /// Returns the Kepler constant of the model, in radians per minute per
    /// Earth radius to the three halves.
    ///
    /// 返回模型的开普勒常数，单位为弧度每分每地球半径的三分之二次方。
    #[must_use]
    pub fn xke(&self) -> f64 {
        self.model.xke()
    }
}

impl Satellite {
    /// Builds the record, including every coefficient that depends only on
    /// the epoch elements.
    ///
    /// 构建该记录，含所有只依赖历元根数的系数。
    fn initialise(elements: &Elements, model: GravityModel) -> Self {
        // The divisor guards the singularity at inclination pi, where the
        // expression is finite in the limit. `1.5e-12` is the threshold used
        // by the reference implementation.
        // 该除数用于保护倾角为 pi 处的奇点，该处表达式在极限下有限。`1.5e-12`
        // 是参考实现所用的阈值。
        const INCLINATION_GUARD: f64 = 1.5e-12;

        let xke = model.xke();
        let j2 = model.j2;
        let j4 = model.j4;

        let ecco = elements.eccentricity;
        let inclo = elements.inclination;
        let no_kozai = elements.mean_motion;

        let no_unkozai = init::unkozai_mean_motion(no_kozai, ecco, inclo, model);
        let method = if init::is_deep_space(no_unkozai) {
            Method::DeepSpace
        } else {
            Method::NearEarth
        };

        let cosio = inclo.cos();
        let sinio = inclo.sin();
        let cosio2 = cosio * cosio;
        let cosio4 = cosio2 * cosio2;
        let omeosq = 1.0 - ecco * ecco;
        let rteosq = omeosq.sqrt();

        let ao = (xke / no_unkozai).powf(2.0 / 3.0);
        let po = ao * omeosq;
        let posq = po * po;
        let rp = ao * (1.0 - ecco);
        let con42 = 1.0 - 5.0 * cosio2;
        let con41 = -con42 - cosio2 - cosio2;

        // The aerodynamic parameters. `sfour` is the effective radius at
        // which the atmosphere is treated as having constant density, and
        // `qzms24` is the corresponding density scale raised to the fourth
        // power.
        // 空气动力学参数。`sfour` 是把大气视为密度恒定时的有效半径，
        // `qzms24` 是相应密度尺度的四次方。
        let ss = 78.0 / model.radius + 1.0;
        let qzms2t = ((120.0 - 78.0) / model.radius).powi(4);

        let mut isimp = false;
        if rp < 220.0 / model.radius + 1.0 {
            isimp = true;
        }
        let mut sfour = ss;
        let mut qzms24 = qzms2t;
        let perige = (rp - 1.0) * model.radius;
        if perige < 156.0 {
            sfour = perige - 78.0;
            if perige < 98.0 {
                sfour = 20.0;
            }
            qzms24 = ((120.0 - sfour) / model.radius).powi(4);
            sfour = sfour / model.radius + 1.0;
        }

        let pinvsq = 1.0 / posq;
        let tsi = 1.0 / (ao - sfour);
        let eta = ao * ecco * tsi;
        let etasq = eta * eta;
        let eeta = ecco * eta;
        let psisq = (1.0 - etasq).abs();
        let coef = qzms24 * tsi.powi(4);
        let coef1 = coef / psisq.powf(3.5);

        let cc2 = coef1
            * no_unkozai
            * (ao * (1.0 + 1.5 * etasq + eeta * (4.0 + etasq))
                + 0.375 * j2 * tsi / psisq * con41 * (8.0 + 3.0 * etasq * (8.0 + etasq)));
        let cc1 = elements.bstar * cc2;

        let mut cc3 = 0.0;
        if ecco > 1.0e-4 {
            cc3 = -2.0 * coef * tsi * model.j3_over_j2() * no_unkozai * sinio / ecco;
        }
        let x1mth2 = 1.0 - cosio2;
        let cc4 = 2.0
            * no_unkozai
            * coef1
            * ao
            * omeosq
            * (eta * (2.0 + 0.5 * etasq) + ecco * (0.5 + 2.0 * etasq)
                - j2 * tsi / (ao * psisq)
                    * (-3.0 * con41 * (1.0 - 2.0 * eeta + etasq * (1.5 - 0.5 * eeta))
                        + 0.75
                            * x1mth2
                            * (2.0 * etasq - eeta * (1.0 + etasq))
                            * (2.0 * elements.argument_of_perigee).cos()));
        let cc5 = 2.0 * coef1 * ao * omeosq * (1.0 + 2.75 * (etasq + eeta) + eeta * etasq);

        let temp1 = 1.5 * j2 * pinvsq * no_unkozai;
        let temp2 = 0.5 * temp1 * j2 * pinvsq;
        let temp3 = -0.46875 * j4 * pinvsq * pinvsq * no_unkozai;
        let mdot = no_unkozai
            + 0.5 * temp1 * rteosq * con41
            + 0.0625 * temp2 * rteosq * (13.0 - 78.0 * cosio2 + 137.0 * cosio4);
        let argpdot = -0.5 * temp1 * con42
            + 0.0625 * temp2 * (7.0 - 114.0 * cosio2 + 395.0 * cosio4)
            + temp3 * (3.0 - 36.0 * cosio2 + 49.0 * cosio4);
        let xhdot1 = -temp1 * cosio;
        let nodedot = xhdot1
            + (0.5 * temp2 * (4.0 - 19.0 * cosio2) + 2.0 * temp3 * (3.0 - 7.0 * cosio2)) * cosio;

        let omgcof = elements.bstar * cc3 * elements.argument_of_perigee.cos();
        let mut xmcof = 0.0;
        if ecco > 1.0e-4 {
            xmcof = -2.0 / 3.0 * coef * elements.bstar / eeta;
        }
        let nodecf = 3.5 * omeosq * xhdot1 * cc1;
        let t2cof = 1.5 * cc1;

        let xlcof = if (cosio + 1.0).abs() > INCLINATION_GUARD {
            -0.25 * model.j3_over_j2() * sinio * (3.0 + 5.0 * cosio) / (1.0 + cosio)
        } else {
            -0.25 * model.j3_over_j2() * sinio * (3.0 + 5.0 * cosio) / INCLINATION_GUARD
        };
        let aycof = -0.5 * model.j3_over_j2() * sinio;

        let delmotemp = 1.0 + eta * elements.mean_anomaly.cos();
        let delmo = delmotemp * delmotemp * delmotemp;
        let sinmao = elements.mean_anomaly.sin();
        let x7thm1 = 7.0 * cosio2 - 1.0;

        let mut satellite = Self {
            model,
            no_unkozai,
            ecco,
            inclo,
            nodeo: elements.right_ascension,
            argpo: elements.argument_of_perigee,
            mo: elements.mean_anomaly,
            bstar: elements.bstar,
            epoch_mjd: elements.epoch_mjd,
            method,
            isimp,
            con41,
            x1mth2,
            x7thm1,
            aycof,
            xlcof,
            cc1,
            cc4,
            cc5,
            d2: 0.0,
            d3: 0.0,
            d4: 0.0,
            delmo,
            eta,
            omgcof,
            sinmao,
            t2cof,
            t3cof: 0.0,
            t4cof: 0.0,
            t5cof: 0.0,
            xmcof,
            mdot,
            argpdot,
            nodedot,
            nodecf,
            gsto: gstime(elements.epoch_mjd + 2_400_000.5),
            ds: crate::apsis_sgp4::deep_space::DeepSpace::ZERO,
        };

        // The higher order drag terms are omitted for the simplified branch.
        // 简化分支省略高阶阻力项。
        if !isimp {
            let cc1sq = cc1 * cc1;
            let d2 = 4.0 * ao * tsi * cc1sq;
            let temp = d2 * tsi * cc1 / 3.0;
            let d3 = (17.0 * ao + sfour) * temp;
            let d4 = 0.5 * temp * ao * tsi * (221.0 * ao + 31.0 * sfour) * cc1;
            satellite.d2 = d2;
            satellite.d3 = d3;
            satellite.d4 = d4;
            satellite.t3cof = d2 + 2.0 * cc1sq;
            satellite.t4cof = 0.25 * (3.0 * d3 + cc1 * (12.0 * d2 + 10.0 * cc1sq));
            satellite.t5cof = 0.2
                * (3.0 * d4 + 12.0 * cc1 * d3 + 6.0 * d2 * d2 + 15.0 * cc1sq * (2.0 * d2 + cc1sq));
        }

        if method == Method::DeepSpace {
            satellite.isimp = true;
            satellite.ds = crate::apsis_sgp4::deep_space::initialise(&satellite);
        }

        satellite
    }
}

/// Returns Greenwich sidereal time for a UT1 Julian Date, in radians.
///
/// The expression is the one used by the reference implementation. It
/// expands the ratio of universal time to sidereal time and is accurate for
/// the purpose of initialising the deep space resonance, which is the only
/// place the value is used.
///
/// Source: Vallado et al., *Revisiting Spacetrack Report #3*, AIAA
/// 2006-6753, 2006.
///
/// 返回 UT1 儒略日对应的格林尼治恒星时，单位为弧度。
///
/// 该表达式取自参考实现。它展开了世界时与恒星时之比，用其初始化深空共振已足够
/// 精确，而该值仅在该处使用。
///
/// 出处：Vallado 等，*Revisiting Spacetrack Report #3*，AIAA 2006-6753，2006。
pub(crate) fn gstime(julian_date_ut1: f64) -> f64 {
    let centuries = (julian_date_ut1 - 2_451_545.0) / 36_525.0;
    let mut seconds = -6.2e-6 * centuries * centuries * centuries
        + 0.093_104 * centuries * centuries
        + (876_600.0 * 3_600.0 + 8_640_184.812_866) * centuries
        + 67_310.548_41;
    seconds = (seconds * (PI / 180.0) / 240.0) % TWO_PI;
    if seconds < 0.0 {
        seconds += TWO_PI;
    }
    seconds
}

/// Returns the propagation error for a mean motion that is not positive.
///
/// 返回平运动不为正时的传播错误。
pub(crate) fn non_positive_mean_motion(mean_motion: f64) -> PropagationError {
    PropagationError::new(PropagationErrorKind::InvalidElements)
        .with_detail(format!("propagated mean motion is {mean_motion}"))
}
