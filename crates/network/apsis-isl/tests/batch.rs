//! Tests for the batch visibility matrix.
//!
//! The parallel form must produce exactly the matrix the sequential form
//! does, in the same row major order. Every test compares the two rather than
//! against a constant, so a content or ordering difference is caught.
//!
//! 批量可见性矩阵的测试。
//!
//! 并行形式必须产生与顺序形式完全相同的矩阵，且行主序相同。每个测试都比较二者
//! 而非与常量比较，故内容或次序差异会被捕获。

use apsis_isl::{visibility_matrix, visibility_matrix_parallel};
use apsis_math::Vec3;

/// The body radius used throughout, in kilometers.
///
/// 全程所用的天体半径，单位为千米。
const RADIUS: f64 = 6_378.137;

/// Returns a set of positions spread around a shell.
///
/// 返回分布在一个壳层上的一组位置。
fn shell(count: usize) -> Vec<Vec3> {
    (0..count)
        .map(|index| {
            let angle = index as f64 * 0.3;
            let radius = 6_900.0;
            Vec3::new(
                radius * angle.cos(),
                radius * angle.sin(),
                radius * 0.2 * (index as f64 * 0.1).sin(),
            )
        })
        .collect()
}

#[test]
fn a_single_position_gives_a_one_by_one_matrix() {
    let positions = shell(1);
    assert_eq!(
        visibility_matrix_parallel(&positions, RADIUS),
        visibility_matrix(&positions, RADIUS)
    );
    // A satellite is visible to itself over a body radius below its own.
    // 卫星对自身可见，只要天体半径低于它自身的半径。
    assert!(visibility_matrix(&positions, RADIUS)[0]);
}

#[test]
fn a_shell_matches_the_sequential_matrix() {
    // Long enough that the parallel path takes the multi thread branch.
    // 长到足以使并行路径走多线程分支。
    for count in [2_usize, 16, 64, 200] {
        let positions = shell(count);
        assert_eq!(
            visibility_matrix_parallel(&positions, RADIUS),
            visibility_matrix(&positions, RADIUS),
            "count {count} differs"
        );
    }
}

#[test]
fn an_empty_set_gives_an_empty_matrix() {
    assert!(visibility_matrix_parallel(&[], RADIUS).is_empty());
}

#[test]
fn the_matrix_is_row_major_and_symmetric() {
    // Row major: the entry at row * len + column is the visibility from the
    // row satellite to the column one. For a spherical body the geometry is
    // symmetric, so the matrix is symmetric, which is what a wrong ordering
    // of a row major layout would break.
    // 行主序：位于 row * len + column 的元素是从第 row 颗卫星到第 column 颗的
    // 可见性。对球形天体几何是对称的，故矩阵对称，而行主序布局的次序错误会破坏这
    // 一点。
    let positions = shell(12);
    let matrix = visibility_matrix_parallel(&positions, RADIUS);
    let len = positions.len();
    assert_eq!(matrix.len(), len * len);
    for row in 0..len {
        for column in 0..len {
            assert_eq!(
                matrix[row * len + column],
                matrix[column * len + row],
                "not symmetric at ({row}, {column})"
            );
        }
    }
}

#[test]
fn the_matrix_matches_the_scalar_test() {
    // Each entry must agree with calling the geometry directly, so a wrong
    // pairing of row and column is caught even when the matrix is symmetric.
    // 每个元素必须与直接调用几何一致，故即便矩阵对称，行列配错也会被捕获。
    let positions = shell(10);
    let matrix = visibility_matrix_parallel(&positions, RADIUS);
    let len = positions.len();
    for row in 0..len {
        for column in 0..len {
            let expected = apsis_isl::mutual_visibility(positions[row], positions[column], RADIUS)
                .unwrap_or(false);
            assert_eq!(matrix[row * len + column], expected, "at ({row}, {column})");
        }
    }
}

#[test]
fn an_unusable_body_radius_yields_all_false() {
    // A non positive radius makes every test undefined, which the sequential
    // form reports as false; the parallel form must do the same.
    // 非正半径使每个判定都未定义，顺序形式报告为假；并行形式必须同样处理。
    let positions = shell(8);
    for radius in [0.0, -1.0] {
        let matrix = visibility_matrix_parallel(&positions, radius);
        assert!(matrix.iter().all(|&visible| !visible));
    }
}

#[test]
fn a_position_that_is_not_finite_yields_false_in_its_row() {
    // A satellite with a non finite position cannot see anything. The rest
    // of the matrix is unaffected.
    // 位置非有限的卫星看不到任何东西。矩阵其余部分不受影响。
    let mut positions = shell(6);
    positions[2] = Vec3::new(f64::NAN, 0.0, 0.0);
    let matrix = visibility_matrix_parallel(&positions, RADIUS);
    let len = positions.len();
    for column in 0..len {
        assert!(
            !matrix[2 * len + column],
            "row 2 column {column} should be false"
        );
        assert!(
            !matrix[column * len + 2],
            "row {column} column 2 should be false"
        );
    }
}
