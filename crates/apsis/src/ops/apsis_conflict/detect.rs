//! The conflict detection itself.
//!
//! 冲突检测本身。

use crate::apsis_conflict::conflict::{Conflict, ConflictKind, ConflictReport};
use crate::apsis_resource::{ResourceId, ResourceSet};
use crate::apsis_schedule::Schedule;
use crate::apsis_task::{Task, TaskId};

/// A precedence constraint: one task must finish before another starts.
///
/// 一条优先约束：一个任务必须先于另一个开始而完成。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Precedence {
    /// The task that must finish first.
    ///
    /// 必须先完成的任务。
    pub before: TaskId,
    /// The task that must start after.
    ///
    /// 必须后开始的任务。
    pub after: TaskId,
}

impl Precedence {
    /// Creates a precedence constraint.
    ///
    /// 创建一条优先约束。
    #[must_use]
    pub const fn new(before: TaskId, after: TaskId) -> Self {
        Self { before, after }
    }
}

/// Returns the placements of a task, which is at most one.
///
/// 返回某任务的放置，至多一处。
#[must_use]
pub fn placements_of(
    schedule: &Schedule,
    task: TaskId,
) -> Option<&crate::apsis_schedule::Placement> {
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
///
/// 返回计划中的所有冲突。
///
/// 检查是完整执行的：报告包含所有发现的问题，而非只有首个。这是有意
/// 的，因为拿到一份计划的规划器通常希望先知道它有多糟，再决定修复还是
/// 重建。
///
/// 检查项为：
///
/// - 每处放置的任务都在列表中，且时长一致；
/// - 每处放置都落在其任务窗口内；
/// - 任何瞬时没有资源超出其容量；
/// - 每处放置都落在其所要求的每个资源的可用窗口内。
///
/// 资源扫掠检查各放置的边界，因为资源上的总需求在相邻边界之间恒定，
/// 故该检查是精确的，耗时正比于放置数而非计划长度。
#[must_use]
pub fn detect(schedule: &Schedule, tasks: &[Task], resources: &ResourceSet) -> ConflictReport {
    let mut conflicts = Vec::new();

    // The requirement and window checks need only the placement and its
    // task.
    // 需求与窗口检查只需放置及其任务。
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
    // 可用性检查：每处放置必须落在其所要求的每个资源的某个窗口内。
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
            // 超过容量的需求永远无法满足，故属需求问题而非放置问题。
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
    // 容量检查，逐个资源进行。
    for resource in resources.resources() {
        // Each event is a time, a change in demand, and the task responsible,
        // so that a conflict can name the pair.
        // 每个事件含时刻、需求变化与责任任务，使冲突得以指明这一对。
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
        // 同一瞬先排释放后排获取，使两个相接的放置不计为重叠。
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
                // 报告首次把该资源推过量的一对，使冲突指明两个任务而
                // 不仅是最后到达的那个。
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
                // 每个资源只报告一次，取其首次超订的时刻。较晚时刻
                // 的第二次报告对一个修复轮毫无用处。
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
///
/// 返回一组约束中的优先冲突。
///
/// 当本应居次的任务在应居首者完成之前开始，即为优先冲突。若某约束的两
/// 个任务未被同时放置，则不构成冲突，因为对它们未作任何承诺。
///
/// 约束按给定形式检查，故必须无环。环是调用方自身模型中的矛盾，此处
/// 不予检测。
#[must_use]
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
