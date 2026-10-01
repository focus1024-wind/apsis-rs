//! Tests for the topology graph.
//!
//! 拓扑图的测试。

use apsis_topology::{Edge, Topology, TopologyErrorKind};

/// A four node ring, with links both ways.
///
/// 一个四节点环，链路双向。
fn ring() -> Topology {
    let mut topology = Topology::new(4);
    for node in 0..4 {
        topology
            .add_bidirectional_edge(node, (node + 1) % 4, 1.0)
            .expect("the nodes exist");
    }
    topology
}

#[test]
fn an_empty_topology_has_no_edges() {
    let topology = Topology::new(5);
    assert_eq!(topology.node_count(), 5);
    assert_eq!(topology.edge_count(), 0);
    assert!(topology.is_empty());
    assert_eq!(topology.total_cost(), 0.0);
    assert!(!topology.has_edge(0, 1));
    assert_eq!(topology.cost(0, 1), None);
    assert!(topology.neighbours(0).is_empty());
}

#[test]
fn an_edge_is_directed() {
    let mut topology = Topology::new(2);
    topology.add_edge(0, 1, 2.5).expect("valid");
    assert!(topology.has_edge(0, 1));
    assert!(!topology.has_edge(1, 0));
    assert_eq!(topology.cost(0, 1), Some(2.5));
    assert_eq!(topology.out_degree(0), 1);
    assert_eq!(topology.in_degree(0), 0);
    assert_eq!(topology.out_degree(1), 0);
    assert_eq!(topology.in_degree(1), 1);
}

#[test]
fn a_bidirectional_edge_adds_both_directions() {
    let mut topology = Topology::new(2);
    topology.add_bidirectional_edge(0, 1, 3.0).expect("valid");
    assert!(topology.has_edge(0, 1));
    assert!(topology.has_edge(1, 0));
    assert_eq!(topology.edge_count(), 2);
    assert_eq!(topology.cost(0, 1), topology.cost(1, 0));
}

#[test]
fn an_out_of_range_node_is_rejected() {
    let mut topology = Topology::new(2);
    let error = topology
        .add_edge(0, 2, 1.0)
        .expect_err("node 2 does not exist");
    assert_eq!(error.kind(), TopologyErrorKind::NoSuchNode);
    assert_eq!(
        topology.add_edge(9, 0, 1.0).expect_err("node 9").kind(),
        TopologyErrorKind::NoSuchNode
    );
    // The topology is unchanged.
    // 拓扑不变。
    assert!(topology.is_empty());
}

#[test]
fn an_invalid_cost_is_rejected() {
    let mut topology = Topology::new(2);
    for cost in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        let error = topology
            .add_edge(0, 1, cost)
            .expect_err("the cost is not usable");
        assert_eq!(error.kind(), TopologyErrorKind::InvalidCost, "cost {cost}");
    }
    assert!(topology.is_empty());
}

#[test]
fn a_duplicate_edge_is_rejected() {
    let mut topology = Topology::new(2);
    topology.add_edge(0, 1, 1.0).expect("valid");
    let error = topology
        .add_edge(0, 1, 2.0)
        .expect_err("the edge already exists");
    assert_eq!(error.kind(), TopologyErrorKind::DuplicateEdge);
    // The original cost is retained.
    // 原代价被保留。
    assert_eq!(topology.cost(0, 1), Some(1.0));
    assert_eq!(topology.edge_count(), 1);
}

#[test]
fn a_self_loop_is_allowed_and_counted() {
    // A self loop is unusual in a satellite network but is not an error:
    // a node may have a local interface, and the graph need not forbid it.
    // 自环在卫星网中不常见但并非错误：节点可能有本地接口，而图无需禁止它。
    let mut topology = Topology::new(1);
    topology.add_edge(0, 0, 1.0).expect("valid");
    assert!(topology.has_edge(0, 0));
    assert_eq!(topology.out_degree(0), 1);
    assert_eq!(topology.in_degree(0), 1);
    assert_eq!(topology.connected_components(), 1);
}

#[test]
fn an_edge_can_be_removed() {
    let mut topology = Topology::new(2);
    topology.add_edge(0, 1, 1.0).expect("valid");
    topology.remove_edge(0, 1).expect("the edge exists");
    assert!(!topology.has_edge(0, 1));
    assert!(topology.is_empty());

    let error = topology.remove_edge(0, 1).expect_err("the edge is gone");
    assert_eq!(error.kind(), TopologyErrorKind::NoSuchEdge);
}

