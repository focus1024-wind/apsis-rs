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


/// A 6x6 state covariance matrix in position-then-velocity order.
///
/// 按位置后速度顺序排列的 6x6 状态协方差矩阵。
pub type Covariance6 = [[f64; 6]; 6];

/// State and covariance at the end of a propagation.
///
/// 传播结束时的状态与协方差。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HpopCovariance {
    /// Final inertial state.
    pub state: HpopState,
    /// Final covariance matrix.
    pub covariance: Covariance6,
}


/// Principal axes of a position covariance ellipsoid.
///
/// 位置协方差椭球的三个主轴。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CovarianceEllipsoid {
    /// Semi-axis lengths, in kilometers, ordered largest to smallest.
    pub semi_axes_km: [f64; 3],
    /// Eigenvectors as columns, ordered to match `semi_axes_km`.
    pub rotation: [[f64; 3]; 3],
}

/// Computes the position covariance ellipsoid at a requested sigma scale.
///
/// Only the upper-left 3x3 position block is used. The symmetric matrix is
/// diagonalized with Jacobi rotations. Negative eigenvalues from numerical
/// roundoff are clamped to zero. Returns an error for non-finite input or a
/// non-positive scale.
///
/// 计算指定 sigma 倍数下的位置协方差椭球。
///
/// 只使用左上角 3x3 位置块。对称矩阵用 Jacobi 旋转对角化，数值舍入造成的
/// 负特征值截断为零。输入非有限或倍率非正时返回错误。
pub fn position_covariance_ellipsoid(
    covariance: &Covariance6,
    sigma_scale: f64,
) -> Result<CovarianceEllipsoid> {
    if !sigma_scale.is_finite() || sigma_scale <= 0.0 {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "sigma scale must be positive and finite",
        ));
    }
    let mut matrix = [[0.0_f64; 3]; 3];
    for row in 0..3 {
        for column in 0..3 {
            if !covariance[row][column].is_finite() {
                return Err(Error::with_message(
                    ErrorKind::InvalidInput,
                    "covariance must be finite",
                ));
            }
            matrix[row][column] = covariance[row][column];
        }
    }
    let mut vectors = [[0.0_f64; 3]; 3];
    for (index, vector) in vectors.iter_mut().enumerate() {
        vector[index] = 1.0;
    }
    for _ in 0..32 {
        let mut p = 0;
        let mut q = 1;
        let mut largest = matrix[0][1].abs();
        for (row, column) in [(0, 2), (1, 2)] {
            if matrix[row][column].abs() > largest {
                largest = matrix[row][column].abs();
                p = row;
                q = column;
            }
        }
        if largest < 1e-15 {
            break;
        }
        let angle = 0.5 * (2.0 * matrix[p][q]).atan2(matrix[q][q] - matrix[p][p]);
        let (sin, cos) = angle.sin_cos();
        let app = matrix[p][p];
        let aqq = matrix[q][q];
        let apq = matrix[p][q];
        matrix[p][p] = cos * cos * app - 2.0 * sin * cos * apq + sin * sin * aqq;
        matrix[q][q] = sin * sin * app + 2.0 * sin * cos * apq + cos * cos * aqq;
        matrix[p][q] = 0.0;
        matrix[q][p] = 0.0;
        for index in 0..3 {
            if index != p && index != q {
                let aip = matrix[index][p];
                let aiq = matrix[index][q];
                matrix[index][p] = cos * aip - sin * aiq;
                matrix[p][index] = matrix[index][p];
                matrix[index][q] = sin * aip + cos * aiq;
                matrix[q][index] = matrix[index][q];
            }
            let vip = vectors[index][p];
            let viq = vectors[index][q];
            vectors[index][p] = cos * vip - sin * viq;
            vectors[index][q] = sin * vip + cos * viq;
        }
    }
    let mut order = [0usize, 1, 2];
    order.sort_by(|left, right| matrix[*right][*right].total_cmp(&matrix[*left][*left]));
    let values = order.map(|index| matrix[index][index].max(0.0).sqrt() * sigma_scale);
    let rotation = [
        [vectors[0][order[0]], vectors[0][order[1]], vectors[0][order[2]]],
        [vectors[1][order[0]], vectors[1][order[1]], vectors[1][order[2]]],
        [vectors[2][order[0]], vectors[2][order[1]], vectors[2][order[2]]],
    ];
    Ok(CovarianceEllipsoid { semi_axes_km: values, rotation })
}

