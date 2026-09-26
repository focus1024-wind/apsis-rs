//! Three by three matrices over `f64`.

use crate::quat::Quat;
use crate::vec3::Vec3;
use core::fmt;
use core::ops::Mul;

/// A three by three matrix stored in column-major order.
///
/// The field names follow the `m<row><column>` convention, so `m01` is the
/// entry in the first row and second column. The fields are listed in
/// column-major order, and the same order is used by [`Mat3::from_columns`]
/// and [`Mat3::column`].
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Mat3 {
    /// First row, first column.
    pub m00: f64,
    /// Second row, first column.
    pub m10: f64,
    /// Third row, first column.
    pub m20: f64,
    /// First row, second column.
    pub m01: f64,
    /// Second row, second column.
    pub m11: f64,
    /// Third row, second column.
    pub m21: f64,
    /// First row, third column.
    pub m02: f64,
    /// Second row, third column.
    pub m12: f64,
    /// Third row, third column.
    pub m22: f64,
}

impl Mat3 {
    /// The zero matrix.
    pub const ZERO: Self = Self::from_rows_raw([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]);

    /// The identity matrix.
    pub const IDENTITY: Self =
        Self::from_rows_raw([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]);

    /// Creates a matrix from its nine entries in row-major order.
    ///
    /// The arguments are grouped by row: `row0`, then `row1`, then `row2`.
    pub const fn from_rows(row0: [f64; 3], row1: [f64; 3], row2: [f64; 3]) -> Self {
        Self::from_rows_raw(row0, row1, row2)
    }

    /// Creates a matrix from three column vectors.
    pub const fn from_columns(c0: Vec3, c1: Vec3, c2: Vec3) -> Self {
        Self {
            m00: c0.x,
            m10: c0.y,
            m20: c0.z,
            m01: c1.x,
            m11: c1.y,
            m21: c1.z,
            m02: c2.x,
            m12: c2.y,
            m22: c2.z,
        }
    }

    /// Creates a diagonal matrix from a vector of diagonal entries.
    pub const fn from_diagonal(diagonal: Vec3) -> Self {
        Self::from_rows_raw(
            [diagonal.x, 0.0, 0.0],
            [0.0, diagonal.y, 0.0],
            [0.0, 0.0, diagonal.z],
        )
    }

    /// Creates a rotation matrix from a unit quaternion.
    ///
    /// The quaternion is assumed to have unit length. A non-unit quaternion
    /// produces a scaled matrix, which [`Mat3::is_rotation`] then rejects.
    pub fn from_quat(q: Quat) -> Self {
        let (w, x, y, z) = (q.w, q.x, q.y, q.z);
        Self::from_rows(
            [
                1.0 - 2.0 * (y * y + z * z),
                2.0 * (x * y - w * z),
                2.0 * (x * z + w * y),
            ],
            [
                2.0 * (x * y + w * z),
                1.0 - 2.0 * (x * x + z * z),
                2.0 * (y * z - w * x),
            ],
            [
                2.0 * (x * z - w * y),
                2.0 * (y * z + w * x),
                1.0 - 2.0 * (x * x + y * y),
            ],
        )
    }

    /// Creates a rotation matrix of `angle` radians about the unit axis
    /// `axis`.
    ///
    /// Returns `None` when the axis has zero or non-finite length.
    pub fn from_axis_angle(axis: Vec3, angle: f64) -> Option<Self> {
        Quat::from_axis_angle(axis, angle).map(Self::from_quat)
    }

    /// Creates a rotation matrix of `angle` radians about the first axis.
    pub fn from_rotation_x(angle: f64) -> Self {
        Self::from_quat(Quat::from_rotation_x(angle))
    }

    /// Creates a rotation matrix of `angle` radians about the second axis.
    pub fn from_rotation_y(angle: f64) -> Self {
        Self::from_quat(Quat::from_rotation_y(angle))
    }

    /// Creates a rotation matrix of `angle` radians about the third axis.
    pub fn from_rotation_z(angle: f64) -> Self {
        Self::from_quat(Quat::from_rotation_z(angle))
    }

    /// Returns the entry in row `row` and column `column`.
    ///
    /// Returns `None` when either index is outside `0..=2`.
    pub const fn get(self, row: usize, column: usize) -> Option<f64> {
        match (row, column) {
            (0, 0) => Some(self.m00),
            (0, 1) => Some(self.m01),
            (0, 2) => Some(self.m02),
            (1, 0) => Some(self.m10),
            (1, 1) => Some(self.m11),
            (1, 2) => Some(self.m12),
            (2, 0) => Some(self.m20),
            (2, 1) => Some(self.m21),
            (2, 2) => Some(self.m22),
            _ => None,
        }
    }

