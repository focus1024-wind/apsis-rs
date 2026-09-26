//! The schedule itself.

use crate::error::{ScheduleError, ScheduleErrorKind};
use crate::placement::{Evaluation, Placement, ResourceUsage};
use apsis_resource::ResourceId;
use apsis_task::TaskId;
use std::collections::BTreeMap;

/// A set of task placements.
///
/// The schedule stores placements in the order they were made and keeps an
/// index from task to placement, so that a lookup by task is direct. A task
/// may be placed at most once: placing it again is an error rather than a
/// silent overwrite, because a schedule that contains two placements for one
/// task is not a schedule.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Schedule {
    placements: Vec<Placement>,
    index: BTreeMap<usize, usize>,
}

impl Schedule {
    /// Creates an empty schedule.
    pub fn new() -> Self {
        Self {
            placements: Vec::new(),
            index: BTreeMap::new(),
        }
    }

    /// Returns the placements in the order they were made.
    pub fn placements(&self) -> &[Placement] {
        &self.placements
    }

    /// Returns the number of placements.
    pub fn len(&self) -> usize {
        self.placements.len()
    }

    /// Returns `true` when there are no placements.
    pub fn is_empty(&self) -> bool {
        self.placements.is_empty()
    }

    /// Returns `true` when a task has been placed.
    pub fn is_placed(&self, task: TaskId) -> bool {
        self.index.contains_key(&task.index())
    }

    /// Returns the placement of a task.
    pub fn placement_of(&self, task: TaskId) -> Option<&Placement> {
        self.index
            .get(&task.index())
            .map(|position| &self.placements[*position])
    }

    /// Places a task at a time.
    ///
    /// The placement is checked against the task's window. The resources of
    /// the task are recorded with the placement through the task itself, so
    /// this method does not need them; the capacity check is performed by
    /// [`Schedule::validate_capacity`], which needs the resources.
    ///
    /// Returns an error when the task is already placed, when the start time
    /// is not finite, or when the placement falls outside the task's window.
    pub fn place(
        &mut self,
        task: &apsis_task::Task,
        start: f64,
    ) -> Result<&Placement, ScheduleError> {
        if !start.is_finite() {
            return Err(ScheduleError::new(ScheduleErrorKind::InvalidStart)
                .with_detail(format!("start {start}")));
        }
        if self.is_placed(task.id) {
            return Err(ScheduleError::new(ScheduleErrorKind::AlreadyPlaced)
                .with_detail(format!("{}", task.id)));
        }
        if start < task.earliest_start {
            return Err(
                ScheduleError::new(ScheduleErrorKind::BeforeWindow).with_detail(format!(
                    "task {} starts at {start}, before its window opens at {}",
                    task.id, task.earliest_start
                )),
            );
        }
        if start + task.duration > task.latest_end {
            return Err(
                ScheduleError::new(ScheduleErrorKind::AfterWindow).with_detail(format!(
                    "task {} ends at {}, after its window closes at {}",
                    task.id,
                    start + task.duration,
                    task.latest_end
                )),
            );
        }
        let placement = Placement::new(task.id, start, task.duration);
        self.index.insert(task.id.index(), self.placements.len());
        self.placements.push(placement);
        Ok(self
            .placements
            .last()
            .expect("the placement was just pushed"))
    }

    /// Removes the placement of a task.
    ///
    /// Returns an error when the task is not placed.
    pub fn unplace(&mut self, task: TaskId) -> Result<Placement, ScheduleError> {
        let position = self.index.remove(&task.index()).ok_or_else(|| {
            ScheduleError::new(ScheduleErrorKind::NotPlaced).with_detail(format!("{task}"))
        })?;
        let removed = self.placements.remove(position);
        // Every placement after the removed one shifts down by one, so the
        // index must be rebuilt. The schedule is small enough that this is
        // cheaper than maintaining a linked structure.
        self.rebuild_index();
        Ok(removed)
    }

    /// Rebuilds the task to position index.
    fn rebuild_index(&mut self) {
        self.index.clear();
        for (position, placement) in self.placements.iter().enumerate() {
            self.index.insert(placement.task.index(), position);
        }
    }

