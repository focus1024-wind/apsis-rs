//! Orbital state facade that integrates the orbit layer.
//!
//! This module is the single entry point to the orbit layer. It re-exports
//! the element parsing, the analytical propagator, and the two-body model,
//! and adds one type that ties them together.
//!
//! [`Orbit`] holds a satellite prepared for propagation together with the
//! gravity model it was initialised against, and offers the conversions
//! between the two ways of describing an orbit that the layer supports:
//!
//! - Mean elements propagated by SGP4, which is the model a two-line
//!   element set is written for and the only way to reproduce the published
//!   positions of a catalogued satellite.
//! - Instantaneous Keplerian elements, which describe the osculating
//!   two-body orbit at one instant and are useful for inspection and for
//!   planning a maneuver.
//!
//! The two descriptions are not interchangeable. Propagating mean elements
//! with the two-body model, or osculating elements with SGP4, gives a
//! different and wrong answer; the conversions here are explicit about which
//! direction they go.
//!
//! Two further operations are built on the propagation and exposed here
//! because they need the same gravity parameter to stay consistent:
//!
//! - [`Orbit::sample_ephemeris`] records the track on a time grid so that it
//!   can be queried by interpolation instead of by propagation, which is
//!   what a coverage or access search wants.
//! - [`Orbit::apply_burn`] applies a finite maneuver to the osculating state
//!   and returns the resulting elements, which is what a maneuver plan
//!   produces.
//!
//! ```
//! use apsis::apsis_orbit::{GravityModel, Orbit};
//! use apsis::apsis_tle::TwoLineElement;
//!
//! let text = concat!(
//!     "1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927\r\n",
//!     "2 25544  51.6416 247.4627 0006703 130.5360 325.0288 15.72125391563537\r\n",
//! );
//! let element = TwoLineElement::parse(text).expect("sample is well formed");
//! let orbit = Orbit::from_element(&element, GravityModel::WGS72)
//!     .expect("sample elements are propagatable");
//!
//! let state = orbit.propagate_minutes(60.0).expect("state exists");
//! assert!(state.speed() > 7.0);
//!
//! // The osculating elements at that instant.
//! let osculating = orbit.osculating_elements(60.0).expect("state exists");
//! assert!(osculating.eccentricity < 0.01);
//! ```
//!
//! 整合 orbit 层的轨道状态门面。
//!
//! 本模块是 orbit 层的单一入口。它重导出根数解析、解析传播器与二体模型，
//! 并加入一个把它们联系起来的类型。
//!
//! [`Orbit`] 保存为传播准备好的卫星，连同其初始化所用的引力模型，并提供该层
//! 所支持的两种轨道描述之间的换算：
//!
//! - 由 SGP4 传播的平根数，这是两行根数集所针对的模型，也是复现已编目卫星
//!   已发布位置的唯一方式。
//! - 瞬时开普勒根数，它描述某一瞬时的密切二体轨道，对检视与规划机动有用。
//!
//! 两种描述不可互换。用二体模型传播平根数，或用 SGP4 传播密切根数，会给出
//! 不同且错误的答案；此处的换算明确说明它们朝哪个方向进行。
//!
//! 另有两项操作建立在传播之上并在此公开，因为它们需要同一引力参数以保持
//! 一致：
//!
//! - [`Orbit::sample_ephemeris`] 在时间网格上记录轨迹，使其可经插值而非
//!   传播查询，这正是覆盖或可见性搜索所需的。
//! - [`Orbit::apply_burn`] 对密切状态施加有限机动并返回所得根数，这正是
//!   机动规划所产生的。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod orbit;

pub use orbit::{EphemerisSamplingError, ManeuverApplicationError, Orbit};

pub use crate::apsis_ephemeris::{Ephemeris, EphemerisError, EphemerisErrorKind, Sample, Sampler};
pub use crate::apsis_kepler::{GravityParameter, KeplerianElements};
pub use crate::apsis_maneuver::{
    BurnOutcome, FiniteBurn, HohmannTransfer, Impulse, ManeuverError, ManeuverErrorKind,
    hohmann_transfer,
};
pub use crate::apsis_sgp4::{
    Elements, GravityModel, PropagationError, PropagationErrorKind, Satellite, State,
};
pub use crate::apsis_tle::{Classification, ElementError, ElementErrorKind, TwoLineElement};
