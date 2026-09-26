//! Preparing a scenario for a run.

use crate::activity::Activity;
use crate::error::{SimulationError, SimulationErrorKind};
use crate::session::Session;
use apsis_resource::ResourceSet;
use apsis_scenario::Scenario;
use apsis_schedule::{Evaluation, Schedule};
use apsis_scheduler::{Planner, SchedulingResult};
use apsis_task::{Task, TaskId};

/// A scenario prepared for a run.
///
/// A simulation holds the scenario, the plan that was made from it, and the
/// timeline that plan implies. Preparing is separate from running so that a
/// caller can inspect the plan, and reject it, before a run commits to it.
#[derive(Debug, Clone)]
pub struct Simulation {
    scenario: Scenario,
    resources: ResourceSet,
    result: SchedulingResult,
    timeline: Vec<(f64, Activity)>,
}

impl Simulation {
    /// Checks a scenario and plans it.
    ///
    /// The scenario is validated, the tasks are planned against the
    /// resources, and the resulting schedule is turned into a timeline of
    /// start and end events. The plan is greedy, so a scenario may be
    /// prepared successfully and still leave tasks unplaced; that is reported
    /// by [`Simulation::has_full_plan`] rather than being an error.
    pub fn prepare(scenario: Scenario) -> Result<Self, SimulationError> {
        scenario.validate().map_err(|error| {
            SimulationError::new(SimulationErrorKind::InvalidScenario)
                .with_detail(error.to_string())
        })?;
        let resources = scenario.resource_set().map_err(|error| {
            SimulationError::new(SimulationErrorKind::InvalidResources)
                .with_detail(error.to_string())
        })?;
        let tasks = scenario.tasks().to_vec();
        let result =
            Planner::new().schedule_with_precedence(&tasks, &resources, scenario.precedence());
        Self::from_parts(scenario, resources, result)
    }

    /// Builds a simulation from an already planned result.
    ///
    /// This is for a caller that has planned with its own settings and wants
    /// the rest of the facade: the timeline and the run loop. The result must
    /// belong to the scenario, which is checked.
    pub fn from_plan(
        scenario: Scenario,
        result: SchedulingResult,
    ) -> Result<Self, SimulationError> {
        scenario.validate().map_err(|error| {
            SimulationError::new(SimulationErrorKind::InvalidScenario)
                .with_detail(error.to_string())
        })?;
        let resources = scenario.resource_set().map_err(|error| {
            SimulationError::new(SimulationErrorKind::InvalidResources)
                .with_detail(error.to_string())
        })?;
        Self::from_parts(scenario, resources, result)
    }

    /// Builds the timeline from a plan.
    fn from_parts(
        scenario: Scenario,
        resources: ResourceSet,
        result: SchedulingResult,
    ) -> Result<Self, SimulationError> {
        let mut timeline = Vec::with_capacity(result.schedule.len() * 2);
        for placement in result.schedule.in_time_order() {
            let Some(task) = scenario
                .tasks()
                .iter()
                .find(|task| task.id == placement.task)
            else {
                return Err(SimulationError::new(SimulationErrorKind::InconsistentPlan)
                    .with_detail(format!("{} is not in the scenario", placement.task)));
            };
            // The duration is copied into the placement, so the plan is
            // checked against the task it claims to place.
            if (placement.duration - task.duration).abs() > f64::EPSILON {
                return Err(SimulationError::new(SimulationErrorKind::InconsistentPlan)
                    .with_detail(format!(
                        "{} lasts {} s in the plan and {} s as a task",
                        placement.task, placement.duration, task.duration
                    )));
            }
            timeline.push((placement.start, Activity::start(placement.task)));
            timeline.push((placement.end(), Activity::end(placement.task)));
        }
        Ok(Self {
            scenario,
            resources,
            result,
            timeline,
        })
    }

    /// Returns the scenario.
    pub const fn scenario(&self) -> &Scenario {
        &self.scenario
    }

    /// Returns the resources in the scenario.
    pub const fn resources(&self) -> &ResourceSet {
        &self.resources
    }

    /// Returns the planning result.
    pub const fn plan(&self) -> &SchedulingResult {
        &self.result
    }

    /// Returns the schedule.
    pub const fn schedule(&self) -> &Schedule {
        &self.result.schedule
    }

    /// Returns the tasks in the scenario.
    pub fn tasks(&self) -> &[Task] {
        self.scenario.tasks()
    }

    /// Returns the timeline as time and activity pairs, in time order.
    ///
    /// Each placed task contributes a start and an end. Ties are ordered so
    /// that an end precedes a start at the same instant, which lets two back
    /// to back tasks share a boundary without the reader seeing them overlap.
    pub fn timeline(&self) -> &[(f64, Activity)] {
        &self.timeline
    }

    /// Returns the number of start and end events in the timeline.
    pub fn activity_count(&self) -> usize {
        self.timeline.len()
    }

    /// Returns `true` when every offered task was placed.
    pub fn has_full_plan(&self) -> bool {
        self.result.is_complete()
    }

    /// Returns `true` when every mandatory task was placed.
    pub fn honours_mandatory(&self) -> bool {
        self.result.honours_mandatory(self.scenario.tasks())
    }

    /// Returns `true` when every precedence constraint is satisfied.
    pub fn honours_precedence(&self) -> bool {
        self.result.honours_precedence(self.scenario.precedence())
    }

    /// Returns the tasks that were not placed for want of time.
    pub fn deferred(&self) -> Vec<TaskId> {
        self.result.deferred()
    }

    /// Returns the tasks that cannot be placed at all.
    pub fn infeasible(&self) -> Vec<TaskId> {
        self.result.infeasible()
    }

    /// Evaluates the schedule against the scenario.
    pub fn evaluate(&self) -> Evaluation {
        self.schedule()
            .evaluate(self.scenario.tasks(), &self.resources)
    }

    /// Starts a run.
    ///
    /// The returned session owns its own engine and recorder, so several
    /// sessions may run from one prepared simulation without interfering. The
    /// clock starts at the scenario epoch and the run limits come from the
    /// scenario.
    pub fn start(&self) -> Session {
        Session::new(self)
    }
}
