//! Ground coverage analysis.
//!
//! 地面覆盖分析。
//!
//! A satellite covers the part of the Earth's surface from which it can be
//! seen above a minimum elevation angle. That part is a circle, called the
//! footprint, whose size depends on the satellite altitude and on the
//! elevation mask. This crate computes the footprint geometry, tests whether
//! a ground point lies inside a footprint, and measures how much of a region
//! is covered.
//!
//! The spherical model in [`geometry`](self) treats the Earth as a sphere.
//! That is accurate enough for constellation design, where the quantities
//! wanted are the footprint radius to a few parts in ten thousand and the
//! number of satellites in view. The exact line-of-sight test in
//! [`line_of_sight`] uses the actual positions, so it accounts for the
//! flattening and for a ground point above the ellipsoid.
//!
//! The geometry of the satellite, the ground point, and the centre of the
//! Earth is a triangle. Its three angles are the elevation at the ground
//! point, the nadir angle at the satellite, and the central angle at the
//! centre, and they sum to a right angle:
//!
//! ```text
//! elevation + nadir + central = 90 degrees
//! ```
//!
//! Source: the relations used here are the standard satellite geometry, as
//! given for example in Vallado, D. A., *Fundamentals of Astrodynamics and
//! Applications*, 4th edition, microcosm Press, 2013, section 5.3.
//!
//! ```
//! use apsis_coverage::{footprint_central_angle, slant_range};
//!
//! // A satellite at 550 km altitude sees to about fifteen degrees of
//! // central angle at a ten degree elevation mask, which is a footprint
//! // radius of some 1665 km along the surface.
//! let central = footprint_central_angle(6_928.137, 6_378.137, 10.0_f64.to_radians())
//!     .expect("valid geometry");
//! assert!((central.to_degrees() - 14.96).abs() < 0.01);
//!
//! let range = slant_range(6_928.137, 6_378.137, 10.0_f64.to_radians())
//!     .expect("valid geometry");
//! assert!((range - 1_815.65).abs() < 0.01);
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod footprint;
mod geometry;
mod visibility;

pub use footprint::{
    Footprint, central_angle_between, footprint, footprint_area, footprint_central_angle,
    footprint_edge_elevation, footprint_fraction, point_in_footprint,
};
pub use geometry::{earth_central_angle, elevation_from_central_angle, nadir_angle, slant_range};
pub use visibility::{
    GroundPoint, Sensor, SubSatellitePoint, central_angle_to_station, elevation_angle,
    line_of_sight, local_up, sub_satellite_point, visible_from, visible_from_geodetic,
};

pub use apsis_geodesy::{Ellipsoid, Geodetic, WGS84};
pub use apsis_math::Vec3;
