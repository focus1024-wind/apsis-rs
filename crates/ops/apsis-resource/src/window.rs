//! Availability windows and the interval algebra over them.
//!
//! 可用性窗口以及其上的区间代数。

use crate::error::{ResourceError, ResourceErrorKind};
use core::fmt;

/// A half-open interval of time, `[start, end)`.
///
/// The half-open convention is the one that makes intervals tile without
/// overlap or gap: an interval that ends where another begins shares only the
/// instant at the boundary, which belongs to the second. Two intervals are
/// therefore disjoint when the end of the first is at or before the start of
/// the second.
///
/// 半开时间区间 `[start, end)`。
///
/// 采用半开约定正是为了使区间既不重叠也无空隙地铺满：一个在某处结束、
/// 另一处从此开始的区间只在边界那一瞬相交，而该瞬归后者。故当第一个区
/// 间的终点不晚于第二个区间的起点时，二者不相交。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interval {
    /// The start of the interval, in seconds.
    ///
    /// 区间起点，单位为秒。
    pub start: f64,
    /// The end of the interval, in seconds.
    ///
    /// 区间终点，单位为秒。
    pub end: f64,
}

impl Interval {
    /// Creates an interval.
    ///
    /// 创建一个区间。
    pub const fn new(start: f64, end: f64) -> Self {
        Self { start, end }
    }

    /// Returns the duration of the interval, in seconds.
    ///
    /// The value is zero for an empty interval and positive otherwise.
    ///
    /// 返回该区间的时长，单位为秒。
    ///
    /// 对空区间该值为零，其余情形为正。
    pub fn duration(&self) -> f64 {
        (self.end - self.start).max(0.0)
    }

    /// Returns `true` when the interval contains no time.
    ///
    /// 当该区间不含任何时刻时返回 `true`。
    pub fn is_empty(&self) -> bool {
        self.end <= self.start
    }

    /// Returns `true` when the interval is finite and ordered.
    ///
    /// 当该区间有限且次序正确时返回 `true`。
    pub fn is_valid(&self) -> bool {
        self.start.is_finite() && self.end.is_finite() && self.end >= self.start
    }

    /// Returns `true` when an instant lies inside the interval.
    ///
    /// The start is included and the end is excluded, so a boundary instant
    /// belongs to the interval that begins there.
    ///
    /// 当某时刻落在该区间内时返回 `true`。
    ///
    /// 含起点不含终点，故边界时刻归由此处开始的区间。
    pub fn contains(&self, time: f64) -> bool {
        time >= self.start && time < self.end
    }

    /// Returns `true` when the interval fully contains another.
    ///
    /// An empty interval is contained by any interval that contains its
    /// instant, so a zero length query inside the range is satisfied.
    ///
    /// 当该区间完全包含另一区间时返回 `true`。
    ///
    /// 空区间被任何包含其瞬时的区间所包含，故范围内的零长度查询得到满足。
    pub fn contains_interval(&self, other: &Self) -> bool {
        if other.is_empty() {
            return other.start >= self.start && other.start <= self.end;
        }
        other.start >= self.start && other.end <= self.end
    }

    /// Returns `true` when two intervals share any time.
    ///
    /// Two intervals that meet at a point do not overlap, because the point
    /// belongs to only one of them under the half-open convention.
    ///
    /// 当两个区间共享任一时刻时返回 `true`。
    ///
    /// 在一点相接的两个区间不重叠，因为在半开约定下该点只属于其中之一。
    pub fn overlaps(&self, other: &Self) -> bool {
        self.start < other.end && other.start < self.end
    }

    /// Returns the intersection of two intervals, or `None` when they do not
    /// overlap.
    ///
    /// 返回两个区间的交集；二者不重叠时返回 `None`。
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        if !self.overlaps(other) {
            return None;
        }
        Some(Self::new(
            self.start.max(other.start),
            self.end.min(other.end),
        ))
    }

    /// Returns the shortest interval containing both.
    ///
    /// 返回包含二者的最短区间。
    pub fn hull(&self, other: &Self) -> Self {
        Self::new(self.start.min(other.start), self.end.max(other.end))
    }

    /// Returns `true` when the two intervals touch or overlap, so that they
    /// can be merged without leaving a gap.
    ///
    /// 当两个区间相接或重叠时返回 `true`，此时合并它们不会留下空隙。
    pub fn is_adjacent_to(&self, other: &Self) -> bool {
        self.start <= other.end && other.start <= self.end
    }
}

impl fmt::Display for Interval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {})", self.start, self.end)
    }
}

/// An availability window of a resource.
///
/// This is an [`Interval`] under a name that reads better where the caller is
/// describing when a resource can be used rather than doing interval algebra.
///
/// 资源的可用窗口。
///
/// 它就是一个 [`Interval`]，只是当调用方描述资源何时可用、而非进行区间运算
/// 时，这个名字读起来更贴切。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Window {
    /// The interval during which the resource is available.
    ///
    /// 资源可用的时间区间。
    pub interval: Interval,
}

