//! Unit quaternions used for rotations.

use crate::vec3::Vec3;
use core::fmt;
use core::ops::Mul;

/// A quaternion `w + xi + yj + zk`.
///
/// Methods that rotate vectors assume the quaternion has unit length.
/// Multiplication and addition do not, because arbitrary quaternion
/// arithmetic is well defined without that assumption. Use
/// [`Quat::normalize`] to enforce unit length, and
/// [`Quat::is_normalized`] to check it.
///
/// A rotation by angle `theta` about a unit axis `u` is
/// `w = cos(theta / 2)`, `(x, y, z) = u * sin(theta / 2)`. Composition uses
/// the same convention as matrix multiplication: `(a * b)` applied to a
/// vector applies `b` first, then `a`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Quat {
    /// Scalar part.
    pub w: f64,
    /// First component of the vector part.
    pub x: f64,
    /// Second component of the vector part.
    pub y: f64,
    /// Third component of the vector part.
    pub z: f64,
}

impl Quat {
    /// The identity rotation.
    pub const IDENTITY: Self = Self::new(1.0, 0.0, 0.0, 0.0);

    /// Creates a quaternion from its four components.
    pub const fn new(w: f64, x: f64, y: f64, z: f64) -> Self {
        Self { w, x, y, z }
    }

    /// Creates a rotation of `angle` radians about the unit axis `axis`.
    ///
    /// The axis is normalized internally. Returns `None` when the axis has
    /// zero or non-finite length.
    pub fn from_axis_angle(axis: Vec3, angle: f64) -> Option<Self> {
        let axis = axis.normalize()?;
        let half = angle / 2.0;
        let (sin, cos) = half.sin_cos();
        Some(Self::new(cos, axis.x * sin, axis.y * sin, axis.z * sin))
    }

    /// Creates a rotation of `angle` radians about the first axis.
    pub fn from_rotation_x(angle: f64) -> Self {
        let (sin, cos) = (angle / 2.0).sin_cos();
        Self::new(cos, sin, 0.0, 0.0)
    }

    /// Creates a rotation of `angle` radians about the second axis.
    pub fn from_rotation_y(angle: f64) -> Self {
        let (sin, cos) = (angle / 2.0).sin_cos();
        Self::new(cos, 0.0, sin, 0.0)
    }

    /// Creates a rotation of `angle` radians about the third axis.
    pub fn from_rotation_z(angle: f64) -> Self {
        let (sin, cos) = (angle / 2.0).sin_cos();
        Self::new(cos, 0.0, 0.0, sin)
    }

    /// Creates a quaternion from the rotation matrix `m`.
    ///
    /// The algorithm picks the largest diagonal entry to divide by, which
    /// keeps the square root well conditioned for every input rotation.
    /// Returns `None` when `m` is not a rotation matrix, that is when it is
    /// not orthonormal with determinant one within a tolerance of `1e-9`.
    pub fn from_mat3(m: crate::mat3::Mat3) -> Option<Self> {
        if !m.is_rotation(1e-9) {
            return None;
        }
        let (m00, m01, m02) = (m.m00, m.m01, m.m02);
        let (m10, m11, m12) = (m.m10, m.m11, m.m12);
        let (m20, m21, m22) = (m.m20, m.m21, m.m22);
        let trace = m00 + m11 + m22;
        let quaternion = if trace > 0.0 {
            let s = (trace + 1.0).sqrt() * 2.0;
            Self::new(0.25 * s, (m21 - m12) / s, (m02 - m20) / s, (m10 - m01) / s)
        } else if m00 > m11 && m00 > m22 {
            let s = (1.0 + m00 - m11 - m22).sqrt() * 2.0;
            Self::new((m21 - m12) / s, 0.25 * s, (m01 + m10) / s, (m02 + m20) / s)
        } else if m11 > m22 {
            let s = (1.0 + m11 - m00 - m22).sqrt() * 2.0;
            Self::new((m02 - m20) / s, (m01 + m10) / s, 0.25 * s, (m12 + m21) / s)
        } else {
            let s = (1.0 + m22 - m00 - m11).sqrt() * 2.0;
            Self::new((m10 - m01) / s, (m02 + m20) / s, (m12 + m21) / s, 0.25 * s)
        };
        Some(quaternion)
    }

    /// Returns the vector part `(x, y, z)`.
    pub const fn vector(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }

