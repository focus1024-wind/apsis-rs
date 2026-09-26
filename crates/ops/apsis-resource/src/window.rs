//! Availability windows and the interval algebra over them.

use crate::error::{ResourceError, ResourceErrorKind};
use core::fmt;

/// A half-open interval of time, `[start, end)`.
///
/// The half-open convention is the one that makes intervals tile without
/// overlap or gap: an interval that ends where another begins shares only the
/// instant at the boundary, which belongs to the second. Two intervals are
/// therefore disjoint when the end of the first is at or before the start of
/// the second.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interval {
    /// The start of the interval, in seconds.
    pub start: f64,
    /// The end of the interval, in seconds.
    pub end: f64,
}

impl Interval {
    /// Creates an interval.
    pub const fn new(start: f64, end: f64) -> Self {
        Self { start, end }
    }

    /// Returns the duration of the interval, in seconds.
    ///
    /// The value is zero for an empty interval and positive otherwise.
    pub fn duration(&self) -> f64 {
        (self.end - self.start).max(0.0)
    }

    /// Returns `true` when the interval contains no time.
    pub fn is_empty(&self) -> bool {
        self.end <= self.start
    }

    /// Returns `true` when the interval is finite and ordered.
    pub fn is_valid(&self) -> bool {
        self.start.is_finite() && self.end.is_finite() && self.end >= self.start
    }

    /// Returns `true` when an instant lies inside the interval.
    ///
    /// The start is included and the end is excluded, so a boundary instant
    /// belongs to the interval that begins there.
    pub fn contains(&self, time: f64) -> bool {
        time >= self.start && time < self.end
    }

    /// Returns `true` when the interval fully contains another.
    ///
    /// An empty interval is contained by any interval that contains its
    /// instant, so a zero length query inside the range is satisfied.
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
    pub fn overlaps(&self, other: &Self) -> bool {
        self.start < other.end && other.start < self.end
    }

    /// Returns the intersection of two intervals, or `None` when they do not
    /// overlap.
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
    pub fn hull(&self, other: &Self) -> Self {
        Self::new(self.start.min(other.start), self.end.max(other.end))
    }

    /// Returns `true` when the two intervals touch or overlap, so that they
    /// can be merged without leaving a gap.
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Window {
    /// The interval during which the resource is available.
    pub interval: Interval,
}

impl Window {
    /// Creates a window from a start and an end time.
    pub const fn new(start: f64, end: f64) -> Self {
        Self {
            interval: Interval::new(start, end),
        }
    }

    /// Creates a window from an interval.
    pub const fn from_interval(interval: Interval) -> Self {
        Self { interval }
    }

    /// Returns the start of the window, in seconds.
    pub const fn start(&self) -> f64 {
        self.interval.start
    }

    /// Returns the end of the window, in seconds.
    pub const fn end(&self) -> f64 {
        self.interval.end
    }

    /// Returns the duration of the window, in seconds.
    pub fn duration(&self) -> f64 {
        self.interval.duration()
    }

    /// Returns `true` when the window is usable.
    pub fn is_valid(&self) -> bool {
        self.interval.is_valid()
    }

    /// Validates the window, returning the reason when it is not usable.
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
