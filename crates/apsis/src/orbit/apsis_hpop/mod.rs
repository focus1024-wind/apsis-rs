//! Numerical J2 propagation for high-precision orbit studies.
//!
//! 用于高精度轨道研究的 J2 数值传播。

use crate::apsis_core::{Error, ErrorKind, Result};
use crate::apsis_math::Vec3;

/// One state of a numerically propagated orbit.
///
/// 数值传播轨道的一个状态。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HpopState {
    /// Seconds from the initial state.
    pub seconds: f64,
    /// Position in the inertial frame, in kilometers.
    pub position: Vec3,
    /// Velocity in the inertial frame, in kilometers per second.
    pub velocity: Vec3,
}

/// Force-model parameters for J2 propagation.
///
/// J2 传播的力模型参数。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct J2Model {
    /// Gravitational parameter, in km^3/s^2.
    pub mu: f64,
    /// Second zonal harmonic.
    pub j2: f64,
    /// Equatorial radius, in kilometers.
    pub equatorial_radius_km: f64,
}

impl Default for J2Model {
    fn default() -> Self {
        Self {
            mu: 398_600.441_8,
            j2: 1.082_626_683e-3,
            equatorial_radius_km: 6_378.137,
        }
    }
}

fn acceleration(position: Vec3, model: J2Model) -> Vec3 {
    let radius = position.norm();
    if radius == 0.0 {
        return Vec3::new(0.0, 0.0, 0.0);
    }
    let two_body = position * (-model.mu / radius.powi(3));
    let z2 = position.z * position.z;
    let r2 = radius * radius;
    let factor = 1.5 * model.j2 * model.mu * model.equatorial_radius_km.powi(2) / radius.powi(5);
    let common = 5.0 * z2 / r2;
    let j2 = Vec3::new(
        factor * position.x * (common - 1.0),
        factor * position.y * (common - 1.0),
        factor * position.z * (common - 3.0),
    );
    two_body + j2
}

/// Propagates an initial inertial state under two-body plus J2 acceleration.
///
/// Uses classical fourth-order Runge-Kutta with the requested fixed step.
/// Returns the initial state followed by one state per step; the final sample
/// is shortened when the duration is not an exact multiple of the step.
///
/// 在二体加 J2 加速度下传播初始惯性状态。
///
/// 使用经典四阶 Runge-Kutta 与固定步长。返回初始状态及每步状态；当总时长
/// 不是步长的整数倍时，最后一步缩短。
pub fn propagate_j2(
    initial_position: Vec3,
    initial_velocity: Vec3,
    duration_seconds: f64,
    step_seconds: f64,
    model: J2Model,
) -> Result<Vec<HpopState>> {
    if !duration_seconds.is_finite() || duration_seconds < 0.0 {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "duration must be a non-negative finite number",
        ));
    }
    if !step_seconds.is_finite() || step_seconds <= 0.0 {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "step must be a positive finite number",
        ));
    }
    if !model.mu.is_finite() || model.mu <= 0.0 || model.equatorial_radius_km <= 0.0 {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "force model must have positive finite mu and radius",
        ));
    }
    let mut states = Vec::new();
    let mut position = initial_position;
    let mut velocity = initial_velocity;
    let mut seconds = 0.0;
    states.push(HpopState { seconds, position, velocity });
    while seconds < duration_seconds {
        let dt = step_seconds.min(duration_seconds - seconds);
        let (next_position, next_velocity) = rk4(position, velocity, dt, model);
        seconds += dt;
        position = next_position;
        velocity = next_velocity;
        states.push(HpopState { seconds, position, velocity });
    }
    Ok(states)
}

fn rk4(position: Vec3, velocity: Vec3, dt: f64, model: J2Model) -> (Vec3, Vec3) {
    let k1_v = acceleration(position, model);
    let k1_r = velocity;

    let k2_r = velocity + k1_v * (dt / 2.0);
    let k2_v = acceleration(position + k1_r * (dt / 2.0), model);

    let k3_r = velocity + k2_v * (dt / 2.0);
    let k3_v = acceleration(position + k2_r * (dt / 2.0), model);

    let k4_r = velocity + k3_v * dt;
    let k4_v = acceleration(position + k3_r * dt, model);

    let next_position = position + (k1_r + k2_r * 2.0 + k3_r * 2.0 + k4_r) * (dt / 6.0);
    let next_velocity = velocity + (k1_v + k2_v * 2.0 + k3_v * 2.0 + k4_v) * (dt / 6.0);
    (next_position, next_velocity)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn circular_state() -> (Vec3, Vec3) {
        let radius = 7_000.0_f64;
        let speed = (398_600.441_8 / radius).sqrt();
        (Vec3::new(radius, 0.0, 0.0), Vec3::new(0.0, speed * 51.6_f64.to_radians().cos(), speed * 51.6_f64.to_radians().sin()))
    }

    #[test]
    fn zero_j2_is_a_two_body_orbit() {
        let (position, velocity) = circular_state();
        let states = propagate_j2(
            position,
            velocity,
            600.0,
            10.0,
            J2Model { j2: 0.0, ..J2Model::default() },
        )
        .expect("propagation");
        assert_eq!(states.len(), 61);
        for state in &states {
            assert!((state.position.norm() - 7_000.0).abs() < 1e-6);
            assert!((state.velocity.norm() - (398_600.441_8_f64 / 7_000.0).sqrt()).abs() < 1e-9);
        }
    }

    #[test]
    fn j2_changes_the_orbit_plane() {
        let (position, velocity) = circular_state();
        let states = propagate_j2(position, velocity, 86_400.0, 60.0, J2Model::default()).expect("propagation");
        let initial = states.first().expect("initial");
        let final_state = states.last().expect("final");
        let h0 = initial.position.cross(initial.velocity).normalize().expect("normal");
        let h1 = final_state.position.cross(final_state.velocity).normalize().expect("normal");
        assert!((h0.dot(h1) - 1.0).abs() > 1e-12);
    }

    #[test]
    fn invalid_sampling_is_rejected() {
        let (position, velocity) = circular_state();
        assert!(propagate_j2(position, velocity, -1.0, 10.0, J2Model::default()).is_err());
        assert!(propagate_j2(position, velocity, 10.0, 0.0, J2Model::default()).is_err());
    }
}
