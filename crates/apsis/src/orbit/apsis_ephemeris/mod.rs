//! Ephemeris sampling and interpolation.
//!
//! A propagator gives the state of a satellite at any instant, but calling
//! it millions of times is wasteful when the result is only needed to draw a
//! track or to search for a crossing. This module records the state at a
//! grid of times, called samples, and answers queries between them by
//! interpolation.
//!
//! The two halves are deliberately independent:
//!
//! - [`Ephemeris`] is a table of times and states. It knows nothing about
//!   where the states came from, so a table propagated by SGP4, computed by
//!   two-body theory, or read from a file all work the same way.
//! - [`Sampler`] fills a table by calling a function that produces a state
//!   from a time. In this workspace that function is normally
//!   `crate::apsis_sgp4::Satellite::propagate_at`, but the sampler only requires
//!   a closure, so a test can use an analytic function.
//!
//! Interpolation is Hermite cubic, which uses both the position and the
//! velocity at each end of an interval. Using the velocity is what makes it
//! accurate: a cubic through two positions alone has no information about
//! how fast the satellite is moving, while Hermite reconstruction matches
//! the slope at both ends and so reproduces a smooth orbit far better for
//! the same sample spacing. On a circular orbit the error is of order the
//! fourth power of the step, so halving the step cuts the error about
//! sixteen fold.
//!
//! Times are in seconds from an origin chosen by the caller, matching the
//! rest of the orbit layer. Positions are in kilometers and velocities in
//! kilometers per second.
//!
//! ```
//! use apsis::apsis_ephemeris::{Sample, Sampler};
//! use apsis::apsis_kepler::{GravityParameter, KeplerianElements};
//!
//! // A circular orbit at the Earth's equatorial radius.
//! let elements = KeplerianElements {
//!     semi_major_axis: 7_000.0,
//!     eccentricity: 0.0,
//!     inclination: 0.9,
//!     right_ascension: 0.0,
//!     argument_of_perigee: 0.0,
//!     true_anomaly: 0.0,
//! };
//! let state_at = |seconds: f64| {
//!     let mean = 2.0 * core::f64::consts::PI * seconds
//!         / elements.period(GravityParameter::WGS84);
//!     let state = elements
//!         .with_mean_anomaly(mean)
//!         .to_state(GravityParameter::WGS84)
//!         .expect("the orbit is valid");
//!     Sample::new(seconds, state.position, state.velocity)
//! };
//!
//! let sampler = Sampler::new(0.0, 600.0, 60.0);
//! let ephemeris = sampler.sample(state_at).expect("the grid is usable");
//! assert_eq!(ephemeris.len(), 11);
//!
//! // A query between two samples is interpolated.
//! let state = ephemeris.state_at(30.0).expect("the time is covered");
//! assert!(state.position.norm() > 6_900.0);
//! ```
//!
//! 星历采样与插值。
//!
//! 传播器给出卫星在任意瞬时的状态，但当结果只用于绘制轨迹或搜索穿越时，调用
//! 它数百万次是浪费的。本模块在一组时刻网格上记录状态（称为样本），并用
//! 插值回答其间的查询。
//!
//! 两半是有意独立的：
//!
//! - [`Ephemeris`] 是时刻与状态的表。它对状态来自何处一无所知，故由 SGP4
//!   传播、由二体理论计算，或从文件读取的表都以相同方式工作。
//! - [`Sampler`] 通过调用一个由时刻产生状态的函数来填充表。在本工作区中该
//!   函数通常是 `crate::apsis_sgp4::Satellite::propagate_at`，但采样器只需要一个
//!   闭包，故测试可用解析函数。
//!
//! 插值是 Hermite 三次插值，它同时使用区间两端的位与速度。使用速度正是它
//! 准确的原因：只过两个位置的三次式没有卫星移动多快的信息，而 Hermite 重构
//! 匹配两端的斜率，故在相同采样间距下能好得多地复现光滑轨道。在圆轨道上误差
//! 是步长的四阶量，故步长减半使误差约减为十六分之一。
//!
//! 时间以距调用方所选原点的秒数计，与 orbit 层其余部分一致。位置以千米计，
//! 速度以千米每秒计。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod ephemeris;
mod error;
mod sampler;

pub use ephemeris::{Ephemeris, Sample, hermite_position, hermite_state, hermite_velocity};
pub use error::{EphemerisError, EphemerisErrorKind};
pub use sampler::Sampler;

pub use crate::apsis_math::Vec3;
