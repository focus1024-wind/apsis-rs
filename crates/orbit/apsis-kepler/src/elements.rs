//! Keplerian elements and their conversion to a Cartesian state.

use crate::anomaly::{eccentric_from_mean, mean_from_eccentric, true_from_eccentric, wrap_to_pi};
use crate::parameter::GravityParameter;
use apsis_math::Vec3;

/// The six Keplerian elements of an orbit.
///
/// The shape and size are given by the semi-major axis and the eccentricity,
/// the orientation by the inclination, the right ascension of the ascending
/// node, and the argument of perigee, and the position along the orbit by
/// the true anomaly.
///
/// Angles are in radians and the semi-major axis is in kilometers. The orbit
/// is elliptical, so the eccentricity must be below one.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KeplerianElements {
    /// Semi-major axis, in kilometers. Must be positive.
    pub semi_major_axis: f64,
    /// Eccentricity, dimensionless. Must be in `[0, 1)`.
    pub eccentricity: f64,
    /// Inclination, in radians. Must be in `[0, pi]`.
    pub inclination: f64,
    /// Right ascension of the ascending node, in radians.
    pub right_ascension: f64,
    /// Argument of perigee, in radians.
    pub argument_of_perigee: f64,
    /// True anomaly, in radians.
    pub true_anomaly: f64,
}

/// The position and velocity of a satellite in the inertial frame the
/// elements are referred to.
///
/// Position is in kilometers and velocity is in kilometers per second. The
/// frame has its first axis toward the reference direction, its third axis
/// along the orbit normal, and its second completing a right-handed set.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct State {
    /// Position, in kilometers.
    pub position: Vec3,
    /// Velocity, in kilometers per second.
    pub velocity: Vec3,
}

impl KeplerianElements {
    /// Returns `true` when the elements describe an elliptical orbit.
    pub fn is_valid(self) -> bool {
        self.semi_major_axis.is_finite()
            && self.semi_major_axis > 0.0
            && self.eccentricity.is_finite()
            && (0.0..1.0).contains(&self.eccentricity)
            && self.inclination.is_finite()
            && (0.0..=core::f64::consts::PI).contains(&self.inclination)
            && self.right_ascension.is_finite()
            && self.argument_of_perigee.is_finite()
            && self.true_anomaly.is_finite()
    }

    /// Returns the semi-latus rectum, in kilometers.
    ///
    /// This is `a (1 - e^2)`, the distance from the focus to the orbit
    /// measured perpendicular to the major axis.
    pub fn semi_latus_rectum(self) -> f64 {
        self.semi_major_axis * (1.0 - self.eccentricity * self.eccentricity)
    }

    /// Returns the semi-minor axis, in kilometers.
    pub fn semi_minor_axis(self) -> f64 {
        self.semi_major_axis * (1.0 - self.eccentricity * self.eccentricity).sqrt()
    }

    /// Returns the perigee radius, in kilometers.
    ///
    /// This is the closest approach to the focus, `a (1 - e)`.
    pub fn perigee_radius(self) -> f64 {
        self.semi_major_axis * (1.0 - self.eccentricity)
    }

    /// Returns the apogee radius, in kilometers.
    ///
    /// This is the farthest point from the focus, `a (1 + e)`.
    pub fn apogee_radius(self) -> f64 {
        self.semi_major_axis * (1.0 + self.eccentricity)
    }

    /// Returns the orbital period, in seconds.
    ///
    /// This is Kepler's third law, `T = 2 pi sqrt(a^3 / mu)`.
    pub fn period(self, parameter: GravityParameter) -> f64 {
        2.0 * core::f64::consts::PI * (self.semi_major_axis.powi(3) / parameter.value()).sqrt()
    }

    /// Returns the eccentric anomaly corresponding to the true anomaly.
    pub fn eccentric_anomaly(self) -> f64 {
        crate::anomaly::eccentric_from_true(self.true_anomaly, self.eccentricity)
    }

    /// Returns the mean anomaly corresponding to the true anomaly.
    pub fn mean_anomaly(self) -> f64 {
        crate::anomaly::mean_from_true(self.true_anomaly, self.eccentricity)
    }

