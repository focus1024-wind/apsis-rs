//! Tasks, their requirements, and the set that holds them.
//!
//! 任务、其需求，以及保存它们的集合。

use crate::error::{TaskError, TaskErrorKind};
use apsis_resource::ResourceId;

/// An identifier for a task.
///
/// 任务的标识符。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TaskId(pub usize);

impl TaskId {
    /// Creates an identifier.
    ///
    /// 创建一个标识符。
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the index.
    ///
    /// 返回该下标。
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
///
/// 任务对某个资源的需求。
///
/// 需求是任务运行期间消耗的资源容量大小。超出资源容量的需求使该任务无法在
/// 此处调度，调度器通过比较二者即可发现。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Requirement {
    /// The resource required.
    ///
    /// 所要求的资源。
    pub resource: ResourceId,
    /// The amount of the resource consumed.
    ///
    /// 消耗的资源量。
    pub demand: f64,
}

impl Requirement {
    /// Creates a requirement.
    ///
    /// 创建一条需求。
    pub const fn new(resource: ResourceId, demand: f64) -> Self {
        Self { resource, demand }
    }

    /// Returns `true` when the demand is positive and finite.
    ///
    /// 当该需求为正且有限时返回 `true`。
    pub fn is_valid(&self) -> bool {
        self.demand.is_finite() && self.demand > 0.0
    }
}

/// A unit of work to be scheduled.
///
/// A task is immutable after construction. The builder methods take `self` by
/// value and return a new task, so a caller can share a base task and derive
/// variants from it without the variants interfering.
///
/// 待调度的一份工作。
///
/// 任务在构造后不可变。各构建方法按值接收 `self` 并返回新任务，故调用方
/// 可以共享一个基础任务并由它派生出若干变体，而变体之间互不干扰。
#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    /// The identifier of this task.
    ///
    /// 该任务的标识符。
    pub id: TaskId,
    /// The duration of the task, in seconds.
    ///
    /// 任务时长，单位为秒。
    pub duration: f64,
    /// The earliest time at which the task may start, in seconds.
    ///
    /// This is the start of the earliest window in which the task may run.
    /// A task with no window has no restriction, which is what the
    /// constructors give.
    ///
    /// 该任务可开始的最早时刻，单位为秒。
    ///
    /// 即该任务可运行的最早窗口的起点。没有窗口的任务不受限制，构造函数
    /// 给出的正是这种情形。
    pub earliest_start: f64,
    /// The latest time at which the task may end, in seconds.
    ///
    /// A task with no window has no restriction.
    ///
    /// 该任务可结束的最晚时刻，单位为秒。
    ///
    /// 没有窗口的任务不受限制。
    pub latest_end: f64,
    /// The resources the task requires, in the order they were added.
    ///
    /// 该任务要求的资源，按添加顺序排列。
    pub requirements: Vec<Requirement>,
    /// The priority of the task.
    ///
    /// A larger value means the task is more important. The scale is the
    /// caller's. A scheduler that maximises weighted value uses this number,
    /// and one that maximises the count of scheduled tasks ignores it.
    ///
    /// 该任务的优先级。
    ///
    /// 值越大表示任务越重要，量纲由调用方确定。最大化加权价值的调度器使用
    /// 该数值，而最大化已调度任务数的调度器则忽略它。
    pub priority: f64,
    /// Whether the task must be scheduled.
    ///
    /// A mandatory task is scheduled even at the cost of other tasks. A
    /// scheduler may still fail to place one when no placement exists, and
    /// reports that rather than dropping it silently.
    ///
    /// 该任务是否必须被调度。
    ///
    /// 强制任务即使以牺牲其他任务为代价也要调度。当不存在可行放置时调度器
    /// 仍可能无法安排它，此时会报告，而不是静默丢弃。
    pub mandatory: bool,
}

impl Task {
    /// Creates a task with the given duration and no requirements.
    ///
    /// The task may start at any finite time, because it has no window. It
    /// has a priority of zero and is not mandatory.
    ///
    /// 以给定时长、无资源需求创建任务。
    ///
    /// 该任务没有窗口，故可在任意有限时刻开始。其优先级为零，且非强制。
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
    ///
    /// 返回带时间窗口的本任务副本。
    ///
    /// 该窗口是整个任务必须容纳其中的区间，故任务起点不早于窗口起点、终点
    /// 不晚于窗口终点。
    pub fn with_window(mut self, start: f64, end: f64) -> Self {
        self.earliest_start = start;
        self.latest_end = end;
        self
    }

    /// Returns a copy of this task with a priority.
    ///
    /// 返回带给定优先级的本任务副本。
    pub fn with_priority(mut self, priority: f64) -> Self {
        self.priority = priority;
        self
    }

