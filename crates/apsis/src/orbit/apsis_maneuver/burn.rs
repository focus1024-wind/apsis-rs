//! The finite burn.
//!
//! 有限点火。

use crate::apsis_kepler::{GravityParameter, KeplerianElements, State};
use crate::apsis_maneuver::error::{ManeuverError, ManeuverErrorKind};
use crate::apsis_math::Vec3;

/// A maneuver that applies a constant acceleration for a duration.
///
/// A finite burn is the model for a thruster that is on for long enough that
/// its duration cannot be ignored: an electric thruster, whose burn may last
/// a large fraction of an orbit, or any burn where the spacecraft moves
/// appreciably while thrusting.
///
/// The acceleration is held constant in an inertial frame chosen by the
/// caller. That is the simplification this model makes: a real thruster
/// fires along a body axis that rotates as the spacecraft turns to hold a
/// direction, and modelling that requires the attitude history, which is not
/// part of this module.
///
/// 在一段时间内施加恒定加速度的机动。
///
/// 有限推力点火适用于推力器工作时间足够长、其时长无法忽略的情形：电推力器的
/// 点火可能持续轨道相当大一部分，任何推力期间航天器有明显位移的点火亦属此类。
///
/// 加速度在调用方选定的惯性系中保持恒定。这是本模型所作的简化：真实推力器
/// 沿随航天器转动以保持方向的本体轴喷焰，建模这一点需要姿态历史，而它不属
/// 于本模块的范围。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FiniteBurn {
    /// The acceleration vector, in kilometers per second squared.
    ///
    /// 加速度矢量，单位为千米每二次方秒。
    acceleration: Vec3,
    /// The duration, in seconds.
    ///
    /// 时长，单位为秒。
    duration: f64,
}

impl FiniteBurn {
    /// Creates a burn from an acceleration and a duration.
    ///
    /// Returns an error when a value is not finite, when the duration is not
    /// positive, or when the acceleration is zero.
    ///
    /// 由加速度与时长创建一次点火。
    ///
    /// 当某个值非有限、时长不为正或加速度为零时返回错误。
    pub fn new(acceleration: Vec3, duration: f64) -> Result<Self, ManeuverError> {
        if !acceleration.is_finite() || !duration.is_finite() {
            return Err(ManeuverError::new(ManeuverErrorKind::NotFinite));
        }
        if duration <= 0.0 {
            return Err(ManeuverError::new(ManeuverErrorKind::NonPositiveDuration)
                .with_detail(format!("duration {duration}")));
        }
        if acceleration.norm() == 0.0 {
            return Err(
                ManeuverError::new(ManeuverErrorKind::NonPositiveAcceleration)
                    .with_detail("the acceleration is zero"),
            );
        }
        Ok(Self {
            acceleration,
            duration,
        })
    }

    /// Creates a burn of a given acceleration magnitude along a direction.
    ///
    /// The direction is normalized, so the acceleration magnitude is fixed
    /// whatever the length of the direction vector.
    ///
    /// 沿给定方向创建指定大小加速度的点火。
    ///
    /// 方向被归一化，故无论方向矢量多长，加速度大小都固定不变。
    pub fn along(direction: Vec3, acceleration: f64, duration: f64) -> Result<Self, ManeuverError> {
        if !acceleration.is_finite() || acceleration <= 0.0 {
            return Err(
                ManeuverError::new(ManeuverErrorKind::NonPositiveAcceleration)
                    .with_detail(format!("acceleration {acceleration}")),
            );
        }
        let unit = direction.normalize().ok_or_else(|| {
            ManeuverError::new(ManeuverErrorKind::InvalidState)
                .with_detail("the direction is zero or not finite")
        })?;
        Self::new(unit * acceleration, duration)
    }

    /// Returns the acceleration vector, in kilometers per second squared.
    ///
    /// 返回加速度矢量，单位为千米每二次方秒。
    #[must_use]
    pub const fn acceleration(self) -> Vec3 {
        self.acceleration
    }

    /// Returns the magnitude of the acceleration, in kilometers per second
    /// squared.
    ///
    /// 返回加速度大小，单位为千米每二次方秒。
    #[must_use]
    pub fn acceleration_magnitude(self) -> f64 {
        self.acceleration.norm()
    }

    /// Returns the duration, in seconds.
    ///
    /// 返回时长，单位为秒。
    #[must_use]
    pub const fn duration(self) -> f64 {
        self.duration
    }

    /// Returns the velocity change the acceleration would give in free space,
    /// in kilometers per second.
    ///
    /// This is the acceleration times the duration, the delta-v the burn
    /// would deliver without gravity acting during it. Under gravity the
    /// achieved change is close to this for a short burn and differs from it
    /// for a long one, which is the difference the finite model exists to
    /// capture.
    ///
    /// 返回该加速度在自由空间中会造成的速度变化，单位为千米每秒。
    ///
    /// 它是加速度乘以时长，即点火在不受引力作用时会提供的 delta-v。在引力
    /// 作用下，短点火的实际变化与之接近，长点火则不同，这正是有限推力模型
    /// 得以存在的意义。
    #[must_use]
    pub fn nominal_delta_v(self) -> f64 {
        self.acceleration.norm() * self.duration
    }

