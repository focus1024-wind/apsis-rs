//! The ephemeris table and its Hermite interpolation.

use crate::error::{EphemerisError, EphemerisErrorKind};
use apsis_math::Vec3;

/// One sampled state.
///
/// A sample is a position and velocity at a time, which is the form every
/// propagator in the workspace produces and the form the interpolation
/// needs.
///
/// 一个采样状态。
///
/// 样本是某时刻的位置与速度，这既是工作区中每个传播器产出的形式，也是插值所需
/// 的形式。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample {
    /// The time of the sample, in seconds from the caller's origin.
    ///
    /// 样本时刻，单位为自调用方原点起的秒数。
    ///
    /// 样本时刻，单位为自调用方原点起的秒数。
    pub seconds: f64,
    /// Position, in kilometers.
    ///
    /// 位置，单位为千米。
    ///
    /// 位置，单位为千米。
    pub position: Vec3,
    /// Velocity, in kilometers per second.
    ///
    /// 速度，单位为千米每秒。
    ///
    /// 速度，单位为千米每秒。
    pub velocity: Vec3,
}

impl Sample {
    /// Creates a sample.
    ///
    /// 创建一个样本。
    ///
    /// 创建一个样本。
    pub const fn new(seconds: f64, position: Vec3, velocity: Vec3) -> Self {
        Self {
            seconds,
            position,
            velocity,
        }
    }

    /// Returns `true` when the time, position, and velocity are finite.
    ///
    /// 当时间、位置与速度都有限时返回 `true`。
    pub fn is_valid(&self) -> bool {
        self.seconds.is_finite() && self.position.is_finite() && self.velocity.is_finite()
    }

    /// Returns the distance from the origin, in kilometers.
    ///
    /// 返回到原点的距离，单位为千米。
    pub fn radius(&self) -> f64 {
        self.position.norm()
    }

    /// Returns the speed, in kilometers per second.
    ///
    /// 返回速率，单位为千米每秒。
    pub fn speed(&self) -> f64 {
        self.velocity.norm()
    }
}

/// Evaluates the Hermite cubic for one component.
///
/// `p0` and `p1` are the endpoint values, `v0` and `v1` the endpoint
/// derivatives with respect to time, `dt` the interval length, and `s` the
/// fraction along the interval in `[0, 1]`.
///
/// The four terms are the standard cubic Hermite basis: the first and third
/// carry the endpoints, the second and fourth the slopes. The derivative
/// terms are scaled by `dt` because the slopes are per unit time while `s`
/// is per unit interval.
///
/// 对一个分量求 Hermite 三次式的值。
///
/// `p0` 与 `p1` 为端点值，`v0` 与 `v1` 为端点处对时间的导数，`dt` 为区间长度，
/// `s` 为区间内位置比例，位于 `[0, 1]`。
///
/// 四项即标准三次 Hermite 基：第一、三项承载端点值，第二、四项承载斜率。导数项
/// 乘以 `dt`，因为斜率以单位时间计而 `s` 以单位区间计。
pub fn hermite_position(p0: Vec3, v0: Vec3, p1: Vec3, v1: Vec3, dt: f64, s: f64) -> Vec3 {
    let s2 = s * s;
    let s3 = s2 * s;
    let h00 = 2.0 * s3 - 3.0 * s2 + 1.0;
    let h10 = s3 - 2.0 * s2 + s;
    let h01 = -2.0 * s3 + 3.0 * s2;
    let h11 = s3 - s2;
    p0 * h00 + v0 * (h10 * dt) + p1 * h01 + v1 * (h11 * dt)
}

/// Evaluates the derivative of the Hermite cubic for one component.
///
/// This is the velocity that goes with [`hermite_position`]. It is the exact
/// derivative of that cubic, so the reconstructed state is internally
/// consistent: a caller that differentiates the position gets this velocity.
///
/// 对一个分量求 Hermite 三次式的导数。
///
/// 这是与 [`hermite_position`] 配对的速度，且是该三次式的精确导数，因此重建
/// 的状态内部自洽：调用方对位置求导会得到这个速度。
pub fn hermite_velocity(p0: Vec3, v0: Vec3, p1: Vec3, v1: Vec3, dt: f64, s: f64) -> Vec3 {
    let s2 = s * s;
    let dh00 = 6.0 * s2 - 6.0 * s;
    let dh10 = 3.0 * s2 - 4.0 * s + 1.0;
    let dh01 = -6.0 * s2 + 6.0 * s;
    let dh11 = 3.0 * s2 - 2.0 * s;
    // The chain rule turns the per interval derivative into a per second
    // one, which divides the position terms by `dt` and leaves the velocity
    // terms, already per second, untouched.
    // 链式法则把每单位区间的导数变为每单位秒的导数：位置项除以 `dt`，而速度项
    // 本就以每秒计，保持不变。
    p0 * (dh00 / dt) + v0 * dh10 + p1 * (dh01 / dt) + v1 * dh11
}

