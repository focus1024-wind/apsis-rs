//! The building blocks a greedy scheduler needs.

use apsis_resource::{ResourceId, ResourceSet};
use apsis_schedule::Schedule;
use apsis_task::{Task, TaskId};

/// Returns `true` when every resource of a task has the capacity to hold its
/// demand.
///
/// A task that demands more of a resource than the resource has can never be
/// scheduled on it, so this is checked before the search for a time. Returns
/// `false` when a required resource does not exist.
pub fn resources_have_capacity(task: &Task, resources: &ResourceSet) -> bool {
    task.requirements.iter().all(|requirement| {
        resources
            .get(requirement.resource)
            .is_some_and(|resource| requirement.demand <= resource.capacity() + 1e-9)
    })
}

/// Returns the total demand placed on each resource at an instant, given a
/// schedule and the tasks it contains.
///
/// The result is in the order the resources appear in the set. A resource
/// that is not in the set is omitted.
pub fn demand_at(
    schedule: &Schedule,
    tasks: &[Task],
    resources: &ResourceSet,
    time: f64,
) -> Vec<(ResourceId, f64)> {
    resources
        .resources()
        .iter()
        .map(|resource| {
            let total: f64 = schedule
                .placements()
                .iter()
                .filter(|placement| placement.covers(time))
                .filter_map(|placement| {
                    tasks
                        .iter()
                        .find(|task| task.id == placement.task)
                        .and_then(|task| task.demand_for(resource.id))
                })
                .sum();
            (resource.id, total)
        })
        .collect()
}

/// Returns the identifiers of the resources that are free at an instant.
///
/// A resource is free when its availability windows cover the instant and
/// the demand already placed on it leaves room for one more unit. This is
/// the query a watermark scheduler makes when it looks for a satellite with
/// spare capacity.
pub fn resources_free_at(
    schedule: &Schedule,
    tasks: &[Task],
    resources: &ResourceSet,
    time: f64,
) -> Vec<ResourceId> {
    resources
        .resources()
        .iter()
        .filter(|resource| resource.is_available_at(time))
        .filter(|resource| {
            let placed: f64 = schedule
                .placements()
                .iter()
                .filter(|placement| placement.covers(time))
                .filter_map(|placement| {
                    tasks
                        .iter()
                        .find(|task| task.id == placement.task)
                        .and_then(|task| task.demand_for(resource.id))
                })
                .sum();
            placed < resource.capacity() - 1e-9
        })
        .map(|resource| resource.id)
        .collect()
}

/// Returns `true` when a task placed at a time is feasible.
///
/// The check is that every required resource is available for the whole
/// duration and that the demand already placed on it, plus the new demand,
/// stays within the capacity at every instant of the placement.
pub fn feasible_at(
    task: &Task,
    tasks: &[Task],
    schedule: &Schedule,
    resources: &ResourceSet,
    start: f64,
) -> bool {
    let index = crate::sweep::demand_index(tasks);
    feasible_at_indexed(task, schedule, resources, start, &index)
}

/// Returns `true` when a task placed at a time is feasible, using a demand
/// index the caller built once.
///
/// The index replaces two linear scans of the task list: the one that finds a
/// placed task while collecting the boundaries, and the one inside that sums
/// the demand over every placement covering a boundary. The boundary
/// arithmetic is unchanged, so the answer is the same as the scans give.
pub(crate) fn feasible_at_indexed(
    task: &Task,
    schedule: &Schedule,
    resources: &ResourceSet,
    start: f64,
    index: &crate::sweep::DemandIndex,
) -> bool {
    if !start.is_finite() {
        return false;
    }
    let end = start + task.duration;
    for requirement in &task.requirements {
        let Some(resource) = resources.get(requirement.resource) else {
            return false;
        };
        if !resource.is_available_over(start, task.duration) {
            return false;
        }
        // The demand of the new task must fit alongside the placements that
        // overlap it. Because the demand of each placement is constant over
        // its own interval, it is enough to test the boundaries of the
        // overlapping placements, where the total is greatest.
        let mut boundaries: Vec<f64> = vec![start];
        for placement in schedule.placements() {
            if crate::sweep::demand_on(index, placement.task, resource.id).is_none() {
                continue;
            }
            if placement.start < end && start < placement.end() {
                boundaries.push(placement.start);
            }
        }
        for boundary in boundaries {
            let placed: f64 = schedule
                .placements()
                .iter()
                .filter(|placement| placement.covers(boundary))
                .filter_map(|placement| crate::sweep::demand_on(index, placement.task, resource.id))
                .sum();
            if placed + requirement.demand > resource.capacity() + 1e-9 {
                return false;
            }
        }
    }
    true
}

/// Returns the identifiers of the tasks that a set of resources cannot
/// accommodate at all.
///
/// A task is infeasible when it demands more of a resource than the resource
/// has, when a resource it requires does not exist, or when its window is
/// shorter than its duration. These cannot be repaired by rescheduling and
/// are worth reporting separately from the tasks that were merely left out.
pub fn infeasible_tasks(tasks: &[Task], resources: &ResourceSet) -> Vec<TaskId> {
    tasks
        .iter()
        .filter(|task| !task.is_valid() || !resources_have_capacity(task, resources))
        .map(|task| task.id)
        .collect()
}
