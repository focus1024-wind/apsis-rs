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
//! The precedence graph is a separate type from [`Task`](crate::Task)
//! because the constraints are a property of the set of tasks rather than of
//! any one task, and because a caller may want to plan the same tasks under
//! different constraint graphs.

use crate::error::{TaskError, TaskErrorKind};
use crate::task::TaskId;
use std::collections::BTreeMap;

/// A directed acyclic graph of precedence constraints.
///
/// An edge from one task to another means the first must finish before the
/// second starts. The order of the tasks themselves is the caller's: the
/// graph only records which pairs are related.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PrecedenceGraph {
    /// The successors of each task, in the order the edges were added.
    successors: BTreeMap<usize, Vec<usize>>,
    /// The predecessors of each task, in the order the edges were added.
    predecessors: BTreeMap<usize, Vec<usize>>,
}

impl PrecedenceGraph {
    /// Creates an empty graph.
    pub fn new() -> Self {
        Self {
            successors: BTreeMap::new(),
            predecessors: BTreeMap::new(),
        }
    }

    /// Returns the number of edges.
    pub fn edge_count(&self) -> usize {
        self.successors.values().map(Vec::len).sum()
    }

    /// Returns `true` when there are no edges.
    pub fn is_empty(&self) -> bool {
        self.successors.values().all(Vec::is_empty)
    }

    /// Returns `true` when the constraint from `before` to `after` exists.
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
    pub fn successors_of(&self, task: TaskId) -> Vec<TaskId> {
        self.successors
            .get(&task.index())
            .map(|list| list.iter().map(|index| TaskId::new(*index)).collect())
            .unwrap_or_default()
    }

    /// Returns the tasks with no predecessor.
    pub fn roots(&self) -> Vec<TaskId> {
        // Every task that appears as a successor has a predecessor, so the
        // roots are the tasks that appear only as a predecessor.
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
    pub fn topological_order(&self, tasks: &[TaskId]) -> Result<Vec<TaskId>, TaskError> {
        // The position of each task in the supplied list, which breaks ties.
        let mut preference: BTreeMap<usize, usize> = BTreeMap::new();
        for (position, task) in tasks.iter().enumerate() {
            preference.insert(task.index(), position);
        }

        // The number of predecessors of each task that are themselves in the
        // list. A predecessor outside the list is ignored, because it cannot
        // be scheduled and so cannot be waited for.
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