    /// Converts these elements to a position and velocity.
    ///
    /// Returns `None` when the elements are not a valid elliptical orbit.
    ///
    /// The conversion places the satellite in the perifocal frame, where the
    /// first axis points to perigee and the third along the orbit normal,
    ///
    /// ```text
    /// r = p / (1 + e cos nu)
    /// x = r cos nu,  y = r sin nu,  z = 0
    /// vx = -sqrt(mu / p) sin nu
    /// vy =  sqrt(mu / p) (e + cos nu)
    /// ```
    ///
    /// and then rotates by the argument of perigee, the inclination, and the
    /// right ascension of the ascending node.
    pub fn to_state(self, parameter: GravityParameter) -> Option<State> {
        if !self.is_valid() {
            return None;
        }
        let p = self.semi_latus_rectum();
        let mu = parameter.value();
        let cos_nu = self.true_anomaly.cos();
        let sin_nu = self.true_anomaly.sin();
        let radius = p / (1.0 + self.eccentricity * cos_nu);
        let scale = (mu / p).sqrt();

        let perifocal_position = Vec3::new(radius * cos_nu, radius * sin_nu, 0.0);
        let perifocal_velocity =
            Vec3::new(-scale * sin_nu, scale * (self.eccentricity + cos_nu), 0.0);

        // Rotate from the perifocal frame to the inertial frame by the
        // argument of perigee about the third axis, the inclination about
        // the first, and the right ascension about the third.
        let rotation = perifocal_to_inertial(
            self.argument_of_perigee,
            self.inclination,
            self.right_ascension,
        );
        Some(State {
            position: rotation * perifocal_position,
            velocity: rotation * perifocal_velocity,
        })
    }

