//! Reference frame definitions and the rotation between the inertial and
//! Earth-fixed frames.
//!
//! A reference frame fixes the orientation and origin that a vector is
//! expressed relative to. This module names the two frames used for
//! near-Earth work and computes the rotation between them. Concrete
//! position types that carry a frame are defined in `apsis_coord`; this
//! crate works with the rotation matrix and the angle that defines it.
//!
//! The inertial frame is the Earth-centered inertial (ECI) frame aligned
//! with the mean equator and equinox of J2000.0. Its axes are fixed in
//! inertial space, so an orbit propagated in it is not affected by the
//! rotation of the Earth.
//!
//! The rotating frame is the Earth-centered Earth-fixed (ECEF) frame. Its
//! third axis is the Earth's rotation axis and its first axis follows the
//! prime meridian, so a point fixed to the ground has constant coordinates
//! in it.
//!
//! The two are related by a rotation of [`EarthRotationAngle`] about the
//! common third axis. The simple rotation at this level uses the Earth
//! Rotation Angle, which is a function of UT1 only; it does not apply
//! precession or nutation, which is sufficient for work that needs a few
//! hundredths of a degree. The [`iau2000`] submodule provides the full IAU
//! 2000/2006 chain, including the frame bias, precession, nutation, and polar
//! motion, when the orientation of the celestial pole matters.
//!
//! Source: the Earth Rotation Angle expression is from the International
//! Astronomical Union, as given in the *IERS Conventions (2010)*, equation
//! 8.32.
//!
//! ```
//! use apsis::apsis_frames::{EarthRotationAngle, rotation_eci_to_ecef};
//!
//! // The angle advances by roughly 360.9856 degrees per day.
//! let first = EarthRotationAngle::from_ut1_days(0.0);
//! let second = EarthRotationAngle::from_ut1_days(1.0);
//! let advance = (second.radians() - first.radians()).to_degrees();
//! assert!((advance - 360.985_612_288_088).abs() < 1e-6);
//!
//! // The rotation is ready to apply to a vector.
//! let _ = rotation_eci_to_ecef(first);
//! ```
//!
//! 参考系定义，以及惯性系与地固系之间的转动。
//!
//! 参考系固定了矢量所相对表达的取向与原点。本模块为近地工作所用的两个
//! 坐标系命名，并计算它们之间的转动。携带坐标系的具位置类型定义在
//! `apsis_coord` 中；本模块处理转动矩阵以及定义它的角。
//!
//! 惯性系是与 J2000.0 平赤道与平春分点对齐的地心惯性（ECI）系。其各轴在
//! 惯性空间中固定，故在其中传播的轨道不受地球转动影响。
//!
//! 转动坐标系是地心地固（ECEF）系。其第三轴是地球自转轴，第一轴随本初
//! 子午线，故固定于地面的点在其中具有恒定坐标。
//!
//! 二者由绕公共第三轴转动 [`EarthRotationAngle`] 相联系。本层较简单的转动
//! 只用地球自转角，它仅是 UT1 的函数，不施加岁差或章动；这对只需要百分之几
//! 度精度的计算已足够。需要天极取向时，[`iau2000`] 子模块给出完整的 IAU
//! 2000/2006 链，包括坐标架偏差、岁差、章动与极移。
//!
//! 出处：地球自转角表达式来自国际天文学联合会，如 *IERS Conventions
//! （2010）* 式 8.32 所给出。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod earth_rotation;
pub mod iau2000;

pub use earth_rotation::{
    ERA_J2000_TURNS, ERA_RATE_TURNS_PER_DAY, EarthRotationAngle, rotation_ecef_to_eci,
    rotation_eci_to_ecef, ut1_days_from_j2000,
};