/// Evaluates both the Hermite position and velocity.
///
/// 同时求 Hermite 位置与速度。
pub fn hermite_state(p0: Vec3, v0: Vec3, p1: Vec3, v1: Vec3, dt: f64, s: f64) -> (Vec3, Vec3) {
    (
        hermite_position(p0, v0, p1, v1, dt, s),
        hermite_velocity(p0, v0, p1, v1, dt, s),
    )
}

/// A table of sampled states that answers queries by interpolation.
///
/// The table holds at least two samples in strictly increasing time order.
/// A query at a sample time returns that sample exactly; a query between two
/// samples returns the Hermite cubic through the four endpoint values. A
/// query outside the covered interval is refused rather than extrapolated,
/// because the interpolation has no information there and an extrapolated
/// orbit can diverge quickly.
///
/// 以插值回答查询的采样状态表。
///
/// 该表至少含两个样本，时间严格递增。在样本时刻查询会精确返回该样本；两个样本
/// 之间的查询返回穿过四个端点值的 Hermite 三次式。落在覆盖区间之外的查询会被
/// 拒绝而非外推，因为插值在那里没有信息，而外推的轨道可能迅速发散。
#[derive(Debug, Clone, PartialEq)]
pub struct Ephemeris {
    samples: Vec<Sample>,
}

impl Ephemeris {
    /// Builds a table from samples in increasing time order.
    ///
    /// Returns an error when there are fewer than two samples, when a sample
    /// is not finite, or when the times do not strictly increase.
    ///
    /// 由时间递增的样本构建该表。
    ///
    /// 当样本少于两个、某个样本非有限，或时间未严格递增时返回错误。
    pub fn from_samples(samples: Vec<Sample>) -> Result<Self, EphemerisError> {
        if samples.len() < 2 {
            return Err(EphemerisError::new(EphemerisErrorKind::TooFewSamples)
                .with_detail(format!("{} sample(s)", samples.len())));
        }
        for sample in &samples {
            if !sample.is_valid() {
                return Err(EphemerisError::new(EphemerisErrorKind::InvalidState)
                    .with_detail(format!("at {} s", sample.seconds)));
            }
        }
        for pair in samples.windows(2) {
            if pair[1].seconds <= pair[0].seconds {
                return Err(
                    EphemerisError::new(EphemerisErrorKind::NotMonotonic).with_detail(format!(
                        "{} s follows {} s",
                        pair[1].seconds, pair[0].seconds
                    )),
                );
            }
        }
        Ok(Self { samples })
    }

    /// Returns the samples in time order.
    ///
    /// 返回按时间排列的样本。
    pub fn samples(&self) -> &[Sample] {
        &self.samples
    }

    /// Returns the number of samples.
    ///
    /// 返回样本数。
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Returns `true` when the table is empty.
    ///
    /// A table built through [`Ephemeris::from_samples`] is never empty, so
    /// this is always `false` for one that exists; it is provided so that
    /// generic code that accepts a slice can use the usual pair of methods.
    ///
    /// 当该表为空时返回 `true`。
    ///
    /// 经 [`Ephemeris::from_samples`] 构建的表不会为空，故已存在的表恒返回
    /// `false`；提供它是为了让接受切片的泛型代码可以配上惯常的一对方法。
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Returns the first sample time, in seconds.
    ///
    /// 返回首个样本时刻，单位为秒。
    pub fn start_time(&self) -> f64 {
        self.samples.first().map_or(0.0, |sample| sample.seconds)
    }

    /// Returns the last sample time, in seconds.
    ///
    /// 返回末个样本时刻，单位为秒。
    pub fn end_time(&self) -> f64 {
        self.samples.last().map_or(0.0, |sample| sample.seconds)
    }

    /// Returns the time the table covers, in seconds.
    ///
    /// 返回该表覆盖的时间，单位为秒。
    pub fn duration(&self) -> f64 {
        self.end_time() - self.start_time()
    }

    /// Returns `true` when a time falls within the covered interval.
    ///
    /// The interval is closed, so its endpoints are covered.
    ///
    /// 当某时刻落在覆盖区间内时返回 `true`。
    ///
    /// 该区间是闭的，故其端点也被覆盖。
    pub fn covers(&self, seconds: f64) -> bool {
        seconds.is_finite() && seconds >= self.start_time() && seconds <= self.end_time()
    }

    /// Returns the nominal step, when the samples are evenly spaced.
    ///
    /// Returns `None` when the spacing varies, which a caller checks before
    /// indexing samples by arithmetic.
    ///
    /// 当样本等间隔时返回标称步长。
    ///
    /// 间隔不一致时返回 `None`；调用方应先检查这一点，再按算术方式索引样本。
    pub fn uniform_step(&self) -> Option<f64> {
        let first = self.samples.windows(2).next()?[1].seconds - self.samples[0].seconds;
        let uniform = self
            .samples
            .windows(2)
            .all(|pair| (pair[1].seconds - pair[0].seconds - first).abs() <= 1e-9);
        uniform.then_some(first)
    }

