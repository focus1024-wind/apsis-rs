//! The near-Earth propagation step.
//!
//! Given a satellite record and a time from epoch, this module evaluates the
//! secular and periodic terms of the SGP4 theory and returns the state in
//! the True Equator Mean Equinox frame.
//!
//! The mean elements advance as polynomials in time: the mean anomaly,
//! argument of perigee, and right ascension are linear, the right ascension
//! has an additional quadratic term, and the semi-major axis, eccentricity,
//! and mean anomaly pick up further drag terms for orbits above the
//! simplified threshold. The orbit is then solved for the eccentric anomaly
//! by Newton's method, and the short period terms are applied.
//!
//! Source: the sequence follows Vallado et al., *Revisiting Spacetrack
//! Report #3*, AIAA 2006-6753, 2006, and the reference implementation
//! distributed with that paper.

use crate::error::{PropagationError, PropagationErrorKind};
use crate::satellite::{Method, Satellite, TWO_PI};
use crate::state::State;
use apsis_math::Vec3;

/// The number of iterations allowed when solving for the eccentric anomaly.
///
/// 求解偏近点角时允许的迭代次数。
const KEPLER_MAX_ITERATIONS: usize = 10;

/// The convergence threshold on the eccentric anomaly correction.
///
/// 偏近点角修正的收敛阈值。
const KEPLER_TOLERANCE: f64 = 1.0e-12;

/// The largest correction applied in one Kepler step.
///
/// Clamping keeps the iteration stable for the eccentric orbits that the
/// near circular approximation of the theory handles poorly.
///
/// 设上限使迭代在理论中近圆近似处理不佳的高偏心轨道上保持稳定。
const KEPLER_MAX_STEP: f64 = 0.95;

/// The smallest eccentricity used after the drag terms are applied.
///
/// A value that would round to zero makes several divisions in the short
/// period terms singular, so it is clamped to this floor.
///
/// 会舍入为零的取值会使短周期项中的若干除法奇异，故钳位到此下限。
const MINIMUM_ECCENTRICITY: f64 = 1.0e-6;

/// The divisor used when the inclination is close to pi.
///
/// 倾角接近 pi 时所用的除数。
const INCLINATION_GUARD: f64 = 1.5e-12;

