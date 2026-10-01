//! The conflict type and the report that collects them.

use apsis_resource::ResourceId;
use apsis_task::TaskId;
use core::fmt;

/// The kind of problem a conflict describes.
///
/// 冲突所描述的问题类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ConflictKind {
    /// Two placements need more of a resource than it has.
    ///
    /// 两处放置所需的资源超过其容量。
    Capacity,
    /// A placement falls outside the time window of its task.
    ///
    /// 某处放置落在其任务的时间窗口之外。
    Window,
    /// A placement does not agree with its task.
    ///
    /// 某处放置与其任务不一致。
    Requirement,
    /// A placement falls outside the availability of a resource.
    ///
    /// 某处放置落在某资源的可用性之外。
    Availability,
    /// A task that must follow another starts too early.
    ///
    /// 某任务未待其前驱完成即过早开始。
    Precedence,
}

impl fmt::Display for ConflictKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::Capacity => "capacity",
            Self::Window => "window",
            Self::Requirement => "requirement",
            Self::Availability => "availability",
            Self::Precedence => "precedence",
        };
        f.write_str(text)
    }
}

/// One problem found in a schedule.
///
/// The conflict names the tasks involved and, where it applies, the resource
/// and the instant at which the problem occurs. That is enough for an
/// operator to locate the placement without re-deriving the analysis.
///
/// 计划中发现的单个问题。
///
/// 冲突指明所涉任务，并在适用时指明资源及问题发生的时刻。这足以让操作
/// 者定位放置，而无须重新推导分析。
#[derive(Debug, Clone, PartialEq)]
pub struct Conflict {
    /// The kind of problem.
    ///
    /// 问题类别。
    pub kind: ConflictKind,
    /// The task the conflict is centred on.
    ///
    /// 该冲突所围绕的任务。
    pub task: TaskId,
    /// The other task involved, for a conflict between two placements.
    ///
    /// 所涉的另一个任务，用于两处放置之间的冲突。
    pub other: Option<TaskId>,
    /// The resource involved, where the conflict is about one.
    ///
    /// 所涉资源，当冲突与之相关时。
    pub resource: Option<ResourceId>,
    /// The time at which the problem occurs, in seconds.
    ///
    /// For a capacity conflict this is the instant at which the demand
    /// exceeds the capacity. For a window conflict it is the offending edge
    /// of the placement.
    ///
    /// 问题发生的时刻，单位为秒。
    ///
    /// 对容量冲突而言，是需求超出容量的那一瞬；对窗口冲突而言，是该
    /// 放置的越界之处。
    pub at: f64,
    /// A description of the problem.
    ///
    /// 该问题的说明。
    pub detail: String,
}

impl Conflict {
    /// Creates a conflict.
    ///
    /// 创建一个冲突。
    pub fn new(kind: ConflictKind, task: TaskId, at: f64, detail: impl Into<String>) -> Self {
        Self {
            kind,
            task,
            other: None,
            resource: None,
            at,
            detail: detail.into(),
        }
    }

    /// Returns a copy of this conflict involving another task.
    ///
    /// 返回涉及另一任务的本冲突副本。
    pub fn with_other(mut self, other: TaskId) -> Self {
        self.other = Some(other);
        self
    }

    /// Returns a copy of this conflict involving a resource.
    ///
    /// 返回涉及某资源的本冲突副本。
    pub fn with_resource(mut self, resource: ResourceId) -> Self {
        self.resource = Some(resource);
        self
    }

    /// Returns `true` when the conflict concerns a placement that should be
    /// moved rather than one that should be dropped.
    ///
    /// A capacity or availability conflict can often be repaired by moving
    /// the placement, while a requirement conflict means the placement does
    /// not belong in the schedule at all.
    ///
    /// 当该冲突涉及一处应当移动而非丢弃的放置时返回 `true`。
    ///
    /// 容量或可用性冲突常可通过移动放置来修复，而需求冲突意味着该
    /// 放置本就不应出现在计划中。
    pub const fn is_movable(&self) -> bool {
        matches!(
            self.kind,
            ConflictKind::Capacity | ConflictKind::Availability
        )
    }
}

impl fmt::Display for Conflict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} conflict on {} at {}: {}",
            self.kind, self.task, self.at, self.detail
        )?;
        if let Some(other) = self.other {
            write!(f, " (with {other})")?;
        }
        if let Some(resource) = self.resource {
            write!(f, " (on {resource})")?;
        }
        Ok(())
    }
}

/// The conflicts found in a schedule.
///
/// The report is ordered by time and then by kind, so that reading it
/// follows the plan. The counts by kind are provided because a planner
/// usually wants to know how many of each problem there are before deciding
/// what to do.
///
/// 计划中发现的所有冲突。
///
/// 报告按时间、再按类别排序，故阅读它即是在阅读该计划。各类计数一并
/// 提供，因为规划器通常希望先知道每类问题各有若干，再决定对策。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConflictReport {
    conflicts: Vec<Conflict>,
    offered_tasks: usize,
    placed_tasks: usize,
}

