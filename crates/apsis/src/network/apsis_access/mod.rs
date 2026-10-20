//! Ground station access windows.
//!
//! A satellite is accessible from a ground station during the intervals when
//! it rises above the station's elevation mask. Each such interval is a
//! pass, or access window, and the sequence of windows over a period is the
//! access schedule. This module finds those windows from a propagated orbit
//! and reports their start, end, duration, and the geometry at their peak.
//!
//! The windows are found by sampling the elevation angle at a fixed step and
//! refining each crossing where the elevation passes the mask. The sampling
//! step sets the shortest pass that can be found, and the refinement gives
//! the crossing times to a tolerance independent of the step.
//!
//! The elevation is evaluated from the satellite position in the
//! Earth-fixed frame, so the caller supplies the transformation from the
//! orbit frame. In this workspace that is the conversion in `apsis_coord`
//! with the Earth rotation angle from `apsis_frames`.
//!
//! ```
//! use apsis::apsis_access::{AccessFinder, Pass};
//! use apsis::apsis_coverage::GroundPoint;
//! use apsis::apsis_geodesy::Geodetic;
//!
//! // A station on the equator with a ten degree mask.
//! let station = GroundPoint::new(Geodetic::new(0.0, 0.0, 0.0), 10.0_f64.to_radians());
//! let finder = AccessFinder {
//!     station,
//!     sample_step_seconds: 60.0,
//!     crossing_tolerance_seconds: 0.001,
//! };
//! assert!(finder.is_valid());
//! ```
//!
//! 地面站可见性窗口。
//!
//! 当卫星升到地面站仰角遮蔽之上时，它在那段时间内可从该站访问。每个这样的区间
//! 是一次过境，或称可见性窗口，而一段时间上的窗口序列即可见性调度。本模块从
//! 传播后的轨道找出这些窗口，并报告其开始、结束、时长以及峰值处的几何。
//!
//! 窗口通过对仰角以固定步长采样、并对仰角穿过遮蔽的每个穿越加以细化来找到。采样
//! 步长设定所能找到的最短过境，而细化给出独立于步长的穿越时刻精度。
//!
//! 仰角由地固系中的卫星位置求值，故调用方提供从轨道坐标系来的变换。在本工作区中
//! 那是 `apsis_coord` 中的换算，配以来自 `apsis_frames` 的地球自转角。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod batch;
mod chain;
mod error;
mod finder;
mod pass;

pub use batch::{find_all, find_all_parallel};
pub use chain::{AccessInterval, chain_passes, intersect_intervals};
pub use error::{AccessError, AccessErrorKind};
pub use finder::{AccessFinder, OrbitSample};
pub use pass::{Pass, PassSummary};

pub use crate::apsis_coverage::{GroundPoint, Sensor, elevation_angle};
pub use crate::apsis_math::Vec3;