/// Propagates a satellite by a number of minutes from its element epoch.
/// 把某卫星自其根数历元起传播若干分钟。
pub(crate) fn propagate(satellite: &Satellite, minutes: f64) -> Result<State, PropagationError> {
    let mean_anomaly = satellite.mo + satellite.mdot * minutes;
    let secular_argument = satellite.argpo + satellite.argpdot * minutes;
    let secular_node = satellite.nodeo + satellite.nodedot * minutes;

    let time_squared = minutes * minutes;
    let mut node = secular_node + satellite.nodecf * time_squared;

    let mut tempa = 1.0 - satellite.cc1 * minutes;
    let mut tempe = satellite.bstar * satellite.cc4 * minutes;
    let mut templ = satellite.t2cof * time_squared;

    let mut argument = secular_argument;
    let mut anomaly = mean_anomaly;

    if !satellite.isimp {
        let delta_argument = satellite.omgcof * minutes;
        let drag_anomaly = 1.0 + satellite.eta * mean_anomaly.cos();
        let delta_anomaly =
            satellite.xmcof * (drag_anomaly * drag_anomaly * drag_anomaly - satellite.delmo);
        let total = delta_argument + delta_anomaly;
        anomaly = mean_anomaly + total;
        argument = secular_argument - total;
        let time_cubed = time_squared * minutes;
        let time_fourth = time_cubed * minutes;
        tempa -=
            satellite.d2 * time_squared + satellite.d3 * time_cubed + satellite.d4 * time_fourth;
        tempe += satellite.bstar * satellite.cc5 * (anomaly.sin() - satellite.sinmao);
        templ = templ
            + satellite.t3cof * time_cubed
            + time_fourth * (satellite.t4cof + minutes * satellite.t5cof);
    }

    let mut eccentricity = satellite.ecco;
    let mut inclination = satellite.inclo;
    let mut mean_motion = satellite.no_unkozai;

    // The deep space branch replaces the mean elements here.
    // 深空分支在此替换平均根数。
    if satellite.method == Method::DeepSpace {
        let deep = crate::deep_space::dspace(
            satellite,
            minutes,
            eccentricity,
            argument,
            inclination,
            anomaly,
            node,
            mean_motion,
        );
        eccentricity = deep.eccentricity;
        argument = deep.argument;
        inclination = deep.inclination;
        anomaly = deep.mean_anomaly;
        node = deep.node;
        mean_motion = deep.mean_motion;
    }

    if mean_motion <= 0.0 {
        return Err(super::satellite::non_positive_mean_motion(mean_motion));
    }

    // Recover the semi-major axis including the drag scaling, then recompute
    // the mean motion from it so the two stay consistent.
    // 含阻力缩放地还原半长轴，再由它重算平运动，使二者保持一致。
    let semi_major = (satellite.xke() / mean_motion).powf(2.0 / 3.0) * tempa * tempa;
    mean_motion = satellite.xke() / semi_major.powf(1.5);

    eccentricity -= tempe;
    if eccentricity >= 1.0 || eccentricity < -0.001 {
        return Err(PropagationError::new(PropagationErrorKind::InvalidElements)
            .with_detail(format!("propagated eccentricity is {eccentricity}")));
    }
    let eccentricity = eccentricity.max(MINIMUM_ECCENTRICITY);

    anomaly += satellite.no_unkozai * templ;
    let mut longitude = anomaly + argument + node;

    // Reduce the angles to a single turn. The reference implementation uses
    // a remainder that keeps the sign of the dividend for the right
    // ascension and a positive remainder elsewhere; both are reproduced here
    // because the eccentric anomaly iteration is sensitive to the branch.
    // 把各角归约到一周之内。参考实现对赤经采用保留被除数符号的余数，其余
    // 处用正余数；此处两者都予以复现，因为偏近点角迭代对分支敏感。
    node = if node >= 0.0 {
        node % TWO_PI
    } else {
        -((-node) % TWO_PI)
    };
    argument %= TWO_PI;
    longitude %= TWO_PI;
    anomaly = (longitude - argument - node) % TWO_PI;

    let mut perturbed_eccentricity = eccentricity;
    let mut perturbed_inclination = inclination;
    let mut perturbed_node = node;
    let mut perturbed_argument = argument;
    let mut perturbed_anomaly = anomaly;
    let mut sin_perturbed = inclination.sin();
    let mut cos_perturbed = inclination.cos();

    if satellite.method == Method::DeepSpace {
        let result = crate::deep_space::dpper(
            satellite,
            minutes,
            false,
            perturbed_eccentricity,
            perturbed_inclination,
            perturbed_node,
            perturbed_argument,
            perturbed_anomaly,
        );
        perturbed_eccentricity = result.eccentricity;
        perturbed_inclination = result.inclination;
        perturbed_node = result.node;
        perturbed_argument = result.argument;
        perturbed_anomaly = result.mean_anomaly;

        if perturbed_inclination < 0.0 {
            perturbed_inclination = -perturbed_inclination;
            perturbed_node += core::f64::consts::PI;
            perturbed_argument -= core::f64::consts::PI;
        }
        if perturbed_eccentricity < 0.0 || perturbed_eccentricity > 1.0 {
            return Err(
                PropagationError::new(PropagationErrorKind::InvalidElements).with_detail(format!(
                    "perturbed eccentricity is {perturbed_eccentricity}"
                )),
            );
        }

        sin_perturbed = perturbed_inclination.sin();
        cos_perturbed = perturbed_inclination.cos();
    }

    // The `aycof` and `xlcof` used below come from the perturbed
    // inclination in the deep space branch and from the epoch inclination
    // otherwise.
    // 下文所用的 `aycof` 与 `xlcof` 来自受扰动的
    let (aycof, xlcof) = if satellite.method == Method::DeepSpace {
        let aycof = -0.5 * satellite.model.j3_over_j2() * sin_perturbed;
        let xlcof = if (cos_perturbed + 1.0).abs() > INCLINATION_GUARD {
            -0.25 * satellite.model.j3_over_j2() * sin_perturbed * (3.0 + 5.0 * cos_perturbed)
                / (1.0 + cos_perturbed)
        } else {
            -0.25 * satellite.model.j3_over_j2() * sin_perturbed * (3.0 + 5.0 * cos_perturbed)
                / INCLINATION_GUARD
        };
        (aycof, xlcof)
    } else {
        (satellite.aycof, satellite.xlcof)
    };

    let axnl = perturbed_eccentricity * perturbed_argument.cos();
    let temp = 1.0 / (semi_major * (1.0 - perturbed_eccentricity * perturbed_eccentricity));
    let aynl = perturbed_eccentricity * perturbed_argument.sin() + temp * aycof;
    let xl = perturbed_anomaly + perturbed_argument + perturbed_node + temp * xlcof * axnl;

    let u = (xl - perturbed_node) % TWO_PI;

    // Solve Kepler's equation by Newton's method.
    // 用 Newton 法求解开普勒方程。
    let mut eo1 = u;
    let (mut sineo1, mut coseo1);
    let mut iterations = 0;
    loop {
        sineo1 = eo1.sin();
        coseo1 = eo1.cos();
        let denominator = 1.0 - coseo1 * axnl - sineo1 * aynl;
        let mut correction = (u - aynl * coseo1 + axnl * sineo1 - eo1) / denominator;
        if correction.abs() >= KEPLER_MAX_STEP {
            correction = if correction > 0.0 {
                KEPLER_MAX_STEP
            } else {
                -KEPLER_MAX_STEP
            };
        }
        eo1 += correction;
        iterations += 1;
        if correction.abs() < KEPLER_TOLERANCE || iterations >= KEPLER_MAX_ITERATIONS {
            break;
        }
    }
    sineo1 = eo1.sin();
    coseo1 = eo1.cos();

    let ecose = axnl * coseo1 + aynl * sineo1;
    let esine = axnl * sineo1 - aynl * coseo1;
    let el2 = axnl * axnl + aynl * aynl;
    let pl = semi_major * (1.0 - el2);
    if pl < 0.0 {
        return Err(PropagationError::new(PropagationErrorKind::InvalidElements)
            .with_detail(format!("semi-latus rectum is {pl}")));
    }

    let rl = semi_major * (1.0 - ecose);
    let rdotl = semi_major.sqrt() * esine / rl;
    let rvdotl = pl.sqrt() / rl;
    let betal = (1.0 - el2).sqrt();
    let temp = esine / (1.0 + betal);
    let sinu = semi_major / rl * (sineo1 - aynl - axnl * temp);
    let cosu = semi_major / rl * (coseo1 - axnl + aynl * temp);
    let mut su = sinu.atan2(cosu);
    let sin2u = (cosu + cosu) * sinu;
    let cos2u = 1.0 - 2.0 * sinu * sinu;

    let temp = 1.0 / pl;
    let temp1 = 0.5 * satellite.model.j2 * temp;
    let temp2 = temp1 * temp;

    let (con41, x1mth2, x7thm1) = if satellite.method == Method::DeepSpace {
        let cosisq = cos_perturbed * cos_perturbed;
        (3.0 * cosisq - 1.0, 1.0 - cosisq, 7.0 * cosisq - 1.0)
    } else {
        (satellite.con41, satellite.x1mth2, satellite.x7thm1)
    };

    let mrt = rl * (1.0 - 1.5 * temp2 * betal * con41) + 0.5 * temp1 * x1mth2 * cos2u;
    su -= 0.25 * temp2 * x7thm1 * sin2u;
    let xnode = perturbed_node + 1.5 * temp2 * cos_perturbed * sin2u;
    let xinc = perturbed_inclination + 1.5 * temp2 * cos_perturbed * sin_perturbed * cos2u;

    let mvt = rdotl - mean_motion * temp1 * x1mth2 * sin2u / satellite.xke();
    let rvdot = rvdotl + mean_motion * temp1 * (x1mth2 * cos2u + 1.5 * con41) / satellite.xke();

    let sinsu = su.sin();
    let cossu = su.cos();
    let snod = xnode.sin();
    let cnod = xnode.cos();
    let sini = xinc.sin();
    let cosi = xinc.cos();

    let xmx = -snod * cosi;
    let xmy = cnod * cosi;
    let ux = xmx * sinsu + cnod * cossu;
    let uy = xmy * sinsu + snod * cossu;
    let uz = sini * sinsu;
    let vx = xmx * cossu - cnod * sinsu;
    let vy = xmy * cossu - snod * sinsu;
    let vz = sini * cossu;

    let radius = mrt * satellite.model.radius;
    let position = Vec3::new(radius * ux, radius * uy, radius * uz);
    let velocity_scale = satellite.model.radius * satellite.xke() / 60.0;
    let velocity = Vec3::new(
        (mvt * ux + rvdot * vx) * velocity_scale,
        (mvt * uy + rvdot * vy) * velocity_scale,
        (mvt * uz + rvdot * vz) * velocity_scale,
    );

    if mrt < 1.0 {
        return Err(PropagationError::new(PropagationErrorKind::Decayed)
            .with_detail(format!("radius is {mrt} Earth radii")));
    }

    Ok(State {
        position,
        velocity,
        semi_major_axis: semi_major * satellite.model.radius,
    })
}
