//! `ApsisRS`: computational kernels for commercial spaceflight and satellite
//! internet systems.
//!
//! The crate is organized into dependency-ordered layers:
//!
//! ```text
//! foundation -> physics -> orbit -> rf -> network -> ops -> sim
//! ```
//!
//! Dependencies flow in one direction only. A layer may use the layers to its
//! left but never the layers to its right. Each layer is a private module
//! under `src/<layer>`, and every module it contains is re-exported below
//! under the name of the crate it used to be, so the public paths are
//! `apsis::apsis_core`, `apsis::apsis_orbit`, and so on.
//!
//! A downstream project enables the feature that matches the layers it needs.
//! Each feature re-exports the modules of one layer and pulls in the features
//! of the layers below it. For example, the `orbit` feature enables `physics`
//! and re-exports `apsis_orbit`, `apsis_tle`, `apsis_sgp4`, `apsis_kepler`,
//! `apsis_ephemeris`, and `apsis_maneuver`.
//!
//! The `full` feature enables every layer, including `sim`. Because `sim`
//! aggregates all other layers and is only needed by simulation workloads,
//! projects that do not run simulations can enable `ops` instead, which
//! enables every layer except `sim`.
//!
//! ApsisRS：面向商业航天与卫星互联网系统的计算内核。
//!
//! 本 crate 按依赖排序的各层组织：
//!
//! ```text
//! foundation -> physics -> orbit -> rf -> network -> ops -> sim
//! ```
//!
//! 依赖只朝一个方向流动。一层可以使用其左侧的各层，但绝不使用其右侧的
//! 层。每一层是 `src/<layer>` 下的一个私有模块，其中包含的每个模块都在
//! 下方以其原本的 crate 名重导出，故公开路径是 `apsis::apsis_core`、
//! `apsis::apsis_orbit` 等。
//!
//! 下游项目启用与其所需层相匹配的特性。每个特性重导出某一层的各模块，
//! 并引入其下方各层的特性。例如，`orbit` 特性启用 `physics`，并重导出
//! `apsis_orbit`、`apsis_tle`、`apsis_sgp4`、`apsis_kepler`、
//! `apsis_ephemeris` 与 `apsis_maneuver`。
//!
//! `full` 特性启用每一层，包括 `sim`。因为 `sim` 聚合了其余所有层且只为
//! 仿真工作负载所需，不运行仿真的项目可以改而启用 `ops`，它启用除 `sim`
//! 之外的每一层。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

// foundation

#[cfg(feature = "foundation")]
mod foundation;

#[cfg(feature = "foundation")]
pub use crate::foundation::apsis_core;

#[cfg(feature = "foundation")]
pub use crate::foundation::apsis_macros;

#[cfg(feature = "foundation")]
pub use crate::foundation::apsis_math;

#[cfg(feature = "foundation")]
pub use crate::foundation::apsis_units;

// physics

#[cfg(feature = "physics")]
mod physics;

#[cfg(feature = "physics")]
pub use crate::physics::apsis_coord;

#[cfg(feature = "physics")]
pub use crate::physics::apsis_frames;

#[cfg(feature = "physics")]
pub use crate::physics::apsis_geodesy;

#[cfg(feature = "physics")]
pub use crate::physics::apsis_leap;

#[cfg(feature = "physics")]
pub use crate::physics::apsis_time;

// orbit

#[cfg(feature = "orbit")]
mod orbit;

#[cfg(feature = "orbit")]
#[cfg(feature = "orbit")]
pub use crate::orbit::apsis_cat;
#[cfg(feature = "orbit")]
pub use crate::orbit::apsis_eoir;
#[cfg(feature = "orbit")]
pub use crate::orbit::apsis_ephemeris;

#[cfg(feature = "orbit")]
pub use crate::orbit::apsis_hpop;
#[cfg(feature = "orbit")]
pub use crate::orbit::apsis_kepler;

#[cfg(feature = "orbit")]
pub use crate::orbit::apsis_maneuver;
#[cfg(feature = "orbit")]
pub use crate::orbit::apsis_mission;

#[cfg(feature = "orbit")]
pub use crate::orbit::apsis_orbit;

#[cfg(feature = "orbit")]
pub use crate::orbit::apsis_seet;
#[cfg(feature = "orbit")]
pub use crate::orbit::apsis_sensor;
#[cfg(feature = "orbit")]
pub use crate::orbit::apsis_sgp4;

#[cfg(feature = "orbit")]
pub use crate::orbit::apsis_tle;

// rf

#[cfg(feature = "rf")]
mod rf;

#[cfg(feature = "rf")]
pub use crate::rf::apsis_antenna;

#[cfg(feature = "rf")]
pub use crate::rf::apsis_atmosphere;
#[cfg(feature = "rf")]
pub use crate::rf::apsis_aviator;

#[cfg(feature = "rf")]
pub use crate::rf::apsis_doppler;

#[cfg(feature = "rf")]
pub use crate::rf::apsis_interference;

#[cfg(feature = "rf")]
pub use crate::rf::apsis_link;

#[cfg(feature = "rf")]
pub use crate::rf::apsis_noise;

#[cfg(feature = "rf")]
pub use crate::rf::apsis_rf;

// network

#[cfg(feature = "network")]
mod network;

#[cfg(feature = "network")]
pub use crate::network::apsis_access;

#[cfg(feature = "network")]
pub use crate::network::apsis_constellation;

#[cfg(feature = "network")]
pub use crate::network::apsis_coverage;

#[cfg(feature = "network")]
pub use crate::network::apsis_handover;

#[cfg(feature = "network")]
pub use crate::network::apsis_isl;

#[cfg(feature = "network")]
pub use crate::network::apsis_mesh;

#[cfg(feature = "network")]
pub use crate::network::apsis_routing;

#[cfg(feature = "network")]
pub use crate::network::apsis_topology;

// ops

#[cfg(feature = "ops")]
mod ops;

#[cfg(feature = "ops")]
pub use crate::ops::apsis_conflict;

#[cfg(feature = "ops")]
pub use crate::ops::apsis_resource;

#[cfg(feature = "ops")]
pub use crate::ops::apsis_schedule;

#[cfg(feature = "ops")]
pub use crate::ops::apsis_scheduler;

#[cfg(feature = "ops")]
pub use crate::ops::apsis_task;

// sim

#[cfg(feature = "sim")]
mod sim;

#[cfg(feature = "sim")]
pub use crate::sim::apsis_event;

#[cfg(feature = "sim")]
pub use crate::sim::apsis_output;

#[cfg(feature = "sim")]
pub use crate::sim::apsis_scenario;

#[cfg(feature = "sim")]
pub use crate::sim::apsis_sim;

#[cfg(feature = "sim")]
pub use crate::sim::apsis_sim_engine;
