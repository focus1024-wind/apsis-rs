//! Tests for the precedence graph.

use apsis_task::{PrecedenceGraph, TaskErrorKind, TaskId};

/// The identifiers used by the tests.
fn id(index: usize) -> TaskId {
    TaskId::new(index)
}

#[test]
fn an_empty_graph_has_no_edges() {
    let graph = PrecedenceGraph::new();
    assert!(graph.is_empty());
    assert_eq!(graph.edge_count(), 0);
    assert_eq!(graph.task_count(), 0);
    assert!(!graph.has_constraint(id(0), id(1)));
    assert!(graph.predecessors_of(id(0)).is_empty());
    assert!(graph.successors_of(id(0)).is_empty());
}

#[test]
fn a_constraint_is_recorded_both_ways() {
    let mut graph = PrecedenceGraph::new();
    graph.add(id(0), id(1)).expect("no cycle");

    assert!(graph.has_constraint(id(0), id(1)));
    // The constraint is directed, so the reverse does not exist.
    assert!(!graph.has_constraint(id(1), id(0)));
    assert_eq!(graph.successors_of(id(0)), vec![id(1)]);
    assert_eq!(graph.predecessors_of(id(1)), vec![id(0)]);
    assert_eq!(graph.edge_count(), 1);
    assert_eq!(graph.task_count(), 2);
}

#[test]
fn a_self_constraint_is_rejected() {
    let mut graph = PrecedenceGraph::new();
    let error = graph
        .add(id(0), id(0))
        .expect_err("a task cannot precede itself");
    assert_eq!(error.kind(), TaskErrorKind::PrecedenceCycle);
    assert!(graph.is_empty());
}

#[test]
fn a_duplicate_constraint_is_accepted_and_ignored() {
    let mut graph = PrecedenceGraph::new();
    graph.add(id(0), id(1)).expect("no cycle");
    graph.add(id(0), id(1)).expect("already present");
    // The edge is recorded once.
    assert_eq!(graph.edge_count(), 1);
    assert_eq!(graph.successors_of(id(0)), vec![id(1)]);
}

#[test]
fn a_cycle_is_rejected() {
    let mut graph = PrecedenceGraph::new();
    graph.add(id(0), id(1)).expect("no cycle");
    graph.add(id(1), id(2)).expect("no cycle");
    // Closing the cycle is refused.
    let error = graph.add(id(2), id(0)).expect_err("this closes a cycle");
    assert_eq!(error.kind(), TaskErrorKind::PrecedenceCycle);
    // The graph is unchanged.
    assert!(!graph.has_constraint(id(2), id(0)));
    assert_eq!(graph.edge_count(), 2);
}

#[test]
fn a_longer_cycle_is_rejected() {
    let mut graph = PrecedenceGraph::new();
    for edge in [(0, 1), (1, 2), (2, 3), (3, 4)] {
        graph.add(id(edge.0), id(edge.1)).expect("no cycle");
    }
    assert_eq!(
        graph
            .add(id(4), id(1))
            .expect_err("this closes a cycle")
            .kind(),
        TaskErrorKind::PrecedenceCycle
    );
}

#[test]
fn reachability_follows_the_direction() {
    let mut graph = PrecedenceGraph::new();
    graph.add(id(0), id(1)).expect("no cycle");
    graph.add(id(1), id(2)).expect("no cycle");

    // The transitive relation holds forward but not backward.
    assert!(graph.is_reachable(id(0), id(1)));
    assert!(graph.is_reachable(id(0), id(2)));
    assert!(graph.is_reachable(id(1), id(2)));
    assert!(!graph.is_reachable(id(2), id(0)));
    // A task reaches itself by a path of no edges.
    assert!(graph.is_reachable(id(0), id(0)));
}

#[test]
fn roots_and_leaves_are_reported() {
    let mut graph = PrecedenceGraph::new();
    // A diamond: 0 before 1 and 2, and both before 3.
    graph.add(id(0), id(1)).expect("no cycle");
    graph.add(id(0), id(2)).expect("no cycle");
    graph.add(id(1), id(3)).expect("no cycle");
    graph.add(id(2), id(3)).expect("no cycle");

    assert_eq!(graph.roots(), vec![id(0)]);
    assert_eq!(graph.leaves(), vec![id(3)]);
}

