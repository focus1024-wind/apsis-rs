//! The conflict detection itself.

use crate::conflict::{Conflict, ConflictKind, ConflictReport};
use apsis_resource::{ResourceId, ResourceSet};
use apsis_schedule::Schedule;
use apsis_task::{Task, TaskId};

/// A precedence constraint: one task must finish before another starts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Precedence {
    /// The task that must finish first.
    pub before: TaskId,
    /// The task that must start after.
    pub after: TaskId,
}

impl Precedence {
    /// Creates a precedence constraint.
    pub const fn new(before: TaskId, after: TaskId) -> Self {
        Self { before, after }
    }
}

/// Returns the placements of a task, which is at most one.
pub fn placements_of(schedule: &Schedule, task: TaskId) -> Option<&apsis_schedule::Placement> {
    schedule.placement_of(task)
}

/// Returns every conflict in a schedule.
///
/// The checks are performed in full: the report contains every problem found,
/// not just the first. That is deliberate, because a planner that is handed a
/// plan usually wants to know how bad it is before deciding whether to repair
/// or rebuild.
///
/// The checks are:
///
/// - every placement's task exists in the list, and the durations agree;
/// - every placement falls inside its task's window;
/// - no resource exceeds its capacity at any instant;
/// - every placement falls inside the availability windows of each resource
///   it requires.
///
/// The resource sweep examines the boundaries of the placements, because the
/// total demand on a resource is constant between them, so the check is exact
/// and takes time proportional to the number of placements rather than to the
/// length of the plan.
pub fn detect(schedule: &Schedule, tasks: &[Task], resources: &ResourceSet) -> ConflictReport {
    let mut conflicts = Vec::new();

    // The requirement and window checks need only the placement and its
    // task.
    for placement in schedule.placements() {
        let task = tasks.iter().find(|task| task.id == placement.task);
        let Some(task) = task else {
            conflicts.push(
                Conflict::new(
                    ConflictKind::Requirement,
                    placement.task,
                    placement.start,
                    "the task is not in the task list",
                )
                .with_resource(ResourceId::new(0)),
            );
            continue;
        };

        if (placement.duration - task.duration).abs() > 1e-12 {
            conflicts.push(Conflict::new(
                ConflictKind::Requirement,
                placement.task,
                placement.start,
                format!(
                    "placed for {} s but the task lasts {} s",
                    placement.duration, task.duration
                ),
            ));
        }

        if placement.start < task.earliest_start {
            conflicts.push(Conflict::new(
                ConflictKind::Window,
                placement.task,
                placement.start,
                format!(
                    "starts at {}, before the window opens at {}",
                    placement.start, task.earliest_start
                ),
            ));
        }
        if placement.end() > task.latest_end {
            conflicts.push(Conflict::new(
                ConflictKind::Window,
                placement.task,
                placement.end(),
                format!(
                    "ends at {}, after the window closes at {}",
                    placement.end(),
                    task.latest_end
                ),
            ));
        }
    }

    // The availability check: a placement must fall inside a window of every
    // resource it requires.
    for placement in schedule.placements() {
        let Some(task) = tasks.iter().find(|task| task.id == placement.task) else {
            continue;
        };
        for requirement in &task.requirements {
            let Some(resource) = resources.get(requirement.resource) else {
                conflicts.push(
                    Conflict::new(
                        ConflictKind::Requirement,
                        placement.task,
                        placement.start,
                        format!("{} does not exist", requirement.resource),
                    )
                    .with_resource(requirement.resource),
                );
                continue;
            };
            // A demand greater than the capacity can never be met, which is a
            // requirement problem rather than a placement problem.
            if requirement.demand > resource.capacity() + 1e-9 {
                conflicts.push(
                    Conflict::new(
                        ConflictKind::Requirement,
                        placement.task,
                        placement.start,
                        format!(
                            "demand {} exceeds the capacity {} of {}",
                            requirement.demand,
                            resource.capacity(),
                            resource.id
                        ),
                    )
                    .with_resource(requirement.resource),
                );
                continue;
            }
            if !resource.is_available_over(placement.start, placement.duration) {
                conflicts.push(
                    Conflict::new(
                        ConflictKind::Availability,
                        placement.task,
                        placement.start,
                        format!(
                            "{} is not available from {} for {} s",
                            resource.id, placement.start, placement.duration
                        ),
                    )
                    .with_resource(requirement.resource),
                );
            }
        }
    }

    // The capacity check, one resource at a time.
    for resource in resources.resources() {
        // Each event is a time, a change in demand, and the task responsible,
        // so that a conflict can name the pair.
        let mut events: Vec<(f64, f64, TaskId)> = Vec::new();
        for placement in schedule.placements() {
            let Some(task) = tasks.iter().find(|task| task.id == placement.task) else {
                continue;
            };
            if let Some(demand) = task.demand_for(resource.id) {
                events.push((placement.start, demand, placement.task));
                events.push((placement.end(), -demand, placement.task));
            }
        }
        if events.is_empty() {
            continue;
        }
        // A release sorts before an acquisition at the same instant, so two
        // placements that touch are not counted as overlapping.
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
                .then_with(|| first.2.cmp(&second.2))
        });

        let mut current = 0.0;
        let mut holders: Vec<TaskId> = Vec::new();
        for (time, change, task) in events {
            if change > 0.0 {
                holders.push(task);
            } else {
                holders.retain(|holder| *holder != task);
            }
            current += change;
            if current > resource.capacity() + 1e-9 {
                // Report the pair that first pushes the resource over, so
                // that the conflict names two tasks rather than only the one
                // that arrived last.
                let mut involved = holders.clone();
                involved.sort();
                let first = involved.first().copied();
                let second = involved.get(1).copied();
                let detail = format!(
                    "demand {} exceeds the capacity {} of {}",
                    current,
                    resource.capacity(),
                    resource.id
                );
                let mut conflict =
                    Conflict::new(ConflictKind::Capacity, first.unwrap_or(task), time, detail)
                        .with_resource(resource.id);
                if let Some(second) = second {
                    conflict = conflict.with_other(second);
                }
                conflicts.push(conflict);
                // Report each resource once, at the first instant it is over
                // committed. A second report at a later instant would add
                // nothing a repair pass can use.
                break;
            }
        }
    }

    ConflictReport::new(conflicts, tasks.len(), schedule.len())
}

/// Returns the precedence conflicts among a set of constraints.
///
/// A precedence conflict occurs when the task that must come second starts
/// before the one that must come first has finished. A constraint whose
/// tasks are not both placed is not a conflict, because nothing has been
/// promised about them.
///
/// The constraints are checked as given, so they must be acyclic. A cycle is
/// a contradiction in the caller's own model and is not detected here.
pub fn detect_precedence(schedule: &Schedule, constraints: &[Precedence]) -> Vec<Conflict> {
    let mut conflicts = Vec::new();
    for constraint in constraints {
        let (Some(before), Some(after)) = (
            schedule.placement_of(constraint.before),
            schedule.placement_of(constraint.after),
        ) else {
            continue;
        };
        if after.start < before.end() {
            conflicts.push(
                Conflict::new(
                    ConflictKind::Precedence,
                    constraint.after,
                    after.start,
                    format!(
                        "{} must follow {} but starts at {} before it ends at {}",
                        constraint.after,
                        constraint.before,
                        after.start,
                        before.end()
                    ),
                )
                .with_other(constraint.before),
            );
        }
    }
    conflicts
}
