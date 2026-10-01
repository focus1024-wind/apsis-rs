//! A second pass that tries to place what the greedy first pass left out.
//!
//! 第二遍，尝试放置贪心第一遍所遗漏的。
//!
//! The greedy pass places each task at the earliest time it fits and never
//! moves a placement once it is made, so a task that does not fit is lost
//! even when moving a placement would make room. This pass addresses the
//! cheapest case of that problem.
//!
//! For a task that was left out, the pass looks at the placements that block
//! it, that is the placements sharing a resource it needs. For each blocker
//! it tries a relocation of the pair:
//!
//! 1. Record where the blocker sits, then remove it.
//! 2. Find the earliest time at which the waiting task fits without it.
//! 3. Place the waiting task there, then ask whether the blocker fits
//!    somewhere else now that it has a new neighbour.
//! 4. Keep the new arrangement when both fit, and undo it otherwise.
//!
//! The check in step 4 is the whole safety argument: the pair is kept only
//! when both placements pass the same feasibility test the greedy pass uses,
//! so every accepted relocation leaves the schedule feasible.
//!
//! The pass is bounded. Each relocated pair costs two feasibility searches,
//! and the pair is attempted only a bounded number of times, so the pass
//! stays linear in the number of tasks. It is still a heuristic: scheduling
//! with resource constraints is intractable, so the pass reduces the loss
//! rather than removing it, and the caller continues to see what was left
//! out.

use crate::sweep::earliest_feasible_start;
use apsis_resource::ResourceSet;
use apsis_schedule::Schedule;
use apsis_task::{Task, TaskId, TaskState};

/// The settings of the improvement pass.
///
/// 改进轮的设置。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImprovementSettings {
    /// Whether the pass runs.
    ///
    /// 该轮是否运行。
    pub enabled: bool,
    /// The greatest number of blocking placements to try per task.
    ///
    /// Each attempt costs two feasibility searches. A small bound keeps the
    /// pass predictable; a larger one finds more placements on a crowded
    /// schedule at proportional cost.
    ///
    /// 每个任务最多尝试的阻塞放置数。
    ///
    /// 每次尝试耗费两次可行性搜索。较小的界使该轮表现可预期；较大的界
    /// 能在拥挤的计划上找到更多放置，代价相应增加。
    pub max_attempts_per_task: usize,
}

impl Default for ImprovementSettings {
    /// Returns settings that run the pass with four attempts per task.
    ///
    /// 返回每任务尝试四次的设置。
    fn default() -> Self {
        Self {
            enabled: true,
            max_attempts_per_task: 4,
        }
    }
}

impl ImprovementSettings {
    /// Creates settings with the given bound.
    ///
    /// 以给定界创建设置。
    pub const fn new(max_attempts_per_task: usize) -> Self {
        Self {
            enabled: true,
            max_attempts_per_task,
        }
    }

    /// Returns settings that do not run the pass.
    ///
    /// 返回不运行该轮的设置。
    pub const fn disabled() -> Self {
        Self {
            enabled: false,
            max_attempts_per_task: 0,
        }
    }
}

/// What the improvement pass changed.
///
/// 改进轮所作的改变。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ImprovementOutcome {
    /// The identifiers of the tasks the pass placed.
    ///
    /// 该轮所放置的任务标识符。
    pub placed: Vec<TaskId>,
    /// The identifiers of the tasks the pass moved to make room.
    ///
    /// 该轮为腾出空隙而移动的任务标识符。
    pub moved: Vec<TaskId>,
}

impl ImprovementOutcome {
    /// Returns the number of tasks the pass placed.
    ///
    /// 返回该轮所放置的任务数。
    pub fn placed_count(&self) -> usize {
        self.placed.len()
    }

    /// Returns `true` when the pass changed nothing.
    ///
    /// 当该轮未作任何改变时返回 `true`。
    pub fn is_empty(&self) -> bool {
        self.placed.is_empty() && self.moved.is_empty()
    }
}

/// Tries to place deferred tasks by relocating the placements that block them.
///
/// `deferred` are the tasks the greedy pass could not place, in the order to
/// consider them. `states` holds the state of every task indexed by
/// identifier and is updated in place, so the caller sees the final
/// assignment. The schedule is modified in place.
///
/// Returns a record of what the pass placed and moved.
///
/// 通过搬迁阻塞它们的放置，尝试安排被推迟的任务。
///
/// `deferred` 是贪心轮无法放置的任务，按考虑顺序排列。`states` 按标识符
/// 索引保存每个任务的状态并就地更新，故调用方看到的是最终指派。该计划
/// 亦被就地修改。
///
/// 返回记录该轮所放置与所移动内容的报告。
pub fn improve(
    tasks: &[Task],
    resources: &ResourceSet,
    deferred: &[TaskId],
    schedule: &mut Schedule,
    states: &mut [TaskState],
    settings: ImprovementSettings,
) -> ImprovementOutcome {
    let mut outcome = ImprovementOutcome::default();
    if !settings.enabled || settings.max_attempts_per_task == 0 {
        return outcome;
    }

    for id in deferred {
        let Some(waiting) = tasks.iter().find(|task| task.id == *id) else {
            continue;
        };
        // A task that is not deferred has been placed by an earlier step.
        // 非被推迟的任务已由先前步骤放置。
        if schedule.is_placed(*id) {
            continue;
        }

        for blocker_id in blocking_tasks(waiting, tasks, schedule)
            .into_iter()
            .take(settings.max_attempts_per_task)
        {
            let Some(blocker) = tasks.iter().find(|task| task.id == blocker_id) else {
                continue;
            };
            if relocate_pair(waiting, blocker, tasks, schedule, resources) {
                let start = schedule
                    .placement_of(*id)
                    .map_or(0.0, |placement| placement.start);
                set_state(states, *id, TaskState::Scheduled { start });
                let blocker_start = schedule
                    .placement_of(blocker_id)
                    .map_or(0.0, |placement| placement.start);
                set_state(
                    states,
                    blocker_id,
                    TaskState::Scheduled {
                        start: blocker_start,
                    },
                );
                outcome.placed.push(*id);
                outcome.moved.push(blocker_id);
                break;
            }
        }
    }

    outcome.placed.sort();
    outcome.placed.dedup();
    outcome.moved.sort();
    outcome.moved.dedup();
    outcome
}