    /// Returns a copy of this task marked mandatory or not.
    ///
    /// 返回标记为强制或非强制的本任务副本。
    pub fn with_mandatory(mut self, mandatory: bool) -> Self {
        self.mandatory = mandatory;
        self
    }

    /// Returns a copy of this task with an additional resource requirement.
    ///
    /// Adding the same resource twice is allowed at this point, because a
    /// builder chain cannot report an error. [`Task::validate`] rejects it,
    /// and [`Task::try_require`] reports it immediately.
    ///
    /// 返回附加一条资源需求的本任务副本。
    ///
    /// 此处允许重复加入同一资源，因为构建者链无法报告错误。
    /// [`Task::validate`] 会拒绝它，[`Task::try_require`] 则立即报告。
    pub fn require(mut self, resource: ResourceId, demand: f64) -> Self {
        self.requirements.push(Requirement::new(resource, demand));
        self
    }

    /// Returns a copy of this task with an additional resource requirement,
    /// reporting a duplicate or an invalid demand immediately.
    ///
    /// 返回附加一条资源需求的本任务副本，并立即报告重复或无效需求。
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
    ///
    /// 返回该任务对某资源的需求。
    ///
    /// 该任务不要求该资源时返回 `None`。
    pub fn demand_for(&self, resource: ResourceId) -> Option<f64> {
        self.requirements
            .iter()
            .find(|requirement| requirement.resource == resource)
            .map(|requirement| requirement.demand)
    }

    /// Returns `true` when the task requires a resource.
    ///
    /// 当该任务要求某资源时返回 `true`。
    pub fn requires(&self, resource: ResourceId) -> bool {
        self.demand_for(resource).is_some()
    }

    /// Returns the number of resources the task requires.
    ///
    /// 返回该任务要求的资源数量。
    pub fn requirement_count(&self) -> usize {
        self.requirements.len()
    }

    /// Returns the identifiers of the required resources.
    ///
    /// 返回所要求资源的标识符。
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
    ///
    /// 返回该任务必须容纳其中的窗口长度，单位为秒。
    ///
    /// 任务两端均无限制时返回无穷；窗口完全容不下该任务时返回零或负值。
    pub fn window_length(&self) -> f64 {
        self.latest_end - self.earliest_start
    }

    /// Returns `true` when the window cannot hold the task.
    ///
    /// 当窗口容不下该任务时返回 `true`。
    pub fn window_is_too_short(&self) -> bool {
        self.duration > self.window_length()
    }

    /// Returns the earliest time the task could end, given its window.
    ///
    /// Returns infinity when the task has no upper bound, and negative
    /// infinity when it has no lower bound.
    ///
    /// 返回在其窗口下该任务可能结束的最早时刻。
    ///
    /// 任务无上界时返回无穷，无下界时返回负无穷。
    pub fn earliest_possible_end(&self) -> f64 {
        self.earliest_start + self.duration
    }

    /// Returns `true` when the task fits its window.
    ///
    /// 当该任务能容纳于其窗口时返回 `true`。
    pub fn fits_window(&self) -> bool {
        !self.window_is_too_short()
    }

    /// Returns `true` when the task uses any resource more than once, or
    /// demands a non-positive amount.
    ///
    /// 当该任务重复使用某资源，或需求非正时返回 `true`。
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
    ///
    /// 校验该任务；不可用时返回原因。
    ///
    /// 任务可用当且仅当：时长有限且为正；窗口在其给出的两端有限并容得下该
    /// 时长；每条需求为正且有限并指向互不相同的资源；各权值有限。
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
    ///
    /// 当该任务可用时返回 `true`。
    pub fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }

    /// Returns the value of the task, which is its priority weighted by its
    /// duration.
    ///
    /// This is the quantity a scheduler maximises when it is asked to make
    /// the most of the available time: a long task of high priority is worth
    /// more than a short one of the same priority.
    ///
    /// 返回该任务的价值，即其优先级与时长的乘积。
    ///
    /// 当调度器被要求充分利用可用时间时，最大化的正是该量：同等优先级下
    /// 时长较长的任务价值更高。
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
///
/// 调度器为某任务指定的状态。
///
/// 状态与任务分开保存，故同一任务可被考虑多次放置而不被修改。已放置的
/// 任务记录了何时何地，从而无需二次查找即可检查一份计划。
#[derive(Debug, Clone, PartialEq)]
pub enum TaskState {
    /// The task has not been placed and is still being considered.
    ///
    /// 该任务尚未放置，仍在考虑之中。
    Pending,
    /// The task has been placed at a time.
    ///
    /// 该任务已在某时刻放置。
    Scheduled {
        /// The start time of the placement, in seconds.
        ///
        /// 放置的起始时刻，单位为秒。
        start: f64,
    },
    /// The task was deliberately left out.
    ///
    /// A scheduler reports this rather than silently dropping a task, so that
    /// an operator can see what was given up and why.
    ///
    /// 该任务被有意排除在外。
    ///
    /// 调度器报告此状态而非静默丢弃任务，使操作者能看出放弃了什么以及
    /// 为什么。
    Deferred,
    /// The task cannot be scheduled at all.
    ///
    /// A task is infeasible when its window is too short, when it demands
    /// more of a resource than the resource has, or when a resource it needs
    /// does not exist.
    ///
    /// 该任务完全无法调度。
    ///
    /// 任务不可行的情形有：窗口过短；需求超过某资源容量；所需资源不存在。
    Infeasible {
        /// A description of why the task cannot be scheduled.
        ///
        /// 该任务无法调度的原因说明。
        reason: String,
    },
}

