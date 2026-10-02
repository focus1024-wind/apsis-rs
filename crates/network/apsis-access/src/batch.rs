//! Searching access windows for many tracks, optionally across several
//! threads.
//!
//! A ground station sees a whole constellation, so the access schedule is
//! the search of one track per satellite. Each track is independent of the
//! others, which makes the search the natural place to use several cores: the
//! work is proportional to the number of satellites, and a shell of a few
//! hundred gives enough of it to be worth the threads.
//!
//! The parallel entry point returns exactly what the sequential one does, in
//! the same order, so a caller can switch between them without changing what
//! it computes.
//!
//! 为许多轨迹搜索可见性窗口，可选跨多个线程。
//!
//! 一座地面站看到整个星座，故可见性调度即是每颗卫星一条轨迹的搜索。每条轨迹与
//! 其余独立，这使该搜索成为使用多核的自然之处：工作量与卫星数成正比，而几百颗的
//! 壳层已足以值得动用线程。
//!
//! 并行入口所返回的与顺序入口完全相同，次序也相同，故调用方可在二者之间切换而
//! 不改变它所计算的东西。

use crate::{AccessError, AccessFinder, OrbitSample, PassSummary};
use apsis_math::parallel::map_index;

/// Searches access for many tracks, in order.
///
/// The sequential form, provided for comparison and for small sets where
/// starting a thread would cost more than the search.
///
/// 按次序为许多轨迹搜索可见性。
///
/// 顺序形式，供比较以及线程启动代价高于搜索本身的小规模情形使用。
#[must_use]
pub fn find_all(
    finder: &AccessFinder,
    tracks: &[Vec<OrbitSample>],
) -> Vec<Result<PassSummary, AccessError>> {
    tracks.iter().map(|track| finder.find(track)).collect()
}

/// Searches access for many tracks, across several threads.
///
/// The result matches [`find_all`] element for element and in the same order.
/// A track that cannot be searched yields its error in the same position the
/// sequential form would put it, so one bad track does not hide the rest.
///
/// 跨多个线程为许多轨迹搜索可见性。
///
/// 结果与 [`find_all`] 逐元素、逐次序相同。无法搜索的轨迹在顺序形式会
/// 放置它的同一位置给出其错误，故一条坏轨迹不会掩盖其余。
#[must_use]
pub fn find_all_parallel(
    finder: &AccessFinder,
    tracks: &[Vec<OrbitSample>],
) -> Vec<Result<PassSummary, AccessError>> {
    map_index(tracks.len(), |index| finder.find(&tracks[index]))
}
