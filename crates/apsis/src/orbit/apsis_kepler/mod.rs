//! Keplerian orbital elements and two-body propagation.
//!
//! A Keplerian element set describes an orbit as an ellipse, a parabola, or
//! a hyperbola with a fixed orientation in space. From the six elements and
//! the gravitational parameter, this module computes the position and
//! velocity at any time under the two-body assumption, and converts in the
//! other direction.
//!
//! The two-body model includes only the point mass gravity of the central
//! body. It omits the oblateness of the Earth, drag, and third body
//! perturbations, so it drifts from reality over time. It is the right model
//! for analysing an orbit whose elements are already known, for example to
//! find the radius at a true anomaly, and the wrong model for propagating a
//! two-line element set, which is the job of `apsis_sgp4`.
//!
//! Angles are in radians. Lengths are kilometers and the gravitational
//! parameter is in cubic kilometers per second squared, matching the rest of
//! the orbit layer.
//!
//! ```
//! use apsis::apsis_kepler::{GravityParameter, KeplerianElements};
//!
//! // A circular orbit at the Earth's equatorial radius has the speed
//! // sqrt(mu / r), about 7.9 km/s.
//! let elements = KeplerianElements {
//!     semi_major_axis: 6_378.137,
//!     eccentricity: 0.0,
//!     inclination: 0.0,
//!     right_ascension: 0.0,
//!     argument_of_perigee: 0.0,
//!     true_anomaly: 0.0,
//! };
//! let state = elements.to_state(GravityParameter::WGS84).expect("valid elements");
//! assert!((state.velocity.norm() - 7.909).abs() < 0.01);
//! ```
//!
//! 开普勒轨道根数与二体传播。
//!
//! 开普勒根数集把轨道描述为空间中具有固定取向的椭圆、抛物线或双曲线。由
//! 六个根数与引力参数，本模块在二体假定下计算任意时刻的位置与速度，并
//! 作反方向换算。
//!
//! 二体模型只包含中心天体的质点引力。它略去地球扁率、阻力与第三体摄动，故
//! 随时间偏离现实。对分析已知根数的轨道（例如求真近点角处的半径），它是恰当
//! 的模型；对传播两行根数集，它是错误的模型，后者是 `apsis_sgp4` 的事。
//!
//! 角度以弧度计。长度以千米计，引力参数以立方千米每平方秒计，与 orbit 层
//! 其余部分一致。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod anomaly;
mod elements;
mod parameter;

pub use anomaly::{
    eccentric_from_mean, eccentric_from_true, mean_from_eccentric, mean_from_true,
    true_from_eccentric, true_from_mean, wrap_to_pi,
};
pub use elements::{KeplerianElements, State, perifocal_to_inertial};
pub use parameter::GravityParameter;
