//! Constellation definition and instantiation.
//!
//! A constellation is a set of satellites described by a small number of
//! parameters rather than as a list of individual orbits. This module models
//! two forms of construction that together cover the constellations in
//! operational use:
//!
//! - A **Walker constellation**, in which satellites are spread evenly over
//!   a set of orbital planes that share an inclination and a semi-major
//!   axis, with a fixed phase relationship between adjacent planes.
//! - A **custom constellation**, where each shell is described separately so
//!   that a design that does not follow the Walker pattern can still be
//!   instantiated.
//!
//! - **Walker 星座**：卫星均匀分布在若干轨道面上，各面共用同一倾角与同一半长轴，
//!   相邻轨道面之间有固定的相位关系。
//! - **自定义星座**：每个壳层单独描述，使不遵循 Walker 构型的设计同样可以
//!   实例化。
//!
//! Both forms produce a list of [`KeplerianElements`] at a reference time,
//! which the orbit layer can propagate. The constellations in use for
//! satellite internet are Walker patterns at several altitudes, so the
//! Walker constructor is the one most callers need.
//!
//! 两种形式都在参考时刻产出一组 [`KeplerianElements`]，可由 orbit 层传播。
//! 当前用于卫星互联网的星座是在若干高度上的 Walker 构型，因此多数调用方需要的
//! 是 Walker 构造函数。
//!
//! The Walker notation is written `i: t/p/f`, where `i` is the inclination
//! in degrees, `t` the total number of satellites, `p` the number of planes,
//! and `f` the phase factor. The satellites are placed so that
//! `t / p` satellites share each plane and adjacent planes are offset in
//! mean anomaly by `f * 360 / t` degrees.
//!
//! 卫星的分布方式为：每个轨道面分到 `t / p` 颗卫星，相邻轨道面的平近点角相差
//! `f * 360 / t` 度。
//!
//! Source: the Walker notation and the phasing rule are the ones defined in
//! Walker, J. G., *Circular Orbit Patterns Providing Continuous Whole Earth
//! Coverage*, Royal Aircraft Establishment Technical Report 70211, 1970.
//!
//! 出处：Walker 记法与相位规则取自 Walker, J. G., *Circular Orbit Patterns
//! Providing Continuous Whole Earth Coverage*, Royal Aircraft Establishment
//! Technical Report 70211, 1970。
//!
//! ```
//! use apsis::apsis_constellation::{Constellation, WalkerPattern};
//!
//! // A Walker 53 degree pattern with 24 satellites in 3 planes.
//! let pattern = WalkerPattern {
//!     total_satellites: 24,
//!     planes: 3,
//!     phase_factor: 1,
//!     inclination: 53.0_f64.to_radians(),
//!     semi_major_axis: 6_878.137,
//! };
//! let constellation =
//!     Constellation::walker(pattern, 0.0).expect("the pattern is consistent");
//! assert_eq!(constellation.satellites().len(), 24);
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod error;
mod walker;

pub use error::{ConstellationError, ConstellationErrorKind};
pub use walker::{Constellation, Satellite, WalkerPattern};

pub use crate::apsis_kepler::{GravityParameter, KeplerianElements};
pub use crate::apsis_sgp4::GravityModel;
