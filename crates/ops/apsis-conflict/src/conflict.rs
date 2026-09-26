//! The conflict type and the report that collects them.

use apsis_resource::ResourceId;
use apsis_task::TaskId;
use core::fmt;

/// The kind of problem a conflict describes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ConflictKind {
    /// Two placements need more of a resource than it has.
    Capacity,
    /// A placement falls outside the time window of its task.
    Window,
    /// A placement does not agree with its task.
    Requirement,
    /// A placement falls outside the availability of a resource.
    Availability,
    /// A task that must follow another starts too early.
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
#[derive(Debug, Clone, PartialEq)]
pub struct Conflict {
    /// The kind of problem.
    pub kind: ConflictKind,
    /// The task the conflict is centred on.
    pub task: TaskId,
    /// The other task involved, for a conflict between two placements.
    pub other: Option<TaskId>,
    /// The resource involved, where the conflict is about one.
    pub resource: Option<ResourceId>,
    /// The time at which the problem occurs, in seconds.
    ///
    /// For a capacity conflict this is the instant at which the demand
    /// exceeds the capacity. For a window conflict it is the offending edge
    /// of the placement.
    pub at: f64,
    /// A description of the problem.
    pub detail: String,
}

impl Conflict {
    /// Creates a conflict.
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
    pub fn with_other(mut self, other: TaskId) -> Self {
        self.other = Some(other);
        self
    }

    /// Returns a copy of this conflict involving a resource.
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
    pub fn conflicts(&self) -> &[Conflict] {
        &self.conflicts
    }

    /// Returns the number of conflicts.
    pub fn len(&self) -> usize {
        self.conflicts.len()
    }

    /// Returns `true` when there are no conflicts.
    ///
    /// This is the conventional spelling of [`ConflictReport::is_clear`], and
    /// the two always agree.
    pub fn is_empty(&self) -> bool {
        self.conflicts.is_empty()
    }

    /// Returns `true` when there are no conflicts.
    ///
    /// A clear report means the schedule is feasible with respect to every
    /// check this crate performs, not that it is optimal.
    pub fn is_clear(&self) -> bool {
        self.conflicts.is_empty()
    }

    /// Returns `true` when there is at least one conflict.
    pub fn has_conflicts(&self) -> bool {
        !self.conflicts.is_empty()
    }

    /// Returns the number of conflicts of one kind.
    pub fn count_of(&self, kind: ConflictKind) -> usize {
        self.conflicts
            .iter()
            .filter(|conflict| conflict.kind == kind)
            .count()
    }

    /// Returns the conflicts of one kind, in time order.
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
    pub fn offered_tasks(&self) -> usize {
        self.offered_tasks
    }

    /// Returns the number of tasks that were placed.
    pub fn placed_tasks(&self) -> usize {
        self.placed_tasks
    }

    /// Returns a one line summary of the report.
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
const fn kind_order(kind: ConflictKind) -> u8 {
    match kind {
        ConflictKind::Requirement => 0,
        ConflictKind::Window => 1,
        ConflictKind::Capacity => 2,
        ConflictKind::Availability => 3,
        ConflictKind::Precedence => 4,
    }
}
