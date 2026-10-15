//! Precedence constraints between tasks.
//!
//! A precedence constraint says that one task must finish before another
//! starts. A set of such constraints forms a directed acyclic graph, and a
//! scheduler must respect it: a task cannot be placed until every task that
//! must precede it has been placed and has finished.
//!
//! The graph is stored as adjacency lists. It rejects a cycle when an edge is
//! added, because a cycle is not a set of constraints that can be satisfied
//! and would otherwise produce an empty topological order that a caller
//! might mistake for a scheduling failure rather than a modelling error.
//!
//! The precedence graph is a separate type from [`Task`](crate::apsis_task::Task)
//! because the constraints are a property of the set of tasks rather than of
//! any one task, and because a caller may want to plan the same tasks under
//! different constraint graphs.
//!
//! 任务之间的先后约束。
//!
//! 一条先后约束说一个任务必须在另一个开始之前完成。一组这样的约束构成有向无环图，
//! 而调度器必须遵守它：一个任务在其必须先行的每个任务都被放置并完成之前无法被
//! 放置。
//!
//! 该图以邻接表存储。它在添加边时拒绝环，因为环不是一组可满足的约束，且否则会产生
//! 一个空的拓扑序，调用方可能把它误认为调度失败而非建模错误。
//!
//! 先后关系图是与 [`Task`](crate::apsis_task::Task) 不同的类型，因为约束是任务集合的性质而非
//! 任一单个任务的性质，也因为调用方可能想在不同约束图下规划同一批任务。

use crate::apsis_task::error::{TaskError, TaskErrorKind};
use crate::apsis_task::task::TaskId;
use std::collections::BTreeMap;

/// A directed acyclic graph of precedence constraints.
///
/// An edge from one task to another means the first must finish before the
/// second starts. The order of the tasks themselves is the caller's: the
/// graph only records which pairs are related.
///
/// 优先约束构成的有向无环图。
///
/// 从一个任务指向另一个任务的边表示前者必须先于后者完成。任务本身的次序
/// 由调用方决定：该图只记录哪些任务对之间存在关系。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PrecedenceGraph {
    /// The successors of each task, in the order the edges were added.
    ///
    /// 各任务的后继，按边添加顺序排列。
    successors: BTreeMap<usize, Vec<usize>>,
    /// The predecessors of each task, in the order the edges were added.
    ///
    /// 各任务的前驱，按边添加顺序排列。
    predecessors: BTreeMap<usize, Vec<usize>>,
}

impl PrecedenceGraph {
    /// Creates an empty graph.
    ///
    /// 创建一个空图。
    #[must_use]
    pub fn new() -> Self {
        Self {
            successors: BTreeMap::new(),
            predecessors: BTreeMap::new(),
        }
    }

    /// Returns the number of edges.
    ///
    /// 返回边的数量。
    pub fn edge_count(&self) -> usize {
        self.successors.values().map(Vec::len).sum()
    }

    /// Returns `true` when there are no edges.
    ///
    /// 当没有边时返回 `true`。
    pub fn is_empty(&self) -> bool {
        self.successors.values().all(Vec::is_empty)
    }

    /// Returns `true` when the constraint from `before` to `after` exists.
    ///
    /// 当存在从 `before` 到 `after` 的约束时返回 `true`。
    #[must_use]
    pub fn has_constraint(&self, before: TaskId, after: TaskId) -> bool {
        self.successors
            .get(&before.index())
            .is_some_and(|successors| successors.contains(&after.index()))
    }

