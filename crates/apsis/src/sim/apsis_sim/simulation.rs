//! Preparing a scenario for a run.
//!
//! 为一次运行准备场景。

use crate::apsis_resource::ResourceSet;
use crate::apsis_scenario::Scenario;
use crate::apsis_schedule::{Evaluation, Schedule};
use crate::apsis_scheduler::{Planner, SchedulingResult};
use crate::apsis_sim::activity::Activity;
use crate::apsis_sim::error::{SimulationError, SimulationErrorKind};
use crate::apsis_sim::session::Session;
use crate::apsis_task::{Task, TaskId};

/// A scenario prepared for a run.
///
/// A simulation holds the scenario, the plan that was made from it, and the
/// timeline that plan implies. Preparing is separate from running so that a
/// caller can inspect the plan, and reject it, before a run commits to it.
///
/// 为一次运行准备好的场景。
///
/// 仿真保存场景、由其作出的计划，以及该计划所蕴含的时间线。准备与运行
/// 分开，使调用方可以在运行承诺之前检查该计划并拒绝它。
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
    ///
    /// 检查场景并为其作计划。
    ///
    /// 场景被校验，任务针对资源被规划，所得调度被转为由开始与结束事件
    /// 构成的时间线。该计划是贪心的，故场景可能被成功准备却仍留下未
    /// 放置的任务；这由 [`Simulation::has_full_plan`] 报告，而不是当作
    /// 错误。
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
    ///
    /// 由已规划好的结果构建仿真。
    ///
    /// 这适用于已用自己的设置完成规划、只想要外观其余部分的调用方：时间
    /// 线与运行循环。该结果必须属于该场景，这一点会被检查。
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
    ///
    /// 由计划构建时间线。
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
            // 时长被复制进放置中，故针对该计划声称要放置的任务检查它。
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
    ///
    /// 返回场景。
    #[must_use]
    pub const fn scenario(&self) -> &Scenario {
        &self.scenario
    }

    /// Returns the resources in the scenario.
    ///
    /// 返回场景中的资源。
    #[must_use]
    pub const fn resources(&self) -> &ResourceSet {
        &self.resources
    }

    /// Returns the planning result.
    ///
    /// 返回规划结果。
    #[must_use]
    pub const fn plan(&self) -> &SchedulingResult {
        &self.result
    }

    /// Returns the schedule.
    ///
    /// 返回调度。
    #[must_use]
    pub const fn schedule(&self) -> &Schedule {
        &self.result.schedule
    }

    /// Returns the tasks in the scenario.
    ///
    /// 返回场景中的任务。
    #[must_use]
    pub fn tasks(&self) -> &[Task] {
        self.scenario.tasks()
    }

    /// Returns the timeline as time and activity pairs, in time order.
    ///
    /// Each placed task contributes a start and an end. Ties are ordered so
    /// that an end precedes a start at the same instant, which lets two back
    /// to back tasks share a boundary without the reader seeing them overlap.
    ///
    /// 按时间次序返回时间与活动对的列表。
    ///
    /// 每处被放置的任务贡献一个开始与一个结束。并列情形按结束先于开始
    /// 排序，使两个前后相接的任务可共享一个边界而读者不会看到它们
    /// 重叠。
    #[must_use]
    pub fn timeline(&self) -> &[(f64, Activity)] {
        &self.timeline
    }

    /// Returns the number of start and end events in the timeline.
    ///
    /// 返回时间线中开始与结束事件的数量。
    #[must_use]
    pub fn activity_count(&self) -> usize {
        self.timeline.len()
    }

    /// Returns `true` when every offered task was placed.
    ///
    /// 当所有提交任务均被放置时返回 `true`。
    #[must_use]
    pub fn has_full_plan(&self) -> bool {
        self.result.is_complete()
    }

    /// Returns `true` when every mandatory task was placed.
    ///
    /// 当每个必需任务都被放置时返回 `true`。
    #[must_use]
    pub fn honours_mandatory(&self) -> bool {
        self.result.honours_mandatory(self.scenario.tasks())
    }

    /// Returns `true` when every precedence constraint is satisfied.
    ///
    /// 当每条先后约束都被满足时返回 `true`。
    #[must_use]
    pub fn honours_precedence(&self) -> bool {
        self.result.honours_precedence(self.scenario.precedence())
    }

    /// Returns the tasks that were not placed for want of time.
    ///
    /// 返回因时间不足而未被放置的任务。
    #[must_use]
    pub fn deferred(&self) -> Vec<TaskId> {
        self.result.deferred()
    }

    /// Returns the tasks that cannot be placed at all.
    ///
    /// 返回完全无法放置的任务。
    #[must_use]
    pub fn infeasible(&self) -> Vec<TaskId> {
        self.result.infeasible()
    }

    /// Evaluates the schedule against the scenario.
    ///
    /// 针对场景评估该调度。
    #[must_use]
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
    ///
    /// 开始一次运行。
    ///
    /// 返回的会话拥有自己的引擎与记录器，故多个会话可从同一个已准备
    /// 仿真运行而互不干扰。时钟从场景历元开始，运行限制来自场景。
    #[must_use]
    pub fn start(&self) -> Session {
        Session::new(self)
    }
}
