//! The propagated state of a satellite.
//!
//! 卫星被传播后的状态。

use crate::apsis_math::Vec3;

/// The position and velocity of a satellite in the True Equator Mean Equinox
/// frame.
///
/// Position is in kilometers and velocity is in kilometers per second. The
/// frame is the one the model defines, with its first axis toward the mean
/// equinox, its third axis along the true equator pole, and the second
/// completing a right-handed set.
///
/// 真赤道平春分点系中卫星的位置与速度。
///
/// 位置单位为千米，速度单位为千米每秒。该系是模型所定义者：第一轴指向平春分点，
/// 第三轴沿真赤道极，第二轴构成右手系。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct State {
    /// Position, in kilometers.
    ///
    /// 位置，单位为千米。
    pub position: Vec3,
    /// Velocity, in kilometers per second.
    ///
    /// 速度，单位为千米每秒。
    pub velocity: Vec3,
    /// The propagated semi-major axis, in kilometers.
    ///
    /// This is the semi-major axis after the secular drag term has been
    /// applied, so it differs from the value implied by the mean motion in
    /// the element set.
    ///
    /// 传播所得的半长轴，单位为千米。
    ///
    /// 这是施加长期阻力项之后的半长轴，故与根数集中平运动所隐含的值不同。
    pub semi_major_axis: f64,
}

impl State {
    /// Returns the distance from the center of the Earth, in kilometers.
    ///
    /// 返回到地心的距离，单位为千米。
    #[must_use]
    pub fn altitude_radius(self) -> f64 {
        self.position.norm()
    }

    /// Returns the speed relative to the Earth's center, in kilometers per
    /// second.
    ///
    /// 返回相对地心的速率，单位为千米每秒。
    #[must_use]
    pub fn speed(self) -> f64 {
        self.velocity.norm()
    }
}
