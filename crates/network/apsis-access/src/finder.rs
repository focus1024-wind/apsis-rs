//! The access window finder.
//!
//! 可见性窗口查找器。

use crate::error::{AccessError, AccessErrorKind};
use crate::pass::{Pass, PassSummary};
use apsis_coverage::{GroundPoint, elevation_angle};
use apsis_math::Vec3;

/// One sample of a satellite track, as the finder consumes it.
///
/// The position is in the Earth-fixed frame, in kilometers, and the time is
/// in seconds from the caller's origin. The caller supplies a sequence of
/// these, which decouples the finder from the propagation and the frame
/// transformation.
///
/// 卫星轨迹的一个样本，按其被查找器消费的形式。
///
/// 位置在地固系中，单位为千米；时间以距调用方原点的秒数表示。调用方
/// 提供这样的一串样本，使查找器与传播及坐标系变换解耦。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OrbitSample {
    /// The time of the sample, in seconds from the caller's origin.
    ///
    /// 样本时刻，单位为自调用方原点起的秒数。
    pub seconds: f64,
    /// The satellite position in the Earth-fixed frame, in kilometers.
    ///
    /// 地固系中的卫星位置，单位为千米。
    pub position_ecef_km: Vec3,
}

impl OrbitSample {
    /// Creates a sample from a time and a position.
    ///
    /// 由时间与位置创建样本。
    pub const fn new(seconds: f64, position_ecef_km: Vec3) -> Self {
        Self {
            seconds,
            position_ecef_km,
        }
    }

    /// Returns `true` when the sample is usable.
    ///
    /// 当该样本可用时返回 `true`。
    pub fn is_valid(&self) -> bool {
        self.seconds.is_finite() && self.position_ecef_km.is_finite()
    }
}

/// Finds access windows for one station.
///
/// The finder holds the station and the numerical settings of the search.
/// The settings are separated from the search so that the same finder can be
/// applied to many satellites.
///
/// 为一个地面站查找可见性窗口。
///
/// 查找器保存地面站与搜索的数值设置。设置与搜索分开，使同一个查找器可
/// 应用于许多卫星。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AccessFinder {
    /// The station and its elevation mask.
    ///
    /// 地面站及其仰角遮蔽。
    pub station: GroundPoint,
    /// The interval between samples of the satellite track, in seconds.
    ///
    /// A smaller step finds shorter passes but costs proportionally more
    /// evaluations. A step of one minute resolves any pass a low Earth
    /// satellite makes, since such a pass lasts several minutes.
    ///
    /// 卫星轨迹样本之间的间隔，单位为秒。
    ///
    /// 更小的步长能发现更短的过境，但求值次数成比例增加。一分钟的步长
    /// 能分辨低轨卫星的任何过境，因为这样的过境持续数分钟。
    pub sample_step_seconds: f64,
    /// The tolerance on a crossing time, in seconds.
    ///
    /// The refinement of a rise or set continues until the interval
    /// containing the crossing is this narrow.
    ///
    /// 穿越时刻的容差，单位为秒。
    ///
    /// 对升起或降落的细化持续到包含该穿越的区间窄于该值为止。
    pub crossing_tolerance_seconds: f64,
}

impl AccessFinder {
    /// Returns `true` when the finder settings are usable.
    ///
    /// 当查找器设置可用时返回 `true`。
    pub fn is_valid(&self) -> bool {
        self.station.is_valid()
            && self.sample_step_seconds.is_finite()
            && self.sample_step_seconds > 0.0
            && self.crossing_tolerance_seconds.is_finite()
            && self.crossing_tolerance_seconds > 0.0
            && self.crossing_tolerance_seconds <= self.sample_step_seconds
    }

