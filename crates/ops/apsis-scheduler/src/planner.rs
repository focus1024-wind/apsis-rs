//! The greedy planner.
//!
//! 贪心规划器。

use crate::greedy::infeasible_tasks;
use crate::improve::{ImprovementSettings, improve};
use crate::sweep::earliest_feasible_start;
use apsis_resource::ResourceSet;
use apsis_schedule::Schedule;
use apsis_task::{PrecedenceGraph, Task, TaskId, TaskState, task_order};

/// What happened to one task during planning.
///
/// The outcome pairs a task with the state the planner assigned, which is the
/// information an operator needs to see. It is separate from the schedule
/// because a task that was not placed has no placement but still has an
/// outcome worth reporting.
///
/// 规划过程中某个任务的结果。
///
/// 它把任务与规划器所指定的状态配对，这是操作者需要看到的信息。它与
/// 计划分开，因为未放置的任务虽无放置，却仍有一份值得报告的结果。
#[derive(Debug, Clone, PartialEq)]
pub struct PlanOutcome {
    /// The task.
    ///
    /// 该任务。
    pub task: TaskId,
    /// The state the planner assigned.
    ///
    /// 规划器所指定的状态。
    pub state: TaskState,
}

impl PlanOutcome {
    /// Returns `true` when the task was placed.
    ///
    /// 当该任务已被放置时返回 `true`。
    #[must_use]
    pub fn is_scheduled(&self) -> bool {
        self.state.is_scheduled()
    }

    /// Returns `true` when the task could not be placed for a structural
    /// reason, rather than being left out for want of time.
    ///
    /// 当该任务因结构性原因无法放置、而非因时间不足被排除时返回 `true`。
    #[must_use]
    pub fn is_infeasible(&self) -> bool {
        matches!(self.state, TaskState::Infeasible { .. })
    }
}

/// The result of planning.
///
/// 规划的结果。
#[derive(Debug, Clone, PartialEq)]
pub struct SchedulingResult {
    /// The schedule that was produced.
    ///
    /// 所产出的计划。
    pub schedule: Schedule,
    /// The outcome of every task, in the order the planner considered them.
    ///
    /// The order is the order of the task list the planner was given, not the
    /// order in which the planner placed them, so that a caller can index the
    /// outcomes by the position of the task in its own list.
    ///
    /// 每个任务的结果，按规划器考虑它们的顺序排列。
    ///
    /// 该顺序是规划器所获任务列表的顺序，而非其放置的顺序，故调用方
    /// 可按任务在自身列表中的位置索引结果。
    pub outcomes: Vec<PlanOutcome>,
    /// The identifiers of the tasks that the improvement pass placed.
    ///
    /// These tasks were left out by the greedy pass and placed afterwards by
    /// moving other placements. They are also recorded as scheduled in the
    /// outcomes.
    ///
    /// 改进轮所放置的任务标识符。
    ///
    /// 这些任务被贪心轮排除，其后通过移动其他放置而被安排。它们在
    /// 结果中同样被记为已调度。
    pub improved: Vec<TaskId>,
}

impl SchedulingResult {
    /// Returns the outcome of a task.
    ///
    /// 返回某任务的结果。
    #[must_use]
    pub fn outcome_of(&self, task: TaskId) -> Option<&PlanOutcome> {
        self.outcomes.iter().find(|outcome| outcome.task == task)
    }

    /// Returns the identifiers of the tasks that were placed.
    ///
    /// 返回已放置任务的标识符。
    #[must_use]
    pub fn scheduled(&self) -> Vec<TaskId> {
        self.outcomes
            .iter()
            .filter(|outcome| outcome.is_scheduled())
            .map(|outcome| outcome.task)
            .collect()
    }

    /// Returns the identifiers of the tasks that were left out.
    ///
    /// 返回被排除任务的标识符。
    #[must_use]
    pub fn deferred(&self) -> Vec<TaskId> {
        self.outcomes
            .iter()
            .filter(|outcome| outcome.state == TaskState::Deferred)
            .map(|outcome| outcome.task)
            .collect()
    }

