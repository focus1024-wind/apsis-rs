//! Conversions between the three anomalies.
//!
//! An anomaly is an angle that measures how far a satellite has travelled
//! around its orbit. The three forms used here are:
//!
//! - The true anomaly, the angle from perigee to the satellite as seen from
//!   the focus where the central body sits. It is the only one with a direct
//!   geometric meaning.
//! - The eccentric anomaly, the angle from perigee to the projection of the
//!   satellite onto the auxiliary circle of the ellipse, as seen from the
//!   center of the ellipse.
//! - The mean anomaly, a linear function of time that is proportional to the
//!   area swept since perigee. It has no direct geometric meaning, but it is
//!   the form in which an orbit propagates in time.
//!
//! Kepler's equation relates the mean and eccentric anomalies and has no
//! closed form solution. It is solved here by Newton's method from a
//! starting guess that is accurate enough to converge in a few iterations
//! for every eccentricity below one.
//!
//! All angles are in radians. The functions reduce their results into
//! `[-pi, pi)` so that a value is comparable without further wrapping.
//!
//! 三种近点角之间的换算。
//!
//! 近点角是度量卫星绕其轨道走了多远的角。此处所用的三种形式是：
//!
//! - 真近点角，自近地点到卫星的角，从中心天体所在的焦点看。它是唯一具有
//!   直接几何意义者。
//! - 偏近点角，自近地点到卫星在椭圆辅助圆上的投影的角，从椭圆中心看。
//! - 平近点角，时间的线性函数，与自近地点起扫过的面积成正比。它没有直接的
//!   几何意义，但它是轨道随时间传播所采用的形式。
//!
//! 开普勒方程联系平近点角与偏近点角，且没有闭式解。此处用牛顿法求解，其
//! 初值足够准确，对小于一的每个偏心率都能在几次迭代内收敛。
//!
//! 所有角度都以弧度计。各函数把结果归约到 `[-pi, pi)`，使值无需再作环绕即可
//! 比较。

use core::f64::consts::PI;

/// The maximum number of Newton iterations used to solve Kepler's equation.
///
/// 求解开普勒方程所用的 Newton 迭代最大步数。
const MAX_ITERATIONS: usize = 30;

/// The convergence tolerance of the Newton iteration, in radians.
///
/// Newton 迭代的收敛容差，单位为弧度。
const TOLERANCE: f64 = 1e-14;

/// Reduces an angle into `[-pi, pi)`.
///
/// 把角度归约到 `[-pi, pi)`。
pub fn wrap_to_pi(angle: f64) -> f64 {
    let wrapped = (angle + PI).rem_euclid(2.0 * PI) - PI;
    // `rem_euclid` can return exactly pi for an input just below pi, which
    // the half-open interval excludes.
    // 对略小于 pi 的输入，`rem_euclid` 可能恰好返回 pi，而半开区间不含该端点。
    if wrapped >= PI {
        wrapped - 2.0 * PI
    } else {
        wrapped
    }
}

/// Returns the true anomaly for an eccentric anomaly and eccentricity.
///
/// The relation follows from the geometry of the auxiliary circle:
///
/// ```text
/// tan(nu / 2) = sqrt((1 + e) / (1 - e)) tan(E / 2)
/// ```
///
/// The half angle form is used rather than the direct cosine relation so
/// that the result is correct in all four quadrants without a quadrant
/// check.
///
/// 由偏近点角与偏心率返回真近点角。
///
/// 该关系来自辅助圆的几何性质：
///
/// ```text
/// tan(nu / 2) = sqrt((1 + e) / (1 - e)) tan(E / 2)
/// ```
///
/// 采用半角形式而非直接的余弦关系，使结果在四个象限中都正确，无需判别象限。
pub fn true_from_eccentric(eccentric_anomaly: f64, eccentricity: f64) -> f64 {
    let half = eccentric_anomaly / 2.0;
    let factor = ((1.0 + eccentricity) / (1.0 - eccentricity)).sqrt();
    wrap_to_pi(2.0 * (factor * half.tan()).atan())
}

