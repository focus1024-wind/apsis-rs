//! The access window type and its summary.
//!
//! 可见性窗口类型及其汇总。

use apsis_math::Vec3;

/// One continuous interval during which a satellite is above a station's
/// elevation mask.
///
/// Times are seconds from an origin chosen by the caller, so the same
/// structure serves a search over a scenario clock and one over elapsed
/// time.
///
/// 卫星高于某地面站仰角遮蔽的一个连续区间。
///
/// 时间是以距调用方所选原点的秒数表示，故同一结构既服务于按场景时钟的
/// 搜索，也服务于按流逝时间的搜索。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pass {
    /// The time the satellite rises above the mask, in seconds from the
    /// caller's origin.
    ///
    /// 卫星升到遮蔽之上的时间，以距调用方原点的秒数表示。
    pub rise_seconds: f64,
    /// The time the satellite falls below the mask.
    ///
    /// 卫星降到遮蔽以下的时间。
    pub set_seconds: f64,
    /// The time of greatest elevation.
    ///
    /// 仰角最大时的时间。
    pub peak_seconds: f64,
    /// The greatest elevation during the pass, in radians.
    ///
    /// 该过境期间的最大仰角，单位为弧度。
    pub peak_elevation: f64,
    /// The elevation at which the satellite crossed the mask, in radians.
    ///
    /// This is the station's mask, recorded so that a pass carries the
    /// geometry it was found under.
    ///
    /// 卫星穿越遮蔽时的仰角，单位为弧度。
    ///
    /// 这是地面站的遮蔽，记录它是为了使过境携带其被发现时所依据的
    /// 几何。
    pub mask_elevation: f64,
    /// The satellite position in the Earth-fixed frame at the peak, in
    /// kilometers.
    ///
    /// 峰值处地固系中的卫星位置，单位为千米。
    pub peak_position: Vec3,
}

impl Pass {
    /// Returns the duration of the pass, in seconds.
    ///
    /// 返回该过境的时长，单位为秒。
    #[must_use]
    pub fn duration(&self) -> f64 {
        self.set_seconds - self.rise_seconds
    }

    /// Returns the time from the rise to the peak, in seconds.
    ///
    /// 返回从升起到峰值的时间，单位为秒。
    #[must_use]
    pub fn rise_to_peak(&self) -> f64 {
        self.peak_seconds - self.rise_seconds
    }

    /// Returns the elevation at a time within the pass, when the caller can
    /// supply it.
    ///
    /// The pass stores only the peak, so this helper reports the stored
    /// values at the three defining times and defers to the caller's
    /// elevation function otherwise.
    ///
    /// 返回过境内某时刻的仰角，当调用方能提供它时。
    ///
    /// 过境只保存峰值，故该辅助函数在三个界定时刻报告所存的值，其余
    /// 情形交由调用方的仰角函数。
    #[must_use]
    pub fn contains(&self, seconds: f64) -> bool {
        seconds >= self.rise_seconds && seconds <= self.set_seconds
    }

    /// Returns the maximum elevation of the pass, in radians.
    ///
    /// This is the same as the peak elevation, named for readability at a
    /// call site that compares passes.
    ///
    /// 返回该过境的最大仰角，单位为弧度。
    ///
    /// 这与峰值仰角相同，其命名是为在比较过境的调用处更具可读性。
    #[must_use]
    pub fn maximum_elevation(&self) -> f64 {
        self.peak_elevation
    }
}

/// A summary of the access windows over an interval.
///
/// 某区间上可见性窗口的汇总。
#[derive(Debug, Clone, PartialEq)]
pub struct PassSummary {
    /// The passes, in time order.
    ///
    /// 按时序排列的各过境。
    pub passes: Vec<Pass>,
    /// The length of the interval searched, in seconds.
    ///
    /// 所搜索区间的长度，单位为秒。
    pub interval_seconds: f64,
}

