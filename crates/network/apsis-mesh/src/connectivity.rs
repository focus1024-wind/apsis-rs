//! Connectivity measures over a topology.

use apsis_topology::Topology;

/// Returns the set of nodes reachable from a node, following edges in their
/// direction.
///
/// The start node is included, since a node reaches itself by a path of no
/// hops. The result is in node order.
///
/// Returns an empty vector when the node is out of range.
pub fn reachable_from(topology: &Topology, source: usize) -> Vec<usize> {
    if source >= topology.node_count() {
        return Vec::new();
    }
    let mut visited = vec![false; topology.node_count()];
    visited[source] = true;
    let mut stack = vec![source];
    while let Some(node) = stack.pop() {
        for edge in topology.edges().iter().filter(|edge| edge.from == node) {
            if !visited[edge.to] {
                visited[edge.to] = true;
                stack.push(edge.to);
            }
        }
    }
    (0..topology.node_count())
        .filter(|node| visited[*node])
        .collect()
}

/// Returns the number of independent paths between two nodes.
///
/// Two paths are independent when they share no edge. By Menger's theorem
/// that number is the minimum number of edges whose removal disconnects the
/// two nodes, so it is the number of link failures the connection survives.
///
/// The value is computed as a maximum flow of unit capacity between the two
/// nodes, using the Edmonds-Karp variant of the Ford-Fulkerson method, which
/// augments along a shortest path at each step. For the small graphs of a
/// satellite mesh this is fast, and it is exact.
///
/// Returns `Some(0)` when no path exists, and `None` when either node is out
/// of range or the two nodes are the same, for which the question is not
/// defined.
pub fn edge_connectivity(topology: &Topology, source: usize, destination: usize) -> Option<usize> {
    if source >= topology.node_count() || destination >= topology.node_count() {
        return None;
    }
    if source == destination {
        return None;
    }

    // A residual capacity for each edge, indexed by the edge position.
    let edges = topology.edges();
    let mut residual: Vec<usize> = vec![1; edges.len()];

    let mut flow = 0;
    loop {
        // Breadth first search for an augmenting path in the residual
        // graph.
        let mut parent_edge: Vec<Option<usize>> = vec![None; topology.node_count()];
        let mut visited = vec![false; topology.node_count()];
        visited[source] = true;
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(source);

        while let Some(node) = queue.pop_front() {
            if node == destination {
                break;
            }
            for (index, edge) in edges.iter().enumerate() {
                // Forward edges with residual capacity.
                if edge.from == node && residual[index] > 0 && !visited[edge.to] {
                    visited[edge.to] = true;
                    parent_edge[edge.to] = Some(index);
                    queue.push_back(edge.to);
                    continue;
                }
                // Reverse edges, which exist when flow has been pushed the
                // other way and which cancel it.
                if edge.to == node && residual[index] == 0 && !visited[edge.from] {
                    visited[edge.from] = true;
                    parent_edge[edge.from] = Some(index);
                    queue.push_back(edge.from);
                }
            }
        }

        if !visited[destination] {
            break;
        }

        // Walk the path back, adjusting the residual capacities. An edge
        // traversed in its own direction consumes one unit; an edge
        // traversed against its direction restores one.
        let mut node = destination;
        while node != source {
            let index = parent_edge[node]?;
            let edge = edges[index];
            if edge.to == node {
                residual[index] -= 1;
                node = edge.from;
            } else {
                residual[index] += 1;
                node = edge.to;
            }
        }
        flow += 1;
    }
    Some(flow)
}

/// Returns the number of strongly connected components.
///
/// A component is strongly connected when every node in it can reach every
/// other following the edges in their direction. This is the count of
/// separate networks that can route internally, which for a satellite mesh
/// with directed inter-satellite links is the right measure of how many
/// pieces it has.
///
/// The algorithm is Kosaraju's: a depth first search records the order in
/// which nodes finish, and a second search over the reversed edges, taking
/// the nodes in reverse finishing order, emits the components.
///
/// Source: Kosaraju's algorithm as described in Cormen et al.,
/// *Introduction to Algorithms*, 3rd edition, MIT Press, 2009, chapter 22.
pub fn strongly_connected_components(topology: &Topology) -> usize {
    let node_count = topology.node_count();
    if node_count == 0 {
        return 0;
    }

    // First pass: the finishing order over the forward edges.
    let mut visited = vec![false; node_count];
    let mut order = Vec::with_capacity(node_count);
    for start in 0..node_count {
        if visited[start] {
            continue;
        }
        // An iterative depth first search, pushing a node again to mark the
        // point at which its children are done.
        let mut stack = vec![(start, false)];
        while let Some((node, processed)) = stack.pop() {
            if processed {
                order.push(node);
                continue;
            }
            if visited[node] {
                continue;
            }
            visited[node] = true;
            stack.push((node, true));
            for edge in topology.edges().iter().filter(|edge| edge.from == node) {
                if !visited[edge.to] {
                    stack.push((edge.to, false));
                }
            }
        }
    }

    // Second pass: the reverse edges, taking nodes in reverse finishing
    // order.
    let mut assigned = vec![false; node_count];
    let mut components = 0;
    for node in order.into_iter().rev() {
        if assigned[node] {
            continue;
        }
        components += 1;
        let mut stack = vec![node];
        assigned[node] = true;
        while let Some(current) = stack.pop() {
            for edge in topology.edges().iter().filter(|edge| edge.to == current) {
                if !assigned[edge.from] {
                    assigned[edge.from] = true;
                    stack.push(edge.from);
                }
            }
        }
    }
    components
}
