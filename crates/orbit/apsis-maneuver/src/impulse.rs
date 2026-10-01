//! The impulsive maneuver.
//!
//! 脉冲机动。

use crate::error::{ManeuverError, ManeuverErrorKind};
use apsis_math::Vec3;

/// An instantaneous change in velocity.
///
/// An impulse is a velocity change applied in zero time, so it changes the
/// velocity of a state but not its position. That is the model for a burn
/// short compared with the orbital period, and it is how a delta-v budget is
/// written.
///
/// The change is stored as a vector in the same frame as the state it will
/// be applied to, so the caller is responsible for expressing it in that
/// frame. The magnitude is the delta-v, the quantity a propellant budget is
/// measured in.
///
/// 瞬时速度变化。
///
/// 冲量是在零时间内施加的速度变化，故只改变状态的速度而不改变其位置。这是相对
/// 轨道周期很短的点火的模型，也是编写 delta-v 预算所用的模型。
///
/// 该变化以矢量形式存储在与待施加状态相同的坐标系中，因此调用方负责把它表达
/// 在该系中。其模即 delta-v，推进剂预算以此计量。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Impulse {
    delta_v: Vec3,
}

impl Impulse {
    /// Creates an impulse from a velocity change vector, in kilometers per
    /// second.
    ///
    /// Returns an error when a component is not finite or when the vector is
    /// zero, because a zero change is not a maneuver and is almost always a
    /// mistake in a plan.
    ///
    /// 由速度变化矢量创建冲量，单位为千米每秒。
    ///
    /// 当某个分量非有限或矢量为零时返回错误，因为零变化不是机动，且几乎总是
    /// 计划中的笔误。
    pub fn new(delta_v: Vec3) -> Result<Self, ManeuverError> {
        if !delta_v.is_finite() {
            return Err(ManeuverError::new(ManeuverErrorKind::NotFinite)
                .with_detail("the velocity change is not finite"));
        }
        if delta_v.norm() == 0.0 {
            return Err(ManeuverError::new(ManeuverErrorKind::InvalidState)
                .with_detail("the velocity change is zero"));
        }
        Ok(Self { delta_v })
    }

    /// Creates an impulse of a given magnitude along a direction.
    ///
    /// The direction is normalized, so the magnitude is the delta-v whatever
    /// the length of the direction vector. Returns an error when the
    /// magnitude is not positive and finite or when the direction is zero or
    /// not finite.
    ///
    /// 沿给定方向创建指定大小的冲量。
    ///
    /// 方向被归一化，故无论方向矢量多长，其大小即 delta-v。当大小非正或非有限，
    /// 或方向为零或非有限时返回错误。
    pub fn along(direction: Vec3, magnitude: f64) -> Result<Self, ManeuverError> {
        if !magnitude.is_finite() || magnitude <= 0.0 {
            return Err(
                ManeuverError::new(ManeuverErrorKind::NonPositiveAcceleration)
                    .with_detail(format!("magnitude {magnitude}")),
            );
        }
        let unit = direction.normalize().ok_or_else(|| {
            ManeuverError::new(ManeuverErrorKind::InvalidState)
                .with_detail("the direction is zero or not finite")
        })?;
        Self::new(unit * magnitude)
    }

    /// Returns the velocity change vector, in kilometers per second.
    ///
    /// 返回速度变化矢量，单位为千米每秒。
    pub const fn delta_v(self) -> Vec3 {
        self.delta_v
    }

    /// Returns the magnitude of the change, in kilometers per second.
    ///
    /// This is the delta-v the burn costs.
    ///
    /// 返回该变化的大小，单位为千米每秒。
    ///
    /// 这就是该次点火所需的 delta-v。
    pub fn magnitude(self) -> f64 {
        self.delta_v.norm()
    }

    /// Returns a copy of the change scaled by a factor.
    ///
    /// A factor of zero produces a zero impulse, which [`Impulse::new`]
    /// rejects, so this returns an error for it rather than a value that
    /// cannot be built.
    ///
    /// 返回该变化按某因子缩放后的副本。
    ///
    /// 因子为零会得到零冲量，而 [`Impulse::new`] 拒绝这种情况，故本函数对该
    /// 因子返回错误，而不是给出一个无法构造的值。
    pub fn scaled(self, factor: f64) -> Result<Self, ManeuverError> {
        Self::new(self.delta_v * factor)
    }

    /// Returns a copy of the change reversed, which undoes the burn.
    ///
    /// 返回取反后的变化副本，即撤销该次点火。
    pub fn reversed(self) -> Self {
        Self {
            delta_v: -self.delta_v,
        }
    }

    /// Applies the impulse to a velocity, returning the new velocity.
    ///
    /// 把该冲量作用于一个速度，返回新的速度。
    pub fn apply_to_velocity(self, velocity: Vec3) -> Vec3 {
        velocity + self.delta_v
    }
}

impl core::fmt::Display for Impulse {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "impulse of {:.6} km/s", self.magnitude())
    }
}
