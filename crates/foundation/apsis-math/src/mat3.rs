//! Three by three matrices over `f64`.
//!
//! 基于 `f64` 的三阶矩阵。

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
///
/// 按列主序存储的三乘三矩阵。
///
/// 字段名遵循 `m<行><列>` 约定，故 `m01` 是第一行第二列的元素。字段按列主序
/// 列出，[`Mat3::from_columns`] 与 [`Mat3::column`] 使用同样的顺序。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Mat3 {
    /// First row, first column.
    ///
    /// 第一行第一列。
    pub m00: f64,
    /// Second row, first column.
    ///
    /// 第二行第一列。
    pub m10: f64,
    /// Third row, first column.
    ///
    /// 第三行第一列。
    pub m20: f64,
    /// First row, second column.
    ///
    /// 第一行第二列。
    pub m01: f64,
    /// Second row, second column.
    ///
    /// 第二行第二列。
    pub m11: f64,
    /// Third row, second column.
    ///
    /// 第三行第二列。
    pub m21: f64,
    /// First row, third column.
    ///
    /// 第一行第三列。
    pub m02: f64,
    /// Second row, third column.
    ///
    /// 第二行第三列。
    pub m12: f64,
    /// Third row, third column.
    ///
    /// 第三行第三列。
    pub m22: f64,
}

impl Mat3 {
    /// The zero matrix.
    ///
    /// 零矩阵。
    pub const ZERO: Self = Self::from_rows_raw([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]);

    /// The identity matrix.
    ///
    /// 单位矩阵。
    pub const IDENTITY: Self =
        Self::from_rows_raw([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]);

    /// Creates a matrix from its nine entries in row-major order.
    ///
    /// The arguments are grouped by row: `row0`, then `row1`, then `row2`.
    ///
    /// 由九个元素按行主序创建矩阵。
    ///
    /// 参数按行分组：先 `row0`，再 `row1`，最后 `row2`。
    pub const fn from_rows(row0: [f64; 3], row1: [f64; 3], row2: [f64; 3]) -> Self {
        Self::from_rows_raw(row0, row1, row2)
    }

    /// Creates a matrix from three column vectors.
    ///
    /// 由三个列向量创建矩阵。
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
    ///
    /// 由对角元素向量创建对角矩阵。
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
    ///
    /// 由单位四元数创建旋转矩阵。
    ///
    /// 假定该四元数为单位长度。非单位四元数会得到一个带缩放的矩阵，随后被
    /// [`Mat3::is_rotation`] 拒绝。
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
    ///
    /// 创建绕单位轴 `axis` 旋转 `angle` 弧度的旋转矩阵。
    ///
    /// 轴长度为零或非有限时返回 `None`。
    pub fn from_axis_angle(axis: Vec3, angle: f64) -> Option<Self> {
        Quat::from_axis_angle(axis, angle).map(Self::from_quat)
    }

    /// Creates a rotation matrix of `angle` radians about the first axis.
    ///
    /// 创建绕第一轴旋转 `angle` 弧度的旋转矩阵。
    pub fn from_rotation_x(angle: f64) -> Self {
        Self::from_quat(Quat::from_rotation_x(angle))
    }

    /// Creates a rotation matrix of `angle` radians about the second axis.
    ///
    /// 创建绕第二轴旋转 `angle` 弧度的旋转矩阵。
    pub fn from_rotation_y(angle: f64) -> Self {
        Self::from_quat(Quat::from_rotation_y(angle))
    }

    /// Creates a rotation matrix of `angle` radians about the third axis.
    ///
    /// 创建绕第三轴旋转 `angle` 弧度的旋转矩阵。
    pub fn from_rotation_z(angle: f64) -> Self {
        Self::from_quat(Quat::from_rotation_z(angle))
    }

    /// Returns the entry in row `row` and column `column`.
    ///
    /// Returns `None` when either index is outside `0..=2`.
    ///
    /// 返回第 `row` 行第 `column` 列的元素。
    ///
    /// 任一下标超出 `0..=2` 时返回 `None`。
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
    ///
    /// 返回第 `index` 列。
    ///
    /// `index` 超出 `0..=2` 时返回 `None`。
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
    ///
    /// 返回第 `index` 行。
    ///
    /// `index` 超出 `0..=2` 时返回 `None`。
    pub const fn row(self, index: usize) -> Option<Vec3> {
        match index {
            0 => Some(Vec3::new(self.m00, self.m01, self.m02)),
            1 => Some(Vec3::new(self.m10, self.m11, self.m12)),
            2 => Some(Vec3::new(self.m20, self.m21, self.m22)),
            _ => None,
        }
    }

    /// Returns the transpose.
    ///
    /// 返回转置。
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
    ///
    /// 返回对角元素之和。
    pub const fn trace(self) -> f64 {
        self.m00 + self.m11 + self.m22
    }

    /// Returns the determinant.
    ///
    /// 返回行列式。
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
    ///
    /// 当该矩阵在 `tolerance` 内为旋转矩阵时返回 `true`。
    ///
    /// 正交且行列式为一的矩阵即旋转矩阵。该检查逐元素比较 `M^T M` 与单位
    /// 矩阵，然后把行列式与一比较。
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
    ///
    /// 当九个元素都有限时返回 `true`。
    pub fn is_finite(self) -> bool {
        [
            self.m00, self.m10, self.m20, self.m01, self.m11, self.m21, self.m02, self.m12,
            self.m22,
        ]
        .iter()
        .all(|entry| entry.is_finite())
    }

    /// Returns a copy of this matrix with one entry replaced.
    ///
    /// 返回把某一个元素替换后的矩阵副本。
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
    ///
    /// 供各关联常量使用的 const 构造函数。
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
