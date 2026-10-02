//! Deciding many links at once, optionally across several threads.
//!
//! Building a constellation's topology asks the same question for every pair
//! of satellites, which is a quadratic number of independent geometry tests.
//! Each one is small, but there are so many that the loop is the dominant
//! cost of building the graph, and the tests do not depend on each other, so
//! they are the natural place to use several cores.
//!
//! The parallel entry point returns exactly what the sequential one does, in
//! the same order, so a caller can switch between them without changing the
//! graph it builds.
//!
//! 一次判定许多链路，可选跨多个线程。
//!
//! 构建星座的拓扑会对每对卫星问同一个问题，即二次数量的独立几何判定。每个都很
//! 小，但数量如此之多，使该循环成为构建图的主要代价，且各判定彼此独立，故它们是
//! 使用多核的自然之处。
//!
//! 并行入口所返回的与顺序入口完全相同，次序也相同，故调用方可在二者之间切换而
//! 不改变它所构建的图。

use crate::geometry::mutual_visibility;
use apsis_math::{Vec3, parallel::map_index};

/// Returns the visibility of every ordered pair of positions, in order.
///
/// The result is row major: the entry at `row * positions.len() + column` is
/// the visibility from `positions[row]` to `positions[column]`. A satellite is
/// always visible to itself, which is the caller's cue to skip the diagonal.
///
/// A pair whose test is undefined, because a position is not finite or the
/// body radius is not positive, yields `false`.
///
/// 按顺序返回各位置有序对的可见性。
///
/// 结果按行主序：位于 `row * positions.len() + column` 的元素是
/// `positions[row]` 对 `positions[column]` 的可见性。卫星对自身总可见，
/// 这提示调用方跳过对角线。
///
/// 测试未定义的点对（位置非有限或天体半径非正）给出 `false`。
#[must_use]
pub fn visibility_matrix(positions: &[Vec3], body_radius: f64) -> Vec<bool> {
    let len = positions.len();
    let mut matrix = vec![false; len * len];
    for (row, first) in positions.iter().enumerate() {
        for (column, second) in positions.iter().enumerate() {
            matrix[row * len + column] =
                mutual_visibility(*first, *second, body_radius).unwrap_or(false);
        }
    }
    matrix
}

/// Returns the visibility of every ordered pair of positions, across several
/// threads.
///
/// The result matches [`visibility_matrix`] element for element and in the
/// same order. The pairs are independent, so splitting the flat index space
/// across threads needs no coordination and produces the same matrix however
/// the work is divided.
///
/// The gain from threading is modest here and shrinks as the matrix shrinks.
/// A single visibility test is a few nanoseconds, so the cost of starting the
/// threads is only repaid by a large matrix: on a sixteen core machine a few
/// hundred satellites already gain, while a few dozen lose. The sequential
/// form is the better choice for a small shell, and this one exists for the
/// large ones where the quadratic sweep dominates.
///
/// 跨多个线程返回各位置有序对的可见性。
///
/// 结果与 [`visibility_matrix`] 逐元素、逐次序相同。各对相互独立，故把扁平的
/// 下标空间跨线程划分无需协调，且无论工作如何划分都产生相同的矩阵。
///
/// 此处的线程收益不大，并随矩阵变小而缩小。单次可见性判定只有几纳秒，故启动
/// 线程的代价只有大矩阵才回收：在十六核机器上，几百颗卫星已有收益，而几十颗则
/// 亏损。顺序形式是小壳层的更好选择，而这个存在是为了二次扫描占主导的大壳层。
#[must_use]
pub fn visibility_matrix_parallel(positions: &[Vec3], body_radius: f64) -> Vec<bool> {
    let len = positions.len();
    // The pairs are independent, so splitting the flat index space across
    // threads needs no coordination. `map_index` owns the output buffer,
    // which is what lets the threads write disjoint slots of one allocation.
    // 各点对相互独立，故把扁平下标空间跨线程划分无需协调。`map_index`
    // 拥有输出缓冲，这正是让各线程写同一分配中互不相交槽位的原因。
    map_index(len * len, |index| {
        let row = index / len;
        let column = index - row * len;
        mutual_visibility(positions[row], positions[column], body_radius).unwrap_or(false)
    })
}
