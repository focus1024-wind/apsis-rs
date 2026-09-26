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

use core::f64::consts::PI;

/// The maximum number of Newton iterations used to solve Kepler's equation.
const MAX_ITERATIONS: usize = 30;

/// The convergence tolerance of the Newton iteration, in radians.
const TOLERANCE: f64 = 1e-14;

/// Reduces an angle into `[-pi, pi)`.
pub fn wrap_to_pi(angle: f64) -> f64 {
    let wrapped = (angle + PI).rem_euclid(2.0 * PI) - PI;
    // `rem_euclid` can return exactly pi for an input just below pi, which
    // the half-open interval excludes.
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
pub fn true_from_eccentric(eccentric_anomaly: f64, eccentricity: f64) -> f64 {
    let half = eccentric_anomaly / 2.0;
    let factor = ((1.0 + eccentricity) / (1.0 - eccentricity)).sqrt();
    wrap_to_pi(2.0 * (factor * half.tan()).atan())
}

/// Returns the eccentric anomaly for a true anomaly and eccentricity.
///
/// This is the inverse of [`true_from_eccentric`].
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
pub fn eccentric_from_mean(mean_anomaly: f64, eccentricity: f64) -> f64 {
    let mean = wrap_to_pi(mean_anomaly);
    // A first order correction to the circular guess.
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
pub fn mean_from_eccentric(eccentric_anomaly: f64, eccentricity: f64) -> f64 {
    wrap_to_pi(eccentric_anomaly - eccentricity * eccentric_anomaly.sin())
}

/// Returns the mean anomaly for a true anomaly and eccentricity.
///
/// The true anomaly is first converted to the eccentric anomaly, and the
/// mean anomaly then follows from Kepler's equation. Going through the
/// eccentric anomaly avoids the separate closed form for the swept area and
/// keeps a single expression responsible for the relation.
pub fn mean_from_true(true_anomaly: f64, eccentricity: f64) -> f64 {
    let eccentric_anomaly = eccentric_from_true(true_anomaly, eccentricity);
    mean_from_eccentric(eccentric_anomaly, eccentricity)
}

/// Returns the true anomaly for a mean anomaly and eccentricity.
pub fn true_from_mean(mean_anomaly: f64, eccentricity: f64) -> f64 {
    true_from_eccentric(
        eccentric_from_mean(mean_anomaly, eccentricity),
        eccentricity,
    )
}