#[test]
fn a_chain_has_one_root_and_one_leaf() {
    let mut graph = PrecedenceGraph::new();
    graph.add(id(0), id(1)).expect("no cycle");
    graph.add(id(1), id(2)).expect("no cycle");
    assert_eq!(graph.roots(), vec![id(0)]);
    assert_eq!(graph.leaves(), vec![id(2)]);
}

#[test]
fn the_topological_order_respects_the_constraints() {
    let mut graph = PrecedenceGraph::new();
    graph.add(id(0), id(1)).expect("no cycle");
    graph.add(id(1), id(2)).expect("no cycle");

    let tasks = [id(2), id(1), id(0)];
    let order = graph
        .topological_order(&tasks)
        .expect("the graph is acyclic");
    // The constraints force 0 before 1 before 2, whatever the input order.
    assert_eq!(order, vec![id(0), id(1), id(2)]);
}

#[test]
fn the_topological_order_breaks_ties_by_position() {
    let mut graph = PrecedenceGraph::new();
    // Only 0 is constrained, before 1. Tasks 2 and 3 are free.
    graph.add(id(0), id(1)).expect("no cycle");

    // The preference order is 3, 2, 0, 1, so among the free tasks 3 comes
    // first.
    let tasks = [id(3), id(2), id(0), id(1)];
    let order = graph
        .topological_order(&tasks)
        .expect("the graph is acyclic");
    assert_eq!(order, vec![id(3), id(2), id(0), id(1)]);
}

#[test]
fn the_topological_order_honours_the_diamond() {
    let mut graph = PrecedenceGraph::new();
    graph.add(id(0), id(1)).expect("no cycle");
    graph.add(id(0), id(2)).expect("no cycle");
    graph.add(id(1), id(3)).expect("no cycle");
    graph.add(id(2), id(3)).expect("no cycle");

    let tasks = [id(0), id(1), id(2), id(3)];
    let order = graph
        .topological_order(&tasks)
        .expect("the graph is acyclic");
    assert_eq!(order, vec![id(0), id(1), id(2), id(3)]);

    // The position of each task in the result relative to its constraints.
    let position_of = |task: TaskId| {
        order
            .iter()
            .position(|candidate| *candidate == task)
            .expect("present")
    };
    assert!(position_of(id(0)) < position_of(id(1)));
    assert!(position_of(id(0)) < position_of(id(2)));
    assert!(position_of(id(1)) < position_of(id(3)));
    assert!(position_of(id(2)) < position_of(id(3)));
}

#[test]
fn the_topological_order_includes_unconstrained_tasks() {
    let mut graph = PrecedenceGraph::new();
    graph.add(id(0), id(1)).expect("no cycle");
    // Task 9 has no constraints at all.
    let tasks = [id(0), id(1), id(9)];
    let order = graph
        .topological_order(&tasks)
        .expect("the graph is acyclic");
    assert_eq!(order.len(), 3);
    assert!(order.contains(&id(9)));
}

#[test]
fn the_topological_order_ignores_predecessors_outside_the_list() {
    // A constraint whose predecessor is not in the list cannot be waited
    // for, so it is ignored rather than blocking the successor.
    let mut graph = PrecedenceGraph::new();
    graph.add(id(5), id(1)).expect("no cycle");
    let tasks = [id(1)];
    assert_eq!(
        graph
            .topological_order(&tasks)
            .expect("the graph is acyclic"),
        vec![id(1)]
    );
}

#[test]
fn the_topological_order_of_an_empty_list_is_empty() {
    let graph = PrecedenceGraph::new();
    assert!(
        graph
            .topological_order(&[])
            .expect("the graph is acyclic")
            .is_empty()
    );
}

#[test]
fn a_graph_with_two_independent_chains_orders_both() {
    let mut graph = PrecedenceGraph::new();
    graph.add(id(0), id(1)).expect("no cycle");
    graph.add(id(2), id(3)).expect("no cycle");

    let tasks = [id(0), id(1), id(2), id(3)];
    let order = graph
        .topological_order(&tasks)
        .expect("the graph is acyclic");
    let position_of = |task: TaskId| {
        order
            .iter()
            .position(|candidate| *candidate == task)
            .expect("present")
    };
    assert!(position_of(id(0)) < position_of(id(1)));
    assert!(position_of(id(2)) < position_of(id(3)));
}