#[test]
fn the_adjacency_lists_every_node() {
    let topology = ring();
    let adjacency = topology.adjacency();
    assert_eq!(adjacency.len(), 4);
    // Each node of a ring has two neighbours in each direction; the
    // bidirectional construction gives two outgoing edges.
    // 环的每个节点在每个方向上有两个邻居；双向构建给出两条出发边。
    for neighbours in &adjacency {
        assert_eq!(neighbours.len(), 2);
    }
    // The order follows the insertion order: node 0 gains its edge to 1
    // when node 0 is processed, and its edge to 3 when node 3 is.
    // 其次序遵循插入次序：节点 0 在节点 0 被处理时获得其到 1 的边，在节点 3 被处理
    // 时获得其到 3 的边。
    assert_eq!(adjacency[0], vec![1, 3]);
    assert_eq!(adjacency[1], vec![0, 2]);
}

#[test]
fn the_ring_is_connected() {
    let topology = ring();
    assert!(topology.is_connected());
    assert_eq!(topology.connected_components(), 1);
}

#[test]
fn removing_a_link_splits_the_ring() {
    let mut topology = ring();
    // Removing one direction of each of two opposite links breaks the ring
    // into two paths.
    // 移除两条相对链路各自的一个方向，把环断成两条路径。
    topology.remove_edge(0, 1).expect("the edge exists");
    topology.remove_edge(1, 0).expect("the edge exists");
    topology.remove_edge(2, 3).expect("the edge exists");
    topology.remove_edge(3, 2).expect("the edge exists");
    assert_eq!(topology.connected_components(), 2);
    assert!(!topology.is_connected());
}

#[test]
fn a_single_direction_is_enough_to_connect_components() {
    // The component count treats edges as undirected, so a link that only
    // carries traffic one way still joins two pieces of the graph.
    // 分量计数把边视为无向，故只单向承载业务的链路仍连接图的两部分。
    let mut topology = Topology::new(2);
    topology.add_edge(0, 1, 1.0).expect("valid");
    assert_eq!(topology.connected_components(), 1);

    // But the reverse direction does not exist, so the node cannot be
    // reached from the other side.
    // 但反向不存在，故该节点无法从另一侧到达。
    assert!(!topology.has_edge(1, 0));
}

#[test]
fn isolated_nodes_are_each_a_component() {
    let topology = Topology::new(4);
    assert_eq!(topology.connected_components(), 4);
    // A graph with no edges is still connected when it has at most one
    // node, and is not when it has more.
    // 无边图在至多一个节点时仍是连通的，节点更多时则不连通。
    assert!(!topology.is_connected());
    assert!(Topology::new(1).is_connected());
    assert!(Topology::new(0).is_connected());
    assert_eq!(Topology::new(0).connected_components(), 0);
}

#[test]
fn the_total_cost_sums_the_edges() {
    let mut topology = Topology::new(3);
    topology.add_edge(0, 1, 1.5).expect("valid");
    topology.add_edge(1, 2, 2.5).expect("valid");
    assert!((topology.total_cost() - 4.0).abs() < 1e-12);
}

#[test]
fn an_edge_can_be_reversed() {
    let edge = Edge::new(3, 7, 2.0);
    let reversed = edge.reversed();
    assert_eq!(reversed.from, 7);
    assert_eq!(reversed.to, 3);
    assert_eq!(reversed.cost, 2.0);
}

#[test]
fn a_failed_bidirectional_add_leaves_the_topology_unchanged() {
    let mut topology = Topology::new(2);
    // The first direction succeeds, but the second is a duplicate, so the
    // whole call must fail and roll back.
    // 第一个方向成功，但第二个是重复，故整个调用必须失败并回滚。
    topology.add_edge(1, 0, 5.0).expect("valid");
    let error = topology
        .add_bidirectional_edge(0, 1, 2.0)
        .expect_err("the reverse edge exists");
    assert_eq!(error.kind(), TopologyErrorKind::DuplicateEdge);
    // The forward direction that was added must have been removed.
    // 所加入的正向必须已被移除。
    assert!(!topology.has_edge(0, 1));
    assert!(topology.has_edge(1, 0));
    assert_eq!(topology.edge_count(), 1);
}

#[test]
fn the_neighbour_order_follows_the_edge_insertion_order() {
    let mut topology = Topology::new(4);
    topology.add_edge(0, 2, 1.0).expect("valid");
    topology.add_edge(0, 1, 1.0).expect("valid");
    topology.add_edge(0, 3, 1.0).expect("valid");
    assert_eq!(topology.neighbours(0), vec![2, 1, 3]);
}
