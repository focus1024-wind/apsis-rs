//! Shortest path computation over a topology.
//!
//! The algorithm is Dijkstra's, using a binary heap keyed by the tentative
//! distance. Costs are positive, so a node's distance is final when it is
//! first removed from the heap, which makes the algorithm exact.
//!
//! The heap is a minimum heap over a reversed ordering key, because the
//! standard library's [`BinaryHeap`](std::collections::BinaryHeap) is a
//! maximum heap. The key is the tentative distance, and ties are broken by
//! the node index so that the result is deterministic for a given topology.

use crate::error::{RoutingError, RoutingErrorKind};
use crate::route::{Route, RouteLeg, RouteStep};
use apsis_topology::{Topology, TopologySeries};
use core::cmp::Ordering;
use std::collections::BinaryHeap;

/// A heap entry holding a node and its tentative distance.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Candidate {
    /// The tentative distance from the source.
    distance: f64,
    /// The node index.
    node: usize,
}

impl Eq for Candidate {}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse the distance comparison so that the smallest distance is
        // the greatest element, which is what the maximum heap needs. Equal
        // distances are ordered by node index so that the sequence of
        // relaxations is reproducible.
        other
            .distance
            .partial_cmp(&self.distance)
            .unwrap_or(Ordering::Equal)
            .then_with(|| other.node.cmp(&self.node))
    }
}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// The result of a single source shortest path computation.
///
/// The distances are to every node reachable from the source, and the
/// predecessor map records the edge by which each node was reached. Both are
/// what a routing table needs, and a single computation fills a whole row of
/// it.
#[derive(Debug, Clone, PartialEq)]
pub struct PathTree {
    /// The source node.
    pub source: usize,
    /// The distance to each node, or infinity when it is unreachable.
    pub distances: Vec<f64>,
    /// The predecessor of each node on the shortest path, if any.
    pub predecessors: Vec<Option<usize>>,
}

impl PathTree {
    /// Returns the distance to a node.
    ///
    /// Returns `None` when the node index is out of range or the node is
    /// unreachable.
    pub fn distance(&self, node: usize) -> Option<f64> {
        let distance = *self.distances.get(node)?;
        distance.is_finite().then_some(distance)
    }

    /// Returns `true` when the node is reachable from the source.
    pub fn is_reachable(&self, node: usize) -> bool {
        self.distance(node).is_some()
    }

    /// Returns the number of reachable nodes, including the source itself.
    pub fn reachable_count(&self) -> usize {
        self.distances
            .iter()
            .filter(|value| value.is_finite())
            .count()
    }

    /// Reconstructs the route to a node.
    ///
    /// Returns an error when the node is out of range or unreachable.
    pub fn route_to(&self, destination: usize, topology: &Topology) -> Result<Route, RoutingError> {
        if destination >= self.distances.len() {
            return Err(RoutingError::new(RoutingErrorKind::NoSuchNode)
                .with_detail(format!("node {destination} of {}", self.distances.len())));
        }
        if !self.is_reachable(destination) {
            return Err(RoutingError::new(RoutingErrorKind::NoRoute)
                .with_detail(format!("no route from {} to {destination}", self.source)));
        }

        // Walk the predecessor chain back to the source, then reverse it.
        let mut nodes = vec![destination];
        let mut current = destination;
        while current != self.source {
            let previous = self.predecessors[current].ok_or_else(|| {
                RoutingError::new(RoutingErrorKind::NoRoute)
                    .with_detail(format!("the predecessor chain breaks at {current}"))
            })?;
            nodes.push(previous);
            current = previous;
        }
        nodes.reverse();

        let mut steps = Vec::with_capacity(nodes.len().saturating_sub(1));
        for pair in nodes.windows(2) {
            let cost = topology.cost(pair[0], pair[1]).ok_or_else(|| {
                RoutingError::new(RoutingErrorKind::NoRoute).with_detail(format!(
                    "the link from {} to {} is absent",
                    pair[0], pair[1]
                ))
            })?;
            steps.push(RouteStep::new(pair[0], pair[1], cost));
        }
        Ok(Route::new(nodes, steps))
    }
}