    /// Validates the settings, returning the reason when they are not
    /// usable.
    ///
    /// 校验设置，不可用时返回原因。
    fn validate(&self) -> Result<(), AccessError> {
        if !self.station.is_valid() {
            return Err(AccessError::new(AccessErrorKind::InvalidStation));
        }
        if !self.sample_step_seconds.is_finite() || self.sample_step_seconds <= 0.0 {
            return Err(AccessError::new(AccessErrorKind::InvalidSampleStep)
                .with_detail(format!("step is {}", self.sample_step_seconds)));
        }
        if !self.crossing_tolerance_seconds.is_finite() || self.crossing_tolerance_seconds <= 0.0 {
            return Err(AccessError::new(AccessErrorKind::InvalidTolerance)
                .with_detail(format!("tolerance is {}", self.crossing_tolerance_seconds)));
        }
        if self.crossing_tolerance_seconds > self.sample_step_seconds {
            return Err(AccessError::new(AccessErrorKind::InvalidTolerance)
                .with_detail("the tolerance exceeds the sampling step"));
        }
        Ok(())
    }

    /// Returns the elevation of a sample, in radians.
    ///
    /// Returns `None` when the sample or the station is not usable.
    ///
    /// 返回某样本的仰角，单位为弧度。
    ///
    /// 样本或地面站不可用时返回 `None`。
    pub fn elevation_of(&self, sample: &OrbitSample) -> Option<f64> {
        if !sample.is_valid() {
            return None;
        }
        elevation_angle(sample.position_ecef_km, &self.station)
    }

    /// Finds the access windows in an ordered sequence of samples.
    ///
    /// The samples must be ordered by time and cover the interval of
    /// interest. The finder assumes the elevation is continuous between
    /// samples, so the step must be short enough that the satellite does not
    /// rise and set within one step. A low Earth satellite is above a
    /// ten degree mask for several minutes, so a step of a minute or less
    /// satisfies that.
    ///
    /// The crossing times are found by interpolating the satellite position
    /// linearly between the two samples that straddle the crossing. The
    /// chord between two samples departs from the true arc by an amount of
    /// the second order in the step, so a ten second step gives a crossing
    /// accurate to about a tenth of a second and a one second step to about
    /// a millisecond. Use [`AccessFinder::find_with`] when the propagator is
    /// available and the crossing is wanted to the full tolerance.
    ///
    /// A pass that is already in progress at the first sample is reported
    /// with its rise at the first sample time, and one still in progress at
    /// the last sample is reported with its set at the last sample time.
    /// That convention means the pass is clipped to the interval searched
    /// rather than extrapolated beyond it.
    ///
    /// Returns an error when the settings are not usable, when the sample
    /// sequence has fewer than two entries, when a sample is not usable, or
    /// when the sequence is not in increasing time order. The order is
    /// checked rather than assumed, because the interpolation between two
    /// samples binary searches the sequence and a binary search over an
    /// unordered list returns a wrong index without failing.
    ///
    /// 在有序的样本序列中查找可见性窗口。
    ///
    /// 各样本必须按时间排序并覆盖所关注的区间。查找器假定仰角在样本
    /// 之间连续，故步长必须短到卫星不会在一个步长内升起又降落。低轨
    /// 卫星在十度遮蔽以上持续数分钟，故一分钟或更短的步长即满足。
    ///
    /// 穿越时刻通过在跨越该穿越的两个样本之间线性插值卫星位置求得。
    /// 两样本之间的弦与真实弧的偏差为步长的二阶量，故十秒步长给出的
    /// 穿越精度约为十分之一秒，一秒步长约为毫秒。当传播器可用且希望
    /// 穿越达到完整容差时，用 [`AccessFinder::find_with`]。
    ///
    /// 在第一个样本处已进行中的过境，其升起时刻报告为第一个样本时间；
    /// 在最后一个样本处仍在进行中的过境，其降落时刻报告为最后一个
    /// 样本时间。该约定意味着过境被裁剪到所搜索的区间，而不外推超出
    /// 它。
    ///
    /// 设置不可用、样本序列少于两项、某样本不可用，或序列未按时间
    /// 递增排列时返回错误。次序是被检查而非假定的，因为两样本之间的
    /// 插值会对序列做二分查找，而对无序列表的二分查找会返回错误的下标
    /// 而不报错。
    pub fn find(&self, samples: &[OrbitSample]) -> Result<PassSummary, AccessError> {
        self.find_internal(samples, |seconds| {
            // Interpolate the position between the bracketing samples. The
            // order of the sequence is checked by `find_internal` before
            // this closure runs, and the step is the caller's to choose.
            // 在夹逼的两个样本之间插值位置。序列的次序在该闭包运行前由
            // `find_internal` 检查，步长由调用方选择。
            let index = match samples.binary_search_by(|sample| {
                sample
                    .seconds
                    .partial_cmp(&seconds)
                    .unwrap_or(core::cmp::Ordering::Equal)
            }) {
                Ok(exact) => exact,
                Err(insert) => insert.min(samples.len() - 1),
            };
            let upper = index.max(1);
            let lower = upper - 1;
            let span = samples[upper].seconds - samples[lower].seconds;
            let fraction = if span == 0.0 {
                0.0
            } else {
                (seconds - samples[lower].seconds) / span
            };
            let position = samples[lower]
                .position_ecef_km
                .lerp(samples[upper].position_ecef_km, fraction);
            self.elevation_of(&OrbitSample::new(seconds, position))
        })
    }