    /// Returns the placements that use a resource, in time order.
    pub fn placements_on(
        &self,
        resource: ResourceId,
        tasks: &[apsis_task::Task],
    ) -> Vec<Placement> {
        let mut selected: Vec<Placement> = self
            .placements
            .iter()
            .filter(|placement| {
                tasks
                    .iter()
                    .find(|task| task.id == placement.task)
                    .is_some_and(|task| task.requires(resource))
            })
            .copied()
            .collect();
        selected.sort_by(|first, second| {
            first
                .start
                .partial_cmp(&second.start)
                .unwrap_or(core::cmp::Ordering::Equal)
        });
        selected
    }

    /// Returns `true` when two placements of different tasks overlap.
    ///
    /// This checks the schedule against itself, ignoring the resources. Two
    /// tasks on disjoint resources may legitimately overlap, so a caller that
    /// wants to know whether a resource is overbooked should use
    /// [`Schedule::validate_capacity`] instead.
    pub fn has_overlapping_placements(&self) -> bool {
        for (index, placement) in self.placements.iter().enumerate() {
            if self.placements[..index]
                .iter()
                .any(|earlier| earlier.overlaps(placement))
            {
                return true;
            }
        }
        false
    }

    /// Returns the total placed duration, in seconds.
    pub fn scheduled_seconds(&self) -> f64 {
        self.placements
            .iter()
            .map(|placement| placement.duration)
            .sum()
    }

    /// Returns the earliest start of any placement.
    ///
    /// Returns `None` when the schedule is empty.
    pub fn earliest_start(&self) -> Option<f64> {
        self.placements
            .iter()
            .map(|placement| placement.start)
            .min_by(|first, second| {
                first
                    .partial_cmp(second)
                    .unwrap_or(core::cmp::Ordering::Equal)
            })
    }

    /// Returns the latest end of any placement.
    ///
    /// Returns `None` when the schedule is empty.
    pub fn latest_end(&self) -> Option<f64> {
        self.placements
            .iter()
            .map(Placement::end)
            .max_by(|first, second| {
                first
                    .partial_cmp(second)
                    .unwrap_or(core::cmp::Ordering::Equal)
            })
    }

    /// Validates a schedule against a set of resources.
    ///
    /// The check has two parts. First every placement must agree with the
    /// requirements of its task, which catches a schedule built with a
    /// different task list. Then, for each resource, the total demand placed
    /// at any instant must not exceed the capacity.
    ///
    /// The second check walks the placement boundaries of each resource in
    /// time order, which is exact because the total demand is constant
    /// between boundaries.
    ///
    /// Returns an error describing the first violation.
    pub fn validate_capacity(
        &self,
        tasks: &[apsis_task::Task],
        resources: &apsis_resource::ResourceSet,
    ) -> Result<(), ScheduleError> {
        for placement in &self.placements {
            let task = tasks
                .iter()
                .find(|task| task.id == placement.task)
                .ok_or_else(|| {
                    ScheduleError::new(ScheduleErrorKind::RequirementMismatch)
                        .with_detail(format!("{} is not in the task list", placement.task))
                })?;
            if (placement.duration - task.duration).abs() > 1e-12 {
                return Err(ScheduleError::new(ScheduleErrorKind::RequirementMismatch)
                    .with_detail(format!(
                        "{} is placed for {} s but takes {} s",
                        task.id, placement.duration, task.duration
                    )));
            }
        }

        for resource in resources.resources() {
            // The boundaries at which the total demand can change.
            let mut events: Vec<(f64, f64)> = Vec::new();
            for placement in &self.placements {
                let Some(task) = tasks.iter().find(|task| task.id == placement.task) else {
                    continue;
                };
                if let Some(demand) = task.demand_for(resource.id) {
                    events.push((placement.start, demand));
                    events.push((placement.end(), -demand));
                }
            }
            if events.is_empty() {
                continue;
            }
            // Sort by time, with a release before an acquisition at the same
            // instant so that two placements that touch are not counted as
            // overlapping.
            events.sort_by(|first, second| {
                first
                    .0
                    .partial_cmp(&second.0)
                    .unwrap_or(core::cmp::Ordering::Equal)
                    .then_with(|| {
                        first
                            .1
                            .partial_cmp(&second.1)
                            .unwrap_or(core::cmp::Ordering::Equal)
                    })
            });

            let mut current = 0.0;
            for (time, change) in events {
                current += change;
                if current > resource.capacity() + 1e-9 {
                    return Err(ScheduleError::new(ScheduleErrorKind::CapacityExceeded)
                        .with_detail(format!(
                            "{} carries demand {} against capacity {} at {time}",
                            resource.id,
                            current,
                            resource.capacity()
                        )));
                }
            }
        }
        Ok(())
    }