    /// Returns the identifiers of the tasks that cannot be placed at all.
    ///
    /// 返回完全无法放置的任务的标识符。
    #[must_use]
    pub fn infeasible(&self) -> Vec<TaskId> {
        self.outcomes
            .iter()
            .filter(|outcome| outcome.is_infeasible())
            .map(|outcome| outcome.task)
            .collect()
    }

    /// Returns `true` when every offered task was placed.
    ///
    /// 当所有提交任务均被放置时返回 `true`。
    pub fn is_complete(&self) -> bool {
        self.outcomes.iter().all(PlanOutcome::is_scheduled)
    }

    /// Returns `true` when every mandatory task was placed.
    ///
    /// A plan that leaves a mandatory task unplaced is not usable, however
    /// much optional work it found room for.
    ///
    /// 当所有强制任务均被放置时返回 `true`。
    ///
    /// 留下强制任务未放置的计划不可用，无论它为多少可选工作腾出了
    /// 空间。
    #[must_use]
    pub fn honours_mandatory(&self, tasks: &[Task]) -> bool {
        tasks
            .iter()
            .filter(|task| task.mandatory)
            .all(|task| self.schedule.is_placed(task.id))
    }

    /// Returns `true` when every precedence constraint is satisfied.
    ///
    /// A constraint is satisfied when the second task starts at or after the
    /// end of the first. A constraint whose tasks are not both placed is
    /// trivially satisfied, because nothing has been promised about them.
    ///
    /// 当所有优先约束都得到满足时返回 `true`。
    ///
    /// 约束被满足，指第二个任务的起点不早于第一个的终点。两个任务未
    /// 同时放置的约束平凡地满足，因为对它们未作任何承诺。
    #[must_use]
    pub fn honours_precedence(&self, graph: &PrecedenceGraph) -> bool {
        self.outcomes
            .iter()
            .filter(|outcome| outcome.is_scheduled())
            .all(|outcome| {
                let Some(after) = self.schedule.placement_of(outcome.task) else {
                    return true;
                };
                graph.predecessors_of(outcome.task).iter().all(|before| {
                    self.schedule
                        .placement_of(*before)
                        .is_none_or(|placement| placement.end() <= after.start)
                })
            })
    }
}

/// A greedy planner.
///
/// The planner holds the settings of the search. The settings are separated
/// from the schedule so that the same planner can be applied to many task
/// sets with the same policy.
///
/// 贪心规划器。
///
/// 该规划器持有搜索的设置。设置与计划分开，故同一个规划器可按同一策略
/// 应用于许多任务集。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Planner {
    /// Whether to place mandatory tasks before the others regardless of
    /// their value.
    ///
    /// A mandatory task must run, so it is worth placing first even when its
    /// weighted value is low: the optional tasks can use whatever time is
    /// left, while the reverse order could consume the only window the
    /// mandatory task had.
    ///
    /// 是否不论价值高低都先放置强制任务。
    ///
    /// 强制任务必须执行，故即便其加权价值很低也值得先放置：可选任务
    /// 可以利用剩下的时间，而相反的顺序可能耗尽强制任务仅有的那个窗口。
    pub mandatory_first: bool,
    /// The settings of the improvement pass.
    ///
    /// 改进轮的设置。
    pub improvement: ImprovementSettings,
}

impl Default for Planner {
    /// Returns a planner that places mandatory tasks first and runs the
    /// improvement pass.
    ///
    /// 返回一个先放置强制任务并运行改进轮的规划器。
    fn default() -> Self {
        Self {
            mandatory_first: true,
            improvement: ImprovementSettings::default(),
        }
    }
}

impl Planner {
    /// Creates a planner with the default settings.
    ///
    /// The defaults are the ones [`Planner::default`] uses: mandatory tasks
    /// first and the improvement pass enabled with four attempts per task.
    ///
    /// 以默认设置创建规划器。
    ///
    /// 默认值即 [`Planner::default`] 所用者：先放置强制任务，改进轮
    /// 启用且每任务尝试四次。
    #[must_use]
    pub const fn new() -> Self {
        Self {
            mandatory_first: true,
            improvement: ImprovementSettings::new(4),
        }
    }

