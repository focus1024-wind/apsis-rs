//! Keplerian elements and their conversion to a Cartesian state.
//!
//! 开普勒根数及其到直角坐标状态的换算。

use crate::apsis_kepler::anomaly::{
    eccentric_from_mean, mean_from_eccentric, true_from_eccentric, wrap_to_pi,
};
use crate::apsis_kepler::parameter::GravityParameter;
use crate::apsis_math::Vec3;

/// The six Keplerian elements of an orbit.
///
/// The shape and size are given by the semi-major axis and the eccentricity,
/// the orientation by the `inclination_rad`, the right ascension of the ascending
/// node, and the argument of perigee, and the `position_km` along the orbit by
/// the true anomaly.
///
/// Angles are in radians and the semi-major axis is in kilometers. The orbit
/// is elliptical, so the eccentricity must be below one.
///
/// 轨道的六个开普勒根数。
///
/// 形状与大小由半长轴与偏心率给出，定向由倾角、升交点赤经与近地点幅角给出，
/// 轨道上的位置由真近点角给出。
///
/// 角度单位为弧度，半长轴单位为千米。轨道为椭圆，故偏心率必须小于一。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KeplerianElements {
    /// Semi-major axis, in kilometers. Must be positive.
    ///
    /// 半长轴，单位为千米，必须为正。
    pub semi_major_axis_km: f64,
    /// Eccentricity, dimensionless. Must be in `[0, 1)`.
    ///
    /// 偏心率，无量纲，必须位于 `[0, 1)`。
    pub eccentricity: f64,
    /// Inclination, in radians. Must be in `[0, pi]`.
    ///
    /// 倾角，单位为弧度，必须位于 `[0, pi]`。
    pub inclination_rad: f64,
    /// Right ascension of the ascending node, in radians.
    ///
    /// 升交点赤经，单位为弧度。
    pub right_ascension_rad: f64,
    /// Argument of perigee, in radians.
    ///
    /// 近地点幅角，单位为弧度。
    pub argument_of_perigee_rad: f64,
    /// True anomaly, in radians.
    ///
    /// 真近点角，单位为弧度。
    pub true_anomaly_rad: f64,
}

/// The `position_km` and `velocity_km_s` of a satellite in the inertial frame the
/// elements are referred to.
///
/// Position is in kilometers and `velocity_km_s` is in kilometers per second. The
/// frame has its first axis toward the reference direction, its third axis
/// along the orbit normal, and its second completing a right-handed set.
///
/// 卫星在根数所参照惯性系中的位置与速度。
///
/// 位置单位为千米，速度单位为千米每秒。该系第一轴指向参考方向，第三轴沿轨道法线，
/// 第二轴构成右手系。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct State {
    /// Position, in kilometers.
    ///
    /// 位置，单位为千米。
    pub position_km: Vec3,
    /// Velocity, in kilometers per second.
    ///
    /// 速度，单位为千米每秒。
    pub velocity_km_s: Vec3,
}

impl KeplerianElements {
    /// Returns `true` when the elements describe an elliptical orbit.
    ///
    /// 当这些根数描述一条椭圆轨道时返回 `true`。
    #[must_use]
    pub fn is_valid(self) -> bool {
        self.semi_major_axis_km.is_finite()
            && self.semi_major_axis_km > 0.0
            && self.eccentricity.is_finite()
            && (0.0..1.0).contains(&self.eccentricity)
            && self.inclination_rad.is_finite()
            && (0.0..=core::f64::consts::PI).contains(&self.inclination_rad)
            && self.right_ascension_rad.is_finite()
            && self.argument_of_perigee_rad.is_finite()
            && self.true_anomaly_rad.is_finite()
    }

    /// Returns the semi-latus rectum, in kilometers.
    ///
    /// This is `a (1 - e^2)`, the distance from the focus to the orbit
    /// measured perpendicular to the major axis.
    ///
    /// 返回半通径，单位为千米。
    ///
    /// 它等于 `a (1 - e^2)`，即从焦点沿垂直于长轴方向量到轨道的距离。
    #[must_use]
    pub fn semi_latus_rectum_km(self) -> f64 {
        self.semi_major_axis_km * (1.0 - self.eccentricity * self.eccentricity)
    }

