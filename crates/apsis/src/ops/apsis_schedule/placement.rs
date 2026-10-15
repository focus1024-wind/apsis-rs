//! Placements and the evaluation of a schedule.
//!
//! 放置以及调度的评估。

use crate::apsis_resource::ResourceId;
use crate::apsis_task::TaskId;

/// One task placed on the schedule.
///
/// 计划上放置的一个任务。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    /// The task placed.
    ///
    /// 被放置的任务。
    pub task: TaskId,
    /// The start time of the placement, in seconds.
    ///
    /// 放置的起始时刻，单位为秒。
    pub start: f64,
    /// The duration of the placement, in seconds.
    ///
    /// This is copied from the task so that a schedule carries everything
    /// needed to evaluate it without the tasks to hand.
    ///
    /// 放置的时长，单位为秒。
    ///
    /// 它从任务复制而来，故一份计划无需持有任务集合也能完成评估。
    pub duration: f64,
}

impl Placement {
    /// Creates a placement.
    ///
    /// 创建一次放置。
    #[must_use]
    pub const fn new(task: TaskId, start: f64, duration: f64) -> Self {
        Self {
            task,
            start,
            duration,
        }
    }

    /// Returns the end time of the placement, in seconds.
    ///
    /// 返回放置的结束时刻，单位为秒。
    #[must_use]
    pub fn end(&self) -> f64 {
        self.start + self.duration
    }

    /// Returns `true` when the placement covers an instant.
    ///
    /// The interval is half-open, so a placement covers its start and not its
    /// end. That convention lets two back to back placements share a boundary
    /// without overlapping.
    ///
    /// 当该放置覆盖某瞬时返回 `true`。
    ///
    /// 区间是半开的，故放置含其起点而不含其终点。该约定使两个首尾相接的
    /// 放置可共享边界而不重叠。
    #[must_use]
    pub fn covers(&self, time: f64) -> bool {
        time >= self.start && time < self.end()
    }

    /// Returns `true` when the placement overlaps another.
    ///
    /// 当该放置与另一个放置重叠时返回 `true`。
    #[must_use]
    pub fn overlaps(&self, other: &Self) -> bool {
        self.start < other.end() && other.start < self.end()
    }

    /// Returns `true` when the placement is usable.
    ///
    /// 当该放置可用时返回 `true`。
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.start.is_finite() && self.duration.is_finite() && self.duration >= 0.0
    }
}

/// How much of one resource a schedule uses.
///
/// 一份计划对某个资源的使用情况。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResourceUsage {
    /// The resource.
    ///
    /// 该资源。
    pub resource: ResourceId,
    /// The greatest total demand placed on the resource at any instant.
    ///
    /// 该资源上任意瞬时的最大总需求。
    pub peak_demand: f64,
    /// The time during which the resource is used at all, in seconds.
    ///
    /// 该资源被使用的总时长，单位为秒。
    pub busy_seconds: f64,
    /// The number of placements that use the resource.
    ///
    /// 使用该资源的放置数量。
    pub placement_count: usize,
}

impl ResourceUsage {
    /// Returns the utilisation of the resource over an interval.
    ///
    /// The value is the busy time divided by the length of the interval. It
    /// is a time utilisation, not a capacity utilisation: a resource that is
    /// only half occupied by demand still counts as busy.
    ///
    /// Returns `None` when the interval is not positive.
    ///
    /// 返回该资源在某区间上的利用率。
    ///
    /// 其值为忙碌时长除以区间长度。这是时间利用率而非容量利用率：仅被
    /// 需求占据一半的资源同样算作忙碌。
    ///
    /// 区间非正时返回 `None`。
    #[must_use]
    pub fn utilisation(&self, interval_seconds: f64) -> Option<f64> {
        if interval_seconds > 0.0 && interval_seconds.is_finite() {
            Some(self.busy_seconds / interval_seconds)
        } else {
            None
        }
    }
}

/// The evaluation of a whole schedule.
///
/// 整份计划的评估结果。
#[derive(Debug, Clone, PartialEq)]
pub struct Evaluation {
    /// The number of placements.
    ///
    /// 放置的数量。
    pub placed_count: usize,
    /// The number of tasks that were offered to the schedule.
    ///
    /// 提交给该计划的任务数量。
    pub offered_count: usize,
    /// The sum of the placed durations, in seconds.
    ///
    /// 已放置时长之和，单位为秒。
    pub scheduled_seconds: f64,
    /// The earliest start time, or zero when nothing is placed.
    ///
    /// 最早的开始时刻；无放置时为零。
    pub makespan_start: f64,
    /// The latest end time, or zero when nothing is placed.
    ///
    /// 最晚的结束时刻；无放置时为零。
    pub makespan_end: f64,
    /// The usage of each resource, in identifier order.
    ///
    /// 各资源的使用情况，按标识符顺序排列。
    pub resource_usage: Vec<ResourceUsage>,
}

impl Evaluation {
    /// Returns the length of the planning horizon the schedule covers, in
    /// seconds.
    ///
    /// This is the time from the earliest start to the latest end, which is
    /// the makespan of the plan.
    ///
    /// 返回该计划覆盖的规划时域长度，单位为秒。
    ///
    /// 它是从最早开始到最晚结束的时间，即该计划的 makespan。
    #[must_use]
    pub fn makespan(&self) -> f64 {
        (self.makespan_end - self.makespan_start).max(0.0)
    }

    /// Returns the fraction of the offered tasks that were placed.
    ///
    /// Returns `None` when no tasks were offered.
    ///
    /// 返回所提交任务中被放置的比例。
    ///
    /// 没有提交任务时返回 `None`。
    #[must_use]
    pub fn fill_rate(&self) -> Option<f64> {
        if self.offered_count == 0 {
            return None;
        }
        Some(self.placed_count as f64 / self.offered_count as f64)
    }

    /// Returns `true` when every offered task was placed.
    ///
    /// 当所有提交任务均被放置时返回 `true`。
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.placed_count == self.offered_count
    }

    /// Returns the usage of one resource.
    ///
    /// 返回某个资源的使用情况。
    #[must_use]
    pub fn usage_of(&self, resource: ResourceId) -> Option<&ResourceUsage> {
        self.resource_usage
            .iter()
            .find(|usage| usage.resource == resource)
    }
}
