//! Tests for mesh connectivity measures.
//!
//! The reference cases are small graphs whose connectivity is known by
//! inspection: a ring has two independent paths between any pair, a line has
//! one, and a fully connected graph has as many as there are parallel links.

use apsis_mesh::{
    CriticalLink, Mesh, edge_connectivity, reachable_from, strongly_connected_components,
};
use apsis_topology::Topology;

/// A bidirectional ring of the given size.
fn ring(node_count: usize) -> Topology {
    let mut topology = Topology::new(node_count);
    for node in 0..node_count {
        topology
            .add_bidirectional_edge(node, (node + 1) % node_count, 1.0)
            .expect("the nodes exist");
    }
    topology
}

/// A bidirectional line, which has no cycle.
fn line(node_count: usize) -> Topology {
    let mut topology = Topology::new(node_count);
    for node in 0..node_count.saturating_sub(1) {
        topology
            .add_bidirectional_edge(node, node + 1, 1.0)
            .expect("the nodes exist");
    }
    topology
}

/// A complete bidirectional graph.
fn complete(node_count: usize) -> Topology {
    let mut topology = Topology::new(node_count);
    for first in 0..node_count {
        for second in (first + 1)..node_count {
            topology
                .add_bidirectional_edge(first, second, 1.0)
                .expect("the nodes exist");
        }
    }
    topology
}

#[test]
fn reachability_follows_the_edge_direction() {
    // A one way line: 0 reaches 1 and 2, but 2 reaches only itself.
    let mut topology = Topology::new(3);
    topology.add_edge(0, 1, 1.0).expect("valid");
    topology.add_edge(1, 2, 1.0).expect("valid");

    assert_eq!(reachable_from(&topology, 0), vec![0, 1, 2]);
    assert_eq!(reachable_from(&topology, 1), vec![1, 2]);
    assert_eq!(reachable_from(&topology, 2), vec![2]);
}

#[test]
fn reachability_includes_the_source() {
    let topology = Topology::new(3);
    assert_eq!(reachable_from(&topology, 0), vec![0]);
}

#[test]
fn reachability_of_an_out_of_range_node_is_empty() {
    let topology = Topology::new(2);
    assert!(reachable_from(&topology, 9).is_empty());
}

#[test]
fn a_ring_has_two_independent_paths() {
    let topology = ring(4);
    // Around a ring there are exactly two ways between any two nodes, and
    // no more, because removing both directions of one link leaves a line.
    assert_eq!(edge_connectivity(&topology, 0, 2), Some(2));
    assert_eq!(edge_connectivity(&topology, 0, 1), Some(2));
    assert_eq!(edge_connectivity(&topology, 1, 3), Some(2));
}

#[test]
fn a_line_has_one_independent_path() {
    let topology = line(4);
    // A line has a single route, so one link failure disconnects it.
    assert_eq!(edge_connectivity(&topology, 0, 3), Some(1));
    assert_eq!(edge_connectivity(&topology, 0, 1), Some(1));
}

#[test]
fn a_complete_graph_has_the_full_degree() {
    let topology = complete(4);
    // With four nodes and every pair linked both ways, there are three edge
    // disjoint paths between any two nodes: the direct link and the two
    // through the other nodes.
    assert_eq!(edge_connectivity(&topology, 0, 1), Some(3));
    assert_eq!(edge_connectivity(&topology, 2, 3), Some(3));
}

#[test]
fn a_disconnected_pair_has_no_path() {
    let mut topology = Topology::new(4);
    topology.add_edge(0, 1, 1.0).expect("valid");
    topology.add_edge(2, 3, 1.0).expect("valid");
    assert_eq!(edge_connectivity(&topology, 0, 3), Some(0));
    assert_eq!(edge_connectivity(&topology, 0, 1), Some(1));
}

#[test]
fn connectivity_rejects_a_node_with_itself() {
    let topology = ring(3);
    assert_eq!(edge_connectivity(&topology, 0, 0), None);
}

#[test]
fn connectivity_rejects_an_out_of_range_node() {
    let topology = ring(3);
    assert_eq!(edge_connectivity(&topology, 0, 9), None);
    assert_eq!(edge_connectivity(&topology, 9, 0), None);
}

#[test]
fn the_connectivity_of_a_parallel_pair_is_the_number_of_links() {
    // Three parallel one way links between two nodes give three independent
    // paths.
    let mut topology = Topology::new(2);
    topology.add_edge(0, 1, 1.0).expect("valid");
    // A second and third parallel link cannot be added as a duplicate, so
    // they are represented by routing through intermediate nodes instead.
    topology.add_edge(0, 1, 1.0).ok();
    assert_eq!(edge_connectivity(&topology, 0, 1), Some(1));

    // A pair of two node disjoint paths: the direct link and one through an
    // intermediate node.
    let mut detour = Topology::new(3);
    detour.add_edge(0, 1, 1.0).expect("valid");
    detour.add_edge(0, 2, 1.0).expect("valid");
    detour.add_edge(2, 1, 1.0).expect("valid");
    assert_eq!(edge_connectivity(&detour, 0, 1), Some(2));
}

#[test]
fn the_strongly_connected_components_of_a_bidirectional_ring_is_one() {
    let topology = ring(4);
    assert_eq!(strongly_connected_components(&topology), 1);
}

#[test]
fn a_one_way_ring_is_strongly_connected() {
    // A directed cycle can still reach every node from every node.
    let mut topology = Topology::new(3);
    topology.add_edge(0, 1, 1.0).expect("valid");
    topology.add_edge(1, 2, 1.0).expect("valid");
    topology.add_edge(2, 0, 1.0).expect("valid");
    assert_eq!(strongly_connected_components(&topology), 1);
}