    /// Returns the semi-minor axis, in kilometers.
    ///
    /// 返回半短轴，单位为千米。
    #[must_use]
    pub fn semi_minor_axis_km(self) -> f64 {
        self.semi_major_axis_km * (1.0 - self.eccentricity * self.eccentricity).sqrt()
    }

    /// Returns the perigee radius, in kilometers.
    ///
    /// This is the closest approach to the focus, `a (1 - e)`.
    ///
    /// 返回近地点半径，单位为千米。
    ///
    /// 它是到焦点的最近距离 `a (1 - e)`。
    #[must_use]
    pub fn perigee_radius_km(self) -> f64 {
        self.semi_major_axis_km * (1.0 - self.eccentricity)
    }

    /// Returns the apogee radius, in kilometers.
    ///
    /// This is the farthest point from the focus, `a (1 + e)`.
    ///
    /// 返回远地点半径，单位为千米。
    ///
    /// 它是离焦点最远的点 `a (1 + e)`。
    #[must_use]
    pub fn apogee_radius_km(self) -> f64 {
        self.semi_major_axis_km * (1.0 + self.eccentricity)
    }

    /// Returns the orbital period, in seconds.
    ///
    /// This is Kepler's third law, `T = 2 pi sqrt(a^3 / mu)`.
    ///
    /// 返回轨道周期，单位为秒。
    ///
    /// 这是开普勒第三定律，`T = 2 pi sqrt(a^3 / mu)`。
    #[must_use]
    pub fn period_s(self, parameter: GravityParameter) -> f64 {
        2.0 * core::f64::consts::PI * (self.semi_major_axis_km.powi(3) / parameter.value()).sqrt()
    }

    /// Returns the eccentric anomaly corresponding to the true anomaly.
    ///
    /// 返回与真近点角对应的偏近点角。
    #[must_use]
    pub fn eccentric_anomaly_rad(self) -> f64 {
        crate::apsis_kepler::anomaly::eccentric_from_true(self.true_anomaly_rad, self.eccentricity)
    }

    /// Returns the mean anomaly corresponding to the true anomaly.
    ///
    /// 返回与真近点角对应的平近点角。
    #[must_use]
    pub fn mean_anomaly_rad(self) -> f64 {
        crate::apsis_kepler::anomaly::mean_from_true(self.true_anomaly_rad, self.eccentricity)
    }

    /// Converts these elements to a `position_km` and `velocity_km_s`.
    ///
    /// Returns `None` when the elements are not a valid elliptical orbit.
    ///
    /// The conversion places the satellite in the perifocal frame, where the
    /// first axis points to perigee and the third along the orbit normal,
    ///
    /// <div>
    /// $$
    /// \begin{aligned}
    /// r &= \frac{p}{1 + e \cos\nu} \\
    /// x &= r \cos\nu,\quad y = r \sin\nu,\quad z = 0 \\
    /// v_x &= -\sqrt{\frac{\mu}{p}}\,\sin\nu \\
    /// v_y &= \sqrt{\frac{\mu}{p}}\,(e + \cos\nu)
    /// \end{aligned}
    /// $$
    /// </div>
    ///
    /// and then rotates by the argument of perigee, the `inclination_rad`, and the
    /// right ascension of the ascending node.
    ///
    /// 把这些根数转换为位置与速度。
    ///
    /// 当根数不是有效椭圆轨道时返回 `None`。
    ///
    /// 换算先在近点坐标系中定位卫星，该系第一轴指向近地点、第三轴沿轨道法线，
    ///
    /// <div>
    /// $$
    /// \begin{aligned}
    /// r &= \frac{p}{1 + e \cos\nu} \\
    /// x &= r \cos\nu,\quad y = r \sin\nu,\quad z = 0 \\
    /// v_x &= -\sqrt{\frac{\mu}{p}}\,\sin\nu \\
    /// v_y &= \sqrt{\frac{\mu}{p}}\,(e + \cos\nu)
    /// \end{aligned}
    /// $$
    /// </div>
    ///
    /// 再依次按近地点幅角、倾角与升交点赤经旋转。
    #[must_use]
    pub fn to_state(self, parameter: GravityParameter) -> Option<State> {
        if !self.is_valid() {
            return None;
        }
        let p = self.semi_latus_rectum_km();
        let mu = parameter.value();
        let cos_nu = self.true_anomaly_rad.cos();
        let sin_nu = self.true_anomaly_rad.sin();
        let radius = p / (1.0 + self.eccentricity * cos_nu);
        let scale = (mu / p).sqrt();

        let perifocal_position = Vec3::new(radius * cos_nu, radius * sin_nu, 0.0);
        let perifocal_velocity =
            Vec3::new(-scale * sin_nu, scale * (self.eccentricity + cos_nu), 0.0);

        // Rotate from the perifocal frame to the inertial frame by the
        // argument of perigee about the third axis, the inclination_rad about
        // the first, and the right ascension about the third.
        // 从近点坐标系旋转到惯性系：先绕第三轴转近地点幅角，再绕第一轴转倾角，
        // 最后绕第三轴转赤经。
        let rotation = perifocal_to_inertial(
            self.argument_of_perigee_rad,
            self.inclination_rad,
            self.right_ascension_rad,
        );
        Some(State {
            position_km: rotation * perifocal_position,
            velocity_km_s: rotation * perifocal_velocity,
        })
    }

