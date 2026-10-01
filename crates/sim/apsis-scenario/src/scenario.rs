//! The scenario type and its text form.

use crate::error::{ScenarioError, ScenarioErrorKind};
use apsis_resource::{Resource, ResourceSet};
use apsis_task::{PrecedenceGraph, Task, TaskId};
use core::fmt;

/// A simulation's initial conditions.
///
/// A scenario holds the resources a plan may use, the tasks it must place,
/// and the constraints between them, together with the limits a run should
/// respect. It is built either with the builder methods or by parsing text,
/// and both routes apply the same checks, so a scenario that parses is one
/// that would have been accepted by the builders.
///
/// 一次仿真的初始条件。
///
/// 场景保存计划可用的资源、必须放置的任务、它们之间的约束，以及一次
/// 运行应遵守的限制。它既可用构建器方法构建，也可由文本解析，两条
/// 路径施加相同的检查，故能解析的场景即是构建器会接受的场景。
#[derive(Debug, Clone, PartialEq)]
pub struct Scenario {
    name: String,
    epoch: f64,
    time_limit: Option<f64>,
    event_limit: Option<u64>,
    resources: Vec<Resource>,
    tasks: Vec<Task>,
    precedence: PrecedenceGraph,
}

impl Scenario {
    /// Creates an empty scenario with an epoch at zero.
    ///
    /// 创建历元为零的空场景。
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            epoch: 0.0,
            time_limit: None,
            event_limit: None,
            resources: Vec::new(),
            tasks: Vec::new(),
            precedence: PrecedenceGraph::new(),
        }
    }

    /// Returns the name of the scenario.
    ///
    /// 返回该场景的名称。
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the time the simulation clock starts at, in seconds.
    ///
    /// 返回仿真时钟的起始时刻，单位为秒。
    pub fn epoch(&self) -> f64 {
        self.epoch
    }

    /// Returns the latest time the run will process, if one is set.
    ///
    /// 返回该运行将处理的最晚时刻（若已设置）。
    pub fn time_limit(&self) -> Option<f64> {
        self.time_limit
    }

    /// Returns the greatest number of events the run will execute, if one is
    /// set.
    ///
    /// 返回该运行将执行的事件数上限（若已设置）。
    pub const fn event_limit(&self) -> Option<u64> {
        self.event_limit
    }

    /// Returns the resources in the order they were added.
    ///
    /// 按添加次序返回各资源。
    pub fn resources(&self) -> &[Resource] {
        &self.resources
    }

    /// Returns the resources for modification.
    ///
    /// A caller that changes a resource is responsible for keeping the
    /// scenario consistent; [`Scenario::validate`] checks it afterwards.
    ///
    /// 返回各资源以供修改。
    ///
    /// 修改资源的调用方负责保持场景一致；[`Scenario::validate`] 随后
    /// 检查它。
    pub fn resources_mut(&mut self) -> &mut Vec<Resource> {
        &mut self.resources
    }

    /// Reads a scenario from its text form.
    ///
    /// The format is described on the crate documentation. A scenario written
    /// with [`Scenario::to_text`] parses back to an equal scenario.
    ///
    /// 由其文本形式读取场景。
    ///
    /// 该格式在 crate 文档中描述。用 [`Scenario::to_text`] 写出的场景
    /// 解析回相等的场景。
    pub fn parse(text: &str) -> Result<Self, ScenarioError> {
        crate::parse::parse(text)
    }

    /// Returns the tasks in the order they were added.
    ///
    /// 按添加次序返回各任务。
    pub fn tasks(&self) -> &[Task] {
        &self.tasks
    }

    /// Returns the precedence constraints.
    ///
    /// 返回先后约束。
    pub const fn precedence(&self) -> &PrecedenceGraph {
        &self.precedence
    }

    /// Sets the time the simulation clock starts at.
    ///
    /// Returns the scenario unchanged in value but not in type; the caller
    /// uses the returned value. The epoch must be finite and non-negative.
    ///
    /// 设置仿真时钟的起始时刻。
    ///
    /// 返回的场景值不变而类型改变；调用方使用返回的值。历元必须有限
    /// 且非负。
    pub fn with_epoch(mut self, epoch: f64) -> Self {
        self.epoch = epoch;
        self
    }

    /// Sets the latest time the run will process.
    ///
    /// A negative or non-finite limit clears the limit rather than storing
    /// it, matching the engine, which ignores a limit it cannot use.
    ///
    /// 设置该运行将处理的最晚时刻。
    ///
    /// 负或非有限的限制清除该限制而非存储它，与引擎一致，后者忽略
    /// 它无法使用的限制。
    pub const fn with_time_limit(mut self, limit: f64) -> Self {
        self.time_limit = if limit.is_finite() && limit >= 0.0 {
            Some(limit)
        } else {
            None
        };
        self
    }

    /// Sets the greatest number of events the run will execute.
    ///
    /// A limit of zero clears the limit rather than storing it, matching the
    /// engine, which treats zero as no limit.
    ///
    /// 设置该运行将执行的事件数上限。
    ///
    /// 限制为零清除该限制而非存储它，与引擎一致，后者把零视为无限制。
    pub const fn with_event_limit(mut self, limit: u64) -> Self {
        self.event_limit = if limit == 0 { None } else { Some(limit) };
        self
    }

    /// Sets the epoch, reporting a value the clock cannot use.
    ///
    /// 设置历元，并报告时钟无法使用的值。
    pub fn set_epoch(&mut self, epoch: f64) -> Result<(), ScenarioError> {
        if !epoch.is_finite() || epoch < 0.0 {
            return Err(ScenarioError::new(ScenarioErrorKind::InvalidEpoch)
                .with_detail(format!("epoch {epoch}")));
        }
        self.epoch = epoch;
        Ok(())
    }

    /// Sets the time limit, reporting a value a run cannot use.
    ///
    /// 设置时限，并报告运行无法使用的值。
    pub fn set_time_limit(&mut self, limit: f64) -> Result<(), ScenarioError> {
        if !limit.is_finite() || limit < 0.0 {
            return Err(ScenarioError::new(ScenarioErrorKind::InvalidTimeLimit)
                .with_detail(format!("time limit {limit}")));
        }
        self.time_limit = Some(limit);
        Ok(())
    }

    /// Adds a resource, reporting one whose identifier is already used.
    ///
    /// The resource itself is also checked, so a scenario cannot hold a
    /// resource that a scheduler would refuse.
    ///
    /// 添加一个资源，并报告标识符已被使用的那种情况。
    ///
    /// 资源自身也受检查，故场景无法保存调度器会拒绝的资源。
    pub fn add_resource(&mut self, resource: Resource) -> Result<(), ScenarioError> {
        if !resource.has_valid_capacity() {
            return Err(
                ScenarioError::new(ScenarioErrorKind::InvalidResource).with_detail(format!(
                    "{} with capacity {}",
                    resource.id,
                    resource.capacity()
                )),
            );
        }
        if self
            .resources
            .iter()
            .any(|existing| existing.id == resource.id)
        {
            return Err(ScenarioError::new(ScenarioErrorKind::DuplicateResource)
                .with_detail(resource.id.to_string()));
        }
        self.resources.push(resource);
        Ok(())
    }

    /// Adds a task, reporting one whose identifier is already used.
    ///
    /// The task's requirements must name resources the scenario holds, and
    /// the task itself must be usable.
    ///
    /// 添加一个任务，并报告标识符已被使用的那种情况。
    ///
    /// 任务的需求必须指名场景所持有的资源，且任务自身必须可用。
    pub fn add_task(&mut self, task: Task) -> Result<(), ScenarioError> {
        if !task.is_valid() {
            return Err(
                ScenarioError::new(ScenarioErrorKind::InvalidTask).with_detail(task.id.to_string())
            );
        }
        if self.tasks.iter().any(|existing| existing.id == task.id) {
            return Err(ScenarioError::new(ScenarioErrorKind::DuplicateTask)
                .with_detail(task.id.to_string()));
        }
        for requirement in &task.requirements {
            if !self
                .resources
                .iter()
                .any(|resource| resource.id == requirement.resource)
            {
                return Err(ScenarioError::new(ScenarioErrorKind::UnknownResource)
                    .with_detail(requirement.resource.to_string()));
            }
        }
        self.tasks.push(task);
        Ok(())
    }

    /// Adds a precedence constraint, reporting a cycle or a missing task.
    ///
    /// 添加一条先后约束，并报告成环或缺失任务的情况。
    pub fn add_precedence(&mut self, before: TaskId, after: TaskId) -> Result<(), ScenarioError> {
        for id in [before, after] {
            if !self.tasks.iter().any(|task| task.id == id) {
                return Err(
                    ScenarioError::new(ScenarioErrorKind::UnknownTask).with_detail(id.to_string())
                );
            }
        }
        self.precedence
            .add(before, after)
            .map_err(|_| ScenarioError::new(ScenarioErrorKind::CyclicPrecedence))
    }

    /// Returns the resources as a set, for a scheduler.
    ///
    /// 返回资源集合，供调度器使用。
    pub fn resource_set(&self) -> Result<ResourceSet, ScenarioError> {
        ResourceSet::from_resources(self.resources.clone()).map_err(|_| {
            ScenarioError::new(ScenarioErrorKind::DuplicateResource).with_detail("resources")
        })
    }

    /// Checks the whole scenario at once.
    ///
    /// A scenario built with the builder methods is already checked as it is
    /// built; this is for one that was assembled field by field or read from
    /// text, and it reports the first problem found.
    ///
    /// 一次性检查整个场景。
    ///
    /// 用构建器方法构建的场景在构建时已被检查；本方法用于逐字段拼装或
    /// 从文本读取的场景，它报告所发现的第一个问题。
    pub fn validate(&self) -> Result<(), ScenarioError> {
        if self.name.trim().is_empty() {
            return Err(ScenarioError::new(ScenarioErrorKind::EmptyName));
        }
        if !self.epoch.is_finite() || self.epoch < 0.0 {
            return Err(ScenarioError::new(ScenarioErrorKind::InvalidEpoch)
                .with_detail(format!("epoch {}", self.epoch)));
        }
        if let Some(limit) = self.time_limit {
            if !limit.is_finite() || limit < 0.0 {
                return Err(ScenarioError::new(ScenarioErrorKind::InvalidTimeLimit)
                    .with_detail(format!("time limit {limit}")));
            }
        }
        for (position, resource) in self.resources.iter().enumerate() {
            if !resource.has_valid_capacity() {
                return Err(ScenarioError::new(ScenarioErrorKind::InvalidResource)
                    .with_detail(resource.id.to_string()));
            }
            if self.resources[..position]
                .iter()
                .any(|earlier| earlier.id == resource.id)
            {
                return Err(ScenarioError::new(ScenarioErrorKind::DuplicateResource)
                    .with_detail(resource.id.to_string()));
            }
        }
        for (position, task) in self.tasks.iter().enumerate() {
            if !task.is_valid() {
                return Err(ScenarioError::new(ScenarioErrorKind::InvalidTask)
                    .with_detail(task.id.to_string()));
            }
            if self.tasks[..position]
                .iter()
                .any(|earlier| earlier.id == task.id)
            {
                return Err(ScenarioError::new(ScenarioErrorKind::DuplicateTask)
                    .with_detail(task.id.to_string()));
            }
            for requirement in &task.requirements {
                if !self
                    .resources
                    .iter()
                    .any(|resource| resource.id == requirement.resource)
                {
                    return Err(ScenarioError::new(ScenarioErrorKind::UnknownResource)
                        .with_detail(requirement.resource.to_string()));
                }
            }
        }
        // A precedence graph cannot hold a cycle once built, because it
        // refuses one as it is added, but the tasks it names may still be
        // missing from the scenario.
        // 先后关系图一旦构建便不能含环，因为它在添加时即拒绝环，但它所
        // 指名的任务仍可能缺失于场景。
        for task in &self.tasks {
            for other in self.precedence.successors_of(task.id) {
                if !self.tasks.iter().any(|candidate| candidate.id == other) {
                    return Err(ScenarioError::new(ScenarioErrorKind::UnknownTask)
                        .with_detail(other.to_string()));
                }
            }
        }
        Ok(())
    }

    /// Returns a summary of what the scenario holds.
    ///
    /// 返回场景所持有内容的摘要。
    pub fn summary(&self) -> ScenarioSummary {
        ScenarioSummary {
            resources: self.resources.len(),
            tasks: self.tasks.len(),
            total_task_duration: self.tasks.iter().map(|task| task.duration).sum(),
            precedence_edges: self.precedence.edge_count(),
            total_availability: self
                .resources
                .iter()
                .map(|resource| resource.total_availability())
                .sum(),
        }
    }

    /// Renders the scenario in the text form [`Scenario::parse`] reads.
    ///
    /// The output parses to an equal scenario, which is what lets a scenario
    /// be stored and reloaded without drift.
    ///
    /// 以 [`Scenario::parse`] 所读取的文本形式呈现场景。
    ///
    /// 输出解析回相等的场景，这正是场景可被存储并重新加载而不漂移的
    /// 原因。
    pub fn to_text(&self) -> String {
        let mut text = String::new();
        text.push_str(&format!("scenario {}\n", self.name));
        text.push_str(&format!("epoch {}\n", self.epoch));
        if let Some(limit) = self.time_limit {
            text.push_str(&format!("time_limit {limit}\n"));
        }
        if let Some(limit) = self.event_limit {
            text.push_str(&format!("event_limit {limit}\n"));
        }

        for resource in &self.resources {
            text.push_str(&format!(
                "\nresource {} capacity {}\n",
                resource.id.index(),
                resource.capacity()
            ));
            for window in resource.windows() {
                text.push_str(&format!(
                    "window {} {} {}\n",
                    resource.id.index(),
                    window.start(),
                    window.end()
                ));
            }
        }

        for task in &self.tasks {
            text.push_str(&format!(
                "\ntask {} duration {}\n",
                task.id.index(),
                task.duration
            ));
            // A task with no window leaves both ends infinite, which the
            // text form omits rather than writing an infinity.
            // 没有窗口的任务使两端均为无限，文本形式省略它而不写出无限。
            if task.earliest_start.is_finite() || task.latest_end.is_finite() {
                let start = if task.earliest_start.is_finite() {
                    task.earliest_start
                } else {
                    0.0
                };
                let end = if task.latest_end.is_finite() {
                    task.latest_end
                } else {
                    start + task.duration
                };
                text.push_str(&format!("task_window {} {start} {end}\n", task.id.index()));
            }
            text.push_str(&format!(
                "task_priority {} {}\n",
                task.id.index(),
                task.priority
            ));
            if task.mandatory {
                text.push_str(&format!("task_mandatory {} true\n", task.id.index()));
            }
            for requirement in &task.requirements {
                text.push_str(&format!(
                    "require {} {} {}\n",
                    task.id.index(),
                    requirement.resource.index(),
                    requirement.demand
                ));
            }
        }

        let mut edges: Vec<(usize, usize)> = Vec::new();
        for task in &self.tasks {
            for successor in self.precedence.successors_of(task.id) {
                edges.push((task.id.index(), successor.index()));
            }
        }
        edges.sort_unstable();
        if !edges.is_empty() {
            text.push('\n');
        }
        for (before, after) in edges {
            text.push_str(&format!("precedence {before} {after}\n"));
        }

        text
    }
}

impl fmt::Display for Scenario {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let summary = self.summary();
        write!(
            f,
            "scenario {} with {} resource(s) and {} task(s)",
            self.name, summary.resources, summary.tasks
        )
    }
}

/// What a scenario holds, as counts and totals.
///
/// 场景所持有的内容，以计数与总计表示。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScenarioSummary {
    /// The number of resources.
    ///
    /// 资源的数量。
    pub resources: usize,
    /// The number of tasks.
    ///
    /// 任务的数量。
    pub tasks: usize,
    /// The sum of the task durations, in seconds.
    ///
    /// 各任务时长之和，单位为秒。
    pub total_task_duration: f64,
    /// The number of precedence constraints.
    ///
    /// 先后约束的数量。
    pub precedence_edges: usize,
    /// The sum of the resource availability, in seconds.
    ///
    /// 各资源可用时长之和，单位为秒。
    pub total_availability: f64,
}
