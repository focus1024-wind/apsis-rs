//! Dependency-free linear algebra for the `ApsisRS` workspace.
//!
//! The crate provides three types:
//!
//! - [`Vec3`], a three component column vector over `f64`
//! - [`Quat`], a unit quaternion used for rotations
//! - [`Mat3`], a three by three matrix stored in column-major order
//!
//! The types are plain data with no interior invariants beyond those
//! documented on each method, so their fields are public and a value can be
//! built by struct literal expression. Operations that are mathematically
//! undefined for some inputs, such as normalizing the zero vector, return
//! [`Option`] instead of panicking or producing a non-finite result.
//!
//! All rotations use the right-hand rule about the given axis, and all
//! angles are in radians. Angles are not wrapped on input; methods that
//! require a canonical range document that requirement.
//!
//! ```
//! use apsis_math::Vec3;
//!
//! let x = Vec3::new(1.0, 0.0, 0.0);
//! let y = Vec3::new(0.0, 1.0, 0.0);
//! let z = x.cross(y);
//! assert_eq!(z, Vec3::new(0.0, 0.0, 1.0));
//! ```
//!
//! `ApsisRS` 工作区无依赖的线性代数。
//!
//! 本 crate 提供三种类型：
//!
//! - [`Vec3`]，基于 `f64` 的三分量列矢量
//! - [`Quat`]，用于转动的单位四元数
//! - [`Mat3`]，按列主序存储的三阶矩阵
//!
//! 这些类型是纯数据，除各方法上所述外没有内部不变量，故其字段是公开的，
//! 值可由结构体字面量表达式构建。对某些输入在数学上未定义的操作（如把零
//! 矢量归一化）返回 [`Option`]，而不 panic 或产生非有限结果。
//!
//! 所有转动都绕给定轴采用右手定则，所有角度都以弧度计。输入时角度不进行
//! 环绕；要求取规范区间的方法会说明该要求。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod mat3;
pub mod parallel;
mod quat;
mod vec3;

pub use mat3::Mat3;
pub use quat::Quat;
pub use vec3::Vec3;
