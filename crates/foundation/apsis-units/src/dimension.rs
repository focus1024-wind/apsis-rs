//! Physical dimension markers.
//!
//! Each marker is a zero sized type naming one physical dimension. A
//! [`Unit`](apsis_core::Unit) declares which dimension it measures through
//! its `Dimension` associated type, and conversion between quantities is
//! only permitted when the dimensions match.
//!
//! 物理量纲标记。
//!
//! 每个标记是一个零大小类型，命名一个物理量纲。[`Unit`](apsis_core::Unit)
//! 通过其 `Dimension` 关联类型声明所度量的量纲，量之间的换算仅在量纲相同时
//! 才被允许。

/// Length.
///
/// 长度。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Length;

/// Duration.
///
/// 时间。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Time;

/// Frequency, the reciprocal of time.
///
/// 频率，时间的倒数。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Frequency;

/// Plane angle.
///
/// 平面角。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Angle;

/// Thermodynamic temperature.
///
/// 热力学温度。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Temperature;
