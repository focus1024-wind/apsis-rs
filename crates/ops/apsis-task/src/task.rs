//! Tasks, their requirements, and the set that holds them.

use crate::error::{TaskError, TaskErrorKind};
use apsis_resource::ResourceId;

/// An identifier for a task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TaskId(pub usize);

impl TaskId {
    /// Creates an identifier.
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the index.
    pub const fn index(self) -> usize {
        self.0
    }
}

impl core::fmt::Display for TaskId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "task {}", self.0)
    }
}

/// A demand that a task makes on one resource.
///
/// The demand is the amount of the resource's capacity the task consumes
/// while it runs. A demand that exceeds the resource's capacity makes the
/// task impossible to schedule there, which a scheduler discovers by
/// comparing the two.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Requirement {
    /// The resource required.
    pub resource: ResourceId,
    /// The amount of the resource consumed.
    pub demand: f64,
}

impl Requirement {
    /// Creates a requirement.
    pub const fn new(resource: ResourceId, demand: f64) -> Self {
        Self { resource, demand }
    }

    /// Returns `true` when the demand is positive and finite.
    pub fn is_valid(&self) -> bool {
        self.demand.is_finite() && self.demand > 0.0
    }
}

/// A unit of work to be scheduled.
///
/// A task is immutable after construction. The builder methods take `self` by
/// value and return a new task, so a caller can share a base task and derive
/// variants from it without the variants interfering.
#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    /// The identifier of this task.
    pub id: TaskId,
    /// The duration of the task, in seconds.
    pub duration: f64,
    /// The earliest time at which the task may start, in seconds.
    ///
    /// This is the start of the earliest window in which the task may run.
    /// A task with no window has no restriction, which is what the
    /// constructors give.
    pub earliest_start: f64,
    /// The latest time at which the task may end, in seconds.
    ///
    /// A task with no window has no restriction.
    pub latest_end: f64,
    /// The resources the task requires, in the order they were added.
    pub requirements: Vec<Requirement>,
    /// The priority of the task.
    ///
    /// A larger value means the task is more important. The scale is the
    /// caller's. A scheduler that maximises weighted value uses this number,
    /// and one that maximises the count of scheduled tasks ignores it.
    pub priority: f64,
    /// Whether the task must be scheduled.
    ///
    /// A mandatory task is scheduled even at the cost of other tasks. A
    /// scheduler may still fail to place one when no placement exists, and
    /// reports that rather than dropping it silently.
    pub mandatory: bool,
}

impl Task {
    /// Creates a task with the given duration and no requirements.
    ///
    /// The task may start at any finite time, because it has no window. It
    /// has a priority of zero and is not mandatory.
    pub fn new(id: TaskId, duration: f64) -> Self {
        Self {
            id,
            duration,
            earliest_start: f64::NEG_INFINITY,
            latest_end: f64::INFINITY,
            requirements: Vec::new(),
            priority: 0.0,
            mandatory: false,
        }
    }

    /// Returns a copy of this task with a time window.
    ///
    /// The window is the interval in which the whole task must fit, so the
    /// task may start no earlier than the window start and must end no later
    /// than the window end.
    pub fn with_window(mut self, start: f64, end: f64) -> Self {
        self.earliest_start = start;
        self.latest_end = end;
        self
    }

    /// Returns a copy of this task with a priority.
    pub fn with_priority(mut self, priority: f64) -> Self {
        self.priority = priority;
        self
    }

    /// Returns a copy of this task marked mandatory or not.
    pub fn with_mandatory(mut self, mandatory: bool) -> Self {
        self.mandatory = mandatory;
        self
    }

    /// Returns a copy of this task with an additional resource requirement.
    ///
    /// Adding the same resource twice is allowed at this point, because a
    /// builder chain cannot report an error. [`Task::validate`] rejects it,
    /// and [`Task::try_require`] reports it immediately.
    pub fn require(mut self, resource: ResourceId, demand: f64) -> Self {
        self.requirements.push(Requirement::new(resource, demand));
        self
    }

    /// Returns a copy of this task with an additional resource requirement,
    /// reporting a duplicate or an invalid demand immediately.
    pub fn try_require(mut self, resource: ResourceId, demand: f64) -> Result<Self, TaskError> {
        let requirement = Requirement::new(resource, demand);
        if !requirement.is_valid() {
            return Err(TaskError::new(TaskErrorKind::InvalidDemand)
                .with_detail(format!("{resource} with demand {demand}")));
        }
        if self
            .requirements
            .iter()
            .any(|existing| existing.resource == resource)
        {
            return Err(TaskError::new(TaskErrorKind::DuplicateRequirement)
                .with_detail(format!("{resource}")));
        }
        self.requirements.push(requirement);
        Ok(self)
    }

    /// Returns the demand the task makes on a resource.
    ///
    /// Returns `None` when the task does not require the resource.
    pub fn demand_for(&self, resource: ResourceId) -> Option<f64> {
        self.requirements
            .iter()
            .find(|requirement| requirement.resource == resource)
            .map(|requirement| requirement.demand)
    }