    /// Returns the impulse equivalent to the nominal change.
    ///
    /// This is the impulsive burn the finite one approximates as its
    /// duration shrinks, and it is what [`FiniteBurn::apply`] returns in the
    /// limit of a constant gravity field.
    ///
    /// 返回与标称变化等效的冲量。
    ///
    /// 它是有限点火在时长趋零时所逼近的脉冲点火，也是 [`FiniteBurn::apply`]
    /// 在恒定引力场极限下给出的量。
    pub fn equivalent_impulse(
        self,
    ) -> Result<crate::apsis_maneuver::impulse::Impulse, ManeuverError> {
        crate::apsis_maneuver::impulse::Impulse::new(self.acceleration * self.duration)
    }

    /// Applies the burn to a state under two-body gravity.
    ///
    /// The position and velocity are integrated forward over the duration
    /// with the constant acceleration added to gravity. The exact solution
    /// of constant acceleration plus point mass gravity has no closed form,
    /// so the integration is a fixed step fourth order Runge-Kutta, which is
    /// accurate for the short arcs a burn occupies.
    ///
    /// The achieved velocity change is measured against a coast of the same
    /// duration, not against the initial velocity. That distinction matters:
    /// gravity changes the velocity during the burn even with no thrust, so
    /// the difference between the final and the initial velocity includes
    /// the natural rotation of the velocity vector. Comparing against the
    /// coast isolates the contribution of the thrust, which is what the
    /// burn bought; it also makes the result converge on the nominal change
    /// as the duration shrinks, which the difference from the initial
    /// velocity would not.
    ///
    /// Returns the state at the end of the burn together with the achieved
    /// velocity change.
    ///
    /// 在二体引力下把该点火作用于一个状态。
    ///
    /// 位置与速度在时长内积分前进，积分时把恒定加速度叠加到引力上。恒定加速度
    /// 加质点引力的精确解没有闭式，故用定步长四阶 Runge-Kutta 积分，对点火所占
    /// 的短弧段是精确的。
    ///
    /// 实际达到的速度变化以同时长的滑行作为比较基准，而非以初始速度。区分这
    /// 一点很重要：即使无推力，引力在点火期间也会改变速度，故末速度与初速度
    /// 之差含速度矢量的自然转动。与滑行比较才能隔离出推力的贡献，即该次点火
    /// 实际买到的东西；这也使结果在时长趋零时收敛到标称变化，而与初速度
    /// 作差则不然。
    ///
    /// 返回点火结束时的状态及实际达到的速度变化。
    pub fn apply(
        self,
        state: State,
        parameter: GravityParameter,
        steps: usize,
    ) -> Result<BurnOutcome, ManeuverError> {
        if !state.position.is_finite() || !state.velocity.is_finite() {
            return Err(ManeuverError::new(ManeuverErrorKind::InvalidState));
        }
        if steps == 0 {
            return Err(ManeuverError::new(ManeuverErrorKind::InvalidState)
                .with_detail("the step count is zero"));
        }
        let mu = parameter.value();
        let step = self.duration / steps as f64;

        // The two-body acceleration at a position.
        // 某位置处的二体加速度。
        let gravity = |position: Vec3| -> Vec3 {
            let radius = position.norm();
            if radius == 0.0 {
                return Vec3::ZERO;
            }
            position * (-mu / (radius * radius * radius))
        };
        // The thrust added to gravity, which is what the burn integrates.
        // 叠加到引力上的推力，即点火所积分的量。
        let thrust_and_gravity = |position: Vec3| -> Vec3 { gravity(position) + self.acceleration };

        // The burn trajectory.
        // 点火轨迹。
        let mut position = state.position;
        let mut velocity = state.velocity;
        for _ in 0..steps {
            let (new_position, new_velocity) =
                rk4_step(position, velocity, step, &thrust_and_gravity);
            position = new_position;
            velocity = new_velocity;
        }

        // The coast trajectory over the same duration, with the same
        // integrator and step, so the only difference is the thrust.
        // 同时长、同积分器同步长的滑行轨迹，故二者唯一差别是推力。
        let mut coast_position = state.position;
        let mut coast_velocity = state.velocity;
        for _ in 0..steps {
            let (new_position, new_velocity) =
                rk4_step(coast_position, coast_velocity, step, &gravity);
            coast_position = new_position;
            coast_velocity = new_velocity;
        }

        if !position.is_finite() || !velocity.is_finite() {
            return Err(ManeuverError::new(ManeuverErrorKind::InvalidState)
                .with_detail("the integration diverged"));
        }

        Ok(BurnOutcome {
            state: State { position, velocity },
            achieved_delta_v: velocity - coast_velocity,
            nominal_delta_v: self.nominal_delta_v(),
            duration: self.duration,
        })
    }