    /// Returns `true` when the samples are evenly spaced.
    ///
    /// 当样本等间隔时返回 `true`。
    pub fn is_uniform(&self) -> bool {
        self.uniform_step().is_some()
    }

    /// Returns the index of the interval that contains a time.
    ///
    /// The index `i` is such that `samples[i].seconds <= seconds <=
    /// samples[i + 1].seconds`. A time outside the interval returns `None`.
    ///
    /// 返回包含某时刻的区间下标。
    ///
    /// 下标 `i` 满足 `samples[i].seconds <= seconds <= samples[i + 1].seconds`。
    /// 落在该区间之外的时刻返回 `None`。
    fn interval_of(&self, seconds: f64) -> Option<usize> {
        if !self.covers(seconds) {
            return None;
        }
        // A binary search for the last sample at or before the time. The
        // partition point is the first sample strictly after it, so the
        // interval starts one before, clamped to the last interval.
        // 二分查找最后一个不晚于该时刻的样本。分区点是其后第一个严格靠后的样本，
        // 故区间起点为其前一，并钳位到最后一个区间。
        let after = self
            .samples
            .partition_point(|sample| sample.seconds <= seconds);
        let index = after.saturating_sub(1);
        Some(index.min(self.samples.len() - 2))
    }

    /// Returns the state at a time, interpolating between samples.
    ///
    /// A time equal to a sample time returns that sample. Returns `None`
    /// when the time is outside the covered interval.
    ///
    /// 返回某时刻的状态，在样本之间插值。
    ///
    /// 时刻与某样本相同时返回该样本。落在覆盖区间之外时返回 `None`。
    pub fn state_at(&self, seconds: f64) -> Option<Sample> {
        let index = self.interval_of(seconds)?;
        let first = self.samples[index];
        let second = self.samples[index + 1];

        // At a sample the interpolation must reproduce the sample exactly,
        // which avoids the round off of evaluating the cubic at its end.
        // 在样本处插值必须精确复现该样本，从而避免在三次式端点求值带来的
        // 舍入误差。
        if seconds == first.seconds {
            return Some(first);
        }
        if seconds == second.seconds {
            return Some(second);
        }

        let dt = second.seconds - first.seconds;
        let s = (seconds - first.seconds) / dt;
        let (position, velocity) = hermite_state(
            first.position,
            first.velocity,
            second.position,
            second.velocity,
            dt,
            s,
        );
        Some(Sample::new(seconds, position, velocity))
    }

    /// Returns the state at a time, or the nearest endpoint when the time is
    /// outside the covered interval.
    ///
    /// This is for a caller that must produce a value for any time and is
    /// content to hold the last known state rather than fail.
    ///
    /// 返回某时刻的状态；时刻落在覆盖区间之外时返回最近的端点。
    ///
    /// 适用于必须对任意时刻给出取值、且宁可保持最近已知状态而不愿失败的调用方。
    pub fn clamped_state_at(&self, seconds: f64) -> Option<Sample> {
        if self.covers(seconds) {
            return self.state_at(seconds);
        }
        if !seconds.is_finite() {
            return None;
        }
        if seconds < self.start_time() {
            self.samples.first().copied()
        } else {
            self.samples.last().copied()
        }
    }

    /// Returns the sample nearest to a time, without interpolating.
    ///
    /// 返回距某时刻最近的样本，不作插值。
    pub fn nearest_sample(&self, seconds: f64) -> Option<Sample> {
        if !seconds.is_finite() {
            return None;
        }
        let index = match self.interval_of(seconds) {
            Some(index) => {
                let first = self.samples[index];
                let second = self.samples[index + 1];
                if (seconds - first.seconds).abs() <= (second.seconds - seconds).abs() {
                    index
                } else {
                    index + 1
                }
            }
            None if seconds < self.start_time() => 0,
            None => self.samples.len() - 1,
        };
        Some(self.samples[index])
    }

    /// Returns the times of every sample, in order.
    ///
    /// 返回各样本时刻，按顺序排列。
    pub fn times(&self) -> Vec<f64> {
        self.samples.iter().map(|sample| sample.seconds).collect()
    }

    /// Returns `true` when every state satisfies a predicate.
    ///
    /// This is a convenience for a caller that wants to check a property of
    /// the whole table, such as every radius being above the surface, once.
    ///
    /// 当每个状态都满足给定谓词时返回 `true`。
    ///
    /// 这是给需要一次性检查整表性质的调用方的便利方法，例如检查所有半径都在
    /// 表面之上。
    pub fn all_samples<F>(&self, predicate: F) -> bool
    where
        F: Fn(&Sample) -> bool,
    {
        self.samples.iter().all(predicate)
    }
}