    /// Returns `true` when the task requires a resource.
    pub fn requires(&self, resource: ResourceId) -> bool {
        self.demand_for(resource).is_some()
    }

    /// Returns the number of resources the task requires.
    pub fn requirement_count(&self) -> usize {
        self.requirements.len()
    }

    /// Returns the identifiers of the required resources.
    pub fn resources(&self) -> Vec<ResourceId> {
        self.requirements
            .iter()
            .map(|requirement| requirement.resource)
            .collect()
    }

    /// Returns the length of the window in which the task must fit, in
    /// seconds.
    ///
    /// Returns infinity when the task has no restriction at either end, and
    /// zero or a negative value when the window cannot hold the task at all.
    pub fn window_length(&self) -> f64 {
        self.latest_end - self.earliest_start
    }

    /// Returns `true` when the window cannot hold the task.
    pub fn window_is_too_short(&self) -> bool {
        self.duration > self.window_length()
    }

    /// Returns the earliest time the task could end, given its window.
    ///
    /// Returns infinity when the task has no upper bound, and negative
    /// infinity when it has no lower bound.
    pub fn earliest_possible_end(&self) -> f64 {
        self.earliest_start + self.duration
    }

    /// Returns `true` when the task fits its window.
    pub fn fits_window(&self) -> bool {
        !self.window_is_too_short()
    }

    /// Returns `true` when the task uses any resource more than once, or
    /// demands a non-positive amount.
    pub fn has_duplicate_requirements(&self) -> bool {
        self.requirements
            .iter()
            .enumerate()
            .any(|(index, requirement)| {
                self.requirements[..index]
                    .iter()
                    .any(|earlier| earlier.resource == requirement.resource)
            })
    }

    /// Validates the task, returning the reason when it is not usable.
    ///
    /// A task is usable when its duration is finite and positive, its window
    /// is finite at the ends it states and holds the duration, every demand
    /// is positive and finite and names a distinct resource, and the weights
    /// are finite.
    pub fn validate(&self) -> Result<(), TaskError> {
        if !self.duration.is_finite() || self.duration <= 0.0 {
            return Err(TaskError::new(TaskErrorKind::InvalidDuration)
                .with_detail(format!("duration {}", self.duration)));
        }
        if self.earliest_start.is_nan() || self.latest_end.is_nan() {
            return Err(
                TaskError::new(TaskErrorKind::InvalidWindow).with_detail(format!(
                    "window [{}, {}]",
                    self.earliest_start, self.latest_end
                )),
            );
        }
        if self.latest_end < self.earliest_start {
            return Err(
                TaskError::new(TaskErrorKind::InvalidWindow).with_detail(format!(
                    "window [{}, {}] runs backwards",
                    self.earliest_start, self.latest_end
                )),
            );
        }
        if self.window_is_too_short() {
            return Err(
                TaskError::new(TaskErrorKind::WindowTooShort).with_detail(format!(
                    "duration {} does not fit the window [{}, {}]",
                    self.duration, self.earliest_start, self.latest_end
                )),
            );
        }
        for requirement in &self.requirements {
            if !requirement.is_valid() {
                return Err(
                    TaskError::new(TaskErrorKind::InvalidDemand).with_detail(format!(
                        "{} with demand {}",
                        requirement.resource, requirement.demand
                    )),
                );
            }
        }
        if self.has_duplicate_requirements() {
            return Err(TaskError::new(TaskErrorKind::DuplicateRequirement));
        }
        if !self.priority.is_finite() {
            return Err(TaskError::new(TaskErrorKind::InvalidWeight)
                .with_detail(format!("priority {}", self.priority)));
        }
        Ok(())
    }

    /// Returns `true` when the task is usable.
    pub fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }

    /// Returns the value of the task, which is its priority weighted by its
    /// duration.
    ///
    /// This is the quantity a scheduler maximises when it is asked to make
    /// the most of the available time: a long task of high priority is worth
    /// more than a short one of the same priority.
    pub fn weighted_value(&self) -> f64 {
        self.priority * self.duration
    }
}

impl core::fmt::Display for Task {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{} of {} s requiring {} resource(s)",
            self.id,
            self.duration,
            self.requirements.len()
        )
    }
}

/// The state a scheduler has assigned to a task.
///
/// The state is kept apart from the task so that the same task can be
/// considered for several placements without being modified. A task that has
/// been placed records when and where, so that a schedule can be examined
/// without a second lookup.
#[derive(Debug, Clone, PartialEq)]
pub enum TaskState {
    /// The task has not been placed and is still being considered.
    Pending,
    /// The task has been placed at a time.
    Scheduled {
        /// The start time of the placement, in seconds.
        start: f64,
    },
    /// The task was deliberately left out.
    ///
    /// A scheduler reports this rather than silently dropping a task, so that
    /// an operator can see what was given up and why.
    Deferred,
    /// The task cannot be scheduled at all.
    ///
    /// A task is infeasible when its window is too short, when it demands
    /// more of a resource than the resource has, or when a resource it needs
    /// does not exist.
    Infeasible {
        /// A description of why the task cannot be scheduled.
        reason: String,
    },
}

