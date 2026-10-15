//! The building blocks a greedy scheduler needs.
//!
//! 贪心调度器所需的基本构件。

use crate::apsis_resource::{ResourceId, ResourceSet};
use crate::apsis_schedule::Schedule;
use crate::apsis_task::{Task, TaskId};

/// Returns `true` when every resource of a task has the capacity to hold its
/// demand.
///
/// A task that demands more of a resource than the resource has can never be
/// scheduled on it, so this is checked before the search for a time. Returns
/// `false` when a required resource does not exist.
///
/// 当任务的每个资源都有容量容纳其需求时返回 `true`。
///
/// 需求超过某资源容量的任务永远无法在其上调度，故在搜索时刻之前先检查
/// 此点。所需资源不存在时返回 `false`。
#[must_use]
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
///
/// 给定一份计划及其任务集合，返回某瞬时各资源上的总需求。
///
/// 结果按资源在集合中的次序排列。不在集合中的资源被省略。
#[must_use]
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
///
/// 返回在某瞬空闲的资源标识符。
///
/// 资源空闲当且仅当其可用窗口覆盖该瞬，且其上已放置的需求还留有一个单
/// 位的余量。这正是水位调度器寻找有余量卫星时所作的查询。
#[must_use]
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
///
/// 当某任务在某时刻的放置可行时返回 `true`。
///
/// 检查内容为：每个所需资源在整个时长上可用，且其上已放置的需求加上
/// 新需求，在放置的每个瞬时都不超过容量。
#[must_use]
pub fn feasible_at(
    task: &Task,
    tasks: &[Task],
    schedule: &Schedule,
    resources: &ResourceSet,
    start: f64,
) -> bool {
    let index = crate::apsis_scheduler::sweep::demand_index(tasks);
    feasible_at_indexed(task, schedule, resources, start, &index)
}

/// Returns `true` when a task placed at a time is feasible, using a demand
/// index the caller built once.
///
/// The index replaces two linear scans of the task list: the one that finds a
/// placed task while collecting the boundaries, and the one inside that sums
/// the demand over every placement covering a boundary. The boundary
/// arithmetic is unchanged, so the answer is the same as the scans give.
///
/// 使用调用方已构建一次的需求索引，判断某任务在某时刻的放置是否可行。
///
/// 该索引取代了对任务列表的两次线性扫描：收集边界时查找已放置任务的那
/// 次，以及对每个覆盖该边界的放置求和需求的那次。边界运算本身不变，故
/// 结果与扫描给出的一致。
pub(crate) fn feasible_at_indexed(
    task: &Task,
    schedule: &Schedule,
    resources: &ResourceSet,
    start: f64,
    index: &crate::apsis_scheduler::sweep::DemandIndex,
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
        // 新任务的需求须与与其重叠的放置并存。由于每个放置的需求在其
        // 自身区间内恒定，只需检查重叠放置的边界，即总量最大之处。
        let mut boundaries: Vec<f64> = vec![start];
        for placement in schedule.placements() {
            if crate::apsis_scheduler::sweep::demand_on(index, placement.task, resource.id)
                .is_none()
            {
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
                .filter_map(|placement| {
                    crate::apsis_scheduler::sweep::demand_on(index, placement.task, resource.id)
                })
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
///
/// 返回某组资源完全无法容纳的任务标识符。
///
/// 任务不可行的情形有：需求超过某资源容量；所需资源不存在；窗口短于其
/// 时长。这些无法通过重排修复，值得与那些只是被排除的任务分开报告。
#[must_use]
pub fn infeasible_tasks(tasks: &[Task], resources: &ResourceSet) -> Vec<TaskId> {
    tasks
        .iter()
        .filter(|task| !task.is_valid() || !resources_have_capacity(task, resources))
        .map(|task| task.id)
        .collect()
}