/// Computes the shortest paths from one source to every reachable node.
///
/// Returns an error when the source node is out of range.
///
/// Source: Dijkstra, E. W., *A note on two problems in connexion with
/// graphs*, Numerische Mathematik 1, 1959.
pub fn shortest_paths_from(topology: &Topology, source: usize) -> Result<PathTree, RoutingError> {
    if source >= topology.node_count() {
        return Err(RoutingError::new(RoutingErrorKind::NoSuchNode)
            .with_detail(format!("node {source} of {}", topology.node_count())));
    }

    let mut distances = vec![f64::INFINITY; topology.node_count()];
    let mut predecessors = vec![None; topology.node_count()];
    let mut settled = vec![false; topology.node_count()];
    distances[source] = 0.0;

    let mut heap = BinaryHeap::new();
    heap.push(Candidate {
        distance: 0.0,
        node: source,
    });

    while let Some(Candidate { distance, node }) = heap.pop() {
        // A stale entry: the node already has a shorter distance.
        if settled[node] {
            continue;
        }
        settled[node] = true;

        for edge in topology.edges().iter().filter(|edge| edge.from == node) {
            let candidate = distance + edge.cost;
            if candidate < distances[edge.to] {
                distances[edge.to] = candidate;
                predecessors[edge.to] = Some(node);
                heap.push(Candidate {
                    distance: candidate,
                    node: edge.to,
                });
            }
        }
    }

    Ok(PathTree {
        source,
        distances,
        predecessors,
    })
}

/// Returns the shortest route between two nodes.
///
/// Returns an error when either node is out of range, when they are the
/// same node, or when no route exists.
pub fn shortest_path(
    topology: &Topology,
    source: usize,
    destination: usize,
) -> Result<Route, RoutingError> {
    if source == destination {
        return Err(RoutingError::new(RoutingErrorKind::SameEndpoint)
            .with_detail(format!("both endpoints are node {source}")));
    }
    let tree = shortest_paths_from(topology, source)?;
    tree.route_to(destination, topology)
}

/// Returns the shortest route valid across an interval of a topology series.
///
/// The route is computed over the links that are present throughout the
/// interval, so it remains valid for the whole of it. The interval is taken
/// from the start of the sample in force at `start_seconds` to the start of
/// the sample in force at `end_seconds`, and the links of every sample in
/// that span are intersected.
///
/// The returned leg records the interval the route is valid for, which is
/// bounded by the next topology change after `end_seconds`.
///
/// Returns an error when either node is out of range or the same, when the
/// interval is not ordered, or when no route survives the whole interval.
///
/// Source: the store and forward routing model over a time-varying graph is
/// the one described in Fraire, J. A. et al., *Routing in Delay-Tolerant
/// Networks Comprising Heterogeneous Populations of Deterministic Link
/// Dynamics*, IEEE Transactions on Mobile Computing, 2020.
pub fn route_over_series(
    series: &TopologySeries,
    source: usize,
    destination: usize,
    start_seconds: f64,
    end_seconds: f64,
) -> Result<RouteLeg, RoutingError> {
    if !start_seconds.is_finite() || !end_seconds.is_finite() || end_seconds < start_seconds {
        return Err(
            RoutingError::new(RoutingErrorKind::InvalidInterval).with_detail(format!(
                "the interval runs from {start_seconds} to {end_seconds}"
            )),
        );
    }
    if source >= series.node_count() || destination >= series.node_count() {
        return Err(
            RoutingError::new(RoutingErrorKind::NoSuchNode).with_detail(format!(
                "nodes {source} and {destination} of {}",
                series.node_count()
            )),
        );
    }
    if source == destination {
        return Err(RoutingError::new(RoutingErrorKind::SameEndpoint)
            .with_detail(format!("both endpoints are node {source}")));
    }

    let first = series.index_at(start_seconds).ok_or_else(|| {
        RoutingError::new(RoutingErrorKind::InvalidInterval)
            .with_detail("the series has no sample at the start of the interval")
    })?;
    let last = series.index_at(end_seconds).ok_or_else(|| {
        RoutingError::new(RoutingErrorKind::InvalidInterval)
            .with_detail("the series has no sample at the end of the interval")
    })?;
    if last > series.len() - 1 {
        return Err(RoutingError::new(RoutingErrorKind::SeriesTooShort)
            .with_detail("the interval extends past the series"));
    }

    // Intersect the links of every sample spanned by the interval, taking
    // the cost from the first sample that has each link.
    let mut intersection = Topology::new(series.node_count());
    let reference = &series.samples()[first].topology;
    for edge in reference.edges() {
        let present_everywhere = (first..=last).all(|index| {
            series.samples()[index]
                .topology
                .has_edge(edge.from, edge.to)
        });
        if present_everywhere {
            // The link was checked above, so a failure here would mean the
            // reference topology contradicts itself.
            let _ = intersection.add_edge(edge.from, edge.to, edge.cost);
        }
    }

    let route = shortest_path(&intersection, source, destination)?;

    // The leg is valid until the topology changes after the interval ends.
    let end_of_validity = series
        .interval_of(last)
        .map_or(end_seconds, |(_, next)| next);
    Ok(RouteLeg::new(route, start_seconds, end_of_validity))
}