/// Propagates a 6x6 state covariance with a finite-difference state transition matrix.
///
/// The nominal trajectory is propagated with [`propagate_j2`]. Each initial
/// state component is then perturbed by a small amount and propagated over
/// the same interval; the resulting differences form the state transition
/// matrix, which maps the initial covariance to the final covariance. This is
/// a global finite-difference linearization, suitable for short arcs and for
/// engineering estimates rather than for operational covariance realism.
///
/// 用有限差分状态转移矩阵传播 6x6 状态协方差。
///
/// 标称轨迹由 [`propagate_j2`] 得到。随后对每个初始状态分量施加小扰动，并在
/// 同一区间内传播；差值构成状态转移矩阵，把初始协方差映射到终端协方差。这是
/// 全局有限差分线性化，适用于短弧段与工程估计，不用于业务级协方差真实性。
pub fn propagate_j2_covariance(
    initial_position: Vec3,
    initial_velocity: Vec3,
    duration_seconds: f64,
    step_seconds: f64,
    model: J2Model,
    initial_covariance: Covariance6,
) -> Result<HpopCovariance> {
    for row in initial_covariance {
        if row.iter().any(|value| !value.is_finite()) {
            return Err(Error::with_message(
                ErrorKind::InvalidInput,
                "initial covariance must be finite",
            ));
        }
    }
    let nominal = propagate_j2(
        initial_position,
        initial_velocity,
        duration_seconds,
        step_seconds,
        model,
    )?;
    let final_state = *nominal.last().expect("propagation returns at least one state");
    let mut transition = [[0.0_f64; 6]; 6];
    let perturbations = [1.0e-3, 1.0e-3, 1.0e-3, 1.0e-6, 1.0e-6, 1.0e-6];
    for column in 0..6 {
        let epsilon = perturbations[column];
        let mut position = initial_position;
        let mut velocity = initial_velocity;
        if column < 3 {
            if column == 0 { position.x += epsilon; }
            if column == 1 { position.y += epsilon; }
            if column == 2 { position.z += epsilon; }
        } else {
            if column == 3 { velocity.x += epsilon; }
            if column == 4 { velocity.y += epsilon; }
            if column == 5 { velocity.z += epsilon; }
        }
        let perturbed = propagate_j2(position, velocity, duration_seconds, step_seconds, model)?;
        let end = *perturbed.last().expect("propagation returns at least one state");
        let delta = [
            end.position.x - final_state.position.x,
            end.position.y - final_state.position.y,
            end.position.z - final_state.position.z,
            end.velocity.x - final_state.velocity.x,
            end.velocity.y - final_state.velocity.y,
            end.velocity.z - final_state.velocity.z,
        ];
        for row in 0..6 {
            transition[row][column] = delta[row] / epsilon;
        }
    }
    let mut covariance = [[0.0_f64; 6]; 6];
    for row in 0..6 {
        for column in 0..6 {
            let mut sum = 0.0;
            for i in 0..6 {
                for j in 0..6 {
                    sum += transition[row][i] * initial_covariance[i][j] * transition[column][j];
                }
            }
            covariance[row][column] = sum;
        }
    }
    Ok(HpopCovariance {
        state: final_state,
        covariance,
    })
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
    fn covariance_propagation_is_finite_and_grows() {
        let (position, velocity) = circular_state();
        let mut covariance = [[0.0_f64; 6]; 6];
        for index in 0..3 {
            covariance[index][index] = 1.0e-6;
        }
        for index in 3..6 {
            covariance[index][index] = 1.0e-10;
        }
        let result = propagate_j2_covariance(position, velocity, 600.0, 60.0, J2Model::default(), covariance).expect("covariance");
        assert!(result.covariance.iter().flatten().all(|value| value.is_finite()));
        assert!(result.covariance[0][0] > 1.0e-6);
        assert!(result.covariance[0][0].is_finite());
    }

    #[test]
    fn zero_covariance_stays_zero() {
        let (position, velocity) = circular_state();
        let result = propagate_j2_covariance(position, velocity, 600.0, 60.0, J2Model::default(), [[0.0; 6]; 6]).expect("covariance");
        assert!(result.covariance.iter().flatten().all(|value| *value == 0.0));
    }

    #[test]
    fn covariance_ellipsoid_uses_diagonal_axes() {
        let mut covariance = [[0.0_f64; 6]; 6];
        covariance[0][0] = 9.0;
        covariance[1][1] = 4.0;
        covariance[2][2] = 1.0;
        let ellipsoid = position_covariance_ellipsoid(&covariance, 3.0).expect("ellipsoid");
        assert!((ellipsoid.semi_axes_km[0] - 9.0).abs() < 1e-12);
        assert!((ellipsoid.semi_axes_km[1] - 6.0).abs() < 1e-12);
        assert!((ellipsoid.semi_axes_km[2] - 3.0).abs() < 1e-12);
        assert!(position_covariance_ellipsoid(&covariance, 0.0).is_err());
    }

    #[test]
    fn invalid_sampling_is_rejected() {
        let (position, velocity) = circular_state();
        assert!(propagate_j2(position, velocity, -1.0, 10.0, J2Model::default()).is_err());
        assert!(propagate_j2(position, velocity, 10.0, 0.0, J2Model::default()).is_err());
    }
}