    /// Returns the elements for a `position_km` and `velocity_km_s`.
    ///
    /// Returns `None` when the state is not finite, when the radius is zero,
    /// or when the orbit is not elliptical. This conversion assumes the
    /// orbit is bound, so a parabolic or hyperbolic state is rejected rather
    /// than returned with an eccentricity at or above one.
    ///
    /// 由位置与速度返回对应的根数。
    ///
    /// 当状态非有限、半径为零或轨道非椭圆时返回 `None`。该换算假定轨道受束缚，
    /// 因此抛物线或双曲线状态被拒绝，而不是返回偏心率大于等于一的结果。
    #[must_use]
    pub fn from_state(state: &State, parameter: GravityParameter) -> Option<Self> {
        let position_km = state.position_km;
        let velocity_km_s = state.velocity_km_s;
        if !position_km.is_finite() || !velocity_km_s.is_finite() {
            return None;
        }
        let mu = parameter.value();
        let radius = position_km.norm();
        if radius == 0.0 {
            return None;
        }

        let velocity_squared = velocity_km_s.norm_squared();
        let radial_velocity = position_km.dot(velocity_km_s) / radius;

        // The specific angular momentum and the node vector, which together
        // give the orientation.
        // 比角动量与交点向量，二者共同给出轨道定向。
        let angular_momentum = position_km.cross(velocity_km_s);
        let angular_momentum_norm = angular_momentum.norm();
        if angular_momentum_norm == 0.0 {
            // A purely radial trajectory has no defined orbit plane.
            // 纯径向轨迹没有良定义的轨道平面。
            return None;
        }

        // The eccentricity vector points toward perigee and has magnitude
        // equal to the eccentricity.
        // 偏心率矢量指向近地点，其模等于偏心率。
        let eccentricity_vector = position_km * (velocity_squared - mu / radius)
            - velocity_km_s * (radius * radial_velocity);
        let eccentricity = eccentricity_vector.norm() / mu;
        if !(0.0..1.0).contains(&eccentricity) {
            return None;
        }

        // The specific mechanical energy gives the semi-major axis.
        // 单位质量机械能给出半长轴。
        let energy = velocity_squared / 2.0 - mu / radius;
        if energy == 0.0 {
            return None;
        }
        let semi_major_axis_km = -mu / (2.0 * energy);
        if semi_major_axis_km <= 0.0 || !semi_major_axis_km.is_finite() {
            return None;
        }

        let node = Vec3::Z.cross(angular_momentum);
        let node_norm = node.norm();
        let inclination_rad = (angular_momentum.z / angular_momentum_norm)
            .clamp(-1.0, 1.0)
            .acos();

        // The node and the argument of perigee are undefined for a zero
        // inclination_rad and for a circular orbit respectively; in those cases
        // the standard convention sets them to zero and measures the angle
        // from the reference direction instead.
        // 倾角为零时交点未定义，圆轨道时近地点幅角未定义；在这两种情形下，标准
        // 约定将其置零，并改为自参考方向量取该角。
        let right_ascension_rad = if node_norm == 0.0 {
            0.0
        } else {
            wrap_to_pi(node.y.atan2(node.x))
        };

        let argument_of_perigee_rad = if eccentricity < 1e-12 {
            0.0
        } else if node_norm == 0.0 {
            // Equatorial orbit: measure the argument from the first axis.
            // 赤道轨道：自第一轴量取该幅角。
            wrap_to_pi(eccentricity_vector.y.atan2(eccentricity_vector.x))
        } else {
            let value = (node.dot(eccentricity_vector) / (node_norm * eccentricity_vector.norm()))
                .clamp(-1.0, 1.0);
            let angle = value.acos();
            if eccentricity_vector.z < 0.0 {
                wrap_to_pi(-angle)
            } else {
                wrap_to_pi(angle)
            }
        };

        // The true anomaly follows from the eccentric anomaly recovered
        // from the radial distance and the radial velocity_km_s, which is more
        // accurate near a circular orbit than the angle between vectors.
        // 真近点角由径向距离与径向速度求得的偏近点角推出；在近圆轨道处，这比
        // 直接求两矢量夹角更准确。
        let true_anomaly_rad =
            true_anomaly_from_state(position_km, velocity_km_s, eccentricity, mu);

        let elements = Self {
            semi_major_axis_km,
            eccentricity,
            inclination_rad,
            right_ascension_rad,
            argument_of_perigee_rad,
            true_anomaly_rad,
        };
        elements.is_valid().then_some(elements)
    }

