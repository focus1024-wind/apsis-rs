//! The earliest start search, done in one pass over the schedule.
//!
//! The sequential form asks the same question in a loop: it lists every time
//! at which feasibility could change, sorts them, and tests each in turn until
//! one passes. That is correct but it recomputes the busy profile of every
//! resource once per candidate, so a plan with many placements on one resource
//! spends quadratic work deciding each task and cubic work across the plan.
//!
//! The profile does not depend on the candidate. A placement is busy between
//! its start and its end whatever time the new task is proposed for, so the
//! intervals during which the resource has no room are the same for every
//! candidate. This module computes them once, subtracts them from the
//! availability windows, and reads the earliest surviving start off the front.
//! That is one sweep per resource and a linear merge, rather than a sweep per
//! candidate.
//!
//! The result is confirmed with [`feasible_at`] before it is returned, and the
//! candidate enumeration is kept as the fallback for the rare case where the
//! sweep and the direct test disagree because of floating point rounding at a
//! capacity boundary. The fast path is therefore an accelerator, never a
//! second definition of what is feasible.

use crate::greedy::{feasible_at_indexed, resources_have_capacity};
use apsis_resource::ResourceId;
use apsis_resource::{Resource, ResourceSet};
use apsis_schedule::Schedule;
use apsis_task::Task;
use apsis_task::TaskId;

/// The slack allowed when comparing a load against a capacity.
///
/// It matches the tolerance in [`feasible_at`], so the sweep and the direct
/// test draw the same line at a boundary.
const CAPACITY_SLACK: f64 = 1e-9;

/// Returns the earliest time at which a task can be placed.
///
/// The earliest feasible start is always one of a small set of candidate
/// times: the start of the task's window, the start of an availability
/// window of one of its resources, or the end of a placement already on one
/// of those resources. Feasibility cannot change at any other instant,
/// because the demand on every resource is constant between those events and
/// the availability is constant between them too.
///
/// Returns `None` when the task cannot be placed anywhere in its window.
pub fn earliest_feasible_start(
    task: &Task,
    tasks: &[Task],
    schedule: &Schedule,
    resources: &ResourceSet,
) -> Option<f64> {
    if !task.is_valid() || !resources_have_capacity(task, resources) {
        return None;
    }
    if task.requirements.is_empty() {
        // A task with no requirements can start as soon as its window opens.
        return Some(task.earliest_start);
    }
    let latest_start = task.latest_end - task.duration;

    // The demand of every task, looked up by identifier once. The sweep
    // visits every placement of every required resource, and reading the
    // demand by scanning the task list at each of those visits would make
    // deciding one task quadratic in the plan. The map is built once per
    // task, which is linear, and shared by every visit.
    let demands = demand_index(tasks);

    // The sweep, confirmed by the direct test.
    if let Some(candidate) = swept_earliest_start(task, schedule, resources, latest_start, &demands)
    {
        if feasible_at_indexed(task, schedule, resources, candidate, &demands) {
            return Some(candidate);
        }
    }

    // The enumeration, which is exact. It runs when the sweep could not
    // produce a confirmed answer, which is when the task does not fit at all
    // or when rounding made the two disagree.
    enumerate_earliest_start(task, schedule, resources, latest_start, &demands)
}

/// Returns the earliest start by testing every candidate in turn.
///
/// This is the original formulation, kept as the fallback for the sweep.
fn enumerate_earliest_start(
    task: &Task,
    schedule: &Schedule,
    resources: &ResourceSet,
    latest_start: f64,
    index: &DemandIndex,
) -> Option<f64> {
    let mut candidates: Vec<f64> = vec![task.earliest_start];
    for requirement in &task.requirements {
        let resource = resources.get(requirement.resource)?;
        for window in resource.windows() {
            candidates.push(window.start());
        }
        for placement in schedule.placements() {
            let overlaps = demand_on(index, placement.task, resource.id).is_some();
            if overlaps {
                candidates.push(placement.end());
            }
        }
    }

    candidates
        .retain(|time| time.is_finite() && *time >= task.earliest_start && *time <= latest_start);
    candidates.sort_by(|first, second| {
        first
            .partial_cmp(second)
            .unwrap_or(core::cmp::Ordering::Equal)
    });
    candidates.dedup();

    candidates
        .into_iter()
        .find(|start| feasible_at_indexed(task, schedule, resources, *start, index))
}

/// Returns the earliest start derived from the swept profile.
///
/// The intervals returned by [`starts_fitting_on`] for each resource are
/// intersected, and the first interval that lies inside the task's window
/// gives the answer.
fn swept_earliest_start(
    task: &Task,
    schedule: &Schedule,
    resources: &ResourceSet,
    latest_start: f64,
    demands: &DemandIndex,
) -> Option<f64> {
    let mut allowed = vec![(f64::NEG_INFINITY, f64::INFINITY)];
    for requirement in &task.requirements {
        let resource = resources.get(requirement.resource)?;
        let fitting = starts_fitting_on(
            task.duration,
            requirement.demand,
            requirement.resource,
            schedule,
            resource,
            demands,
        );
        allowed = intersect(&allowed, &fitting);
        if allowed.is_empty() {
            return None;
        }
    }
    let &(low, high) = allowed.first()?;
    let start = low.max(task.earliest_start);
    let end = high.min(latest_start);
    (start <= end).then_some(start)
}

