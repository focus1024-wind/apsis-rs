//! The orbit layer: TLE parsing, SGP4/SDP4, two-body propagation, ephemeris
//! sampling, maneuvers, mission sequences, conjunction assessment, electro-
//! optical and infrared sensing, sensor fields of view, space environment
//! models, and numerical J2 propagation.
//!
//! 轨道层：TLE 解析、SGP4/SDP4、二体传播、星历采样、机动、任务序列、交会
//! 分析、光电与红外探测、传感器视场、空间环境模型与 J2 数值传播。

pub mod apsis_cat;
pub mod apsis_eoir;
pub mod apsis_ephemeris;
pub mod apsis_hpop;
pub mod apsis_kepler;
pub mod apsis_maneuver;
pub mod apsis_mission;
pub mod apsis_orbit;
pub mod apsis_seet;
pub mod apsis_sensor;
pub mod apsis_sgp4;
pub mod apsis_tle;
