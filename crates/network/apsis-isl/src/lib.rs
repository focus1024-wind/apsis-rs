//! Inter-satellite link geometry and visibility.
//!
//! Two satellites can form a link when the straight line between them does
//! not pass through the Earth. This crate decides that condition, gives the
//! geometry a link budget needs, and reports the range and the angles at
//! which each satellite must point.
//!
//! The blocking test is whether the line segment between the two positions
//! has any point inside the body. For a spherical body that is a
//! perpendicular distance test, which is exact and cheap, so it is the one
//! used here.
//!
//! An optical link adds a second condition: the Sun must not be within the
//! field of view of either terminal, because the background would swamp the
//! signal. That is a separate test, offered as [`sun_blocks_link`], which
//! takes the direction to the Sun.
//!
//! Lengths are in kilometers and angles in radians. The positions are in a
//! frame in which the Earth is a sphere of the given radius, which for link
//! geometry is the same frame the orbit layer produces.
//!
//! ```
//! use apsis_isl::{LinkGeometry, link_geometry, mutual_visibility};
//! use apsis_math::Vec3;
//!
//! // Two satellites on the same side of the Earth see each other.
//! let first = Vec3::new(6_928.137, 0.0, 0.0);
//! let second = Vec3::new(6_868.137, 2_500.0, 0.0);
//! assert_eq!(mutual_visibility(first, second, 6_378.137), Some(true));
//!
//! // Two satellites on opposite sides do not.
//! let opposite = Vec3::new(-6_928.137, 0.0, 0.0);
//! assert_eq!(mutual_visibility(first, opposite, 6_378.137), Some(false));
//! ```
//!
//! 星间链路几何与可见性。
//!
//! 当两颗卫星之间的直线不穿过地球时，它们可以形成一条链路。本 crate 判定该条件、
//! 给出链路预算所需的几何，并报告距离以及每颗卫星必须指向的角度。
//!
//! 遮挡判据是两点之间的线段是否有点位于天体内部。对球形天体而言这是一个垂距判定，
//! 它精确且廉价，故此处采用它。
//!
//! 光链路还加一个条件：太阳不得位于任一终端的视场内，因为背景会淹没信号。那是一个
//! 单独的判定，作为 [`sun_blocks_link`] 提供，它接收太阳方向。
//!
//! 长度以千米计，角度以弧度计。各位置所在的坐标系中地球是给定半径的球，对链路
//! 几何而言它与 orbit 层所产生的是同一坐标系。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod batch;
mod geometry;
mod link;

pub use batch::{visibility_matrix, visibility_matrix_parallel};
pub use geometry::{closest_approach_to_centre, link_geometry, mutual_visibility, sun_blocks_link};
pub use link::{LinkGeometry, LinkState};

pub use apsis_math::Vec3;