/// Returns the intervals of start times at which a placement of the given
/// duration and demand fits on one resource.
///
/// A start is in the result when the whole placement lies inside one
/// availability window and the demand already placed on the resource never
/// exceeds the capacity less the new demand. The result is sorted and the
/// intervals are disjoint.
fn starts_fitting_on(
    duration: f64,
    demand: f64,
    resource_id: ResourceId,
    schedule: &Schedule,
    resource: &Resource,
    demands: &DemandIndex,
) -> Vec<(f64, f64)> {
    let blocked = blocked_intervals(demand, resource_id, schedule, resource, demands);
    let mut result = Vec::new();
    for window in resource.windows() {
        for (from, to) in subtract(&blocked, window.start(), window.end()) {
            // The placement has to fit inside, so the last start is the end
            // of the free interval less the duration.
            let last_start = to - duration;
            if last_start >= from {
                result.push((from, last_start));
            }
        }
    }
    result
}

/// Returns the intervals during which a resource has no room for a demand.
///
/// The intervals are where the demand already placed on the resource exceeds
/// the capacity less `demand`. They are computed with a single sweep over the
/// placement boundaries, and are sorted and disjoint.
fn blocked_intervals(
    demand: f64,
    resource_id: ResourceId,
    schedule: &Schedule,
    resource: &Resource,
    demands: &DemandIndex,
) -> Vec<(f64, f64)> {
    let spare = resource.capacity() - demand;

    // The load profile changes only where a placement starts or ends, so the
    // sweep needs only those two instants per placement.
    let mut events: Vec<(f64, f64)> = Vec::new();
    for placement in schedule.placements() {
        let Some(placed) = demand_on(demands, placement.task, resource_id) else {
            continue;
        };
        events.push((placement.start, placed));
        events.push((placement.end(), -placed));
    }
    if events.is_empty() {
        return Vec::new();
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

    let limit = spare + CAPACITY_SLACK;
    let mut blocked: Vec<(f64, f64)> = Vec::new();
    let mut load = 0.0;
    let mut opened: Option<f64> = None;
    let mut index = 0;
    while index < events.len() {
        let time = events[index].0;
        // The load is constant on the interval that ends at this instant.
        if load > limit {
            opened.get_or_insert(time);
        }
        // Apply every change at this instant at once, so the load compared
        // above belongs to the interval before it and the one after belongs
        // to the next comparison.
        let mut next = index;
        while next < events.len() && events[next].0 == time {
            load += events[next].1;
            next += 1;
        }
        if load <= limit {
            if let Some(start) = opened.take() {
                blocked.push((start, time));
            }
        }
        index = next;
    }
    // A run that never closes extends past every placement, so it is
    // recorded as open ended and swallows the rest of the horizon.
    if let Some(start) = opened {
        blocked.push((start, f64::INFINITY));
    }
    blocked
}

/// The demand each task puts on each resource, by task identifier.
///
/// A task identifier is a small dense integer, so the demands are held in a
/// vector indexed by it rather than in a map. Reading one is then an array
/// index rather than a hash, and a hash of a small integer costs more than
/// the scan it replaces.
pub(crate) type DemandIndex = Vec<Option<std::collections::HashMap<ResourceId, f64>>>;

/// Builds a [`DemandIndex`] in one pass over the task list.
pub(crate) fn demand_index(tasks: &[Task]) -> DemandIndex {
    let highest = tasks.iter().map(|task| task.id.index()).max().unwrap_or(0);
    let mut index: DemandIndex = (0..=highest).map(|_| None).collect();
    for task in tasks {
        if task.requirements.is_empty() {
            continue;
        }
        let demands: std::collections::HashMap<ResourceId, f64> = task
            .requirements
            .iter()
            .map(|requirement| (requirement.resource, requirement.demand))
            .collect();
        index[task.id.index()] = Some(demands);
    }
    index
}

/// Returns the demand a placed task puts on a resource, if any.
pub(crate) fn demand_on(index: &DemandIndex, task: TaskId, resource: ResourceId) -> Option<f64> {
    index
        .get(task.index())
        .and_then(Option::as_ref)
        .and_then(|required| required.get(&resource))
        .copied()
}

/// Returns the pieces of an interval that no blocked interval covers.
///
/// Both inputs are sorted and disjoint. The result is sorted and disjoint.
fn subtract(blocked: &[(f64, f64)], from: f64, to: f64) -> Vec<(f64, f64)> {
    let mut pieces = Vec::new();
    let mut cursor = from;
    for &(start, end) in blocked {
        if end <= cursor {
            continue;
        }
        if start >= to {
            break;
        }
        if start > cursor {
            pieces.push((cursor, start.min(to)));
        }
        cursor = cursor.max(end);
        if cursor >= to {
            break;
        }
    }
    if cursor < to {
        pieces.push((cursor, to));
    }
    pieces
}

/// Returns the intersection of two sorted, disjoint lists of intervals.
///
/// The result is sorted and disjoint.
fn intersect(left: &[(f64, f64)], right: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let mut result = Vec::new();
    let mut i = 0;
    let mut j = 0;
    while i < left.len() && j < right.len() {
        let start = left[i].0.max(right[j].0);
        let end = left[i].1.min(right[j].1);
        if start < end {
            result.push((start, end));
        }
        if left[i].1 < right[j].1 {
            i += 1;
        } else {
            j += 1;
        }
    }
    result
}