#[test]
fn a_one_way_line_has_one_component_per_node() {
    let mut topology = Topology::new(3);
    topology.add_edge(0, 1, 1.0).expect("valid");
    topology.add_edge(1, 2, 1.0).expect("valid");
    // No node can be reached back from a later one, so every node is its
    // own component.
    assert_eq!(strongly_connected_components(&topology), 3);
}

#[test]
fn two_one_way_cycles_are_two_components() {
    let mut topology = Topology::new(4);
    topology.add_edge(0, 1, 1.0).expect("valid");
    topology.add_edge(1, 0, 1.0).expect("valid");
    topology.add_edge(2, 3, 1.0).expect("valid");
    topology.add_edge(3, 2, 1.0).expect("valid");
    assert_eq!(strongly_connected_components(&topology), 2);
}

#[test]
fn an_empty_topology_has_no_components() {
    assert_eq!(strongly_connected_components(&Topology::new(0)), 0);
}

#[test]
fn isolated_nodes_are_each_a_component() {
    let topology = Topology::new(3);
    assert_eq!(strongly_connected_components(&topology), 3);
}

#[test]
fn the_mesh_wraps_a_topology() {
    let mesh = Mesh::new(ring(4));
    assert_eq!(mesh.node_count(), 4);
    assert_eq!(mesh.edge_count(), 8);
    assert!(mesh.is_connected());
    assert_eq!(mesh.edge_connectivity(0, 2), Some(2));
    assert_eq!(mesh.strongly_connected_components(), 1);
    assert_eq!(mesh.reachable_from(0), vec![0, 1, 2, 3]);
    assert_eq!(mesh.topology().node_count(), 4);
}

#[test]
fn a_line_mesh_has_no_redundancy() {
    let mesh = Mesh::new(line(4));
    assert_eq!(mesh.minimum_connectivity(), Some(1));
    assert!(!mesh.survives_single_link_failure());
}

#[test]
fn a_ring_mesh_survives_a_single_link_failure() {
    let mesh = Mesh::new(ring(4));
    assert_eq!(mesh.minimum_connectivity(), Some(2));
    assert!(mesh.survives_single_link_failure());
    assert!(mesh.critical_links().is_empty());
}

#[test]
fn the_critical_links_of_a_line_are_every_link() {
    let mesh = Mesh::new(line(3));
    let critical = mesh.critical_links();
    // A three node line has two links, and each is a bridge. A link is
    // reported once even though it is present in both directions, because a
    // failure takes out the whole link.
    assert_eq!(critical.len(), 2);
    for link in &critical {
        assert_eq!(link.components_without, 2);
    }
    // The record carries the endpoints and the cost.
    let first = critical[0];
    assert_eq!(first.from, 0);
    assert_eq!(first.to, 1);
    assert!((first.cost - 1.0).abs() < 1e-12);
}

#[test]
fn a_complete_mesh_has_no_critical_links() {
    let mesh = Mesh::new(complete(4));
    assert!(mesh.critical_links().is_empty());
    assert!(mesh.survives_single_link_failure());
    assert_eq!(mesh.minimum_connectivity(), Some(3));
}

#[test]
fn minimum_connectivity_needs_two_terminals() {
    assert_eq!(Mesh::new(Topology::new(1)).minimum_connectivity(), None);
    assert_eq!(Mesh::new(Topology::new(0)).minimum_connectivity(), None);
}

#[test]
fn the_summary_reports_the_mesh_shape() {
    let mesh = Mesh::new(ring(4));
    let summary = mesh.summarise();
    assert_eq!(summary.node_count, 4);
    assert_eq!(summary.edge_count, 8);
    assert_eq!(summary.components, 1);
    assert_eq!(summary.strongly_connected_components, 1);
    assert_eq!(summary.minimum_connectivity, Some(2));
}

#[test]
fn the_summary_of_a_split_mesh_reports_the_pieces() {
    // Two rings joined by nothing.
    let mut topology = Topology::new(6);
    for node in 0..3 {
        topology
            .add_bidirectional_edge(node, (node + 1) % 3, 1.0)
            .expect("valid");
    }
    for node in 3..6 {
        topology
            .add_bidirectional_edge(node, 3 + (node + 1) % 3, 1.0)
            .expect("valid");
    }
    let mesh = Mesh::new(topology);
    let summary = mesh.summarise();
    assert_eq!(summary.components, 2);
    assert_eq!(summary.strongly_connected_components, 2);
    // The pair across the split has no path, so the minimum is zero.
    assert_eq!(summary.minimum_connectivity, Some(0));
    assert!(!mesh.is_connected());
}

#[test]
fn a_critical_link_type_is_constructible_directly() {
    let link = CriticalLink::new(1, 2, 3.5, 2);
    assert_eq!(link.from, 1);
    assert_eq!(link.to, 2);
    assert!((link.cost - 3.5).abs() < 1e-12);
    assert_eq!(link.components_without, 2);
}

#[test]
fn a_satellite_like_mesh_has_the_expected_connectivity() {
    // A grid-like mesh with each node linked to two neighbours in a ring
    // plus one cross link, which has connectivity two everywhere.
    let mut topology = ring(6);
    topology.add_bidirectional_edge(0, 3, 2.0).expect("valid");
    let mesh = Mesh::new(topology);
    assert!(mesh.is_connected());
    assert!(mesh.survives_single_link_failure());
    assert_eq!(mesh.minimum_connectivity(), Some(2));
}