    /// Adds a constraint that `before` must finish before `after` starts.
    ///
    /// Returns an error when the two identifiers are the same, because a task
    /// cannot precede itself, or when the constraint would close a cycle.
    ///
    /// Adding a constraint that already exists is accepted and has no effect,
    /// so a caller may assemble a graph from overlapping sources without
    /// having to deduplicate.
    ///
    /// 添加一条约束：`before` 必须在 `after` 开始之前完成。
    ///
    /// 两个标识符相同时返回错误，因为任务不能先于自身；该约束会闭成环时
    /// 亦返回错误。
    ///
    /// 添加已存在的约束被接受且不起作用，故调用方可以从多个有重叠的数据源
    /// 汇集一个图，无须先去重。
    pub fn add(&mut self, before: TaskId, after: TaskId) -> Result<(), TaskError> {
        if before == after {
            return Err(TaskError::new(TaskErrorKind::PrecedenceCycle)
                .with_detail(format!("{before} cannot precede itself")));
        }
        if self.has_constraint(before, after) {
            return Ok(());
        }
        // A new edge from `before` to `after` closes a cycle exactly when
        // `before` is already reachable from `after`. That is checked before
        // the edge is inserted, so a rejected edge leaves the graph
        // unchanged.
        // 从 `before` 到 `after` 的新边恰在 `before` 已可自 `after` 到达时
        // 闭成环。该检查先于插入，故被拒的边不会改变图。
        if self.is_reachable(after, before) {
            return Err(
                TaskError::new(TaskErrorKind::PrecedenceCycle).with_detail(format!(
                    "{before} must precede {after}, but {after} already precedes {before}"
                )),
            );
        }
        self.successors
            .entry(before.index())
            .or_default()
            .push(after.index());
        self.predecessors
            .entry(after.index())
            .or_default()
            .push(before.index());
        Ok(())
    }

    /// Returns `true` when `to` is reachable from `from` following the
    /// constraints.
    ///
    /// 当沿约束可自 `from` 到达 `to` 时返回 `true`。
    #[must_use]
    pub fn is_reachable(&self, from: TaskId, to: TaskId) -> bool {
        if from == to {
            return true;
        }
        let mut visited: Vec<usize> = Vec::new();
        let mut stack = vec![from.index()];
        while let Some(node) = stack.pop() {
            if node == to.index() {
                return true;
            }
            if visited.contains(&node) {
                continue;
            }
            visited.push(node);
            if let Some(successors) = self.successors.get(&node) {
                stack.extend(successors.iter().copied());
            }
        }
        false
    }

    /// Returns the tasks that must finish before a task starts.
    ///
    /// The result is in the order the constraints were added. It is the
    /// direct predecessors only, not the transitive closure.
    ///
    /// 返回必须在某任务开始前完成的任务。
    ///
    /// 结果按约束添加顺序排列。它只含直接前驱，不含传递闭包。
    #[must_use]
    pub fn predecessors_of(&self, task: TaskId) -> Vec<TaskId> {
        self.predecessors
            .get(&task.index())
            .map(|list| list.iter().map(|index| TaskId::new(*index)).collect())
            .unwrap_or_default()
    }

    /// Returns the tasks that must wait for a task to finish.
    ///
    /// The result is in the order the constraints were added. It is the
    /// direct successors only, not the transitive closure.
    ///
    /// 返回必须等待某任务完成的任务。
    ///
    /// 结果按约束添加顺序排列。它只含直接后继，不含传递闭包。
    #[must_use]
    pub fn successors_of(&self, task: TaskId) -> Vec<TaskId> {
        self.successors
            .get(&task.index())
            .map(|list| list.iter().map(|index| TaskId::new(*index)).collect())
            .unwrap_or_default()
    }

    /// Returns the tasks with no predecessor.
    ///
    /// 返回没有前驱的任务。
    pub fn roots(&self) -> Vec<TaskId> {
        // Every task that appears as a successor has a predecessor, so the
        // roots are the tasks that appear only as a predecessor.
        // 凡出现为后继的任务都有前驱，故根是那些只作为前驱出现的任务。
        let mut roots: Vec<usize> = Vec::new();
        for task in self.successors.keys() {
            if !self.predecessors.contains_key(task) && !roots.contains(task) {
                roots.push(*task);
            }
        }
        roots.sort_unstable();
        roots.into_iter().map(TaskId::new).collect()
    }

