//! Doppler shift computation.
//!
//! A satellite and a ground station move relative to each other, so the
//! frequency that arrives at the receiver differs from the frequency that
//! left the transmitter. This module computes that shift from the relative
//! velocity.
//!
//! Two effects are present and they are separate. The classical shift is
//! proportional to the range rate divided by the speed of light, and it is
//! what a Doppler tracker compensates. The relativistic time dilation
//! between the two clocks is a second order correction that is far smaller
//! for orbital velocities, and it is offered separately so that a caller can
//! include it when the application needs it.
//!
//! The sign convention used here is that a positive range rate, meaning the
//! two terminals are moving apart, produces a negative shift and therefore a
//! lower received frequency. This matches the everyday observation that a
//! receding source sounds lower.
//!
//! ```
//! use apsis::apsis_doppler::{DopplerShift, range_rate_from_velocity};
//! use apsis::apsis_math::Vec3;
//!
//! // A satellite at the first axis moving away along it.
//! let range_rate = range_rate_from_velocity(
//!     Vec3::new(7_000_000.0, 0.0, 0.0),
//!     Vec3::new(1_000.0, 0.0, 0.0),
//! );
//! assert_eq!(range_rate, Some(1_000.0));
//!
//! let shift = DopplerShift::from_range_rate(12.0e9, 1_000.0).expect("valid");
//! assert!(shift.shifted_frequency() < 12.0e9);
//! ```
//!
//! 多普勒频移计算。
//!
//! 卫星与地面站相对运动，故到达接收机的频率不同于离开发射机的频率。本模块
//! 由相对速度计算该频移。
//!
//! 存在两种效应，它们是分开的。经典频移与距离变化率除以光速成正比，它是
//! 多普勒跟踪器所补偿的。两个时钟之间的相对论时间膨胀是二阶改正，对轨道
//! 速度而言小得多，它被分开提供，使调用方在应用需要时可以计入它。
//!
//! 此处所用的符号约定是：正的距离变化率（即两终端正在远离）产生负频移，
//! 因而接收频率更低。这与日常观察——远离的声源听起来更低——一致。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod shift;

pub use shift::{
    DopplerShift, acquisition_span, angle_from_range_rate, classical_shift, maximum_shift,
    range_rate_from_velocity, relativistic_doppler_factor,
};

pub use crate::apsis_core::constant::SPEED_OF_LIGHT;
pub use crate::apsis_math::Vec3;
