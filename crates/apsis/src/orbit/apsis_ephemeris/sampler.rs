//! Filling an ephemeris by calling a state function on a grid.
//!
//! 通过在网格上调用状态函数来填充星历。

use crate::apsis_ephemeris::ephemeris::{Ephemeris, Sample};
use crate::apsis_ephemeris::error::{EphemerisError, EphemerisErrorKind};
use crate::apsis_math::Vec3;

/// A uniform time grid that a state function is evaluated on.
///
/// A sampler describes when to sample and how finely. The grid is defined by
/// a start time, an end time, and a step, and the number of samples follows
/// from them: the samples are placed at the start and at every step up to
/// the end, with the end itself included whether or not it falls on the
/// grid. Including the end matters, because a caller that asks for coverage
/// up to a time expects the table to reach it.
///
/// 状态函数求值的均匀时间网格。
///
/// 采样器说明何时采样以及采样多密。网格由起始时刻、结束时刻与步长定义，样本数
/// 随之而定：样本置于起点及此后每隔一步处，并始终包含结束时刻，无论它是否落在
/// 网格上。包含结束时刻很重要，因为要求覆盖到某时刻的调用方期望表格确实延伸到
/// 那里。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sampler {
    /// The time of the first sample, in seconds.
    ///
    /// 首个样本时刻，单位为秒。
    pub start_seconds: f64,
    /// The time of the last sample, in seconds.
    ///
    /// 末个样本时刻，单位为秒。
    pub end_seconds: f64,
    /// The spacing between samples, in seconds.
    ///
    /// 样本间隔，单位为秒。
    pub step_seconds: f64,
}

impl Sampler {
    /// Creates a sampler over an interval with a step.
    ///
    /// 按给定步长在某区间上创建采样器。
    #[must_use]
    pub const fn new(start_seconds: f64, end_seconds: f64, step_seconds: f64) -> Self {
        Self {
            start_seconds,
            end_seconds,
            step_seconds,
        }
    }