impl TaskState {
    /// Returns `true` when the task has been placed.
    ///
    /// 当该任务已被放置时返回 `true`。
    pub const fn is_scheduled(&self) -> bool {
        matches!(self, Self::Scheduled { .. })
    }

    /// Returns the start time of the placement, if there is one.
    ///
    /// 返回放置的起始时刻，若有。
    pub const fn start(&self) -> Option<f64> {
        match self {
            Self::Scheduled { start } => Some(*start),
            _ => None,
        }
    }

    /// Returns the end time of the placement, if there is one.
    ///
    /// 返回放置的结束时刻，若有。
    pub fn end(&self, duration: f64) -> Option<f64> {
        self.start().map(|start| start + duration)
    }

    /// Returns a short name for the state.
    ///
    /// 返回该状态的简称。
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
///
/// 以标识符为索引的任务集合。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TaskSet {
    tasks: Vec<Task>,
}

impl TaskSet {
    /// Creates an empty set.
    ///
    /// 创建一个空集合。
    pub const fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    /// Creates a set from a list of tasks.
    ///
    /// Returns an error when two tasks share an identifier or when any task
    /// is not usable.
    ///
    /// 由一组任务创建集合。
    ///
    /// 两个任务标识符相同或任一任务不可用时返回错误。
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
    ///
    /// 插入一个任务。
    ///
    /// 该标识符已存在或该任务不可用时返回错误。
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
    ///
    /// 返回具有所给标识符的任务。
    pub fn get(&self, id: TaskId) -> Option<&Task> {
        self.tasks.iter().find(|task| task.id == id)
    }

    /// Removes a task.
    ///
    /// Returns an error when no task has the identifier.
    ///
    /// 移除一个任务。
    ///
    /// 没有任务具有该标识符时返回错误。
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
    ///
    /// 返回按添加顺序排列的任务。
    pub fn tasks(&self) -> &[Task] {
        &self.tasks
    }

    /// Returns the number of tasks.
    ///
    /// 返回任务数量。
    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    /// Returns `true` when there are no tasks.
    ///
    /// 当没有任务时返回 `true`。
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    /// Returns the total duration of every task, in seconds.
    ///
    /// This is the load the tasks would place on the schedule if each used
    /// one unit of resource for its whole duration.
    ///
    /// 返回所有任务的总时长，单位为秒。
    ///
    /// 这是各任务在整个时长内各占一个单位资源时，会施加到计划上的负载。
    pub fn total_duration(&self) -> f64 {
        self.tasks.iter().map(|task| task.duration).sum()
    }

    /// Returns the total weighted value of every task.
    ///
    /// 返回所有任务的总加权价值。
    pub fn total_weighted_value(&self) -> f64 {
        self.tasks.iter().map(Task::weighted_value).sum()
    }

    /// Returns the identifiers of the tasks that require a resource.
    ///
    /// 返回要求某资源的任务标识符。
    pub fn tasks_requiring(&self, resource: ResourceId) -> Vec<TaskId> {
        self.tasks
            .iter()
            .filter(|task| task.requires(resource))
            .map(|task| task.id)
            .collect()
    }

    /// Returns the identifiers of the mandatory tasks.
    ///
    /// 返回强制任务的标识符。
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
///
/// 返回适合贪心调度器的任务次序。
///
/// 该次序按加权价值递减排列，这是一个有效启发式：先放置贡献最大的任务，
/// 把价值最小的留给填补空隙。平局先按更早的截止时刻、再按标识符打破，故
/// 对给定集合次序是确定的。
///
/// 它不保证找到最优计划。带资源约束的调度在一般情况下不可解，故启发式
/// 是现实的选择，调用方亦可自行提供次序。
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