impl ConflictReport {
    /// Creates a report from a list of conflicts and the task counts.
    ///
    /// The conflicts are sorted by time, then by kind, then by task, so that
    /// the order does not depend on the order they were found.
    ///
    /// 由一组冲突与任务计数创建报告。
    ///
    /// 冲突按时间、类别、任务依次排序，故其次序不取决于发现它们的
    /// 顺序。
    pub fn new(mut conflicts: Vec<Conflict>, offered_tasks: usize, placed_tasks: usize) -> Self {
        conflicts.sort_by(|first, second| {
            first
                .at
                .partial_cmp(&second.at)
                .unwrap_or(core::cmp::Ordering::Equal)
                .then_with(|| kind_order(first.kind).cmp(&kind_order(second.kind)))
                .then_with(|| first.task.cmp(&second.task))
                .then_with(|| first.other.cmp(&second.other))
        });
        Self {
            conflicts,
            offered_tasks,
            placed_tasks,
        }
    }

    /// Returns the conflicts, in time order.
    ///
    /// 返回各冲突，按时间排序。
    pub fn conflicts(&self) -> &[Conflict] {
        &self.conflicts
    }

    /// Returns the number of conflicts.
    ///
    /// 返回冲突数量。
    pub fn len(&self) -> usize {
        self.conflicts.len()
    }

    /// Returns `true` when there are no conflicts.
    ///
    /// This is the conventional spelling of [`ConflictReport::is_clear`], and
    /// the two always agree.
    ///
    /// 当没有冲突时返回 `true`。
    ///
    /// 这是 [`ConflictReport::is_clear`] 的惯用拼写，二者始终一致。
    pub fn is_empty(&self) -> bool {
        self.conflicts.is_empty()
    }

    /// Returns `true` when there are no conflicts.
    ///
    /// A clear report means the schedule is feasible with respect to every
    /// check this crate performs, not that it is optimal.
    ///
    /// 当没有冲突时返回 `true`。
    ///
    /// 报告无冲突意味着该计划就本 crate 执行的每项检查而言是可行的，
    /// 并不意味着它最优。
    pub fn is_clear(&self) -> bool {
        self.conflicts.is_empty()
    }

    /// Returns `true` when there is at least one conflict.
    ///
    /// 当至少存在一个冲突时返回 `true`。
    pub fn has_conflicts(&self) -> bool {
        !self.conflicts.is_empty()
    }

    /// Returns the number of conflicts of one kind.
    ///
    /// 返回某一类别的冲突数量。
    pub fn count_of(&self, kind: ConflictKind) -> usize {
        self.conflicts
            .iter()
            .filter(|conflict| conflict.kind == kind)
            .count()
    }

    /// Returns the conflicts of one kind, in time order.
    ///
    /// 返回某一类别的冲突，按时间排序。
    pub fn of_kind(&self, kind: ConflictKind) -> Vec<&Conflict> {
        self.conflicts
            .iter()
            .filter(|conflict| conflict.kind == kind)
            .collect()
    }

    /// Returns the tasks involved in at least one conflict.
    ///
    /// A task appears once even when it has several conflicts, which is what
    /// a repair pass wants: the set of tasks to reconsider.
    ///
    /// 返回至少卷入一个冲突的任务。
    ///
    /// 同一任务即使有多个冲突也只出现一次，这正是一个修复轮所需：待重新
    /// 考虑的任务集合。
    pub fn conflicting_tasks(&self) -> Vec<TaskId> {
        let mut tasks: Vec<TaskId> = Vec::new();
        for conflict in &self.conflicts {
            if !tasks.contains(&conflict.task) {
                tasks.push(conflict.task);
            }
            if let Some(other) = conflict.other {
                if !tasks.contains(&other) {
                    tasks.push(other);
                }
            }
        }
        tasks
    }

    /// Returns the number of tasks that were offered to the schedule.
    ///
    /// 返回提交给该计划的任务数量。
    pub fn offered_tasks(&self) -> usize {
        self.offered_tasks
    }

    /// Returns the number of tasks that were placed.
    ///
    /// 返回已被放置的任务数量。
    pub fn placed_tasks(&self) -> usize {
        self.placed_tasks
    }

    /// Returns a one line summary of the report.
    ///
    /// 返回该报告的单行摘要。
    pub fn summary(&self) -> String {
        format!(
            "{} conflict(s) among {} placed of {} offered task(s)",
            self.conflicts.len(),
            self.placed_tasks,
            self.offered_tasks
        )
    }
}

/// Returns a sort key that groups the kinds in a readable order.
///
/// The order runs from the problems that invalidate a plan to the ones that
/// merely make it expensive, so that reading the report top to bottom
/// addresses the most serious problems first.
///
/// 返回使各类别按可读次序分组的排序键。
///
/// 该次序从使计划失效的问题排到仅使计划昂贵的问题，故自顶向下阅读报告
/// 时先处理最严重者。
const fn kind_order(kind: ConflictKind) -> u8 {
    match kind {
        ConflictKind::Requirement => 0,
        ConflictKind::Window => 1,
        ConflictKind::Capacity => 2,
        ConflictKind::Availability => 3,
        ConflictKind::Precedence => 4,
    }
}