    /// Runs the search given a function that returns the elevation at a
    /// time.
    ///
    /// The function must be defined over the whole interval and must be
    /// continuous, and the sample sequence must be ordered by time. The
    /// closure returns `None` when the position at that time is not usable.
    ///
    /// 在给定返回某时刻仰角的函数下运行搜索。
    ///
    /// 该函数必须定义在整个区间上且必须连续，样本序列必须按时间排序。
    /// 当该时刻的位置不可用时，该闭包返回 `None`。
    fn find_internal<F>(
        &self,
        samples: &[OrbitSample],
        mut elevation_at: F,
    ) -> Result<PassSummary, AccessError>
    where
        F: FnMut(f64) -> Option<f64>,
    {
        self.validate()?;
        if samples.len() < 2 {
            return Err(AccessError::new(AccessErrorKind::InvalidInterval)
                .with_detail("at least two samples are needed"));
        }

        // The whole sequence must be in strictly increasing time, not just its
        // ends. The interpolation between two samples binary searches this
        // list, and a binary search over an unordered list returns a wrong
        // index rather than failing, so an unordered sequence would refine a
        // crossing against the wrong pair of positions and report a pass that
        // never happened. Checking only the ends, as this once did, let that
        // through silently.
        // 整个序列必须严格按时间递增，而不只是其两端。两样本之间的插值
        // 会对该列表做二分查找，而对无序列表的二分查找会返回错误的下标
        // 而不报错，故无序序列会针对错误的位置对细化穿越，并报告一个
        // 从未发生过的过境。只检查两端（如本代码过去所做的）会使其无声
        // 通过。
        for (index, pair) in samples.windows(2).enumerate() {
            // The test is written out rather than as `!(a < b)` so that it
            // also rejects a pair whose times are not finite, and so that a
            // reader can see that equality and disorder are both refused.
            // 该测试是写开的而非写成 `!(a < b)`，使它也拒绝时间非有限的
            // 点对，并使读者能看出相等与失序都被拒绝。
            let ordered = pair[0].seconds.is_finite()
                && pair[1].seconds.is_finite()
                && pair[0].seconds < pair[1].seconds;
            if !ordered {
                return Err(
                    AccessError::new(AccessErrorKind::UnorderedSamples).with_detail(format!(
                        "sample {} at {} s does not precede {} s",
                        index + 1,
                        pair[0].seconds,
                        pair[1].seconds
                    )),
                );
            }
        }

        let first = samples[0].seconds;
        let last = samples[samples.len() - 1].seconds;
        if !first.is_finite() || !last.is_finite() {
            return Err(AccessError::new(AccessErrorKind::InvalidInterval)
                .with_detail(format!("times run from {first} to {last}")));
        }

        let mask = self.station.elevation_mask;
        let mut elevations = Vec::with_capacity(samples.len());
        for sample in samples {
            let elevation = elevation_at(sample.seconds).ok_or_else(|| {
                AccessError::new(AccessErrorKind::InvalidPosition)
                    .with_detail(format!("sample at {} s is not usable", sample.seconds))
            })?;
            if !elevation.is_finite() {
                return Err(AccessError::new(AccessErrorKind::InvalidPosition)
                    .with_detail(format!("elevation at {} s is not finite", sample.seconds)));
            }
            elevations.push(elevation);
        }

        let mut passes = Vec::new();
        let mut rise: Option<f64> = None;
        // Track the highest elevation seen inside the current pass so that
        // the peak is known without a second pass over the samples.
        // 跟踪当前过境内所见的最大仰角，使峰值无需再遍历一遍样本即可
        // 得知。
        let mut peak = f64::NEG_INFINITY;
        let mut peak_seconds = first;
        let mut peak_position = samples[0].position_ecef_km;

        for index in 0..samples.len() {
            let above = elevations[index] >= mask;
            if above {
                if rise.is_none() {
                    rise = Some(if index == 0 {
                        // The pass was already open at the start of the
                        // window, so it is clipped to the first sample.
                        // 该过境在窗口开始时就已打开，故被裁剪到第一个
                        // 样本。
                        samples[0].seconds
                    } else {
                        self.refine_crossing(samples, index, true, &mut elevation_at)
                    });
                }
                if elevations[index] > peak {
                    peak = elevations[index];
                    peak_seconds = samples[index].seconds;
                    peak_position = samples[index].position_ecef_km;
                }
            } else if let Some(rise_seconds) = rise.take() {
                let set_seconds = self.refine_crossing(samples, index, false, &mut elevation_at);
                passes.push(Pass {
                    rise_seconds,
                    set_seconds,
                    peak_seconds,
                    peak_elevation: peak,
                    mask_elevation: mask,
                    peak_position,
                });
                peak = f64::NEG_INFINITY;
            }
        }

        // A pass still open at the end of the sequence is clipped to the
        // last sample.
        // 在序列末尾仍打开的过境被裁剪到最后一个样本。
        if let Some(rise_seconds) = rise {
            passes.push(Pass {
                rise_seconds,
                set_seconds: last,
                peak_seconds,
                peak_elevation: peak,
                mask_elevation: mask,
                peak_position,
            });
        }

        Ok(PassSummary::new(passes, last - first))
    }