impl PassSummary {
    /// Creates a summary from a list of passes and the interval length.
    ///
    /// 由过境列表与区间长度创建汇总。
    #[must_use]
    pub fn new(passes: Vec<Pass>, interval_seconds: f64) -> Self {
        Self {
            passes,
            interval_seconds,
        }
    }

    /// Returns the number of passes.
    ///
    /// 返回过境的数量。
    #[must_use]
    pub fn count(&self) -> usize {
        self.passes.len()
    }

    /// Returns the total access time, in seconds.
    ///
    /// The passes are disjoint, so the durations add.
    ///
    /// 返回总可见时间，单位为秒。
    ///
    /// 各过境互不相交，故时长相加。
    pub fn total_access_seconds(&self) -> f64 {
        self.passes.iter().map(Pass::duration).sum()
    }

    /// Returns the fraction of the interval during which the satellite was
    /// accessible.
    ///
    /// Returns `None` when the interval length is not positive.
    ///
    /// 返回卫星可见所占该区间的比例。
    ///
    /// 区间长度非正时返回 `None`。
    #[must_use]
    pub fn access_fraction(&self) -> Option<f64> {
        if self.interval_seconds > 0.0 && self.interval_seconds.is_finite() {
            Some(self.total_access_seconds() / self.interval_seconds)
        } else {
            None
        }
    }

    /// Returns the mean pass duration, in seconds.
    ///
    /// Returns `None` when there are no passes.
    ///
    /// 返回过境的平均时长，单位为秒。
    ///
    /// 没有过境时返回 `None`。
    #[must_use]
    pub fn mean_duration_seconds(&self) -> Option<f64> {
        if self.passes.is_empty() {
            return None;
        }
        Some(self.total_access_seconds() / self.passes.len() as f64)
    }

    /// Returns the longest pass, if there is one.
    ///
    /// 返回最长的过境（若存在）。
    #[must_use]
    pub fn longest(&self) -> Option<&Pass> {
        self.passes.iter().max_by(|first, second| {
            first
                .duration()
                .partial_cmp(&second.duration())
                .unwrap_or(core::cmp::Ordering::Equal)
        })
    }

    /// Returns the highest pass, if there is one.
    ///
    /// 返回仰角最高的过境（若存在）。
    #[must_use]
    pub fn highest(&self) -> Option<&Pass> {
        self.passes.iter().max_by(|first, second| {
            first
                .peak_elevation
                .partial_cmp(&second.peak_elevation)
                .unwrap_or(core::cmp::Ordering::Equal)
        })
    }

    /// Returns the gap between two consecutive passes, in seconds.
    ///
    /// Returns `None` when either index is out of range or the two indices
    /// are not consecutive and increasing.
    ///
    /// 返回两个相邻过境之间的间隙，单位为秒。
    ///
    /// 任一下标越界，或两个下标不相邻且递增时返回 `None`。
    #[must_use]
    pub fn gap_seconds(&self, first: usize, second: usize) -> Option<f64> {
        if second != first + 1 {
            return None;
        }
        let earlier = self.passes.get(first)?;
        let later = self.passes.get(second)?;
        Some(later.rise_seconds - earlier.set_seconds)
    }

    /// Returns the largest gap between consecutive passes, in seconds.
    ///
    /// This is the longest time a station waits between contacts.
    /// Returns `None` when there are fewer than two passes.
    ///
    /// 返回相邻过境之间的最大间隙，单位为秒。
    ///
    /// 这是一个地面站在两次通信之间等待的最长时间。过境少于两个时返回
    /// `None`。
    #[must_use]
    pub fn longest_gap_seconds(&self) -> Option<f64> {
        if self.passes.len() < 2 {
            return None;
        }
        self.passes
            .windows(2)
            .map(|pair| pair[1].rise_seconds - pair[0].set_seconds)
            .max_by(|first, second| {
                first
                    .partial_cmp(second)
                    .unwrap_or(core::cmp::Ordering::Equal)
            })
    }
}