    /// Returns `true` when the sampler describes a usable grid.
    ///
    /// The times must be finite, the end must be strictly after the start,
    /// and the step must be finite and positive. A zero length interval is
    /// refused because a table needs at least two distinct times and the
    /// interpolation has no interval to work over.
    ///
    /// 当该采样器描述的网格可用时返回 `true`。
    ///
    /// 各时刻须有限，结束时刻须严格晚于起始时刻，步长须有限且为正。零长度区间
    /// 会被拒绝，因为表格至少需要两个不同时刻，而插值没有区间可用。
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.start_seconds.is_finite()
            && self.end_seconds.is_finite()
            && self.step_seconds.is_finite()
            && self.step_seconds > 0.0
            && self.end_seconds > self.start_seconds
    }

    /// Returns the number of samples the grid produces.
    ///
    /// This is the number of whole steps that fit in the interval plus one
    /// for the start and one for the end when the end is off the grid.
    /// Returns `None` when the sampler is not usable.
    ///
    /// 返回该网格产生的样本数。
    ///
    /// 它等于区间内容纳的整步数，加上起点那一个；当结束时刻不在网格上时再加
    /// 结束那一个。采样器不可用时返回 `None`。
    #[must_use]
    pub fn sample_count(&self) -> Option<usize> {
        if !self.is_valid() {
            return None;
        }
        let span = self.end_seconds - self.start_seconds;
        let whole_steps = (span / self.step_seconds).floor();
        // The end sample is added unless it already coincides with the last
        // whole step within a rounding tolerance.
        // 除非结束样本在舍入容差内已与最后一个整步重合，否则补上它。
        let remainder = span - whole_steps * self.step_seconds;
        let extra = usize::from(remainder > self.step_seconds * 1e-9);
        Some(whole_steps as usize + 1 + extra)
    }

    /// Returns the times of the grid, in order.
    ///
    /// Returns `None` when the sampler is not usable. The last time is the
    /// end when the end does not fall on the grid, and the last grid point
    /// otherwise.
    ///
    /// 返回网格上的各时刻，按顺序排列。
    ///
    /// 采样器不可用时返回 `None`。结束时刻不落在网格上时最后一项为结束时刻，
    /// 否则为最后一个网格点。
    #[must_use]
    pub fn times(&self) -> Option<Vec<f64>> {
        let count = self.sample_count()?;
        let mut times = Vec::with_capacity(count);
        for index in 0..count {
            let time = self.start_seconds + index as f64 * self.step_seconds;
            if time >= self.end_seconds {
                break;
            }
            times.push(time);
        }
        // The end is always included, whether or not it landed on the grid.
        // 无论结束时刻是否落在网格上，它总被包含。
        if times.last().copied() != Some(self.end_seconds) {
            times.push(self.end_seconds);
        }
        Some(times)
    }

    /// Fills an ephemeris by evaluating a state function on the grid.
    ///
    /// The function maps a time in seconds to a position and velocity. In
    /// this workspace it is normally a closure over
    /// `crate::apsis_sgp4::Satellite::propagate_at`, but any function works, so
    /// the sampler can be tested against an analytic orbit.
    ///
    /// Returns an error when the sampler is not usable or when the function
    /// produces a state that is not finite.
    ///
    /// 在网格上求值状态函数以填充星历。
    ///
    /// 该函数把以秒计的时刻映射为位置与速度。在本工作区它通常是
    /// `crate::apsis_sgp4::Satellite::propagate_at` 上的闭包，但任何函数都可以，因此
    /// 采样器也可针对解析轨道测试。
    ///
    /// 采样器不可用或函数产生非有限状态时返回错误。
    pub fn sample<F>(&self, mut state_at: F) -> Result<Ephemeris, EphemerisError>
    where
        F: FnMut(f64) -> Sample,
    {
        if !self.is_valid() {
            return Err(
                EphemerisError::new(EphemerisErrorKind::NonPositiveStep).with_detail(format!(
                    "start {}, end {}, step {}",
                    self.start_seconds, self.end_seconds, self.step_seconds
                )),
            );
        }
        let times = self
            .times()
            .ok_or_else(|| EphemerisError::new(EphemerisErrorKind::NotFinite))?;
        let mut samples = Vec::with_capacity(times.len());
        for time in times {
            let sample = state_at(time);
            if !sample.is_valid() {
                return Err(EphemerisError::new(EphemerisErrorKind::InvalidState)
                    .with_detail(format!("at {time} s")));
            }
            // The time is set from the grid rather than trusted from the
            // function, so the table stays strictly ordered even if the
            // function returns a slightly different time.
            // 时刻取自网格而不采信函数返回值，因此即便函数给出略有差异的时刻，
            // 表格仍保持严格有序。
            samples.push(Sample::new(time, sample.position, sample.velocity));
        }
        Ephemeris::from_samples(samples)
    }

    /// Fills an ephemeris from a function that returns only a position.
    ///
    /// The velocity is reconstructed from neighbouring positions by a
    /// central difference. This is for a position-only source, such as a
    /// tabulated file; where a velocity is available it should be used, so
    /// prefer [`Sampler::sample`].
    ///
    /// 用只返回位置的函数填充星历。
    ///
    /// 速度由相邻位置用中心差分重建。这适用于只有位置的数据源，例如制表文件；
    /// 若可得速度应予使用，故优先选用 [`Sampler::sample`]。
    pub fn sample_positions<F>(&self, mut position_at: F) -> Result<Ephemeris, EphemerisError>
    where
        F: FnMut(f64) -> Vec3,
    {
        if !self.is_valid() {
            return Err(EphemerisError::new(EphemerisErrorKind::NonPositiveStep));
        }
        let times = self
            .times()
            .ok_or_else(|| EphemerisError::new(EphemerisErrorKind::NotFinite))?;
        let mut positions = Vec::with_capacity(times.len());
        for &time in &times {
            let position = position_at(time);
            if !position.is_finite() {
                return Err(EphemerisError::new(EphemerisErrorKind::InvalidState)
                    .with_detail(format!("at {time} s")));
            }
            positions.push(position);
        }

        // A central difference inside the interval and a one sided one at
        // each end. The finite difference is second order, which matches the
        // accuracy of the Hermite interpolation it feeds.
        // 区间内部用中心差分，两端各用单边差分。该有限差分为二阶，与其所供给的
        // Hermite 插值精度相匹配。
        let count = positions.len();
        let mut samples = Vec::with_capacity(count);
        for (index, &position) in positions.iter().enumerate() {
            let velocity = if count == 1 {
                Vec3::new(0.0, 0.0, 0.0)
            } else if index == 0 {
                (positions[1] - positions[0]) / (times[1] - times[0])
            } else if index == count - 1 {
                (positions[count - 1] - positions[count - 2])
                    / (times[count - 1] - times[count - 2])
            } else {
                (positions[index + 1] - positions[index - 1])
                    / (times[index + 1] - times[index - 1])
            };
            samples.push(Sample::new(times[index], position, velocity));
        }
        Ephemeris::from_samples(samples)
    }
}