    /// Refines the time at which the elevation crosses the mask.
    ///
    /// The crossing lies between the sample at `index` and the one before
    /// it, and the two straddle the mask. Bisection halves the interval
    /// until it is narrower than the tolerance.
    ///
    /// When `rising` is true the crossing is a rise, where the elevation
    /// increases through the mask; otherwise it is a set. The flag selects
    /// which side of the crossing is above the mask, which keeps the search
    /// bracket correct.
    ///
    /// 细化仰角穿越遮蔽的时刻。
    ///
    /// 该穿越位于下标 `index` 处的样本与其前一个之间，两者夹逼遮蔽。
    /// 二分法把区间减半直到它窄于容差。
    ///
    /// 当 `rising` 为真时该穿越是升起，即仰角递增地穿过遮蔽；否则是
    /// 降落。该标志选择穿越的哪一侧在遮蔽之上，从而保持搜索夹逼正确。
    fn refine_crossing<F>(
        &self,
        samples: &[OrbitSample],
        index: usize,
        rising: bool,
        elevation_at: &mut F,
    ) -> f64
    where
        F: FnMut(f64) -> Option<f64>,
    {
        let mask = self.station.elevation_mask;
        let mut low = samples[index - 1].seconds;
        let mut high = samples[index].seconds;

        // The elevation difference from the mask, with the sign arranged so
        // that the function is negative at `low` and positive at `high`.
        // 相对遮蔽的仰角差，其符号被安排为使该函数在 `low` 处为负、在
        // `high` 处为正。
        let sign = if rising { 1.0 } else { -1.0 };
        let mut difference = |seconds: f64| {
            let elevation = elevation_at(seconds).unwrap_or(mask);
            sign * (elevation - mask)
        };

        let mut low_value = difference(low);
        for _ in 0..64 {
            if high - low <= self.crossing_tolerance_seconds {
                break;
            }
            let middle = 0.5 * (low + high);
            let value = difference(middle);
            if value == 0.0 {
                return middle;
            }
            if (value > 0.0) == (low_value > 0.0) {
                low = middle;
                low_value = value;
            } else {
                high = middle;
            }
        }
        0.5 * (low + high)
    }