    /// Returns column `index`.
    ///
    /// Returns `None` when `index` is outside `0..=2`.
    pub const fn column(self, index: usize) -> Option<Vec3> {
        match index {
            0 => Some(Vec3::new(self.m00, self.m10, self.m20)),
            1 => Some(Vec3::new(self.m01, self.m11, self.m21)),
            2 => Some(Vec3::new(self.m02, self.m12, self.m22)),
            _ => None,
        }
    }

    /// Returns row `index`.
    ///
    /// Returns `None` when `index` is outside `0..=2`.
    pub const fn row(self, index: usize) -> Option<Vec3> {
        match index {
            0 => Some(Vec3::new(self.m00, self.m01, self.m02)),
            1 => Some(Vec3::new(self.m10, self.m11, self.m12)),
            2 => Some(Vec3::new(self.m20, self.m21, self.m22)),
            _ => None,
        }
    }

    /// Returns the transpose.
    pub const fn transpose(self) -> Self {
        Self {
            m00: self.m00,
            m10: self.m01,
            m20: self.m02,
            m01: self.m10,
            m11: self.m11,
            m21: self.m12,
            m02: self.m20,
            m12: self.m21,
            m22: self.m22,
        }
    }

    /// Returns the sum of the diagonal entries.
    pub const fn trace(self) -> f64 {
        self.m00 + self.m11 + self.m22
    }

    /// Returns the determinant.
    pub fn determinant(self) -> f64 {
        self.m00 * (self.m11 * self.m22 - self.m12 * self.m21)
            - self.m01 * (self.m10 * self.m22 - self.m12 * self.m20)
            + self.m02 * (self.m10 * self.m21 - self.m11 * self.m20)
    }

    /// Returns `true` when this matrix is a rotation within `tolerance`.
    ///
    /// A matrix is a rotation when it is orthonormal with determinant one.
    /// The check compares `M^T M` to the identity entry by entry and then
    /// compares the determinant to one.
    pub fn is_rotation(self, tolerance: f64) -> bool {
        let product = self.transpose() * self;
        let identity = Self::IDENTITY;
        let orthonormal = (0..3).all(|row| {
            (0..3).all(
                |column| match (product.get(row, column), identity.get(row, column)) {
                    (Some(lhs), Some(rhs)) => (lhs - rhs).abs() <= tolerance,
                    _ => false,
                },
            )
        });
        orthonormal && (self.determinant() - 1.0).abs() <= tolerance
    }

    /// Returns `true` when all nine entries are finite.
    pub fn is_finite(self) -> bool {
        [
            self.m00, self.m10, self.m20, self.m01, self.m11, self.m21, self.m02, self.m12,
            self.m22,
        ]
        .iter()
        .all(|entry| entry.is_finite())
    }

    /// Returns a copy of this matrix with one entry replaced.
    const fn with_entry(self, row: usize, column: usize, value: f64) -> Self {
        let mut entries = [
            [self.m00, self.m01, self.m02],
            [self.m10, self.m11, self.m12],
            [self.m20, self.m21, self.m22],
        ];
        entries[row][column] = value;
        Self::from_rows_raw(entries[0], entries[1], entries[2])
    }

    /// Const constructor used by the associated constants.
    const fn from_rows_raw(row0: [f64; 3], row1: [f64; 3], row2: [f64; 3]) -> Self {
        Self {
            m00: row0[0],
            m10: row1[0],
            m20: row2[0],
            m01: row0[1],
            m11: row1[1],
            m21: row2[1],
            m02: row0[2],
            m12: row1[2],
            m22: row2[2],
        }
    }
}

impl Mul for Mat3 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let mut column = 0;
        let mut result = Self::ZERO;
        while column < 3 {
            let mut row = 0;
            while row < 3 {
                let sum = match (self.row(row), rhs.column(column)) {
                    (Some(lhs), Some(rhs)) => lhs.dot(rhs),
                    _ => 0.0,
                };
                result = result.with_entry(row, column, sum);
                row += 1;
            }
            column += 1;
        }
        result
    }
}

impl Mul<Vec3> for Mat3 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3::new(
            self.m00 * rhs.x + self.m01 * rhs.y + self.m02 * rhs.z,
            self.m10 * rhs.x + self.m11 * rhs.y + self.m12 * rhs.z,
            self.m20 * rhs.x + self.m21 * rhs.y + self.m22 * rhs.z,
        )
    }
}