    /// Returns a copy of this planner with the mandatory placement rule set.
    ///
    /// 返回设置了强制放置规则的规划器副本。
    #[must_use]
    pub const fn with_mandatory_first(mut self, mandatory_first: bool) -> Self {
        self.mandatory_first = mandatory_first;
        self
    }

    /// Returns a copy of this planner with the improvement pass configured.
    ///
    /// 返回配置了改进轮的规划器副本。
    #[must_use]
    pub const fn with_improvement(mut self, improvement: ImprovementSettings) -> Self {
        self.improvement = improvement;
        self
    }

    /// Returns the order in which the planner considers the tasks.
    ///
    /// Without a precedence graph the order is mandatory tasks first, when
    /// the planner is configured that way, and then decreasing weighted
    /// value with ties broken by deadline and identifier.
    ///
    /// With a precedence graph the order is a topological order of the
    /// graph, using the value order to break ties. That places a task as
    /// early as its constraints allow and the most valuable one first among
    /// the tasks that are free.
    ///
    /// 返回规划器考虑任务的顺序。
    ///
    /// 无优先关系图时，顺序是先强制任务（若规划器如此配置），再按加权
    /// 价值递减，平局按截止时刻与标识符打破。
    ///
    /// 有优先关系图时，顺序是该图的拓扑序，并以价值次序打破平局。这
    /// 使任务在其约束允许的最早时刻被放置，而在自由的任务中优先放置
    /// 价值最高者。
    #[must_use]
    pub fn order(&self, tasks: &[Task]) -> Vec<TaskId> {
        self.order_with_precedence(tasks, None)
    }

    /// Returns the order in which the planner considers the tasks, honouring
    /// a precedence graph.
    ///
    /// A graph whose constraints contain a cycle produces the value order
    /// unchanged, because the constraints then have no satisfiable
    /// interpretation. The caller should validate the graph before
    /// planning; the order is a fallback so that planning still produces
    /// something rather than failing outright.
    ///
    /// 返回规划器考虑任务的顺序，并遵从给定的优先关系图。
    ///
    /// 约束含环的图会原样产生价值次序，因为这些约束此时没有可满足的
    /// 解释。调用方应在规划前校验该图；此处的次序是一个回退，使规划
    /// 仍能产出结果而非直接失败。
    #[must_use]
    pub fn order_with_precedence(
        &self,
        tasks: &[Task],
        precedence: Option<&PrecedenceGraph>,
    ) -> Vec<TaskId> {
        let ordered = task_order(tasks);
        let valued = if self.mandatory_first {
            let mandatory: Vec<TaskId> = tasks
                .iter()
                .filter(|task| task.mandatory)
                .map(|task| task.id)
                .collect();
            let mut result = Vec::with_capacity(ordered.len());
            result.extend(ordered.iter().filter(|id| mandatory.contains(id)).copied());
            result.extend(ordered.iter().filter(|id| !mandatory.contains(id)).copied());
            result
        } else {
            ordered
        };

        let Some(graph) = precedence else {
            return valued;
        };
        if graph.is_empty() {
            return valued;
        }
        graph.topological_order(&valued).unwrap_or(valued)
    }

    /// Plans a set of tasks on a set of resources.
    ///
    /// Every task is considered once, in the order the planner chooses. A
    /// task is placed at the earliest time at which all of its resources are
    /// free together. A task that cannot be placed is reported as deferred,
    /// or as infeasible when no placement could ever exist.
    ///
    /// The schedule that is returned is feasible by construction, because
    /// each placement is checked against the capacity before it is made. The
    /// caller may confirm that with `apsis-conflict`.
    ///
    /// 在一组资源上规划一组任务。
    ///
    /// 每个任务按规划器所选的顺序被考虑一次，被放置在其全部资源恰好同时
    /// 空闲的最早时刻。无法放置的任务被报告为被推迟；不存在任何可行
    /// 放置的则报告为不可行。
    ///
    /// 返回的计划按构造即可行，因为每处放置在做下之前都对照容量校验
    /// 过。调用方可用 `apsis-conflict` 复核。
    #[must_use]
    pub fn schedule(&self, tasks: &[Task], resources: &ResourceSet) -> SchedulingResult {
        self.plan(tasks, resources, None)
    }