    /// Finds the access windows over an interval, calling a propagator for
    /// each sample.
    ///
    /// The propagator receives a time in seconds from the origin of the
    /// search and returns the satellite position in the Earth-fixed frame.
    /// This adapts the finder to any orbit source without this crate
    /// depending on one.
    ///
    /// Returns an error when the settings are not usable, when the interval
    /// is not ordered, or when the propagator fails at a sample.
    ///
    /// 在某个区间上查找可见性窗口，对每个样本调用一次传播器。
    ///
    /// 传播器接收以距搜索原点的秒数表示的时间，返回地固系中的卫星
    /// 位置。这使查找器可适配任何轨道来源，而本 crate 不依赖于其一。
    ///
    /// 设置不可用、区间次序不对，或传播器在某样本处失败时返回错误。
    pub fn find_with<F>(
        &self,
        start_seconds: f64,
        end_seconds: f64,
        mut propagator: F,
    ) -> Result<PassSummary, AccessError>
    where
        F: FnMut(f64) -> Option<Vec3>,
    {
        self.validate()?;
        if !start_seconds.is_finite() || !end_seconds.is_finite() || end_seconds <= start_seconds {
            return Err(
                AccessError::new(AccessErrorKind::InvalidInterval).with_detail(format!(
                    "interval runs from {start_seconds} to {end_seconds}"
                )),
            );
        }

        // Build the sample sequence, including the end point.
        // 构建样本序列，包含端点。
        let step = self.sample_step_seconds;
        let count = ((end_seconds - start_seconds) / step).ceil() as usize + 1;
        let mut samples = Vec::with_capacity(count);
        for index in 0..count {
            let seconds = (start_seconds + index as f64 * step).min(end_seconds);
            let position = propagator(seconds).ok_or_else(|| {
                AccessError::new(AccessErrorKind::InvalidPosition)
                    .with_detail(format!("the propagator failed at {seconds} s"))
            })?;
            samples.push(OrbitSample::new(seconds, position));
        }
        // The loop above may append the end point twice when the interval is
        // an exact multiple of the step, so the last sample is replaced when
        // it duplicates the end.
        // 当区间是步长的精确整数倍时，上面的循环可能把端点追加两次，故
        // 当最后一个样本与端点重复时将其替换。
        if let Some(last) = samples.last_mut() {
            if last.seconds != end_seconds {
                let position = propagator(end_seconds).ok_or_else(|| {
                    AccessError::new(AccessErrorKind::InvalidPosition)
                        .with_detail(format!("the propagator failed at {end_seconds} s"))
                })?;
                samples.push(OrbitSample::new(end_seconds, position));
            } else {
                last.seconds = end_seconds;
            }
        }

        // The refinement calls the propagator at each bisection point, so
        // the crossing is found to the tolerance rather than to the
        // accuracy of a chord between two samples.
        // 细化在每个二分点调用传播器，故穿越是按容差求得，而非按两样本
        // 之间弦的精度求得。
        let mut failure = None;
        let summary = self.find_internal(&samples, |seconds| match propagator(seconds) {
            Some(position) if position.is_finite() => elevation_angle(position, &self.station),
            _ => {
                failure = Some(seconds);
                None
            }
        });
        match summary {
            Ok(summary) => Ok(summary),
            Err(error) => Err(match failure {
                Some(seconds) => error.with_detail(format!("the propagator failed at {seconds} s")),
                None => error,
            }),
        }
    }
}