    /// Evaluates the schedule against a task list and a resource set.
    ///
    /// The evaluation reports the counts, the makespan, and the usage of each
    /// resource. The peak demand of a resource is found by the same event
    /// sweep that the capacity check uses, so the two agree.
    pub fn evaluate(
        &self,
        tasks: &[apsis_task::Task],
        resources: &apsis_resource::ResourceSet,
    ) -> Evaluation {
        let mut resource_usage = Vec::with_capacity(resources.len());
        for resource in resources.resources() {
            let mut events: Vec<(f64, f64)> = Vec::new();
            let mut placement_count = 0;
            for placement in &self.placements {
                let Some(task) = tasks.iter().find(|task| task.id == placement.task) else {
                    continue;
                };
                if let Some(demand) = task.demand_for(resource.id) {
                    events.push((placement.start, demand));
                    events.push((placement.end(), -demand));
                    placement_count += 1;
                }
            }
            events.sort_by(|first, second| {
                first
                    .0
                    .partial_cmp(&second.0)
                    .unwrap_or(core::cmp::Ordering::Equal)
                    .then_with(|| {
                        first
                            .1
                            .partial_cmp(&second.1)
                            .unwrap_or(core::cmp::Ordering::Equal)
                    })
            });

            let mut current = 0.0;
            let mut peak = 0.0_f64;
            let mut busy = 0.0_f64;
            let mut previous: Option<f64> = None;
            for (time, change) in events {
                if let Some(previous) = previous {
                    if current > 0.0 {
                        busy += time - previous;
                    }
                }
                current += change;
                peak = peak.max(current);
                previous = Some(time);
            }

            resource_usage.push(ResourceUsage {
                resource: resource.id,
                peak_demand: peak,
                busy_seconds: busy,
                placement_count,
            });
        }

        Evaluation {
            placed_count: self.placements.len(),
            offered_count: tasks.len(),
            scheduled_seconds: self.scheduled_seconds(),
            makespan_start: self.earliest_start().unwrap_or(0.0),
            makespan_end: self.latest_end().unwrap_or(0.0),
            resource_usage,
        }
    }

    /// Returns the schedule sorted by start time, leaving the original alone.
    ///
    /// A schedule is stored in the order it was built, which is the order a
    /// scheduler considered the tasks. A caller that wants to read the plan
    /// in time order uses this.
    pub fn in_time_order(&self) -> Vec<Placement> {
        let mut ordered = self.placements.clone();
        ordered.sort_by(|first, second| {
            first
                .start
                .partial_cmp(&second.start)
                .unwrap_or(core::cmp::Ordering::Equal)
                .then_with(|| first.task.cmp(&second.task))
        });
        ordered
    }

    /// Returns the time at which a resource becomes free if it is busy at a
    /// given instant.
    ///
    /// Returns `None` when the resource is idle there. This is what a
    /// sequential scheduler asks when it looks for the next gap.
    pub fn resource_busy_until(
        &self,
        resource: ResourceId,
        tasks: &[apsis_task::Task],
        time: f64,
    ) -> Option<f64> {
        self.placements
            .iter()
            .filter(|placement| {
                tasks
                    .iter()
                    .find(|task| task.id == placement.task)
                    .is_some_and(|task| task.requires(resource) && placement.covers(time))
            })
            .map(Placement::end)
            .max_by(|first, second| {
                first
                    .partial_cmp(second)
                    .unwrap_or(core::cmp::Ordering::Equal)
            })
    }
}