    /// Applies the burn to a set of Keplerian elements.
    ///
    /// The elements are converted to a state, the burn is applied to it, and
    /// the result is converted back. Returns an error when the elements are
    /// not valid, when the integration fails, or when the resulting state is
    /// no longer an elliptical orbit, which happens when the burn is large
    /// enough to escape.
    ///
    /// 把该点火作用于一组开普勒根数。
    ///
    /// 先把根数转为状态，对状态施加点火，再转回根数。当根数无效、积分失败，或
    /// 结果状态不再是椭圆轨道时返回错误；后者在点火足够大以致逃逸时发生。
    pub fn apply_to_elements(
        self,
        elements: KeplerianElements,
        parameter: GravityParameter,
        steps: usize,
    ) -> Result<BurnOutcome, ManeuverError> {
        let state = elements.to_state(parameter).ok_or_else(|| {
            ManeuverError::new(ManeuverErrorKind::InvalidState)
                .with_detail("the elements are not a valid elliptical orbit")
        })?;
        self.apply(state, parameter, steps)
    }
}

/// What a finite burn produced.
///
/// The outcome keeps the final state, the change the burn achieved, and the
/// change the acceleration would have given in free space, so a caller can
/// see how much gravity cost or added during the burn.
///
/// 一次有限推力点火的产物。
///
/// 结果保留末状态、点火实际达到的变化，以及该加速度在自由空间中会造成的
/// 变化，使调用方可以看出点火期间引力造成或增加了多少。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BurnOutcome {
    /// The state at the end of the burn.
    ///
    /// 点火结束时的状态。
    pub state: State,
    /// The velocity change actually achieved, in kilometers per second.
    ///
    /// 实际达到的速度变化，单位为千米每秒。
    pub achieved_delta_v: Vec3,
    /// The velocity change the acceleration would give in free space, in
    /// kilometers per second.
    ///
    /// 该加速度在自由空间中会造成的速度变化，单位为千米每秒。
    pub nominal_delta_v: f64,
    /// The burn duration, in seconds.
    ///
    /// 点火时长，单位为秒。
    pub duration: f64,
}

impl BurnOutcome {
    /// Returns the magnitude of the achieved velocity change.
    ///
    /// 返回实际达到的速度变化的大小。
    #[must_use]
    pub fn achieved_magnitude(&self) -> f64 {
        self.achieved_delta_v.norm()
    }

    /// Returns the difference between the achieved and nominal change, in
    /// kilometers per second.
    ///
    /// A negative value means gravity turned the velocity against the thrust
    /// during the burn, so the burn bought less than its free space delta-v.
    ///
    /// 返回实际变化与标称变化之差，单位为千米每秒。
    ///
    /// 负值表示点火期间引力把速度转向了推力的反方向，使该次点火的收获小于其
    /// 自由空间 delta-v。
    #[must_use]
    pub fn gravity_loss(&self) -> f64 {
        self.achieved_magnitude() - self.nominal_delta_v
    }

    /// Returns the elements at the end of the burn.
    ///
    /// Returns `None` when the final state is not an elliptical orbit, which
    /// happens when the burn is large enough to escape.
    ///
    /// 返回点火结束时的根数。
    ///
    /// 当末状态不是椭圆轨道时返回 `None`；这在点火足够大以致逃逸时发生。
    #[must_use]
    pub fn elements(&self, parameter: GravityParameter) -> Option<KeplerianElements> {
        KeplerianElements::from_state(&self.state, parameter)
    }
}

/// One classical Runge-Kutta fourth order step of the two-body equations
/// with a constant acceleration added.
///
/// The state is `(position, velocity)` and the equations are the usual
/// `dr/dt = v` and `dv/dt = a(r)`, with `a` combining thrust and gravity.
/// The four stages are the standard ones; writing them out keeps the loop
/// allocation free.
///
/// 二体方程加上恒定加速度的一步经典四阶 Runge-Kutta。
///
/// 状态为 `(位置, 速度)`，方程即通常的 `dr/dt = v` 与 `dv/dt = a(r)`，其中
/// `a` 合并推力与引力。四个级次是标准形式；直接写出使循环无需分配内存。
fn rk4_step(
    position: Vec3,
    velocity: Vec3,
    step: f64,
    acceleration: &impl Fn(Vec3) -> Vec3,
) -> (Vec3, Vec3) {
    let k1_v = acceleration(position);
    let k1_r = velocity;

    let k2_v = acceleration(position + k1_r * (step / 2.0));
    let k2_r = velocity + k1_v * (step / 2.0);

    let k3_v = acceleration(position + k2_r * (step / 2.0));
    let k3_r = velocity + k2_v * (step / 2.0);

    let k4_v = acceleration(position + k3_r * step);
    let k4_r = velocity + k3_v * step;

    let new_position = position + (k1_r + k2_r * 2.0 + k3_r * 2.0 + k4_r) * (step / 6.0);
    let new_velocity = velocity + (k1_v + k2_v * 2.0 + k3_v * 2.0 + k4_v) * (step / 6.0);
    (new_position, new_velocity)
}
