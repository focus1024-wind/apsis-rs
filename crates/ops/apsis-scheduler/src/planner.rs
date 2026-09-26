//! The greedy planner.

use crate::greedy::{earliest_feasible_start, infeasible_tasks};
use crate::improve::{ImprovementSettings, improve};
use apsis_resource::ResourceSet;
use apsis_schedule::Schedule;
use apsis_task::{PrecedenceGraph, Task, TaskId, TaskState, task_order};

/// What happened to one task during planning.
///
/// The outcome pairs a task with the state the planner assigned, which is the
/// information an operator needs to see. It is separate from the schedule
/// because a task that was not placed has no placement but still has an
/// outcome worth reporting.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanOutcome {
    /// The task.
    pub task: TaskId,
    /// The state the planner assigned.
    pub state: TaskState,
}

impl PlanOutcome {
    /// Returns `true` when the task was placed.
    pub fn is_scheduled(&self) -> bool {
        self.state.is_scheduled()
    }

    /// Returns `true` when the task could not be placed for a structural
    /// reason, rather than being left out for want of time.
    pub fn is_infeasible(&self) -> bool {
        matches!(self.state, TaskState::Infeasible { .. })
    }
}

/// The result of planning.
#[derive(Debug, Clone, PartialEq)]
pub struct SchedulingResult {
    /// The schedule that was produced.
    pub schedule: Schedule,
    /// The outcome of every task, in the order the planner considered them.
    ///
    /// The order is the order of the task list the planner was given, not the
    /// order in which the planner placed them, so that a caller can index the
    /// outcomes by the position of the task in its own list.
    pub outcomes: Vec<PlanOutcome>,
    /// The identifiers of the tasks that the improvement pass placed.
    ///
    /// These tasks were left out by the greedy pass and placed afterwards by
    /// moving other placements. They are also recorded as scheduled in the
    /// outcomes.
    pub improved: Vec<TaskId>,
}

impl SchedulingResult {
    /// Returns the outcome of a task.
    pub fn outcome_of(&self, task: TaskId) -> Option<&PlanOutcome> {
        self.outcomes.iter().find(|outcome| outcome.task == task)
    }

    /// Returns the identifiers of the tasks that were placed.
    pub fn scheduled(&self) -> Vec<TaskId> {
        self.outcomes
            .iter()
            .filter(|outcome| outcome.is_scheduled())
            .map(|outcome| outcome.task)
            .collect()
    }

    /// Returns the identifiers of the tasks that were left out.
    pub fn deferred(&self) -> Vec<TaskId> {
        self.outcomes
            .iter()
            .filter(|outcome| outcome.state == TaskState::Deferred)
            .map(|outcome| outcome.task)
            .collect()
    }

    /// Returns the identifiers of the tasks that cannot be placed at all.
    pub fn infeasible(&self) -> Vec<TaskId> {
        self.outcomes
            .iter()
            .filter(|outcome| outcome.is_infeasible())
            .map(|outcome| outcome.task)
            .collect()
    }

    /// Returns `true` when every offered task was placed.
    pub fn is_complete(&self) -> bool {
        self.outcomes.iter().all(PlanOutcome::is_scheduled)
    }

    /// Returns `true` when every mandatory task was placed.
    ///
    /// A plan that leaves a mandatory task unplaced is not usable, however
    /// much optional work it found room for.
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Planner {
    /// Whether to place mandatory tasks before the others regardless of
    /// their value.
    ///
    /// A mandatory task must run, so it is worth placing first even when its
    /// weighted value is low: the optional tasks can use whatever time is
    /// left, while the reverse order could consume the only window the
    /// mandatory task had.
    pub mandatory_first: bool,
    /// The settings of the improvement pass.
    pub improvement: ImprovementSettings,
}

impl Default for Planner {
    /// Returns a planner that places mandatory tasks first and runs the
    /// improvement pass.
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
    pub const fn new() -> Self {
        Self {
            mandatory_first: true,
            improvement: ImprovementSettings::new(4),
        }
    }

    /// Returns a copy of this planner with the mandatory placement rule set.
    pub const fn with_mandatory_first(mut self, mandatory_first: bool) -> Self {
        self.mandatory_first = mandatory_first;
        self
    }

    /// Returns a copy of this planner with the improvement pass configured.
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
    pub fn order_with_precedence(
        &self,
        tasks: &[Task],
        precedence: Option<&PrecedenceGraph>,
    ) -> Vec<TaskId> {
        let ordered = task_order(tasks);
        let valued = if !self.mandatory_first {
            ordered
        } else {
            let mandatory: Vec<TaskId> = tasks
                .iter()
                .filter(|task| task.mandatory)
                .map(|task| task.id)
                .collect();
            let mut result = Vec::with_capacity(ordered.len());
            result.extend(ordered.iter().filter(|id| mandatory.contains(id)).copied());
            result.extend(ordered.iter().filter(|id| !mandatory.contains(id)).copied());
            result
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
    pub fn schedule_with_precedence(
        &self,
        tasks: &[Task],
        resources: &ResourceSet,
        precedence: &PrecedenceGraph,
    ) -> SchedulingResult {
        self.plan(tasks, resources, Some(precedence))
    }

    /// The shared planning routine.
    fn plan(
        &self,
        tasks: &[Task],
        resources: &ResourceSet,
        precedence: Option<&PrecedenceGraph>,
    ) -> SchedulingResult {
        let mut schedule = Schedule::new();
        // The state of every task, indexed by identifier, so that a
        // predecessor's placement can be looked up by index.
        let mut states = vec![TaskState::Pending; tasks.len()];
        let mut outcomes = Vec::with_capacity(tasks.len());
        let mut order: Vec<TaskId> = self.order_with_precedence(tasks, precedence);
        // A deferred task is one whose search failed, as opposed to one whose
        // structural preconditions failed.
        let mut deferred: Vec<TaskId> = Vec::new();
        let infeasible = infeasible_tasks(tasks, resources);

        for id in order.drain(..) {
            let task = tasks
                .iter()
                .find(|task| task.id == id)
                .expect("the order was built from the task list");

            if infeasible.contains(&id) {
                let reason = if !task.is_valid() {
                    "the task is not usable".to_string()
                } else {
                    format!("the resources cannot hold the demand of {id}")
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
            let earliest = match self.precedence_floor(id, tasks, precedence, &schedule, &states) {
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
            let constrained = match earliest {
                Some(floor) if floor > task.earliest_start => {
                    let mut copy = task.clone();
                    copy.earliest_start = floor;
                    copy
                }
                _ => task.clone(),
            };

            match earliest_feasible_start(&constrained, tasks, &schedule, resources) {
                Some(start) => match schedule.place(task, start) {
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
                },
                None => {
                    set_state(&mut states, id, TaskState::Deferred);
                    outcomes.push(PlanOutcome {
                        task: id,
                        state: TaskState::Deferred,
                    });
                    deferred.push(id);
                }
            }
        }

        // The improvement pass runs only for a plan without precedence
        // constraints, for the reason given on `schedule_with_precedence`.
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
    fn precedence_floor(
        &self,
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
fn set_state(states: &mut [TaskState], id: TaskId, state: TaskState) {
    if let Some(slot) = states.get_mut(id.index()) {
        *slot = state;
    }
}
