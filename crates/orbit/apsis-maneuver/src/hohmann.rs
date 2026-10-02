//! The two impulse Hohmann transfer between coplanar circular orbits.
//!
//! 共面圆轨道之间的双脉冲 Hohmann 转移。

use crate::error::{ManeuverError, ManeuverErrorKind};
use apsis_kepler::GravityParameter;

/// A two impulse transfer between two coplanar circular orbits.
///
/// The transfer is the classic Hohmann maneuver: a burn onto an ellipse
/// whose perigee touches the lower orbit and whose apogee touches the upper
/// one, a coast for half the ellipse's period, and a second burn onto the
/// destination circle. It is the cheapest two impulse transfer between two
/// coplanar circular orbits in the impulsive model, which is why it is the
/// reference every other transfer is measured against.
///
/// The burns are signed along the direction of motion: a positive value
/// raises the orbit and a negative one lowers it, so the same type covers a
/// climb and a descent. The total is the sum of the magnitudes, which is the
/// quantity a propellant budget uses.
///
/// 两个共面圆轨道之间的双脉冲转移。
///
/// 该转移即经典的 Hohmann 机动：一次点火进入椭圆，其近地点相切于较低轨道、
/// 远地点相切于较高轨道，沿该椭圆滑行半个周期，再一次点火进入目标圆轨道。它
/// 是脉冲模型中两个共面圆轨道之间最省的双脉冲转移，因此成为衡量其他转移的
/// 基准。
///
/// 两次点火沿运动方向带符号：正值抬升轨道，负值降低轨道，故同一类型同时
/// 表达上升与下降。总和取两者模之和，即推进剂预算所用的量。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HohmannTransfer {
    /// The radius of the orbit the transfer starts from, in kilometers.
    ///
    /// 转移起始轨道的半径，单位为千米。
    pub from_radius: f64,
    /// The radius of the orbit the transfer arrives at, in kilometers.
    ///
    /// 转移到达轨道的半径，单位为千米。
    pub to_radius: f64,
    /// The semi-major axis of the transfer ellipse, in kilometers.
    ///
    /// 转移椭圆的半长轴，单位为千米。
    pub transfer_semi_major_axis: f64,
    /// The first burn, along the direction of motion, in kilometers per
    /// second.
    ///
    /// 第一次点火，沿运动方向，单位为千米每秒。
    pub first_burn: f64,
    /// The second burn, along the direction of motion, in kilometers per
    /// second.
    ///
    /// 第二次点火，沿运动方向，单位为千米每秒。
    pub second_burn: f64,
    /// The coast time between the burns, in seconds.
    ///
    /// 两次点火之间的滑行时间，单位为秒。
    pub transfer_time: f64,
}

impl HohmannTransfer {
    /// Returns the total velocity change, in kilometers per second.
    ///
    /// This is the sum of the magnitudes of the two burns, which is what the
    /// transfer costs whatever its direction.
    ///
    /// 返回总速度变化，单位为千米每秒。
    ///
    /// 它是两次点火大小之和，即无论方向如何该转移所需的代价。
    #[must_use]
    pub fn total_delta_v(&self) -> f64 {
        self.first_burn.abs() + self.second_burn.abs()
    }

    /// Returns `true` when the transfer raises the orbit.
    ///
    /// 当该转移抬升轨道时返回 `true`。
    #[must_use]
    pub fn is_raising(&self) -> bool {
        self.to_radius > self.from_radius
    }

    /// Returns the velocity change of the first burn, unsigned.
    ///
    /// 返回第一次点火的速度变化，不带符号。
    #[must_use]
    pub fn first_burn_magnitude(&self) -> f64 {
        self.first_burn.abs()
    }

    /// Returns the velocity change of the second burn, unsigned.
    ///
    /// 返回第二次点火的速度变化，不带符号。
    #[must_use]
    pub fn second_burn_magnitude(&self) -> f64 {
        self.second_burn.abs()
    }

    /// Returns the mean radius the transfer is symmetric about, in
    /// kilometers.
    ///
    /// This is the semi-major axis of the ellipse, which is the arithmetic
    /// mean of the two orbit radii.
    ///
    /// 返回该转移所对称的平均半径，单位为千米。
    ///
    /// 它即该椭圆的半长轴，为两个轨道半径的算术平均。
    #[must_use]
    pub fn mean_radius(&self) -> f64 {
        self.transfer_semi_major_axis
    }
}

