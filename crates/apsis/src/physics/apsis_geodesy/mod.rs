//! Reference ellipsoids and geodetic computations.
//!
//! A geodetic position is written as latitude, longitude, and height above
//! an ellipsoid. Latitude here is always geodetic, the angle between the
//! ellipsoid normal and the equatorial plane, not geocentric. The two
//! differ by up to about 0.19 degrees at mid latitudes, so the distinction
//! matters for pointing and ranging.
//!
//! [`WGS84`] is the ellipsoid used by GPS and by almost all satellite work.
//! Its defining parameters are the semi-major axis and the inverse
//! flattening; every other quantity is derived from those two, so the
//! derived values agree with the ones published in the standard to the
//! precision of the defining parameters.
//!
//! Source: National Imagery and Mapping Agency, *Department of Defense World
//! Geodetic System 1984*, Technical Report TR8350.2, third edition, 2000.
//!
//! ```
//! use apsis::apsis_geodesy::{Geodetic, WGS84};
//!
//! // The equator has zero height and a radius equal to the semi-major axis.
//! let point = Geodetic::new(0.0, 0.0, 0.0);
//! let cartesian = point.to_cartesian(WGS84);
//! assert!((cartesian.norm() - WGS84.semi_major_axis()).abs() < 1e-9);
//! ```
//!
//! 参考椭球与大地计算。
//!
//! 大地位置写为某椭球之上的纬度、经度与高度。此处的纬度总是大地纬度，即
//! 椭球法线与赤道面之间的角，而非地心纬度。在中纬度处二者相差至多约 0.19
//! 度，故该区别对指向与测距是要紧的。
//!
//! [`WGS84`] 是 GPS 以及几乎所有卫星工作所用的椭球。其定义参数是半长轴与
//! 扁率倒数；其余每个量都由这两个导出，故各导出值与标准中发布的值在定义
//! 参数的精度内一致。
//!
//! 出处：National Imagery and Mapping Agency，*Department of Defense World
//! Geodetic System 1984*，技术报告 TR8350.2，第三版，2000。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod ellipsoid;
mod geodetic;

pub use ellipsoid::{Ellipsoid, WGS84, WGS84_GRAVITATIONAL_PARAMETER};
pub use geodetic::{Geodetic, cartesian_from_geodetic, geodetic_from_cartesian};