    /// Returns a copy of these elements with a different true anomaly.
    ///
    /// 返回真近点角不同的这些根数的副本。
    #[must_use]
    pub fn with_true_anomaly(self, true_anomaly_rad: f64) -> Self {
        Self {
            true_anomaly_rad,
            ..self
        }
    }

    /// Returns a copy of these elements at a mean anomaly.
    ///
    /// The true anomaly is recovered from the mean anomaly through Kepler's
    /// equation.
    ///
    /// 返回给定平近点角处这些根数的副本。
    ///
    /// 真近点角由平近点角经开普勒方程求得。
    #[must_use]
    pub fn with_mean_anomaly_rad(self, mean_anomaly: f64) -> Self {
        let eccentric = eccentric_from_mean(mean_anomaly, self.eccentricity);
        Self {
            true_anomaly_rad: true_from_eccentric(eccentric, self.eccentricity),
            ..self
        }
    }

    /// Returns the mean anomaly, which is the linear measure of time along
    /// the orbit.
    ///
    /// This is the same value as [`KeplerianElements::mean_anomaly_rad`], named
    /// for symmetry with [`KeplerianElements::with_mean_anomaly_rad`].
    ///
    /// 返回平近点角，即沿轨道的时间线性度量。
    ///
    /// 其值与 [`KeplerianElements::mean_anomaly_rad`] 相同，此处命名是为了与
    /// [`KeplerianElements::with_mean_anomaly_rad`] 对称。
    #[must_use]
    pub fn to_mean_anomaly_rad(self) -> f64 {
        mean_from_eccentric(self.eccentric_anomaly_rad(), self.eccentricity)
    }
}

/// Returns the rotation from the perifocal frame to the inertial frame.
///
/// The rotation is a product of three elementary rotations: by the argument
/// of perigee about the third axis, then the `inclination_rad` about the first,
/// then the right ascension about the third.
///
/// 返回从近点坐标系到惯性系的旋转。
///
/// 该旋转是三个基本旋转之积：先绕第三轴转近地点幅角，再绕第一轴转倾角，最后
/// 绕第三轴转赤经。
#[must_use]
pub fn perifocal_to_inertial(
    argument_of_perigee_rad: f64,
    inclination_rad: f64,
    right_ascension_rad: f64,
) -> crate::apsis_math::Mat3 {
    use crate::apsis_math::Mat3;
    let (sin_w, cos_w) = argument_of_perigee_rad.sin_cos();
    let (sin_i, cos_i) = inclination_rad.sin_cos();
    let (sin_o, cos_o) = right_ascension_rad.sin_cos();

    // Expanded product R3(O) R1(i) R3(w). Writing it out avoids three matrix
    // products and keeps the result well conditioned.
    // 展开后的乘积 R3(O) R1(i) R3(w)。直接写出可省去三次矩阵相乘，并使结果
    // 保持良态。
    Mat3::from_rows(
        [
            cos_o * cos_w - sin_o * sin_w * cos_i,
            -cos_o * sin_w - sin_o * cos_w * cos_i,
            sin_o * sin_i,
        ],
        [
            sin_o * cos_w + cos_o * sin_w * cos_i,
            -sin_o * sin_w + cos_o * cos_w * cos_i,
            -cos_o * sin_i,
        ],
        [sin_w * sin_i, cos_w * sin_i, cos_i],
    )
}

