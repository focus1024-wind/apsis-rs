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

#[cfg(test)]
mod tests {
    use super::Vec3;

    const EPS: f64 = 1e-12;

    fn assert_close(lhs: Vec3, rhs: Vec3) {
        assert!((lhs - rhs).norm() < EPS, "left {lhs}, right {rhs}");
    }

    #[test]
    fn constructors_set_the_components() {
        assert_eq!(
            Vec3::new(1.0, 2.0, 3.0),
            Vec3 {
                x: 1.0,
                y: 2.0,
                z: 3.0
            }
        );
        assert_eq!(Vec3::splat(2.0), Vec3::new(2.0, 2.0, 2.0));
        assert_eq!(Vec3::default(), Vec3::ZERO);
    }

    #[test]
    fn get_selects_by_index_and_rejects_out_of_range() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        assert_eq!(v.get(0), Some(1.0));
        assert_eq!(v.get(1), Some(2.0));
        assert_eq!(v.get(2), Some(3.0));
        assert_eq!(v.get(3), None);
    }

    #[test]
    fn dot_product_is_symmetric() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(-4.0, 5.0, 0.5);
        assert!((a.dot(b) - b.dot(a)).abs() < EPS);
        assert!((a.dot(b) - 7.5).abs() < EPS);
    }

    #[test]
    fn cross_product_follows_the_right_hand_rule() {
        assert_eq!(Vec3::X.cross(Vec3::Y), Vec3::Z);
        assert_eq!(Vec3::Y.cross(Vec3::Z), Vec3::X);
        assert_eq!(Vec3::Z.cross(Vec3::X), Vec3::Y);
        // The cross product of parallel vectors vanishes.
        assert_eq!(Vec3::X.cross(Vec3::X), Vec3::ZERO);
    }

    #[test]
    fn cross_product_is_orthogonal_to_its_inputs() {
        let a = Vec3::new(1.0, -2.0, 3.5);
        let b = Vec3::new(0.5, 4.0, -1.0);
        let c = a.cross(b);
        assert!(c.dot(a).abs() < EPS);
        assert!(c.dot(b).abs() < EPS);
    }

    #[test]
    fn cross_product_is_antisymmetric() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);
        assert_eq!(a.cross(b), -b.cross(a));
    }

    #[test]
    fn norm_matches_the_pythagorean_length() {
        let v = Vec3::new(3.0, 4.0, 12.0);
        assert!((v.norm_squared() - 169.0).abs() < EPS);
        assert!((v.norm() - 13.0).abs() < EPS);
    }

    #[test]
    fn normalize_produces_a_unit_vector() {
        let v = Vec3::new(-3.0, 0.0, 4.0);
        let unit = v.normalize().expect("non-zero vector has a direction");
        assert!((unit.norm() - 1.0).abs() < EPS);
        assert_close(unit, Vec3::new(-0.6, 0.0, 0.8));
    }

    #[test]
    fn normalize_rejects_undefined_directions() {
        assert_eq!(Vec3::ZERO.normalize(), None);
        assert_eq!(Vec3::splat(f64::INFINITY).normalize(), None);
        assert_eq!(Vec3::splat(f64::NAN).normalize(), None);
    }

    #[test]
    fn distance_measures_separation() {
        let a = Vec3::new(1.0, 1.0, 1.0);
        let b = Vec3::new(1.0, 1.0, 4.0);
        assert!((a.distance(b) - 3.0).abs() < EPS);
        assert!((a.distance_squared(b) - 9.0).abs() < EPS);
    }

    #[test]
    fn angle_between_uses_the_right_hand_geometry() {
        let angle = Vec3::X
            .angle_between(Vec3::Y)
            .expect("both axes have unit length");
        assert!((angle - core::f64::consts::FRAC_PI_2).abs() < EPS);
        let opposite = Vec3::X
            .angle_between(-Vec3::X)
            .expect("both vectors have unit length");
        assert!((opposite - core::f64::consts::PI).abs() < EPS);
        let parallel = Vec3::X
            .angle_between(Vec3::X)
            .expect("both vectors have unit length");
        assert!(parallel.abs() < EPS);
    }

    #[test]
    fn angle_between_rejects_undefined_directions() {
        assert_eq!(Vec3::ZERO.angle_between(Vec3::X), None);
        assert_eq!(Vec3::X.angle_between(Vec3::ZERO), None);
    }

    #[test]
    fn lerp_hits_both_endpoints() {
        let a = Vec3::new(0.0, 0.0, 0.0);
        let b = Vec3::new(2.0, 4.0, 6.0);
        assert_eq!(a.lerp(b, 0.0), a);
        assert_eq!(a.lerp(b, 1.0), b);
        assert_eq!(a.lerp(b, 0.5), Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn component_reductions_report_extremes() {
        let v = Vec3::new(-2.0, 5.0, 1.0);
        assert!((v.max_component() - 5.0).abs() < EPS);
        assert!((v.min_component() + 2.0).abs() < EPS);
        assert_close(v.abs(), Vec3::new(2.0, 5.0, 1.0));
    }

    #[test]
    fn finiteness_is_reported() {
        assert!(Vec3::new(1.0, 2.0, 3.0).is_finite());
        assert!(!Vec3::new(1.0, f64::NAN, 3.0).is_finite());
        assert!(!Vec3::splat(f64::INFINITY).is_finite());
    }

    #[test]
    fn arithmetic_operators_match_component_wise_results() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);
        assert_eq!(a + b, Vec3::new(5.0, 7.0, 9.0));
        assert_eq!(b - a, Vec3::new(3.0, 3.0, 3.0));
        assert_eq!(-a, Vec3::new(-1.0, -2.0, -3.0));
        assert_eq!(a * 2.0, Vec3::new(2.0, 4.0, 6.0));
        assert_eq!(2.0 * a, a * 2.0);
        assert_eq!(b / 2.0, Vec3::new(2.0, 2.5, 3.0));

        let mut acc = a;
        acc += b;
        assert_eq!(acc, a + b);
        acc -= b;
        assert_eq!(acc, a);
        acc *= 3.0;
        assert_eq!(acc, a * 3.0);
        acc /= 3.0;
        assert_eq!(acc, a);
    }

    #[test]
    fn display_lists_the_components() {
        assert_eq!(Vec3::new(1.0, 2.5, -3.0).to_string(), "(1, 2.5, -3)");
    }
}