impl Mul<f64> for Mat3 {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self {
        Self {
            m00: self.m00 * rhs,
            m10: self.m10 * rhs,
            m20: self.m20 * rhs,
            m01: self.m01 * rhs,
            m11: self.m11 * rhs,
            m21: self.m21 * rhs,
            m02: self.m02 * rhs,
            m12: self.m12 * rhs,
            m22: self.m22 * rhs,
        }
    }
}

impl Mul<Mat3> for f64 {
    type Output = Mat3;

    fn mul(self, rhs: Mat3) -> Mat3 {
        rhs * self
    }
}

impl fmt::Display for Mat3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}, {}, {}; {}, {}, {}; {}, {}, {}]",
            self.m00,
            self.m01,
            self.m02,
            self.m10,
            self.m11,
            self.m12,
            self.m20,
            self.m21,
            self.m22
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Mat3;
    use crate::quat::Quat;
    use crate::vec3::Vec3;
    use core::f64::consts::FRAC_PI_2;

    const EPS: f64 = 1e-12;

    fn assert_mat_close(lhs: Mat3, rhs: Mat3) {
        for row in 0..3 {
            for column in 0..3 {
                let (lhs, rhs) = (
                    lhs.get(row, column).expect("indices are in range"),
                    rhs.get(row, column).expect("indices are in range"),
                );
                assert!(
                    (lhs - rhs).abs() < EPS,
                    "at ({row}, {column}): {lhs} vs {rhs}"
                );
            }
        }
    }

    #[test]
    fn constructors_place_entries_in_the_right_slots() {
        let m = Mat3::from_rows([1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]);
        assert_eq!(m.get(0, 0), Some(1.0));
        assert_eq!(m.get(0, 1), Some(2.0));
        assert_eq!(m.get(0, 2), Some(3.0));
        assert_eq!(m.get(1, 0), Some(4.0));
        assert_eq!(m.get(2, 2), Some(9.0));
        assert_eq!(m.get(3, 0), None);
        assert_eq!(m.get(0, 3), None);

        assert_eq!(m.row(1), Some(Vec3::new(4.0, 5.0, 6.0)));
        assert_eq!(m.column(1), Some(Vec3::new(2.0, 5.0, 8.0)));
        assert_eq!(m.row(3), None);
        assert_eq!(m.column(3), None);
    }

    #[test]
    fn from_columns_matches_from_rows() {
        let c0 = Vec3::new(1.0, 4.0, 7.0);
        let c1 = Vec3::new(2.0, 5.0, 8.0);
        let c2 = Vec3::new(3.0, 6.0, 9.0);
        let expected = Mat3::from_rows([1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]);
        assert_eq!(Mat3::from_columns(c0, c1, c2), expected);
    }

    #[test]
    fn identity_is_the_multiplicative_unit() {
        let m = Mat3::from_rotation_z(FRAC_PI_2);
        assert_eq!(Mat3::IDENTITY * m, m);
        assert_eq!(m * Mat3::IDENTITY, m);
        assert_eq!(Mat3::IDENTITY.determinant(), 1.0);
        assert_eq!(Mat3::default(), Mat3::ZERO);
    }

    #[test]
    fn zero_annihilates() {
        assert_eq!(Mat3::ZERO * Vec3::new(1.0, 2.0, 3.0), Vec3::ZERO);
        assert_eq!(Mat3::ZERO * Mat3::IDENTITY, Mat3::ZERO);
        assert_eq!(Mat3::ZERO.determinant(), 0.0);
    }

    #[test]
    fn transpose_swaps_rows_and_columns() {
        let m = Mat3::from_rows([1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]);
        let t = m.transpose();
        assert_eq!(t.row(0), m.column(0));
        assert_eq!(t.row(1), m.column(1));
        assert_eq!(t.row(2), m.column(2));
        assert_eq!(t.transpose(), m);
    }

    #[test]
    fn determinant_matches_hand_computation() {
        let m = Mat3::from_rows([1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 10.0]);
        assert!((m.determinant() + 3.0).abs() < EPS);
        assert_eq!(
            Mat3::from_diagonal(Vec3::new(2.0, 3.0, 4.0)).determinant(),
            24.0
        );
    }

    #[test]
    fn trace_sums_the_diagonal() {
        let m = Mat3::from_rows([1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]);
        assert_eq!(m.trace(), 15.0);
    }

    #[test]
    fn matrix_vector_product_uses_rows() {
        let m = Mat3::from_rows([1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]);
        let v = Vec3::new(1.0, 1.0, 1.0);
        assert_eq!(m * v, Vec3::new(6.0, 15.0, 24.0));
    }

    #[test]
    fn matrix_product_is_associative() {
        let a = Mat3::from_rows([1.0, 2.0, 0.0], [0.0, 1.0, 3.0], [1.0, 0.0, 1.0]);
        let b = Mat3::from_rows([0.0, 1.0, 2.0], [1.0, 0.0, 1.0], [2.0, 1.0, 0.0]);
        let c = Mat3::from_rows([1.0, 1.0, 1.0], [2.0, 2.0, 2.0], [3.0, 3.0, 3.0]);
        assert_eq!((a * b) * c, a * (b * c));
    }

    #[test]
    fn matrix_product_preserves_the_determinant_product() {
        let a = Mat3::from_rows([1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 10.0]);
        let b = Mat3::from_rows([2.0, 0.0, 1.0], [1.0, 3.0, 2.0], [0.0, 1.0, 4.0]);
        let expected = a.determinant() * b.determinant();
        assert!(((a * b).determinant() - expected).abs() < 1e-9);
    }

    #[test]
    fn scalar_product_scales_every_entry() {
        let m = Mat3::from_rows([1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]);
        let scaled = m * 2.0;
        assert_eq!(scaled.get(0, 0), Some(2.0));
        assert_eq!(scaled.get(2, 2), Some(18.0));
        assert_eq!(2.0 * m, scaled);
    }

    #[test]
    fn rotation_matrices_satisfy_the_rotation_predicate() {
        let m = Mat3::from_rotation_z(FRAC_PI_2);
        assert!(m.is_rotation(1e-12));
        assert!((m.determinant() - 1.0).abs() < EPS);
        assert_mat_close(m * m.transpose(), Mat3::IDENTITY);
    }

    #[test]
    fn is_rotation_rejects_non_rotations() {
        assert!(!Mat3::ZERO.is_rotation(1e-9));
        assert!(!Mat3::from_diagonal(Vec3::splat(2.0)).is_rotation(1e-9));
        // A reflection has determinant minus one.
        assert!(!Mat3::from_diagonal(Vec3::new(1.0, 1.0, -1.0)).is_rotation(1e-9));
    }

    #[test]
    fn axis_angle_matches_the_quaternion_rotation() {
        let axis = Vec3::new(0.5, -2.0, 1.0);
        let m = Mat3::from_axis_angle(axis, 0.9).expect("non-zero axis");
        let q = Quat::from_axis_angle(axis, 0.9).expect("non-zero axis");
        let v = Vec3::new(1.0, -2.0, 3.0);
        assert!((m * v - q.rotate(v)).norm() < EPS);
        assert_eq!(Mat3::from_axis_angle(Vec3::ZERO, 1.0), None);
    }

    #[test]
    fn rotation_maps_axes_as_expected() {
        let m = Mat3::from_rotation_z(FRAC_PI_2);
        assert!((m * Vec3::X - Vec3::Y).norm() < EPS);
        assert!((m * Vec3::Y + Vec3::X).norm() < EPS);
        assert!((m * Vec3::Z - Vec3::Z).norm() < EPS);
    }

    #[test]
    fn per_axis_constructors_agree_with_the_axis_angle_form() {
        let angle = 0.6;
        let about_x = Mat3::from_rotation_x(angle);
        assert_eq!(
            about_x,
            Mat3::from_axis_angle(Vec3::X, angle).expect("unit axis")
        );
        assert_eq!(about_x * Vec3::X, Vec3::X);

        let about_y = Mat3::from_rotation_y(angle);
        assert_eq!(
            about_y,
            Mat3::from_axis_angle(Vec3::Y, angle).expect("unit axis")
        );
        assert_eq!(about_y * Vec3::Y, Vec3::Y);

        let about_z = Mat3::from_rotation_z(angle);
        assert_eq!(
            about_z,
            Mat3::from_axis_angle(Vec3::Z, angle).expect("unit axis")
        );
        assert_eq!(about_z * Vec3::Z, Vec3::Z);
        assert!((about_z.determinant() - 1.0).abs() < EPS);
    }

    #[test]
    fn finiteness_is_reported() {
        assert!(Mat3::IDENTITY.is_finite());
        let mut m = Mat3::IDENTITY;
        m.m01 = f64::NAN;
        assert!(!m.is_finite());
    }

    #[test]
    fn display_uses_row_major_groups() {
        let m = Mat3::from_diagonal(Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(m.to_string(), "[1, 0, 0; 0, 2, 0; 0, 0, 3]");
    }

    #[test]
    fn from_quat_is_consistent_with_quaternion_rotation() {
        let q = Quat::from_axis_angle(Vec3::new(1.0, 1.0, 0.0), 1.3).expect("non-zero axis");
        let m = Mat3::from_quat(q);
        let v = Vec3::new(2.0, -1.0, 0.5);
        assert!((m * v - q.rotate(v)).norm() < EPS);
        assert!(m.is_rotation(1e-12));
    }
}
