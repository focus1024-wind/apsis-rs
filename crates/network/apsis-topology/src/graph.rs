//! The topology graph.

use crate::error::{TopologyError, TopologyErrorKind};

/// One link in a topology.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Edge {
    /// The index of the first endpoint.
    pub from: usize,
    /// The index of the second endpoint.
    pub to: usize,
    /// The cost of traversing the link.
    ///
    /// The meaning is the caller's: a delay in seconds, a hop count, or any
    /// other measure a routing algorithm minimises. It must be finite and
    /// positive, so that adding edges never shortens a path.
    pub cost: f64,
}

impl Edge {
    /// Creates an edge.
    pub const fn new(from: usize, to: usize, cost: f64) -> Self {
        Self { from, to, cost }
    }

    /// Returns the same edge with its endpoints exchanged.
    ///
    /// A link between two satellites carries traffic both ways, so most
    /// topologies add both directions.
    pub const fn reversed(self) -> Self {
        Self {
            from: self.to,
            to: self.from,
            cost: self.cost,
        }
    }
}

/// A graph of nodes and links at one instant.
///
/// The node count is fixed at construction and nodes are identified by their
/// index. Edges are directed: adding an edge from one node to another does
/// not add the reverse, so a caller that wants a bidirectional link adds
/// both directions. This is deliberate, because an inter-satellite link may
/// have different capacity in each direction and a ground link is often
/// asymmetric.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Topology {
    node_count: usize,
    edges: Vec<Edge>,
}

impl Topology {
    /// Creates an empty topology with the given number of nodes.
    pub const fn new(node_count: usize) -> Self {
        Self {
            node_count,
            edges: Vec::new(),
        }
    }

    /// Returns the number of nodes.
    pub const fn node_count(&self) -> usize {
        self.node_count
    }

    /// Returns the number of edges.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Returns `true` when there are no edges.
    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }

    /// Returns the edges.
    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    /// Adds a directed edge.
    ///
    /// Returns an error when either node index is out of range, when the
    /// cost is not finite and positive, or when the edge already exists.
    /// Rejecting a duplicate rather than replacing it keeps a topology free
    /// of silent contradictions about the cost of a link.
    pub fn add_edge(&mut self, from: usize, to: usize, cost: f64) -> Result<(), TopologyError> {
        if from >= self.node_count || to >= self.node_count {
            return Err(TopologyError::new(TopologyErrorKind::NoSuchNode)
                .with_detail(format!("nodes {from} and {to} of {}", self.node_count)));
        }
        if !cost.is_finite() || cost <= 0.0 {
            return Err(TopologyError::new(TopologyErrorKind::InvalidCost)
                .with_detail(format!("cost {cost}")));
        }
        if self.has_edge(from, to) {
            return Err(TopologyError::new(TopologyErrorKind::DuplicateEdge)
                .with_detail(format!("edge {from} to {to}")));
        }
        self.edges.push(Edge::new(from, to, cost));
        Ok(())
    }

    /// Adds an edge in both directions with the same cost.
    ///
    /// If the second direction fails after the first succeeds, the first is
    /// rolled back, so the topology is left unchanged when the call reports
    /// an error.
    pub fn add_bidirectional_edge(
        &mut self,
        first: usize,
        second: usize,
        cost: f64,
    ) -> Result<(), TopologyError> {
        self.add_edge(first, second, cost)?;
        if let Err(error) = self.add_edge(second, first, cost) {
            // Remove the first direction so the topology is unchanged.
            self.edges
                .retain(|edge| !(edge.from == first && edge.to == second));
            return Err(error);
        }
        Ok(())
    }

    /// Returns `true` when an edge exists from one node to another.
    pub fn has_edge(&self, from: usize, to: usize) -> bool {
        self.edges
            .iter()
            .any(|edge| edge.from == from && edge.to == to)
    }

    /// Returns the cost of the edge between two nodes.
    ///
    /// Returns `None` when there is no such edge.
    pub fn cost(&self, from: usize, to: usize) -> Option<f64> {
        self.edges
            .iter()
            .find(|edge| edge.from == from && edge.to == to)
            .map(|edge| edge.cost)
    }

    /// Removes the edge between two nodes.
    ///
    /// Returns an error when there is no such edge.
    pub fn remove_edge(&mut self, from: usize, to: usize) -> Result<(), TopologyError> {
        let before = self.edges.len();
        self.edges
            .retain(|edge| !(edge.from == from && edge.to == to));
        if self.edges.len() == before {
            return Err(TopologyError::new(TopologyErrorKind::NoSuchEdge)
                .with_detail(format!("edge {from} to {to}")));
        }
        Ok(())
    }

    /// Returns the indices of the nodes reachable from a node in one hop.
    ///
    /// The result is in the order the edges were added, so it is
    /// deterministic for a given construction sequence.
    pub fn neighbours(&self, node: usize) -> Vec<usize> {
        self.edges
            .iter()
            .filter(|edge| edge.from == node)
            .map(|edge| edge.to)
            .collect()
    }

    /// Returns the number of edges leaving a node.
    pub fn out_degree(&self, node: usize) -> usize {
        self.edges.iter().filter(|edge| edge.from == node).count()
    }

    /// Returns the number of edges arriving at a node.
    pub fn in_degree(&self, node: usize) -> usize {
        self.edges.iter().filter(|edge| edge.to == node).count()
    }

    /// Returns the adjacency of every node.
    ///
    /// The result has one entry per node, in node order, each listing the
    /// neighbours of that node.
    pub fn adjacency(&self) -> Vec<Vec<usize>> {
        (0..self.node_count)
            .map(|node| self.neighbours(node))
            .collect()
    }

    /// Returns the number of connected components, treating the edges as
    /// undirected.
    ///
    /// This answers how many separate networks the topology forms, which is
    /// the first thing to check about a constellation snapshot: a
    /// constellation that has split into pieces can still carry traffic
    /// within each piece but not between them.
    pub fn connected_components(&self) -> usize {
        if self.node_count == 0 {
            return 0;
        }
        let mut visited = vec![false; self.node_count];
        let mut components = 0;
        for start in 0..self.node_count {
            if visited[start] {
                continue;
            }
            components += 1;
            // A breadth first walk over the edges in both directions.
            let mut stack = vec![start];
            visited[start] = true;
            while let Some(node) = stack.pop() {
                for edge in &self.edges {
                    let neighbour = if edge.from == node {
                        edge.to
                    } else if edge.to == node {
                        edge.from
                    } else {
                        continue;
                    };
                    if !visited[neighbour] {
                        visited[neighbour] = true;
                        stack.push(neighbour);
                    }
                }
            }
        }
        components
    }

    /// Returns `true` when every node can reach every other node, treating
    /// the edges as undirected.
    pub fn is_connected(&self) -> bool {
        self.node_count <= 1 || self.connected_components() == 1
    }

    /// Returns the total cost of every edge.
    ///
    /// This is a crude measure of the network's extent, useful for comparing
    /// snapshots rather than for routing.
    pub fn total_cost(&self) -> f64 {
        self.edges.iter().map(|edge| edge.cost).sum()
    }
}