/// Returns the placed tasks that share a resource with a waiting task, in
/// time order.
///
/// Only these can block it: a placement on a resource the task does not need
/// consumes capacity that the task was never going to use.
///
/// 返回与某等待任务共用资源的已放置任务，按时间排序。
///
/// 只有这些能阻塞它：某个该任务并不需要的资源上的放置，消耗的是它本就
/// 不会用到的容量。
fn blocking_tasks(waiting: &Task, tasks: &[Task], schedule: &Schedule) -> Vec<TaskId> {
    let mut blockers: Vec<(f64, TaskId)> = schedule
        .placements()
        .iter()
        .filter(|placement| {
            tasks
                .iter()
                .find(|other| other.id == placement.task)
                .is_some_and(|other| {
                    waiting
                        .requirements
                        .iter()
                        .any(|requirement| other.demand_for(requirement.resource).is_some())
                })
        })
        .map(|placement| (placement.start, placement.task))
        .collect();
    blockers.sort_by(|first, second| {
        first
            .0
            .partial_cmp(&second.0)
            .unwrap_or(core::cmp::Ordering::Equal)
            .then_with(|| first.1.cmp(&second.1))
    });
    blockers.into_iter().map(|(_, task)| task).collect()
}

/// Tries to place a waiting task by moving a blocker, keeping both only when
/// both fit.
///
/// Returns `true` when the pair was rearranged and both tasks are placed. On
/// a `false` return the schedule is exactly as it was.
///
/// 尝试通过移动某个阻塞者来放置等待任务，仅当二者都能容纳时保留二者。
///
/// 当该对被重排且两个任务均已放置时返回 `true`。返回 `false` 时计划与
/// 原状完全一致。
fn relocate_pair(
    waiting: &Task,
    blocker: &Task,
    tasks: &[Task],
    schedule: &mut Schedule,
    resources: &ResourceSet,
) -> bool {
    let Some(blocker_start) = schedule.placement_of(blocker.id).map(|p| p.start) else {
        return false;
    };
    // Remove the blocker and look for a home for the waiting task.
    // 移除阻塞者，为等待任务寻找安身之处。
    if schedule.unplace(blocker.id).is_err() {
        return false;
    }
    let waiting_start = earliest_feasible_start(waiting, tasks, schedule, resources);
    if waiting_start.is_none() {
        // Nothing changed, so restore the blocker.
        // 没有任何改变，故恢复阻塞者。
        let _ = schedule.place(blocker, blocker_start);
        return false;
    }
    let waiting_start = waiting_start.expect("checked above");
    if schedule.place(waiting, waiting_start).is_err() {
        let _ = schedule.place(blocker, blocker_start);
        return false;
    }

    // With the waiting task placed, look for a new home for the blocker.
    // 等待任务已放置后，为阻塞者寻找新的去处。
    match earliest_feasible_start(blocker, tasks, schedule, resources) {
        Some(blocker_new_start) => {
            if schedule.place(blocker, blocker_new_start).is_ok() {
                return true;
            }
            // The search said the placement was feasible, so a failure here
            // can only come from the blocker being placed already, which it
            // is not. Restore the original arrangement and report failure
            // rather than leaving a half applied move.
            // 搜索已判定该放置可行，故此处的失败只可能源于阻塞者已被
            // 放置，而它并未被放置。恢复原安排并报告失败，以免留下
            // 半完成的移动。
            let _ = schedule.unplace(waiting.id);
            let _ = schedule.place(blocker, blocker_start);
            false
        }
        None => {
            // The blocker has nowhere to go, so the move is abandoned and
            // the original arrangement restored.
            // 阻塞者无处可去，故放弃该次移动并恢复原安排。
            let _ = schedule.unplace(waiting.id);
            let _ = schedule.place(blocker, blocker_start);
            false
        }
    }
}

/// Updates the recorded state of a task, ignoring an identifier outside the
/// state list.
///
/// 更新某任务所记录的状态；标识符不在状态列表之内时忽略。
fn set_state(states: &mut [TaskState], id: TaskId, state: TaskState) {
    if let Some(slot) = states.get_mut(id.index()) {
        *slot = state;
    }
}
