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
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the time the simulation clock starts at, in seconds.
    pub fn epoch(&self) -> f64 {
        self.epoch
    }

    /// Returns the latest time the run will process, if one is set.
    pub fn time_limit(&self) -> Option<f64> {
        self.time_limit
    }

    /// Returns the greatest number of events the run will execute, if one is
    /// set.
    pub const fn event_limit(&self) -> Option<u64> {
        self.event_limit
    }

    /// Returns the resources in the order they were added.
    pub fn resources(&self) -> &[Resource] {
        &self.resources
    }

    /// Returns the resources for modification.
    ///
    /// A caller that changes a resource is responsible for keeping the
    /// scenario consistent; [`Scenario::validate`] checks it afterwards.
    pub fn resources_mut(&mut self) -> &mut Vec<Resource> {
        &mut self.resources
    }

    /// Reads a scenario from its text form.
    ///
    /// The format is described on the crate documentation. A scenario written
    /// with [`Scenario::to_text`] parses back to an equal scenario.
    pub fn parse(text: &str) -> Result<Self, ScenarioError> {
        crate::parse::parse(text)
    }

    /// Returns the tasks in the order they were added.
    pub fn tasks(&self) -> &[Task] {
        &self.tasks
    }

    /// Returns the precedence constraints.
    pub const fn precedence(&self) -> &PrecedenceGraph {
        &self.precedence
    }

    /// Sets the time the simulation clock starts at.
    ///
    /// Returns the scenario unchanged in value but not in type; the caller
    /// uses the returned value. The epoch must be finite and non-negative.
    pub fn with_epoch(mut self, epoch: f64) -> Self {
        self.epoch = epoch;
        self
    }

    /// Sets the latest time the run will process.
    ///
    /// A negative or non-finite limit clears the limit rather than storing
    /// it, matching the engine, which ignores a limit it cannot use.
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
    pub const fn with_event_limit(mut self, limit: u64) -> Self {
        self.event_limit = if limit == 0 { None } else { Some(limit) };
        self
    }

    /// Sets the epoch, reporting a value the clock cannot use.
    pub fn set_epoch(&mut self, epoch: f64) -> Result<(), ScenarioError> {
        if !epoch.is_finite() || epoch < 0.0 {
            return Err(ScenarioError::new(ScenarioErrorKind::InvalidEpoch)
                .with_detail(format!("epoch {epoch}")));
        }
        self.epoch = epoch;
        Ok(())
    }

    /// Sets the time limit, reporting a value a run cannot use.
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScenarioSummary {
    /// The number of resources.
    pub resources: usize,
    /// The number of tasks.
    pub tasks: usize,
    /// The sum of the task durations, in seconds.
    pub total_task_duration: f64,
    /// The number of precedence constraints.
    pub precedence_edges: usize,
    /// The sum of the resource availability, in seconds.
    pub total_availability: f64,
}
