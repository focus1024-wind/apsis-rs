//! Placements and the evaluation of a schedule.

use apsis_resource::ResourceId;
use apsis_task::TaskId;

/// One task placed on the schedule.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    /// The task placed.
    pub task: TaskId,
    /// The start time of the placement, in seconds.
    pub start: f64,
    /// The duration of the placement, in seconds.
    ///
    /// This is copied from the task so that a schedule carries everything
    /// needed to evaluate it without the tasks to hand.
    pub duration: f64,
}

impl Placement {
    /// Creates a placement.
    pub const fn new(task: TaskId, start: f64, duration: f64) -> Self {
        Self {
            task,
            start,
            duration,
        }
    }

    /// Returns the end time of the placement, in seconds.
    pub fn end(&self) -> f64 {
        self.start + self.duration
    }

    /// Returns `true` when the placement covers an instant.
    ///
    /// The interval is half-open, so a placement covers its start and not its
    /// end. That convention lets two back to back placements share a boundary
    /// without overlapping.
    pub fn covers(&self, time: f64) -> bool {
        time >= self.start && time < self.end()
    }

    /// Returns `true` when the placement overlaps another.
    pub fn overlaps(&self, other: &Self) -> bool {
        self.start < other.end() && other.start < self.end()
    }

    /// Returns `true` when the placement is usable.
    pub fn is_valid(&self) -> bool {
        self.start.is_finite() && self.duration.is_finite() && self.duration >= 0.0
    }
}

/// How much of one resource a schedule uses.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResourceUsage {
    /// The resource.
    pub resource: ResourceId,
    /// The greatest total demand placed on the resource at any instant.
    pub peak_demand: f64,
    /// The time during which the resource is used at all, in seconds.
    pub busy_seconds: f64,
    /// The number of placements that use the resource.
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
    pub fn utilisation(&self, interval_seconds: f64) -> Option<f64> {
        if interval_seconds > 0.0 && interval_seconds.is_finite() {
            Some(self.busy_seconds / interval_seconds)
        } else {
            None
        }
    }
}

/// The evaluation of a whole schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct Evaluation {
    /// The number of placements.
    pub placed_count: usize,
    /// The number of tasks that were offered to the schedule.
    pub offered_count: usize,
    /// The sum of the placed durations, in seconds.
    pub scheduled_seconds: f64,
    /// The earliest start time, or zero when nothing is placed.
    pub makespan_start: f64,
    /// The latest end time, or zero when nothing is placed.
    pub makespan_end: f64,
    /// The usage of each resource, in identifier order.
    pub resource_usage: Vec<ResourceUsage>,
}

impl Evaluation {
    /// Returns the length of the planning horizon the schedule covers, in
    /// seconds.
    ///
    /// This is the time from the earliest start to the latest end, which is
    /// the makespan of the plan.
    pub fn makespan(&self) -> f64 {
        (self.makespan_end - self.makespan_start).max(0.0)
    }

    /// Returns the fraction of the offered tasks that were placed.
    ///
    /// Returns `None` when no tasks were offered.
    pub fn fill_rate(&self) -> Option<f64> {
        if self.offered_count == 0 {
            return None;
        }
        Some(self.placed_count as f64 / self.offered_count as f64)
    }

    /// Returns `true` when every offered task was placed.
    pub fn is_complete(&self) -> bool {
        self.placed_count == self.offered_count
    }

    /// Returns the usage of one resource.
    pub fn usage_of(&self, resource: ResourceId) -> Option<&ResourceUsage> {
        self.resource_usage
            .iter()
            .find(|usage| usage.resource == resource)
    }
}
