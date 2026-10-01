//! Coordinate frames and the transformations between them.
//!
//! This crate defines positions that carry their reference frame in the
//! type, so a position expressed in one frame cannot be passed where another
//! is expected. The two frames are the Earth-centered inertial (ECI) frame
//! and the Earth-centered Earth-fixed (ECEF) frame.
//!
//! Three position representations are offered:
//!
//! - [`Eci`], an inertial Cartesian position in meters
//! - [`Ecef`], an Earth-fixed Cartesian position in meters
//! - [`Geodetic`], a latitude, longitude, and height above the WGS 84
//!   ellipsoid
//!
//! The transformation between [`Eci`] and [`Ecef`] is the rotation defined
//! by the Earth Rotation Angle, which `apsis-frames` computes. The
//! transformation between [`Ecef`] and [`Geodetic`] uses the WGS 84
//! ellipsoid from `apsis-geodesy`.
//!
//! ```
//! use apsis_coord::{Ecef, Eci, Geodetic, WGS84};
//! use apsis_frames::EarthRotationAngle;
//! use apsis_math::Vec3;
//!
//! // A station on the equator at zero longitude.
//! let station = Geodetic::new(0.0, 0.0, 0.0);
//! let fixed = station.to_ecef();
//! assert!((fixed.vector().x - WGS84.semi_major_axis()).abs() < 1e-6);
//!
//! // Rotating it into the inertial frame preserves its length.
//! let inertial = fixed.to_eci(EarthRotationAngle::from_ut1_days(0.0));
//! assert!((inertial.vector().norm() - fixed.vector().norm()).abs() < 1e-6);
//! let _ = Eci::new(Vec3::X);
//! ```
//!
//! Sources:
//!
//! - Earth Rotation Angle: *IERS Conventions (2010)*, equation 8.32, as
//!   implemented by `apsis-frames`.
//! - Geodetic conversions and the WGS 84 ellipsoid: NIMA TR8350.2, third
//!   edition, 2000, as implemented by `apsis-geodesy`.
//!
//! 坐标系及它们之间的变换。
//!
//! 本 crate 定义在类型中携带其参考系的各类位置，故在某坐标系中表达的位置
//! 无法传递到期望另一坐标系之处。这两个坐标系是地心惯性（ECI）系与地心
//! 地固（ECEF）系。
//!
//! 提供三种位置表示：
//!
//! - [`Eci`]，以米计的惯性直角坐标位置
//! - [`Ecef`]，以米计的地固直角坐标位置
//! - [`Geodetic`]，WGS 84 椭球之上的纬度、经度与高度
//!
//! [`Eci`] 与 [`Ecef`] 之间的变换是由地球自转角定义的转动，由
//! `apsis-frames` 计算。[`Ecef`] 与 [`Geodetic`] 之间的变换使用来自
//! `apsis-geodesy` 的 WGS 84 椭球。
//!
//! 出处：
//!
//! - 地球自转角：*IERS Conventions（2010）*，式 8.32，由 `apsis-frames`
//!   实现。
//! - 大地坐标换算与 WGS 84 椭球：NIMA TR8350.2，第三版，2000，由
//!   `apsis-geodesy` 实现。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod frames;

pub use frames::{Ecef, Eci, Geodetic};

pub use apsis_frames::EarthRotationAngle;
pub use apsis_geodesy::{Ellipsoid, WGS84};
