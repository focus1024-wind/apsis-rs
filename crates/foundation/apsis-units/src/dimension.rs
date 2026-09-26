//! Physical dimension markers.
//!
//! Each marker is a zero sized type naming one physical dimension. A
//! [`Unit`](apsis_core::Unit) declares which dimension it measures through
//! its `Dimension` associated type, and conversion between quantities is
//! only permitted when the dimensions match.

/// Length.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Length;

/// Duration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Time;

/// Frequency, the reciprocal of time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Frequency;

/// Plane angle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Angle;

/// Thermodynamic temperature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Temperature;