    /// Plans a set of tasks, honouring a precedence graph.
    ///
    /// A task is placed only after every task that must precede it has been
    /// placed, so the search for its start begins no earlier than the latest
    /// end among its predecessors. A predecessor that could not be placed
    /// makes the successor deferred as well, because the constraint cannot be
    /// met.
    ///
    /// The improvement pass is not applied to a plan with precedence
    /// constraints, because relocating a placement could break a constraint
    /// that the pass does not consider. A caller that needs both should
    /// improve the plan and then verify it with
    /// [`SchedulingResult::honours_precedence`].
    ///
    /// 遵从优先关系图规划一组任务。
    ///
    /// 任务仅在其所有前驱均已放置之后才被放置，故其起点搜索不早于
    /// 前驱中最晚的结束时刻。无法放置的前驱会使其后继同样被推迟，
    /// 因为该约束无法满足。
    ///
    /// 改进轮不作用于带优先约束的计划，因为移动某处放置可能破坏该轮
    /// 并不考虑的约束。两者皆需的调用方应先改进计划，再用
    /// [`SchedulingResult::honours_precedence`] 复核。
    #[must_use]
    pub fn schedule_with_precedence(
        &self,
        tasks: &[Task],
        resources: &ResourceSet,
        precedence: &PrecedenceGraph,
    ) -> SchedulingResult {
        self.plan(tasks, resources, Some(precedence))
    }

    /// The shared planning routine.
    ///
    /// 共享的规划例程。
    fn plan(
        &self,
        tasks: &[Task],
        resources: &ResourceSet,
        precedence: Option<&PrecedenceGraph>,
    ) -> SchedulingResult {
        let mut schedule = Schedule::new();
        // The state of every task, indexed by identifier, so that a
        // predecessor's placement can be looked up by index.
        // 各任务的状态，按标识符索引，使前驱的放置可按下标查得。
        let mut states = vec![TaskState::Pending; tasks.len()];
        let mut outcomes = Vec::with_capacity(tasks.len());
        let mut order: Vec<TaskId> = self.order_with_precedence(tasks, precedence);
        // A deferred task is one whose search failed, as opposed to one whose
        // structural preconditions failed.
        // 被推迟的任务指搜索失败者，区别于结构性前置条件失败者。
        let mut deferred: Vec<TaskId> = Vec::new();
        let infeasible = infeasible_tasks(tasks, resources);

        for id in order.drain(..) {
            let task = tasks
                .iter()
                .find(|task| task.id == id)
                .expect("the order was built from the task list");

            if infeasible.contains(&id) {
                let reason = if task.is_valid() {
                    format!("the resources cannot hold the demand of {id}")
                } else {
                    "the task is not usable".to_string()
                };
                set_state(
                    &mut states,
                    id,
                    TaskState::Infeasible {
                        reason: reason.clone(),
                    },
                );
                outcomes.push(PlanOutcome {
                    task: id,
                    state: TaskState::Infeasible { reason },
                });
                continue;
            }

            // A task may not start before the latest end among the tasks that
            // must precede it.
            // 任务的起点不得早于其前驱中最晚的结束时刻。
            let earliest = match Self::precedence_floor(id, tasks, precedence, &schedule, &states) {
                Ok(floor) => floor,
                Err(reason) => {
                    set_state(&mut states, id, TaskState::Deferred);
                    outcomes.push(PlanOutcome {
                        task: id,
                        state: TaskState::Deferred,
                    });
                    let _ = reason;
                    deferred.push(id);
                    continue;
                }
            };

            // The task with the floor applied, so that the search respects
            // the constraints.
            // 施加该下界后的任务，使搜索遵从各约束。
            let constrained = match earliest {
                Some(floor) if floor > task.earliest_start => {
                    let mut copy = task.clone();
                    copy.earliest_start = floor;
                    copy
                }
                _ => task.clone(),
            };

            if let Some(start) = earliest_feasible_start(&constrained, tasks, &schedule, resources)
            {
                match schedule.place(task, start) {
                    Ok(_) => {
                        set_state(&mut states, id, TaskState::Scheduled { start });
                        outcomes.push(PlanOutcome {
                            task: id,
                            state: TaskState::Scheduled { start },
                        });
                    }
                    Err(error) => {
                        let reason = error.to_string();
                        set_state(
                            &mut states,
                            id,
                            TaskState::Infeasible {
                                reason: reason.clone(),
                            },
                        );
                        outcomes.push(PlanOutcome {
                            task: id,
                            state: TaskState::Infeasible { reason },
                        });
                    }
                }
            } else {
                set_state(&mut states, id, TaskState::Deferred);
                outcomes.push(PlanOutcome {
                    task: id,
                    state: TaskState::Deferred,
                });
                deferred.push(id);
            }
        }

        // The improvement pass runs only for a plan without precedence
        // constraints, for the reason given on `schedule_with_precedence`.
        // 改进轮只作用于无优先约束的计划，原因见
        // `schedule_with_precedence`。
        let improved = if precedence.is_some() {
            Vec::new()
        } else {
            let outcome = improve(
                tasks,
                resources,
                &deferred,
                &mut schedule,
                &mut states,
                self.improvement,
            );
            // The outcomes that the pass changed must be updated, and the
            // order of the outcomes follows the task list so that a caller
            // can index them by position.
            // 该轮改动的结果必须更新，且结果次序遵循任务列表，使调用
            // 方可按位置索引。
            for id in &outcome.placed {
                if let Some(slot) = outcomes.iter_mut().find(|entry| entry.task == *id) {
                    let start = schedule
                        .placement_of(*id)
                        .map_or(0.0, |placement| placement.start);
                    slot.state = TaskState::Scheduled { start };
                }
            }
            outcome.placed
        };

        // Reorder the outcomes to follow the task list, which is the order a
        // caller expects when it indexes by its own position.
        // 将结果重排为遵循任务列表，即调用方按自身位置索引时所期望的
        // 次序。
        outcomes.sort_by_key(|outcome| outcome.task.index());

        SchedulingResult {
            schedule,
            outcomes,
            improved,
        }
    }

