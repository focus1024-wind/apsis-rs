//! The mesh type and the summary of its redundancy.

use crate::connectivity::{edge_connectivity, reachable_from, strongly_connected_components};
use apsis_topology::Topology;

/// A link whose removal would change the connectivity of the mesh.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CriticalLink {
    /// The node the link leaves.
    pub from: usize,
    /// The node the link arrives at.
    pub to: usize,
    /// The cost of the link.
    pub cost: f64,
    /// The number of components the mesh has after the link is removed.
    pub components_without: usize,
}

impl CriticalLink {
    /// Creates a critical link record.
    pub const fn new(from: usize, to: usize, cost: f64, components_without: usize) -> Self {
        Self {
            from,
            to,
            cost,
            components_without,
        }
    }
}

/// A summary of the connectivity of a mesh.
#[derive(Debug, Clone, PartialEq)]
pub struct MeshSummary {
    /// The number of terminals.
    pub node_count: usize,
    /// The number of directed links.
    pub edge_count: usize,
    /// The number of components when the links are treated as undirected.
    pub components: usize,
    /// The number of strongly connected components.
    pub strongly_connected_components: usize,
    /// The smallest number of independent paths between any two distinct
    /// terminals, or `None` when there are fewer than two terminals.
    ///
    /// This is the worst connection in the mesh. A value of one means some
    /// pair of terminals relies on a single link and the mesh has no
    /// redundancy anywhere.
    pub minimum_connectivity: Option<usize>,
}

/// A mesh of terminals joined by links.
///
/// The mesh wraps a topology and adds the connectivity measures a mesh
/// application needs. It does not own a schedule or a traffic model; those
/// belong to the caller.
#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    topology: Topology,
}

impl Mesh {
    /// Creates a mesh over a topology.
    pub const fn new(topology: Topology) -> Self {
        Self { topology }
    }

    /// Returns the underlying topology.
    pub const fn topology(&self) -> &Topology {
        &self.topology
    }

    /// Returns the number of terminals.
    pub fn node_count(&self) -> usize {
        self.topology.node_count()
    }

    /// Returns the number of links.
    pub fn edge_count(&self) -> usize {
        self.topology.edge_count()
    }

    /// Returns `true` when every terminal reaches every other, treating the
    /// links as undirected.
    pub fn is_connected(&self) -> bool {
        self.topology.is_connected()
    }

    /// Returns the terminals reachable from a terminal.
    pub fn reachable_from(&self, source: usize) -> Vec<usize> {
        reachable_from(&self.topology, source)
    }

    /// Returns the number of independent paths between two terminals.
    ///
    /// Returns `None` when either terminal is out of range or the two are
    /// the same.
    pub fn edge_connectivity(&self, source: usize, destination: usize) -> Option<usize> {
        edge_connectivity(&self.topology, source, destination)
    }

    /// Returns the smallest number of independent paths between any two
    /// distinct terminals.
    ///
    /// Returns `None` when there are fewer than two terminals.
    pub fn minimum_connectivity(&self) -> Option<usize> {
        let node_count = self.topology.node_count();
        if node_count < 2 {
            return None;
        }
        let mut minimum = usize::MAX;
        for source in 0..node_count {
            for destination in (source + 1)..node_count {
                let value = edge_connectivity(&self.topology, source, destination)?;
                minimum = minimum.min(value);
            }
        }
        Some(minimum)
    }

    /// Returns the links whose removal would increase the number of
    /// connected components, treating each link as undirected.
    ///
    /// These are the bridges of the undirected graph underlying the mesh. A
    /// link is reported once, even when it is present in both directions,
    /// because a failure takes out the whole link rather than one direction
    /// of it.
    ///
    /// Each candidate is found by removing the link in both directions and
    /// comparing the component count, which is exact but costs one traversal
    /// per link. For the size of graph a satellite mesh presents that is
    /// acceptable, and it is easier to verify than a linear time bridge
    /// finding algorithm.
    ///
    /// Links that are not bridges are omitted, so an empty result means
    /// every link lies on a cycle and the mesh survives any single link
    /// failure without splitting.
    pub fn critical_links(&self) -> Vec<CriticalLink> {
        let baseline = self.topology.connected_components();
        let mut critical = Vec::new();
        let mut seen: Vec<(usize, usize)> = Vec::new();

        for edge in self.topology.edges() {
            // Canonicalize the pair so that the two directions of one link
            // are considered together and reported once.
            let pair = if edge.from <= edge.to {
                (edge.from, edge.to)
            } else {
                (edge.to, edge.from)
            };
            if seen.contains(&pair) {
                continue;
            }
            seen.push(pair);

            // Remove the link in every direction in which it is present,
            // which is what a link failure does.
            let mut without = self.topology.clone();
            let _ = without.remove_edge(pair.0, pair.1);
            let _ = without.remove_edge(pair.1, pair.0);

            let components = without.connected_components();
            if components > baseline {
                critical.push(CriticalLink::new(edge.from, edge.to, edge.cost, components));
            }
        }
        critical
    }

    /// Returns `true` when the mesh survives the failure of any single link
    /// without splitting.
    ///
    /// This is the condition for the mesh to have a single link failure
    /// tolerance, which is the minimum redundancy a network carrying traffic
    /// without interruption should have.
    pub fn survives_single_link_failure(&self) -> bool {
        self.critical_links().is_empty()
    }

    /// Returns the number of strongly connected components.
    ///
    /// For a mesh of directed links this counts the pieces that can route
    /// internally. A value above one means some terminals cannot be reached
    /// from some others.
    pub fn strongly_connected_components(&self) -> usize {
        strongly_connected_components(&self.topology)
    }

    /// Returns a summary of the mesh connectivity.
    pub fn summarise(&self) -> MeshSummary {
        MeshSummary {
            node_count: self.topology.node_count(),
            edge_count: self.topology.edge_count(),
            components: self.topology.connected_components(),
            strongly_connected_components: strongly_connected_components(&self.topology),
            minimum_connectivity: self.minimum_connectivity(),
        }
    }
}
