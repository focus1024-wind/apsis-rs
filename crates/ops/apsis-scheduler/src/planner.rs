//! The greedy planner.

use crate::greedy::{earliest_feasible_start, infeasible_tasks};
use apsis_resource::ResourceSet;
use apsis_schedule::Schedule;
use apsis_task::{Task, TaskId, TaskState, task_order};

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
    pub outcomes: Vec<PlanOutcome>,
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
        let mandatory: Vec<TaskId> = tasks
            .iter()
            .filter(|task| task.mandatory)
            .map(|task| task.id)
            .collect();
        mandatory.iter().all(|id| self.schedule.is_placed(*id))
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
}

impl Default for Planner {
    /// Returns a planner that places mandatory tasks first.
    fn default() -> Self {
        Self {
            mandatory_first: true,
        }
    }
}

impl Planner {
    /// Creates a planner with the default settings.
    pub const fn new() -> Self {
        Self {
            mandatory_first: true,
        }
    }

    /// Returns a copy of this planner with the mandatory placement rule set.
    pub const fn with_mandatory_first(mut self, mandatory_first: bool) -> Self {
        self.mandatory_first = mandatory_first;
        self
    }

    /// Returns the order in which the planner considers the tasks.
    ///
    /// Mandatory tasks come first when the planner is configured that way,
    /// each group ordered by [`task_order`], which is decreasing weighted
    /// value with ties broken by deadline and identifier.
    pub fn order(&self, tasks: &[Task]) -> Vec<TaskId> {
        let ordered = task_order(tasks);
        if !self.mandatory_first {
            return ordered;
        }
        let mandatory: Vec<TaskId> = tasks
            .iter()
            .filter(|task| task.mandatory)
            .map(|task| task.id)
            .collect();
        // The stable partition keeps the relative order within each group.
        let mut result = Vec::with_capacity(ordered.len());
        result.extend(ordered.iter().filter(|id| mandatory.contains(id)).copied());
        result.extend(ordered.iter().filter(|id| !mandatory.contains(id)).copied());
        result
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
        let mut schedule = Schedule::new();
        let mut outcomes = Vec::with_capacity(tasks.len());

        // Tasks that cannot be placed at all are reported without a search.
        let infeasible = infeasible_tasks(tasks, resources);

        for id in self.order(tasks) {
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
                outcomes.push(PlanOutcome {
                    task: id,
                    state: TaskState::Infeasible { reason },
                });
                continue;
            }

            match earliest_feasible_start(task, tasks, &schedule, resources) {
                Some(start) => {
                    // The placement was checked to be feasible, so the call
                    // succeeds. The schedule may still refuse it if the task
                    // is already placed, which it is not, because each task
                    // is considered once.
                    match schedule.place(task, start) {
                        Ok(_) => outcomes.push(PlanOutcome {
                            task: id,
                            state: TaskState::Scheduled { start },
                        }),
                        Err(error) => outcomes.push(PlanOutcome {
                            task: id,
                            state: TaskState::Infeasible {
                                reason: error.to_string(),
                            },
                        }),
                    }
                }
                None => outcomes.push(PlanOutcome {
                    task: id,
                    state: TaskState::Deferred,
                }),
            }
        }

        SchedulingResult { schedule, outcomes }
    }
}
