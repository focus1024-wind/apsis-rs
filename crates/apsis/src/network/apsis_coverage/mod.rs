//! Ground coverage analysis.
//!
//! A satellite covers the part of the Earth's surface from which it can be
//! seen above a minimum elevation angle. That part is a circle, called the
//! footprint, whose size depends on the satellite altitude and on the
//! elevation mask. This module computes the footprint geometry, tests whether
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
//! <div>
//! $$
//! \text{elevation} + \text{nadir} + \text{central} = 90^\circ
//! $$
//! </div>
//!
//! Source: the relations used here are the standard satellite geometry, as
//! given for example in Vallado, D. A., *Fundamentals of Astrodynamics and
//! Applications*, 4th edition, microcosm Press, 2013, section 5.3.
//!
//! ```
//! use apsis::apsis_coverage::{footprint_central_angle, slant_range};
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
//!
//! 地面覆盖分析。
//!
//! 卫星覆盖地表上能在最小仰角以上看到它的那部分。那部分是一个圆，称为覆盖区，
//! 其大小取决于卫星高度与仰角遮蔽。本模块计算覆盖区几何、判定某地面点是否落在
//! 覆盖区内，并度量某区域被覆盖的比例。
//!
//! [`geometry`](self) 中的球面模型把地球当作一个球。对星座设计已足够准确，那里
//! 所要的量是覆盖区半径到万分之几以及可见卫星数。[`line_of_sight`] 中的精确视线
//! 判定使用实际位置，故它计入了扁率以及高于椭球的地面点。
//!
//! 卫星、地面点与地心构成的几何是一个三角形。其三个角是地面点处的仰角、卫星处的
//! 天底角，以及地心处的中心角，它们相加为直角：
//!
//! <div>
//! $$
//! \text{elevation} + \text{nadir} + \text{central} = 90^\circ
//! $$
//! </div>
//!
//! 出处：此处所用的关系是标准的卫星几何，例如见 Vallado, D. A.，*Fundamentals of
//! Astrodynamics and Applications*，第 4 版，microcosm Press，2013，第 5.3 节。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod area;
mod footprint;
mod geometry;
mod grid;
mod visibility;

pub use area::{AreaAccessResult, AreaCoverageResult, GeoPolygon, area_access, area_coverage};
pub use footprint::{
    Footprint, central_angle_between, footprint, footprint_area, footprint_central_angle,
    footprint_edge_elevation, footprint_fraction, point_in_footprint,
};
pub use geometry::{earth_central_angle, elevation_from_central_angle, nadir_angle, slant_range};
pub use grid::{CoverageGrid, CoverageResult, CoverageSample, grid_coverage};
pub use visibility::{
    GroundPoint, Sensor, SubSatellitePoint, central_angle_to_station, elevation_angle,
    line_of_sight, local_up, sub_satellite_point, visible_from, visible_from_geodetic,
};

pub use crate::apsis_geodesy::{Ellipsoid, Geodetic, WGS84};
pub use crate::apsis_math::Vec3;
