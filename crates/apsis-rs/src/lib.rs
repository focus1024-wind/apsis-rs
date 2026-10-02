//! ApsisRS: computational kernels for commercial spaceflight and satellite
//! internet systems.
//!
//! The workspace is organized into dependency-ordered layers:
//!
//! ```text
//! foundation -> physics -> orbit -> rf -> network -> ops -> sim
//! ```
//!
//! Dependencies flow in one direction only. A layer may use the layers to
//! its left but never the layers to its right.
//!
//! This crate is the public facade named `apsis-rs`. Downstream projects
//! depend on it directly and enable the feature that matches the layers they
//! need. Each feature re-exports the crates of one layer and pulls in the
//! features of the layers below it. For example, the `orbit` feature enables
//! `physics` and re-exports `apsis_orbit`, `apsis_tle`, `apsis_sgp4`,
//! `apsis_kepler`, and `apsis_ephemeris`.
//!
//! The `full` feature enables every layer, including `sim`. Because `sim`
//! aggregates all other layers and is only needed by simulation workloads,
//! projects that do not run simulations can enable `ops` instead, which
//! enables every layer except `sim`.
//!
//! ApsisRS：面向商业航天与卫星互联网系统的计算内核。
//!
//! 工作区按依赖排序的各层组织：
//!
//! ```text
//! foundation -> physics -> orbit -> rf -> network -> ops -> sim
//! ```
//!
//! 依赖只朝一个方向流动。一层可以使用其左侧的各层，但绝不使用其右侧的
//! 层。
//!
//! 本 crate 是名为 `apsis-rs` 的公共门面。下游项目直接依赖它，并启用与其
//! 所需层相匹配的特性。每个特性重导出某一层的各 crate，并引入其下方各层
//! 的特性。例如，`orbit` 特性启用 `physics`，并重导出 `apsis_orbit`、
//! `apsis_tle`、`apsis_sgp4`、`apsis_kepler` 与 `apsis_ephemeris`。
//!
//! `full` 特性启用每一层，包括 `sim`。因为 `sim` 聚合了其余所有层且只为
//! 仿真工作负载所需，不运行仿真的项目可以改而启用 `ops`，它启用除 `sim`
//! 之外的每一层。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

// foundation

#[cfg(feature = "foundation")]
pub use apsis_core;

#[cfg(feature = "foundation")]
pub use apsis_macros;

#[cfg(feature = "foundation")]
pub use apsis_math;

#[cfg(feature = "foundation")]
pub use apsis_units;

// physics

#[cfg(feature = "physics")]
pub use apsis_coord;

#[cfg(feature = "physics")]
pub use apsis_frames;

#[cfg(feature = "physics")]
pub use apsis_geodesy;

#[cfg(feature = "physics")]
pub use apsis_leap;

#[cfg(feature = "physics")]
pub use apsis_time;

// orbit

#[cfg(feature = "orbit")]
pub use apsis_ephemeris;

#[cfg(feature = "orbit")]
pub use apsis_kepler;

#[cfg(feature = "orbit")]
pub use apsis_maneuver;

#[cfg(feature = "orbit")]
pub use apsis_orbit;

#[cfg(feature = "orbit")]
pub use apsis_sgp4;

#[cfg(feature = "orbit")]
pub use apsis_tle;

// rf

#[cfg(feature = "rf")]
pub use apsis_antenna;

#[cfg(feature = "rf")]
pub use apsis_atmosphere;

#[cfg(feature = "rf")]
pub use apsis_doppler;

#[cfg(feature = "rf")]
pub use apsis_interference;

#[cfg(feature = "rf")]
pub use apsis_link;

#[cfg(feature = "rf")]
pub use apsis_noise;

#[cfg(feature = "rf")]
pub use apsis_rf;

// network

#[cfg(feature = "network")]
pub use apsis_access;

#[cfg(feature = "network")]
pub use apsis_constellation;

#[cfg(feature = "network")]
pub use apsis_coverage;

#[cfg(feature = "network")]
pub use apsis_handover;

#[cfg(feature = "network")]
pub use apsis_isl;

#[cfg(feature = "network")]
pub use apsis_mesh;

#[cfg(feature = "network")]
pub use apsis_routing;

#[cfg(feature = "network")]
pub use apsis_topology;

// ops

#[cfg(feature = "ops")]
pub use apsis_conflict;

#[cfg(feature = "ops")]
pub use apsis_resource;

#[cfg(feature = "ops")]
pub use apsis_schedule;

#[cfg(feature = "ops")]
pub use apsis_scheduler;

#[cfg(feature = "ops")]
pub use apsis_task;

// sim

#[cfg(feature = "sim")]
pub use apsis_event;

#[cfg(feature = "sim")]
pub use apsis_output;

#[cfg(feature = "sim")]
pub use apsis_scenario;

#[cfg(feature = "sim")]
pub use apsis_sim;

#[cfg(feature = "sim")]
pub use apsis_sim_engine;
