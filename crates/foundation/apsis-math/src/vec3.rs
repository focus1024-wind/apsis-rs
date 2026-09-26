//! Three component vectors over `f64`.

use core::fmt;
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// A three component column vector.
///
/// The components are named `x`, `y`, and `z`. No meaning is attached to
/// them here; the reference frame is defined by the crate that uses the
/// vector.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec3 {
    /// First component.
    pub x: f64,
    /// Second component.
    pub y: f64,
    /// Third component.
    pub z: f64,
}

impl Vec3 {
    /// The zero vector.
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    /// The unit vector along the first axis.
    pub const X: Self = Self::new(1.0, 0.0, 0.0);

    /// The unit vector along the second axis.
    pub const Y: Self = Self::new(0.0, 1.0, 0.0);

    /// The unit vector along the third axis.
    pub const Z: Self = Self::new(0.0, 0.0, 1.0);

    /// Creates a vector from its three components.
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Creates a vector whose three components all equal `value`.
    pub const fn splat(value: f64) -> Self {
        Self::new(value, value, value)
    }

    /// Returns the first component.
    pub const fn x(self) -> f64 {
        self.x
    }

    /// Returns the second component.
    pub const fn y(self) -> f64 {
        self.y
    }

    /// Returns the third component.
    pub const fn z(self) -> f64 {
        self.z
    }

    /// Returns the component selected by `index`, where `0`, `1`, and `2`
    /// select `x`, `y`, and `z` respectively.
    ///
    /// Returns `None` for an index outside `0..=2`.
    pub const fn get(self, index: usize) -> Option<f64> {
        match index {
            0 => Some(self.x),
            1 => Some(self.y),
            2 => Some(self.z),
            _ => None,
        }
    }

    /// Returns the dot product of `self` and `rhs`.
    pub fn dot(self, rhs: Self) -> f64 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    /// Returns the cross product `self x rhs`.
    ///
    /// The result is perpendicular to both inputs and follows the
    /// right-hand rule, so `Vec3::X.cross(Vec3::Y)` equals `Vec3::Z`.
    pub fn cross(self, rhs: Self) -> Self {
        Self::new(
            self.y * rhs.z - self.z * rhs.y,
            self.z * rhs.x - self.x * rhs.z,
            self.x * rhs.y - self.y * rhs.x,
        )
    }

    /// Returns the squared Euclidean length of this vector.
    ///
    /// Prefer this over [`Vec3::norm`] when comparing lengths, because it
    /// avoids a square root.
    pub fn norm_squared(self) -> f64 {
        self.dot(self)
    }

    /// Returns the Euclidean length of this vector.
    pub fn norm(self) -> f64 {
        self.norm_squared().sqrt()
    }

    /// Returns this vector scaled to unit length.
    ///
    /// Returns `None` when the length is zero or not finite, in which case
    /// no direction is defined.
    pub fn normalize(self) -> Option<Self> {
        let norm = self.norm();
        if norm > 0.0 && norm.is_finite() {
            Some(self / norm)
        } else {
            None
        }
    }

    /// Returns the Euclidean distance between `self` and `rhs`.
    pub fn distance(self, rhs: Self) -> f64 {
        (self - rhs).norm()
    }

    /// Returns the squared Euclidean distance between `self` and `rhs`.
    pub fn distance_squared(self, rhs: Self) -> f64 {
        (self - rhs).norm_squared()
    }

    /// Returns the angle between `self` and `rhs`, in radians.
    ///
    /// The result lies in `[0, pi]`. Returns `None` when either vector has
    /// zero or non-finite length, or when either vector has a non-finite
    /// component.
    pub fn angle_between(self, rhs: Self) -> Option<f64> {
        let denominator = self.norm() * rhs.norm();
        if denominator > 0.0 && denominator.is_finite() {
            // The cosine can exceed one in magnitude through rounding, so
            // it is clamped to keep the arc cosine defined.
            Some((self.dot(rhs) / denominator).clamp(-1.0, 1.0).acos())
        } else {
            None
        }
    }

    /// Returns the linear interpolation `self + t * (rhs - self)`.
    ///
    /// `t = 0` returns `self` and `t = 1` returns `rhs`. Values outside
    /// `[0, 1]` extrapolate.
    pub fn lerp(self, rhs: Self, t: f64) -> Self {
        self + (rhs - self) * t
    }

    /// Returns a vector with each component replaced by its absolute value.
    pub fn abs(self) -> Self {
        Self::new(self.x.abs(), self.y.abs(), self.z.abs())
    }

    /// Returns the largest component.
    pub fn max_component(self) -> f64 {
        self.x.max(self.y).max(self.z)
    }

    /// Returns the smallest component.
    pub fn min_component(self) -> f64 {
        self.x.min(self.y).min(self.z)
    }

    /// Returns `true` when all three components are finite.
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }
}

impl Mul<f64> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl Mul<Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Vec3 {
        rhs * self
    }
}

impl Div<f64> for Vec3 {
    type Output = Self;

    fn div(self, rhs: f64) -> Self {
        Self::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl MulAssign<f64> for Vec3 {
    fn mul_assign(&mut self, rhs: f64) {
        *self = *self * rhs;
    }
}

impl DivAssign<f64> for Vec3 {
    fn div_assign(&mut self, rhs: f64) {
        *self = *self / rhs;
    }
}

impl fmt::Display for Vec3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}