    /// Returns the tasks with no successor.
    ///
    /// 返回没有后继的任务。
    pub fn leaves(&self) -> Vec<TaskId> {
        let mut leaves: Vec<usize> = Vec::new();
        for task in self.predecessors.keys() {
            if !self.successors.contains_key(task) && !leaves.contains(task) {
                leaves.push(*task);
            }
        }
        leaves.sort_unstable();
        leaves.into_iter().map(TaskId::new).collect()
    }

    /// Returns a topological order of the given tasks.
    ///
    /// The order respects every constraint: a task appears after all of its
    /// predecessors. Ties are broken by the position of the task in the
    /// supplied list, so that a caller can impose its own preference among
    /// tasks that are otherwise unordered. That is what lets a planner place
    /// the more valuable tasks first within the freedom the constraints
    /// allow.
    ///
    /// Tasks in the list that have no constraints still appear.
    ///
    /// Returns an error when the graph contains a cycle among the given
    /// tasks. The graph rejects cycles as they are added, so this can only
    /// happen when a caller builds the graph from one task set and orders
    /// another, or when the tasks are not all in the graph.
    ///
    /// 返回所给任务的拓扑序。
    ///
    /// 该次序满足每一条约束：任务出现在其所有前驱之后。平局按该任务在所给
    /// 列表中的位置打破，使调用方可在本来无序的任务间表达自己的偏好。这
    /// 正是让规划器在约束允许的自由度内先放置更有价值的任务。
    ///
    /// 列表中无任何约束的任务同样出现。
    ///
    /// 当图在所给任务之间含环时返回错误。该图在添加时就拒绝成环，故这只
    /// 可能发生在调用方以一组任务建图却对另一组排序，或任务未全在图中时。
    pub fn topological_order(&self, tasks: &[TaskId]) -> Result<Vec<TaskId>, TaskError> {
        // The position of each task in the supplied list, which breaks ties.
        // 各任务在所给列表中的位置，用于打破平局。
        let mut preference: BTreeMap<usize, usize> = BTreeMap::new();
        for (position, task) in tasks.iter().enumerate() {
            preference.insert(task.index(), position);
        }

        // The number of predecessors of each task that are themselves in the
        // list. A predecessor outside the list is ignored, because it cannot
        // be scheduled and so cannot be waited for.
        // 各任务在列表内的前驱个数。列表外的前驱被忽略，因为它无法被
        // 调度，也就无从等待。
        let mut pending: BTreeMap<usize, usize> = BTreeMap::new();
        for task in tasks {
            let count = self
                .predecessors_of(*task)
                .iter()
                .filter(|predecessor| preference.contains_key(&predecessor.index()))
                .count();
            pending.insert(task.index(), count);
        }

        let mut order: Vec<usize> = Vec::with_capacity(tasks.len());
        let mut placed: Vec<usize> = Vec::with_capacity(tasks.len());

        while order.len() < tasks.len() {
            // Among the tasks whose predecessors are all placed, take the
            // one the caller prefers.
            // 在前驱均已放置的任务中，取调用方偏好者。
            let next = tasks
                .iter()
                .map(|task| task.index())
                .filter(|index| !placed.contains(index))
                .filter(|index| pending.get(index).copied().unwrap_or(0) == 0)
                .min_by_key(|index| preference.get(index).copied().unwrap_or(usize::MAX));

            let Some(next) = next else {
                return Err(TaskError::new(TaskErrorKind::PrecedenceCycle)
                    .with_detail("no task is free of unplaced predecessors"));
            };

            placed.push(next);
            order.push(next);
            for successor in self.successors_of(TaskId::new(next)) {
                if let Some(count) = pending.get_mut(&successor.index()) {
                    *count = count.saturating_sub(1);
                }
            }
        }

        Ok(order.into_iter().map(TaskId::new).collect())
    }

    /// Returns the number of tasks in the graph.
    ///
    /// 返回图中的任务数。
    #[must_use]
    pub fn task_count(&self) -> usize {
        let mut all: Vec<usize> = self.successors.keys().copied().collect();
        for task in self.predecessors.keys() {
            if !all.contains(task) {
                all.push(*task);
            }
        }
        all.len()
    }
}