/// Returns the two impulse transfer between two circular orbits.
///
/// The radii are in kilometers and must be finite and positive. The two may
/// be given in either order: a larger second radius is a climb and a smaller
/// one a descent.
///
/// Returns an error when a radius is not usable or when the two are equal,
/// because a transfer between an orbit and itself is not a maneuver and is
/// almost always a mistake in a plan.
///
/// The burns follow from the vis-viva equation. At the first radius the
/// circular speed is `sqrt(mu / r1)` and the speed on the transfer ellipse
/// is `sqrt(mu (2 / r1 - 1 / a))`, where `a` is the semi-major axis of the
/// ellipse, so the first burn is the difference. The second burn is the
/// difference between the circular speed at the destination and the speed on
/// the ellipse there, taken the other way round because the ellipse arrives
/// slower than the circle it must join. The coast time is half the period of
/// the ellipse.
///
/// 返回两个圆轨道之间的双脉冲转移。
///
/// 半径单位为千米，须有限且为正。二者可按任意次序给出：第二个半径较大者为
/// 上升，较小者为下降。
///
/// 当某半径不可用或二者相等时返回错误，因为轨道到其自身的转移不是机动，且
/// 几乎总是计划中的笔误。
///
/// 两次点火由活力公式导出。在第一个半径处圆轨道速度为 `sqrt(mu / r1)`，而
/// 转移椭圆上的速度为 `sqrt(mu (2 / r1 - 1 / a))`，其中 `a` 为椭圆半长轴，
/// 第一次点火即二者之差。第二次点火是目标处圆轨道速度与椭圆上速度之差，取
/// 相反方向，因为椭圆到达时比它必须汇入的圆轨道更慢。滑行时间为该椭圆
/// 周期的一半。
pub fn hohmann_transfer(
    from_radius: f64,
    to_radius: f64,
    parameter: GravityParameter,
) -> Result<HohmannTransfer, ManeuverError> {
    for radius in [from_radius, to_radius] {
        if !radius.is_finite() || radius <= 0.0 {
            return Err(ManeuverError::new(ManeuverErrorKind::InvalidRadius)
                .with_detail(format!("radius {radius}")));
        }
    }
    if from_radius == to_radius {
        return Err(ManeuverError::new(ManeuverErrorKind::DegenerateTransfer)
            .with_detail(format!("both radii are {from_radius}")));
    }

    let mu = parameter.value();
    let transfer_semi_major_axis = f64::midpoint(from_radius, to_radius);

    // The circular speeds at each radius.
    // 各半径处的圆轨道速度。
    let circular_from = (mu / from_radius).sqrt();
    let circular_to = (mu / to_radius).sqrt();

    // The speeds on the transfer ellipse at each radius, from the vis-viva
    // equation `v^2 = mu (2 / r - 1 / a)`.
    // 各半径处转移椭圆上的速度，由活力公式 `v^2 = mu (2 / r - 1 / a)` 得出。
    let transfer_from = (mu * (2.0 / from_radius - 1.0 / transfer_semi_major_axis)).sqrt();
    let transfer_to = (mu * (2.0 / to_radius - 1.0 / transfer_semi_major_axis)).sqrt();

    // The first burn accelerates onto the ellipse; the second accelerates
    // from the ellipse speed up to the circular speed at the destination.
    // Both are along the direction of motion, and the sign follows from
    // which speed is larger.
    // 第一次点火把卫星加速到椭圆上；第二次把椭圆上的速度加速到目标处圆轨道
    // 速度。两者均沿运动方向，符号取决于哪个速度更大。
    let first_burn = transfer_from - circular_from;
    let second_burn = circular_to - transfer_to;

    // Half the period of the transfer ellipse.
    // 转移椭圆周期的一半。
    let transfer_time = core::f64::consts::PI * (transfer_semi_major_axis.powi(3) / mu).sqrt();

    Ok(HohmannTransfer {
        from_radius,
        to_radius,
        transfer_semi_major_axis,
        first_burn,
        second_burn,
        transfer_time,
    })
}