/// Recovers the true anomaly from a state vector.
///
/// Using the eccentric anomaly as an intermediate keeps the result accurate
/// for near circular orbits, where the eccentricity vector is short and its
/// direction is poorly determined. The eccentric anomaly follows from the
/// radial distance and the radial `velocity_km_s` through
///
/// <div>
/// $$
/// \begin{aligned}
/// \cos E &= \frac{1 - r / a}{e} \\
/// \sin E &= \frac{r \dot r}{e \sqrt{\mu a}}
/// \end{aligned}
/// $$
/// </div>
///
/// and the true anomaly follows from the eccentric anomaly. A circular
/// orbit, where the eccentricity is too small for those expressions, is
/// handled by measuring the angle from the node instead, which is the same
/// convention [`KeplerianElements::from_state`] uses for the argument of
/// perigee.
///
/// 由状态矢量反推真近点角。
///
/// 以偏近点角为中间量，可在近圆轨道处保持精度，因为此时偏心率矢量很短、方向
/// 难以确定。偏近点角由径向距离与径向速度经下式求得：
///
/// <div>
/// $$
/// \begin{aligned}
/// \cos E &= \frac{1 - r / a}{e} \\
/// \sin E &= \frac{r \dot r}{e \sqrt{\mu a}}
/// \end{aligned}
/// $$
/// </div>
///
/// 真近点角再由偏近点角推出。对偏心率过小而使上述式子失效的圆轨道，改为
/// 自交点量取夹角处理，这与 [`KeplerianElements::from_state`] 处理近地点幅角
/// 时所用的约定相同。
fn true_anomaly_from_state(
    position_km: Vec3,
    velocity_km_s: Vec3,
    eccentricity: f64,
    mu: f64,
) -> f64 {
    // Below this eccentricity the angle is recovered from the node instead,
    // because the division by the eccentricity would amplify the rounding
    // in the radial terms.
    // 偏心率低于此值时改为自交点反求夹角，因为除以偏心率会放大径向项中的
    // 舍入误差。
    const CIRCULAR_THRESHOLD: f64 = 1e-8;

    let radius = position_km.norm();
    let radial_velocity = position_km.dot(velocity_km_s) / radius;
    let semi_major_axis_km = 1.0 / (2.0 / radius - velocity_km_s.norm_squared() / mu);
    if !semi_major_axis_km.is_finite() || semi_major_axis_km <= 0.0 {
        return 0.0;
    }

    if eccentricity < CIRCULAR_THRESHOLD {
        let angular_momentum = position_km.cross(velocity_km_s);
        let node = Vec3::Z.cross(angular_momentum);
        if node.norm() == 0.0 {
            // Equatorial circular orbit: measure from the first axis.
            // 赤道圆轨道：自第一轴量取。
            return wrap_to_pi(position_km.y.atan2(position_km.x));
        }
        // The argument of latitude_rad is the angle from the node to the
        // position_km, measured in the orbit plane.
        // 纬度幅角是自交点到该位置在轨道平面内量取的夹角。
        let node_unit = node.normalize().expect("node is non-zero");
        let in_plane = position_km.normalize().expect("radius is non-zero");
        let cross = node_unit.cross(in_plane);
        let along = angular_momentum
            .normalize()
            .expect("angular momentum is non-zero");
        return wrap_to_pi(cross.dot(along).atan2(node_unit.dot(in_plane)));
    }

    let sin_e = radius * radial_velocity / (eccentricity * (mu * semi_major_axis_km).sqrt());
    let cos_e = (1.0 - radius / semi_major_axis_km) / eccentricity;
    let eccentric_anomaly = sin_e.atan2(cos_e);
    true_from_eccentric(eccentric_anomaly, eccentricity)
}
