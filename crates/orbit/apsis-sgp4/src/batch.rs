//! Batch propagation over many instants, optionally across several threads.
//!
//! A single satellite's track is the common case for a coverage run or an
//! access search: the caller wants the state at every minute of a day, for
//! every satellite in a shell. Each state is independent of the others, so
//! the loop is a candidate for several cores, and the work is large enough
//! that starting a thread pays for itself.
//!
//! The parallel entry point produces the same values as the sequential one,
//! in the same order. It differs only in how the work is distributed.
//!
//! 在许多瞬时上的批量传播，可选跨多个线程。
//!
//! 单颗卫星的轨迹是覆盖运行或可见性搜索的常见情形：调用方想要某一天每一
//! 分钟、某一壳层每颗卫星的状态。每个状态与其他独立，故该循环是多核的
//! 候选，且工作量大到启动线程本身就值回代价。
//!
//! 并行入口产生与顺序入口相同的值，次序也相同。它只在工作的分配方式上
//! 不同。

use crate::{Satellite, State};
use apsis_math::parallel::map_index;

/// Propagates a satellite at a list of minutes from its epoch, in order.
///
/// The sequential form, provided for comparison and for small lists where
/// starting a thread would cost more than the work.
///
/// 按顺序在历元后若干分钟列表处传播该卫星。
///
/// 顺序形式，供比较以及线程开销高于工作时的小列表使用。
#[must_use]
pub fn propagate_minutes_at(satellite: &Satellite, minutes: &[f64]) -> Vec<Option<State>> {
    minutes
        .iter()
        .map(|&value| satellite.propagate_minutes(value).ok())
        .collect()
}

/// Propagates a satellite at a list of minutes from its epoch, across several
/// threads.
///
/// The result matches [`propagate_minutes_at`] element for element and in the
/// same order. A minute at which the model reports a decay yields `None`, as
/// it does in the sequential form; the threads do not stop at the first such
/// minute, because a caller usually wants the whole track and will decide for
/// itself what a decay means.
///
/// Propagation is deterministic and depends only on the minutes, so the
/// result does not depend on how the work was split.
///
/// The gain depends on how much work the list holds, because starting the
/// threads costs a fixed amount that a short list cannot repay. A day at a
/// one minute step is fourteen hundred states and gains little; a month is
/// forty three thousand and gains several fold. A caller with many short
/// tracks is better served by the per satellite form below, which takes the
/// whole list in one call.
///
/// 跨多个线程在历元后若干分钟列表处传播该卫星。
///
/// 结果与 [`propagate_minutes_at`] 逐元素相同且顺序一致。模型报告衰减的
/// 分钟给出 `None`，与顺序形式相同；线程不会在首个这样的分钟处停下，因为
/// 调用方通常需要整条轨迹，自行判断衰减的含义。
///
/// 传播是确定性的且只依赖分钟，故结果与工作如何划分无关。
///
/// 收益取决于列表所含的工作量，因为启动线程有固定开销，短列表无法收回。一
/// 天按一分钟步长是一千四百个状态，收益很小；一个月是四万三千个，可获数倍
/// 提升。拥有许多短轨迹的调用方更适合用下方的按卫星形式，它一次调用处理
/// 整个列表。
#[must_use]
pub fn propagate_minutes_at_parallel(satellite: &Satellite, minutes: &[f64]) -> Vec<Option<State>> {
    map_index(minutes.len(), |index| {
        satellite.propagate_minutes(minutes[index]).ok()
    })
}

/// Propagates several satellites at one instant, in order.
///
/// The per satellite work is independent, so a shell's state at a single
/// instant is a natural batch. The satellites are read only, so they are
/// shared across the threads by reference.
///
/// 按顺序在同一瞬时传播多颗卫星。
///
/// 各卫星的工作彼此独立，故整壳在同一瞬时的状态是天然的批处理单元。卫星
/// 只读，因此在各线程间按引用共享。
#[must_use]
pub fn propagate_at(satellites: &[Satellite], tai_mjd: f64) -> Vec<Option<State>> {
    satellites
        .iter()
        .map(|satellite| satellite.propagate_at(tai_mjd).ok())
        .collect()
}

/// Propagates several satellites at one instant, across several threads.
///
/// The result matches [`propagate_at`] element for element and in the same
/// order.
///
/// This is the form a coverage run wants: it holds the whole shell in one
/// call, so the threads start once for every satellite rather than once per
/// satellite, and even a shell of a few hundred gains.
///
/// 跨多个线程在同一瞬时传播多颗卫星。
///
/// 结果与 [`propagate_at`] 逐元素相同且顺序一致。
///
/// 覆盖计算需要的形式：它一次调用持有整壳，故线程为所有卫星启动一次而非
/// 每颗一次，即便几百颗的壳也能获益。
#[must_use]
pub fn propagate_at_parallel(satellites: &[Satellite], tai_mjd: f64) -> Vec<Option<State>> {
    map_index(satellites.len(), |index| {
        satellites[index].propagate_at(tai_mjd).ok()
    })
}