impl Window {
    /// Creates a window from a start and an end time.
    ///
    /// 由起始时刻与结束时刻创建窗口。
    pub const fn new(start: f64, end: f64) -> Self {
        Self {
            interval: Interval::new(start, end),
        }
    }

    /// Creates a window from an interval.
    ///
    /// 由区间创建窗口。
    pub const fn from_interval(interval: Interval) -> Self {
        Self { interval }
    }

    /// Returns the start of the window, in seconds.
    ///
    /// 返回窗口起点，单位为秒。
    pub const fn start(&self) -> f64 {
        self.interval.start
    }

    /// Returns the end of the window, in seconds.
    ///
    /// 返回窗口终点，单位为秒。
    pub const fn end(&self) -> f64 {
        self.interval.end
    }

    /// Returns the duration of the window, in seconds.
    ///
    /// 返回窗口时长，单位为秒。
    pub fn duration(&self) -> f64 {
        self.interval.duration()
    }

    /// Returns `true` when the window is usable.
    ///
    /// 当该窗口可用时返回 `true`。
    pub fn is_valid(&self) -> bool {
        self.interval.is_valid()
    }

    /// Validates the window, returning the reason when it is not usable.
    ///
    /// 校验该窗口；不可用时返回原因。
    pub fn validate(&self) -> Result<(), ResourceError> {
        if !self.interval.start.is_finite() || !self.interval.end.is_finite() {
            return Err(ResourceError::new(ResourceErrorKind::NotFinite)
                .with_detail(format!("window {}", self.interval)));
        }
        if self.interval.end < self.interval.start {
            return Err(ResourceError::new(ResourceErrorKind::ReversedInterval)
                .with_detail(format!("window {}", self.interval)));
        }
        Ok(())
    }
}

impl fmt::Display for Window {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.interval)
    }
}

/// Merges a list of windows into a list of disjoint windows in time order.
///
/// Windows that overlap or touch are combined, and empty windows are dropped.
/// The result is the canonical form of a set of availability windows: sorted,
/// disjoint, and non-empty, so that two sets can be compared directly.
///
/// Returns an error when a window is not usable.
///
/// 把一组窗口合并为按时间排序的互不相交窗口。
///
/// 重叠或相接的窗口被合并，空窗口被丢弃。结果是一组可用窗口的规范形式：
/// 有序、互不相交且无空项，故两组可直接比较。
///
/// 当某个窗口不可用时返回错误。
pub fn merge_windows(windows: &[Window]) -> Result<Vec<Window>, ResourceError> {
    for window in windows {
        window.validate()?;
    }
    let mut intervals: Vec<Interval> = windows
        .iter()
        .map(|window| window.interval)
        .filter(|interval| !interval.is_empty())
        .collect();
    if intervals.is_empty() {
        return Ok(Vec::new());
    }
    // Sorting on the start and then the end gives a deterministic order for
    // intervals with equal starts.
    // 先按起点再按终点排序，使起点相同的区间具有确定次序。
    intervals.sort_by(|first, second| {
        first
            .start
            .partial_cmp(&second.start)
            .unwrap_or(core::cmp::Ordering::Equal)
            .then_with(|| {
                first
                    .end
                    .partial_cmp(&second.end)
                    .unwrap_or(core::cmp::Ordering::Equal)
            })
    });

    let mut merged: Vec<Interval> = Vec::with_capacity(intervals.len());
    for interval in intervals {
        match merged.last_mut() {
            Some(current) if current.is_adjacent_to(&interval) => {
                current.end = current.end.max(interval.end);
            }
            _ => merged.push(interval),
        }
    }
    Ok(merged.into_iter().map(Window::from_interval).collect())
}

/// Removes a window from each of a list of windows.
///
/// The result is the same set of availability with the removed window taken
/// out. Windows that are entirely inside the removed one disappear, and one
/// that contains it is split in two.
///
/// Returns an error when any input window is not usable.
///
/// 从一组窗口中逐个减去某个窗口。
///
/// 结果是同一组可用性去掉该窗口之后的情形。完全落在被减窗口内部的窗口
/// 消失，包含它的窗口被一分为二。
///
/// 当任一输入窗口不可用时返回错误。
pub fn subtract_window(windows: &[Window], removed: Window) -> Result<Vec<Window>, ResourceError> {
    removed.validate()?;
    let merged = merge_windows(windows)?;
    if removed.interval.is_empty() {
        return Ok(merged);
    }

    let mut result = Vec::with_capacity(merged.len() + 1);
    for window in merged {
        match window.interval.intersection(&removed.interval) {
            None => result.push(window),
            Some(overlap) => {
                // The part before the removed window, and the part after it.
                // 被减窗口之前的一段，以及之后的一段。
                if overlap.start > window.interval.start {
                    result.push(Window::new(window.interval.start, overlap.start));
                }
                if overlap.end < window.interval.end {
                    result.push(Window::new(overlap.end, window.interval.end));
                }
            }
        }
    }
    Ok(result)
}