impl TaskState {
    /// Returns `true` when the task has been placed.
    pub const fn is_scheduled(&self) -> bool {
        matches!(self, Self::Scheduled { .. })
    }

    /// Returns the start time of the placement, if there is one.
    pub const fn start(&self) -> Option<f64> {
        match self {
            Self::Scheduled { start } => Some(*start),
            _ => None,
        }
    }

    /// Returns the end time of the placement, if there is one.
    pub fn end(&self, duration: f64) -> Option<f64> {
        self.start().map(|start| start + duration)
    }

    /// Returns a short name for the state.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Scheduled { .. } => "scheduled",
            Self::Deferred => "deferred",
            Self::Infeasible { .. } => "infeasible",
        }
    }
}

/// A collection of tasks, indexed by identifier.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TaskSet {
    tasks: Vec<Task>,
}

impl TaskSet {
    /// Creates an empty set.
    pub const fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    /// Creates a set from a list of tasks.
    ///
    /// Returns an error when two tasks share an identifier or when any task
    /// is not usable.
    pub fn from_tasks(tasks: Vec<Task>) -> Result<Self, TaskError> {
        let mut set = Self::new();
        for task in tasks {
            set.insert(task)?;
        }
        Ok(set)
    }

    /// Inserts a task.
    ///
    /// Returns an error when the identifier is already present or when the
    /// task is not usable.
    pub fn insert(&mut self, task: Task) -> Result<(), TaskError> {
        task.validate()?;
        if self.get(task.id).is_some() {
            return Err(
                TaskError::new(TaskErrorKind::DuplicateTask).with_detail(format!("{}", task.id))
            );
        }
        self.tasks.push(task);
        Ok(())
    }

    /// Returns the task with the given identifier.
    pub fn get(&self, id: TaskId) -> Option<&Task> {
        self.tasks.iter().find(|task| task.id == id)
    }

    /// Removes a task.
    ///
    /// Returns an error when no task has the identifier.
    pub fn remove(&mut self, id: TaskId) -> Result<Task, TaskError> {
        let index = self
            .tasks
            .iter()
            .position(|task| task.id == id)
            .ok_or_else(|| {
                TaskError::new(TaskErrorKind::NoSuchTask).with_detail(format!("{id}"))
            })?;
        Ok(self.tasks.remove(index))
    }

    /// Returns the tasks in the order they were added.
    pub fn tasks(&self) -> &[Task] {
        &self.tasks
    }

    /// Returns the number of tasks.
    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    /// Returns `true` when there are no tasks.
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    /// Returns the total duration of every task, in seconds.
    ///
    /// This is the load the tasks would place on the schedule if each used
    /// one unit of resource for its whole duration.
    pub fn total_duration(&self) -> f64 {
        self.tasks.iter().map(|task| task.duration).sum()
    }

    /// Returns the total weighted value of every task.
    pub fn total_weighted_value(&self) -> f64 {
        self.tasks.iter().map(Task::weighted_value).sum()
    }

    /// Returns the identifiers of the tasks that require a resource.
    pub fn tasks_requiring(&self, resource: ResourceId) -> Vec<TaskId> {
        self.tasks
            .iter()
            .filter(|task| task.requires(resource))
            .map(|task| task.id)
            .collect()
    }

    /// Returns the identifiers of the mandatory tasks.
    pub fn mandatory(&self) -> Vec<TaskId> {
        self.tasks
            .iter()
            .filter(|task| task.mandatory)
            .map(|task| task.id)
            .collect()
    }
}

/// Returns a task ordering suitable for a greedy scheduler.
///
/// The order is by decreasing weighted value, which is an efficient
/// heuristic: placing the tasks that contribute most first leaves the least
/// valuable ones to fill the gaps. Ties are broken by the earlier deadline
/// and then by the identifier, so that the order is deterministic for a given
/// set.
///
/// This is not guaranteed to find the best possible schedule. Scheduling
/// with resource constraints is intractable in general, so a heuristic is the
/// practical choice and the caller may supply its own order instead.
pub fn task_order(tasks: &[Task]) -> Vec<TaskId> {
    let mut ordered: Vec<&Task> = tasks.iter().collect();
    ordered.sort_by(|first, second| {
        second
            .weighted_value()
            .partial_cmp(&first.weighted_value())
            .unwrap_or(core::cmp::Ordering::Equal)
            .then_with(|| {
                first
                    .latest_end
                    .partial_cmp(&second.latest_end)
                    .unwrap_or(core::cmp::Ordering::Equal)
            })
            .then_with(|| first.id.cmp(&second.id))
    });
    ordered.into_iter().map(|task| task.id).collect()
}