/// Returns the eccentric anomaly for a true anomaly and eccentricity.
///
/// This is the inverse of [`true_from_eccentric`].
///
/// 由真近点角与偏心率返回偏近点角。
///
/// 这是 [`true_from_eccentric`] 的逆运算。
pub fn eccentric_from_true(true_anomaly: f64, eccentricity: f64) -> f64 {
    let half = true_anomaly / 2.0;
    let factor = ((1.0 - eccentricity) / (1.0 + eccentricity)).sqrt();
    wrap_to_pi(2.0 * (factor * half.tan()).atan())
}

/// Returns the eccentric anomaly for a mean anomaly and eccentricity.
///
/// This solves Kepler's equation
///
/// ```text
/// M = E - e sin E
/// ```
///
/// by Newton's method. The starting guess is the mean anomaly plus a term
/// proportional to the eccentricity, which keeps the iteration stable for
/// eccentricities approaching one. The iteration always terminates, and the
/// result is the value reached after at most a fixed number of steps.
///
/// 由平近点角与偏心率返回偏近点角。
///
/// 用 Newton 法求解开普勒方程
///
/// ```text
/// M = E - e sin E
/// ```
///
/// 初值取平近点角加上一个正比于偏心率的项，使迭代在偏心率趋近一时保持稳定。
/// 迭代总会终止，结果是至多固定步数后达到的值。
pub fn eccentric_from_mean(mean_anomaly: f64, eccentricity: f64) -> f64 {
    let mean = wrap_to_pi(mean_anomaly);
    // A first order correction to the circular guess.
    // 对圆轨道猜测的一阶修正。
    let mut eccentric = mean + eccentricity * mean.sin() * (1.0 + eccentricity * mean.cos());
    for _ in 0..MAX_ITERATIONS {
        let residual = eccentric - eccentricity * eccentric.sin() - mean;
        let derivative = 1.0 - eccentricity * eccentric.cos();
        if derivative.abs() < f64::MIN_POSITIVE {
            break;
        }
        let step = residual / derivative;
        eccentric -= step;
        if step.abs() < TOLERANCE {
            break;
        }
    }
    wrap_to_pi(eccentric)
}

/// Returns the mean anomaly for an eccentric anomaly and eccentricity.
///
/// This is Kepler's equation evaluated directly, `M = E - e sin E`.
///
/// 由偏近点角与偏心率返回平近点角。
///
/// 这是直接求值的开普勒方程，`M = E - e sin E`。
pub fn mean_from_eccentric(eccentric_anomaly: f64, eccentricity: f64) -> f64 {
    wrap_to_pi(eccentric_anomaly - eccentricity * eccentric_anomaly.sin())
}

/// Returns the mean anomaly for a true anomaly and eccentricity.
///
/// The true anomaly is first converted to the eccentric anomaly, and the
/// mean anomaly then follows from Kepler's equation. Going through the
/// eccentric anomaly avoids the separate closed form for the swept area and
/// keeps a single expression responsible for the relation.
///
/// 由真近点角与偏心率返回平近点角。
///
/// 先把真近点角转为偏近点角，再由开普勒方程得平近点角。绕经偏近点角避免了
/// 另立扫过面积的闭式解，使该关系只由一个表达式负责。
pub fn mean_from_true(true_anomaly: f64, eccentricity: f64) -> f64 {
    let eccentric_anomaly = eccentric_from_true(true_anomaly, eccentricity);
    mean_from_eccentric(eccentric_anomaly, eccentricity)
}

/// Returns the true anomaly for a mean anomaly and eccentricity.
///
/// 由平近点角与偏心率返回真近点角。
pub fn true_from_mean(mean_anomaly: f64, eccentricity: f64) -> f64 {
    true_from_eccentric(
        eccentric_from_mean(mean_anomaly, eccentricity),
        eccentricity,
    )
}
