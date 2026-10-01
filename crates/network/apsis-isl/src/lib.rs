//! Inter-satellite link geometry and visibility.
//!
//! 星间链路几何与可见性。
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

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod batch;
mod geometry;
mod link;

pub use batch::{visibility_matrix, visibility_matrix_parallel};
pub use geometry::{closest_approach_to_centre, link_geometry, mutual_visibility, sun_blocks_link};
pub use link::{LinkGeometry, LinkState};

pub use apsis_math::Vec3;