    /// Returns the squared length of this quaternion.
    pub fn norm_squared(self) -> f64 {
        self.w * self.w + self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// Returns the length of this quaternion.
    pub fn norm(self) -> f64 {
        self.norm_squared().sqrt()
    }

    /// Returns this quaternion scaled to unit length.
    ///
    /// Returns `None` when the length is zero or not finite.
    pub fn normalize(self) -> Option<Self> {
        let norm = self.norm();
        if norm > 0.0 && norm.is_finite() {
            Some(Self::new(
                self.w / norm,
                self.x / norm,
                self.y / norm,
                self.z / norm,
            ))
        } else {
            None
        }
    }

    /// Returns the conjugate `w - xi - yj - zk`.
    ///
    /// For a unit quaternion this is the inverse rotation.
    pub const fn conjugate(self) -> Self {
        Self::new(self.w, -self.x, -self.y, -self.z)
    }

    /// Returns the inverse rotation.
    ///
    /// Returns `None` when the length is zero or not finite.
    pub fn inverse(self) -> Option<Self> {
        let norm_squared = self.norm_squared();
        if norm_squared > 0.0 && norm_squared.is_finite() {
            let conjugate = self.conjugate();
            Some(Self::new(
                conjugate.w / norm_squared,
                conjugate.x / norm_squared,
                conjugate.y / norm_squared,
                conjugate.z / norm_squared,
            ))
        } else {
            None
        }
    }

    /// Returns the dot product of `self` and `rhs`, treating both as four
    /// component vectors.
    pub fn dot(self, rhs: Self) -> f64 {
        self.w * rhs.w + self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    /// Rotates `v` by this rotation.
    ///
    /// Assumes this quaternion has unit length. The rotation is computed as
    /// `v + 2 * cross(vector, cross(vector, v) + w * v)`, which avoids
    /// forming the quaternion product matrix twice.
    pub fn rotate(self, v: Vec3) -> Vec3 {
        let q = self.vector();
        let t = q.cross(v) * 2.0;
        v + t * self.w + q.cross(t)
    }

    /// Returns the rotation angle in radians, in `[0, pi]`.
    ///
    /// Assumes this quaternion has unit length.
    pub fn angle(self) -> f64 {
        2.0 * self.w.clamp(-1.0, 1.0).acos()
    }

    /// Returns the rotation axis as a unit vector.
    ///
    /// Returns `None` for the identity rotation, where no axis is defined.
    /// Assumes this quaternion has unit length.
    pub fn axis(self) -> Option<Vec3> {
        self.vector().normalize()
    }

    /// Returns the linear interpolation from `self` to `rhs`.
    ///
    /// This is a chord interpolation, so the result is not a unit
    /// quaternion and the angular speed is not constant. Use
    /// [`Quat::slerp`] when the intermediate rotations must be uniform.
    pub fn lerp(self, rhs: Self, t: f64) -> Self {
        let w = self.w + (rhs.w - self.w) * t;
        let x = self.x + (rhs.x - self.x) * t;
        let y = self.y + (rhs.y - self.y) * t;
        let z = self.z + (rhs.z - self.z) * t;
        Self::new(w, x, y, z)
    }

    /// Returns the spherical linear interpolation from `self` to `rhs`.
    ///
    /// The shortest of the two arcs is taken, so the result follows the
    /// path a rotation would take if the angle were kept minimal. Falls
    /// back to [`Quat::lerp`] when the inputs are nearly parallel, where
    /// the sine in the denominator would otherwise lose precision.
    pub fn slerp(self, rhs: Self, t: f64) -> Self {
        let mut end = rhs;
        let mut dot = self.dot(rhs);
        if dot < 0.0 {
            // Negating one endpoint selects the shorter arc, because `q`
            // and `-q` describe the same rotation.
            end = Self::new(-rhs.w, -rhs.x, -rhs.y, -rhs.z);
            dot = -dot;
        }
        if dot > 0.999_5 {
            return self.lerp(end, t);
        }
        let theta = dot.clamp(-1.0, 1.0).acos();
        let sin_theta = theta.sin();
        if sin_theta == 0.0 {
            return self;
        }
        let scale_start = ((1.0 - t) * theta).sin() / sin_theta;
        let scale_end = (t * theta).sin() / sin_theta;
        Self::new(
            self.w * scale_start + end.w * scale_end,
            self.x * scale_start + end.x * scale_end,
            self.y * scale_start + end.y * scale_end,
            self.z * scale_start + end.z * scale_end,
        )
    }

    /// Returns `true` when the length is one within a tolerance of `1e-9`.
    pub fn is_normalized(self) -> bool {
        (self.norm_squared() - 1.0).abs() < 1e-9
    }

    /// Returns `true` when all four components are finite.
    pub fn is_finite(self) -> bool {
        self.w.is_finite() && self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }

    /// Returns a rotation of a quarter turn about the first axis.
    pub fn quarter_turn_x() -> Self {
        Self::from_rotation_x(core::f64::consts::FRAC_PI_2)
    }

    /// Returns a rotation of a half turn about the third axis.
    pub fn half_turn_z() -> Self {
        Self::from_rotation_z(core::f64::consts::PI)
    }
}

impl Mul for Quat {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        // Hamilton product, matching the convention that `(a * b)`
        // applies `b` first.
        Self::new(
            self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
            self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            self.w * rhs.y - self.x * rhs.z + self.y * rhs.w + self.z * rhs.x,
            self.w * rhs.z + self.x * rhs.y - self.y * rhs.x + self.z * rhs.w,
        )
    }
}

impl fmt::Display for Quat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {}, {})", self.w, self.x, self.y, self.z)
    }
}
