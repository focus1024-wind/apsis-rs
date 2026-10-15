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
//!
//! 最早开始搜索，对调度做一遍完成。
//!
//! 顺序形式在循环中问同一个问题：它列出可行性可能改变的每个时刻，排序，并逐个
//! 检验直到有一个通过。这正确，但它对每个候选都重算每个资源的繁忙廓线，故在一
//! 资源上有很多放置的计划，判定每个任务花费二次工作，整个计划花费三次工作。
//!
//! 廓线并不依赖于候选。一处放置在它的开始与结束之间繁忙，无论新任务被提议在什么
//! 时间，故该资源没有空档的各区间对每个候选都相同。本模块把它们计算一次，从可用性
//! 窗口中减去，并从最前读出所存活的最早开始。这是每个资源一次扫描加一次线性合并，
//! 而不是每个候选一次扫描。
//!
//! 结果在返回前用 [`feasible_at`] 确认，而候选枚举被保留为回退，用于扫描与直接判定
//! 因容量边界处的浮点舍入而不一致的罕见情形。故该快路径是加速器，绝不是可行性的
//! 第二种定义。

use crate::apsis_resource::ResourceId;
use crate::apsis_resource::{Resource, ResourceSet};
use crate::apsis_schedule::Schedule;
use crate::apsis_scheduler::greedy::{feasible_at_indexed, resources_have_capacity};
use crate::apsis_task::Task;
use crate::apsis_task::TaskId;

/// The slack allowed when comparing a load against a capacity.
///
/// It matches the tolerance in [`feasible_at`], so the sweep and the direct
/// test draw the same line at a boundary.
///
/// 比较负载与容量时允许的余量。
///
/// 它与 [`feasible_at`] 中的容差一致，故扫掠与直接测试在边界处划同一条
/// 线。
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
///
/// 返回某任务可被放置的最早时刻。
///
/// 最早可行起点总是某个小候选集合之一：任务窗口的起点、其某资源可用窗口
/// 的起点，或其某资源上已有放置的结束时刻。可行性在其他任何瞬时都不会
/// 改变，因为各资源的需求在这些事件之间恒定，可用性亦然。
///
/// 该任务在其窗口内无处可放时返回 `None`。
#[must_use]
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
        // 无需求的任务可在其窗口开启时即开始。
        return Some(task.earliest_start);
    }
    let latest_start = task.latest_end - task.duration;

    // The demand of every task, looked up by identifier once. The sweep
    // visits every placement of every required resource, and reading the
    // demand by scanning the task list at each of those visits would make
    // deciding one task quadratic in the plan. The map is built once per
    // task, which is linear, and shared by every visit.
    // 各任务的需求，按标识符一次性建好索引。扫掠会访问每个所需资源上
    // 的每处放置，若每次访问都扫描任务列表来读需求，判定一个任务就会
    // 对整个计划呈二次。该索引每个任务构建一次，是线性的，并由所有
    // 访问共享。
    let demands = demand_index(tasks);

    // The sweep, confirmed by the direct test.
    // 扫掠，其结果由直接测试确认。
    if let Some(candidate) = swept_earliest_start(task, schedule, resources, latest_start, &demands)
    {
        if feasible_at_indexed(task, schedule, resources, candidate, &demands) {
            return Some(candidate);
        }
    }

    // The enumeration, which is exact. It runs when the sweep could not
    // produce a confirmed answer, which is when the task does not fit at all
    // or when rounding made the two disagree.
    // 枚举，它是精确的。当扫掠无法给出经确认的答案时运行，即该任务
    // 完全放不下，或舍入使二者不一致时。
    enumerate_earliest_start(task, schedule, resources, latest_start, &demands)
}

/// Returns the earliest start by testing every candidate in turn.
///
/// This is the original formulation, kept as the fallback for the sweep.
///
/// 依次测试每个候选，返回最早起点。
///
/// 这是最初的表述，保留为扫掠的回退。
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
///
/// 由扫掠所得的负载曲线导出最早起点。
///
/// [`starts_fitting_on`] 为各资源返回的区间被求交，落在该任务窗口内的
/// 第一个区间即给出答案。
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
///
/// 返回可使给定时长与需求的放置落在某资源上的起点区间。
///
/// 当整个放置位于某个可用窗口内、且资源上已放置的需求始终不超过容量减去
/// 新需求时，该起点被计入结果。结果有序且各区间互不相交。
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
            // 放置须容纳于其中，故最晚起点为该空闲区间的终点减去时长。
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
///
/// 返回某资源无余量容纳某需求的区间。
///
/// 这些区间即资源上已放置的需求超过容量减去 `demand` 的时段。它们由
/// 对各放置边界的一次扫掠算出，有序且互不相交。
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
    // 负载曲线只在某处放置开始或结束时改变，故扫掠每个放置只需那两个
    // 瞬时。
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
        // 负载在以该瞬时结束的区间上恒定。
        if load > limit {
            opened.get_or_insert(time);
        }
        // Apply every change at this instant at once, so the load compared
        // above belongs to the interval before it and the one after belongs
        // to the next comparison.
        // 一次性施加该瞬时的全部变化，故上面比较的负载属于其前的区间，
        // 其后的负载则用于下一次比较。
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
    // 始终不闭合的区段延伸越过所有放置，故记为开区间，吞没时域的
    // 其余部分。
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
///
/// 各任务在各资源上的需求，按任务标识符索引。
///
/// 任务标识符是小的稠密整数，故需求存于以其为下标的向量而非映射中。读取
/// 时即数组下标而非哈希，而哈希一个小整数的代价高于它所取代的扫描。
pub(crate) type DemandIndex = Vec<Option<std::collections::HashMap<ResourceId, f64>>>;

/// Builds a [`DemandIndex`] in one pass over the task list.
///
/// 由任务列表一趟构建 [`DemandIndex`]。
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
///
/// 返回某已放置任务在某资源上的需求，若有。
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
///
/// 返回某区间中未被任何阻塞区间覆盖的部分。
///
/// 两个输入均有序且互不相交，结果亦然。
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
///
/// 返回两份有序且互不相交的区间列表之交。
///
/// 结果有序且互不相交。
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