    /// Returns the elements for a position and velocity.
    ///
    /// Returns `None` when the state is not finite, when the radius is zero,
    /// or when the orbit is not elliptical. This conversion assumes the
    /// orbit is bound, so a parabolic or hyperbolic state is rejected rather
    /// than returned with an eccentricity at or above one.
    pub fn from_state(state: &State, parameter: GravityParameter) -> Option<Self> {
        let position = state.position;
        let velocity = state.velocity;
        if !position.is_finite() || !velocity.is_finite() {
            return None;
        }
        let mu = parameter.value();
        let radius = position.norm();
        if radius == 0.0 {
            return None;
        }

        let velocity_squared = velocity.norm_squared();
        let radial_velocity = position.dot(velocity) / radius;

        // The specific angular momentum and the node vector, which together
        // give the orientation.
        let angular_momentum = position.cross(velocity);
        let angular_momentum_norm = angular_momentum.norm();
        if angular_momentum_norm == 0.0 {
            // A purely radial trajectory has no defined orbit plane.
            return None;
        }

        // The eccentricity vector points toward perigee and has magnitude
        // equal to the eccentricity.
        let eccentricity_vector =
            position * (velocity_squared - mu / radius) - velocity * (radius * radial_velocity);
        let eccentricity = eccentricity_vector.norm() / mu;
        if !(0.0..1.0).contains(&eccentricity) {
            return None;
        }

        // The specific mechanical energy gives the semi-major axis.
        let energy = velocity_squared / 2.0 - mu / radius;
        if energy == 0.0 {
            return None;
        }
        let semi_major_axis = -mu / (2.0 * energy);
        if semi_major_axis <= 0.0 || !semi_major_axis.is_finite() {
            return None;
        }

        let node = Vec3::Z.cross(angular_momentum);
        let node_norm = node.norm();
        let inclination = (angular_momentum.z / angular_momentum_norm)
            .clamp(-1.0, 1.0)
            .acos();

        // The node and the argument of perigee are undefined for a zero
        // inclination and for a circular orbit respectively; in those cases
        // the standard convention sets them to zero and measures the angle
        // from the reference direction instead.
        let right_ascension = if node_norm == 0.0 {
            0.0
        } else {
            wrap_to_pi(node.y.atan2(node.x))
        };

        let argument_of_perigee = if eccentricity < 1e-12 {
            0.0
        } else if node_norm == 0.0 {
            // Equatorial orbit: measure the argument from the first axis.
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
        // from the radial distance and the radial velocity, which is more
        // accurate near a circular orbit than the angle between vectors.
        let true_anomaly = true_anomaly_from_state(position, velocity, eccentricity, mu);

        let elements = Self {
            semi_major_axis,
            eccentricity,
            inclination,
            right_ascension,
            argument_of_perigee,
            true_anomaly,
        };
        elements.is_valid().then_some(elements)
    }

    /// Returns a copy of these elements with a different true anomaly.
    pub fn with_true_anomaly(self, true_anomaly: f64) -> Self {
        Self {
            true_anomaly,
            ..self
        }
    }

    /// Returns a copy of these elements at a mean anomaly.
    ///
    /// The true anomaly is recovered from the mean anomaly through Kepler's
    /// equation.
    pub fn with_mean_anomaly(self, mean_anomaly: f64) -> Self {
        let eccentric = eccentric_from_mean(mean_anomaly, self.eccentricity);
        Self {
            true_anomaly: true_from_eccentric(eccentric, self.eccentricity),
            ..self
        }
    }

    /// Returns the mean anomaly, which is the linear measure of time along
    /// the orbit.
    ///
    /// This is the same value as [`KeplerianElements::mean_anomaly`], named
    /// for symmetry with [`KeplerianElements::with_mean_anomaly`].
    pub fn to_mean_anomaly(self) -> f64 {
        mean_from_eccentric(self.eccentric_anomaly(), self.eccentricity)
    }
}

/// Returns the rotation from the perifocal frame to the inertial frame.
///
/// The rotation is a product of three elementary rotations: by the argument
/// of perigee about the third axis, then the inclination about the first,
/// then the right ascension about the third.
pub(crate) fn perifocal_to_inertial(
    argument_of_perigee: f64,
    inclination: f64,
    right_ascension: f64,
) -> apsis_math::Mat3 {
    use apsis_math::Mat3;
    let (sin_w, cos_w) = argument_of_perigee.sin_cos();
    let (sin_i, cos_i) = inclination.sin_cos();
    let (sin_o, cos_o) = right_ascension.sin_cos();

    // Expanded product R3(O) R1(i) R3(w). Writing it out avoids three matrix
    // products and keeps the result well conditioned.
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
/// radial distance and the radial velocity through
///
/// ```text
/// cos E = (1 - r / a) / e
/// sin E = r rdot / (e sqrt(mu a))
/// ```
///
/// and the true anomaly follows from the eccentric anomaly. A circular
/// orbit, where the eccentricity is too small for those expressions, is
/// handled by measuring the angle from the node instead, which is the same
/// convention [`KeplerianElements::from_state`] uses for the argument of
/// perigee.
fn true_anomaly_from_state(position: Vec3, velocity: Vec3, eccentricity: f64, mu: f64) -> f64 {
    let radius = position.norm();
    let radial_velocity = position.dot(velocity) / radius;
    let semi_major_axis = 1.0 / (2.0 / radius - velocity.norm_squared() / mu);
    if !semi_major_axis.is_finite() || semi_major_axis <= 0.0 {
        return 0.0;
    }

    // Below this eccentricity the angle is recovered from the node instead,
    // because the division by the eccentricity would amplify the rounding
    // in the radial terms.
    const CIRCULAR_THRESHOLD: f64 = 1e-8;
    if eccentricity < CIRCULAR_THRESHOLD {
        let angular_momentum = position.cross(velocity);
        let node = Vec3::Z.cross(angular_momentum);
        if node.norm() == 0.0 {
            // Equatorial circular orbit: measure from the first axis.
            return wrap_to_pi(position.y.atan2(position.x));
        }
        // The argument of latitude is the angle from the node to the
        // position, measured in the orbit plane.
        let node_unit = node.normalize().expect("node is non-zero");
        let in_plane = position.normalize().expect("radius is non-zero");
        let cross = node_unit.cross(in_plane);
        let along = angular_momentum
            .normalize()
            .expect("angular momentum is non-zero");
        return wrap_to_pi(cross.dot(along).atan2(node_unit.dot(in_plane)));
    }

    let sin_e = radius * radial_velocity / (eccentricity * (mu * semi_major_axis).sqrt());
    let cos_e = (1.0 - radius / semi_major_axis) / eccentricity;
    let eccentric_anomaly = sin_e.atan2(cos_e);
    true_from_eccentric(eccentric_anomaly, eccentricity)
}
