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

/// Exponential-atmosphere drag parameters.
///
/// 指数大气阻力参数。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DragModel {
    /// Drag coefficient, dimensionless.
    pub drag_coefficient: f64,
    /// Cross-sectional area, in square meters.
    pub area_m2: f64,
    /// Spacecraft mass, in kilograms.
    pub mass_kg: f64,
    /// Reference atmospheric density, in kilograms per cubic meter.
    pub reference_density_kg_m3: f64,
    /// Altitude of the reference density, in kilometers.
    pub reference_altitude_km: f64,
    /// Exponential scale height, in kilometers.
    pub scale_height_km: f64,
}

impl DragModel {
    /// Returns a representative LEO drag model: 1 m^2, 100 kg, Cd 2.2.
    ///
    /// 返回代表性 LEO 阻力模型：1 m^2、100 kg、Cd 2.2。
    #[must_use]
    pub fn wgs84_defaults() -> Self {
        Self {
            drag_coefficient: 2.2,
            area_m2: 1.0,
            mass_kg: 100.0,
            reference_density_kg_m3: 2.0e-11,
            reference_altitude_km: 350.0,
            scale_height_km: 50.0,
        }
    }
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
    /// Fourth zonal harmonic; zero disables the J4 term.
    pub j4: f64,
    /// Equatorial radius, in kilometers.
    pub equatorial_radius_km: f64,
    /// Optional exponential-atmosphere drag.
    pub drag: Option<DragModel>,
}

impl Default for J2Model {
    fn default() -> Self {
        Self {
            mu: 398_600.441_8,
            j2: 1.082_626_683e-3,
            j4: 0.0,
            equatorial_radius_km: 6_378.137,
            drag: None,
        }
    }
}

fn acceleration(position: Vec3, velocity: Vec3, model: J2Model) -> Vec3 {
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
    let j4 = if model.j4 == 0.0 {
        Vec3::new(0.0, 0.0, 0.0)
    } else {
        let z4 = z2 * z2;
        let factor = 0.625 * model.j4 * model.mu * model.equatorial_radius_km.powi(4) / radius.powi(7);
        Vec3::new(
            factor * position.x * (3.0 - 30.0 * z2 / r2 + 35.0 * z4 / (r2 * r2)),
            factor * position.y * (3.0 - 30.0 * z2 / r2 + 35.0 * z4 / (r2 * r2)),
            factor * position.z * (15.0 - 70.0 * z2 / r2 + 63.0 * z4 / (r2 * r2)),
        )
    };
    let drag = if let Some(drag) = model.drag {
        if drag.mass_kg > 0.0 && drag.area_m2 > 0.0 {
            let altitude = radius - model.equatorial_radius_km;
            let density = drag.reference_density_kg_m3
                * (-(altitude - drag.reference_altitude_km) / drag.scale_height_km).exp();
            let earth_rate = 7.292_115_9e-5;
            let atmosphere = Vec3::new(-earth_rate * position.y, earth_rate * position.x, 0.0);
            let relative = velocity - atmosphere;
            let speed = relative.norm();
            if speed > 0.0 && density.is_finite() {
                let acceleration_m_s2 = 0.5
                    * density
                    * speed
                    * speed
                    * drag.drag_coefficient
                    * drag.area_m2
                    / drag.mass_kg;
                relative.normalize().unwrap_or(Vec3::new(0.0, 0.0, 0.0))
                    * (-acceleration_m_s2 / 1_000.0)
            } else {
                Vec3::new(0.0, 0.0, 0.0)
            }
        } else {
            Vec3::new(0.0, 0.0, 0.0)
        }
    } else {
        Vec3::new(0.0, 0.0, 0.0)
    };
    two_body + j2 + j4 + drag
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
    if let Some(drag) = model.drag {
        if !drag.drag_coefficient.is_finite()
            || drag.drag_coefficient <= 0.0
            || !drag.area_m2.is_finite()
            || drag.area_m2 <= 0.0
            || !drag.mass_kg.is_finite()
            || drag.mass_kg <= 0.0
            || !drag.reference_density_kg_m3.is_finite()
            || drag.reference_density_kg_m3 < 0.0
            || !drag.reference_altitude_km.is_finite()
            || !drag.scale_height_km.is_finite()
            || drag.scale_height_km <= 0.0
        {
            return Err(Error::with_message(
                ErrorKind::InvalidInput,
                "drag model must have positive finite parameters and a non-negative density",
            ));
        }
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
    let k1_v = acceleration(position, velocity, model);
    let k1_r = velocity;

    let k2_r = velocity + k1_v * (dt / 2.0);
    let k2_v = acceleration(position + k1_r * (dt / 2.0), velocity + k2_r * (dt / 2.0), model);

    let k3_r = velocity + k2_v * (dt / 2.0);
    let k3_v = acceleration(position + k2_r * (dt / 2.0), velocity + k3_r * (dt / 2.0), model);

    let k4_r = velocity + k3_v * dt;
    let k4_v = acceleration(position + k3_r * dt, velocity + k4_r * dt, model);

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
    fn j4_perturbation_is_finite_and_changes_the_state() {
        let position = Vec3::new(0.0, 6_778.137, 0.0);
        let velocity = Vec3::new(7.660, 0.0, 0.0);
        let without = propagate_j2(position, velocity, 600.0, 60.0, J2Model::default()).expect("J2");
        let with = propagate_j2(
            position,
            velocity,
            600.0,
            60.0,
            J2Model { j4: -1.620_000e-6, ..J2Model::default() },
        )
        .expect("J2+J4");
        let last_without = without.last().expect("state");
        let last_with = with.last().expect("state");
        assert!(last_with.position.norm().is_finite());
        assert!((last_with.position - last_without.position).norm() > 1.0e-12);
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
    fn drag_removes_energy_and_changes_the_state() {
        let radius = 6_778.137_f64;
        let speed = (398_600.441_8 / radius).sqrt();
        let position = Vec3::new(radius, 0.0, 0.0);
        let velocity = Vec3::new(0.0, speed, 0.0);
        let without = propagate_j2(position, velocity, 600.0, 60.0, J2Model::default()).expect("J2");
        let with = propagate_j2(
            position,
            velocity,
            600.0,
            60.0,
            J2Model { drag: Some(DragModel::wgs84_defaults()), ..J2Model::default() },
        )
        .expect("J2+drag");
        let last_without = without.last().expect("state");
        let last_with = with.last().expect("state");
        assert!((last_with.position - last_without.position).norm() > 1.0e-9);
        assert!(last_with.velocity.norm() < last_without.velocity.norm() + 1.0e-12);
    }

    #[test]
    fn an_invalid_drag_model_is_rejected() {
        let (position, velocity) = circular_state();
        let drag = DragModel { mass_kg: 0.0, ..DragModel::wgs84_defaults() };
        assert!(propagate_j2(
            position,
            velocity,
            60.0,
            10.0,
            J2Model { drag: Some(drag), ..J2Model::default() },
        )
        .is_err());
    }

    #[test]
    fn invalid_sampling_is_rejected() {
        let (position, velocity) = circular_state();
        assert!(propagate_j2(position, velocity, -1.0, 10.0, J2Model::default()).is_err());
        assert!(propagate_j2(position, velocity, 10.0, 0.0, J2Model::default()).is_err());
    }
}