    /// Returns the earliest start a task may have, given its predecessors.
    ///
    /// Returns `Ok(None)` when there is no constraint, `Ok(Some(t))` when the
    /// task may start no earlier than `t`, and `Err` when a predecessor was
    /// not placed, which makes the constraint unsatisfiable.
    ///
    /// 给定其前驱，返回某任务可有的最早起点。
    ///
    /// 无约束时返回 `Ok(None)`；任务起点不早于 `t` 时返回 `Ok(Some(t))`；
    /// 某前驱未被放置致使约束不可满足时返回 `Err`。
    fn precedence_floor(
        id: TaskId,
        _tasks: &[Task],
        precedence: Option<&PrecedenceGraph>,
        schedule: &Schedule,
        states: &[TaskState],
    ) -> Result<Option<f64>, String> {
        let Some(graph) = precedence else {
            return Ok(None);
        };
        let mut floor: Option<f64> = None;
        for predecessor in graph.predecessors_of(id) {
            // A predecessor that was deferred or is infeasible cannot be
            // waited for.
            // 被推迟或不可行的前驱无从等待。
            let placed = schedule.placement_of(predecessor);
            let Some(placement) = placed else {
                let reason = match states.get(predecessor.index()) {
                    Some(TaskState::Infeasible { reason }) => reason.clone(),
                    _ => format!("{predecessor} was not placed"),
                };
                return Err(reason);
            };
            floor =
                Some(floor.map_or(placement.end(), |current: f64| current.max(placement.end())));
        }
        Ok(floor)
    }
}

/// Records the state of a task, ignoring an identifier outside the list.
///
/// 记录某任务的状态；标识符不在列表之内时忽略。
fn set_state(states: &mut [TaskState], id: TaskId, state: TaskState) {
    if let Some(slot) = states.get_mut(id.index()) {
        *slot = state;
    }
}
