//! Connectivity measures over a topology.
//!
//! 某拓扑上的连通性度量。

use apsis_topology::Topology;

/// Returns the set of nodes reachable from a node, following edges in their
/// direction.
///
/// The start node is included, since a node reaches itself by a path of no
/// hops. The result is in node order.
///
/// Returns an empty vector when the node is out of range.
///
/// 沿边的方向返回自某节点可达的节点集合。
///
/// 起点节点被包含在内，因为节点经零跳路径可达自身。结果按节点次序
/// 排列。
///
/// 节点越界时返回空向量。
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
///
/// 返回两节点之间相互独立的路径条数。
///
/// 两条路径在不含公共边时相互独立。依 Menger 定理，该条数即使这两节点
/// 断连所需移除的最少边数，故即该连接可承受的链路失效次数。
///
/// 该值以两节点之间单位容量的最大流计算，采用 Ford-Fulkerson 方法的
/// Edmonds-Karp 变体，每步沿最短增广。对卫星网这样的小图它既快又
/// 精确。
///
/// 不存在路径时返回 `Some(0)`；任一节点越界或两节点相同（此时该问题
/// 无定义）时返回 `None`。
pub fn edge_connectivity(topology: &Topology, source: usize, destination: usize) -> Option<usize> {
    if source >= topology.node_count() || destination >= topology.node_count() {
        return None;
    }
    if source == destination {
        return None;
    }

    // A residual capacity for each edge, indexed by the edge position.
    // 每条边一个残余容量，按边的位置索引。
    let edges = topology.edges();
    let mut residual: Vec<usize> = vec![1; edges.len()];

    let mut flow = 0;
    loop {
        // Breadth first search for an augmenting path in the residual
        // graph.
        // 在残余图中广度优先搜索一条增广路径。
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
                // 具有残余容量的正向边。
                if edge.from == node && residual[index] > 0 && !visited[edge.to] {
                    visited[edge.to] = true;
                    parent_edge[edge.to] = Some(index);
                    queue.push_back(edge.to);
                    continue;
                }
                // Reverse edges, which exist when flow has been pushed the
                // other way and which cancel it.
                // 反向边，当流量已沿另一方向推送时存在，用以抵消它。
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
        // 沿路径回溯，调整残余容量。沿自身方向穿越的边消耗一个单位；
        // 逆自身方向穿越的边恢复一个单位。
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
///
/// 返回强连通分量的数量。
///
/// 一个分量内每个节点都能沿边的方向到达其余每个节点时，它即强连通。
/// 该计数是各自内部可路由的独立网络数，对含定向星间链路的卫星网而
/// 言，它正是衡量该网由几部分构成的恰当度量。
///
/// 算法为 Kosaraju：一次深度优先搜索记录各节点的完成次序，再在反向
/// 边上按完成次序的逆序做第二次搜索，输出各分量。
///
/// 出处：Kosaraju 算法，见 Cormen 等，*Introduction to Algorithms*，
/// 第 3 版，MIT Press，2009，第 22 章。
pub fn strongly_connected_components(topology: &Topology) -> usize {
    let node_count = topology.node_count();
    if node_count == 0 {
        return 0;
    }

    // First pass: the finishing order over the forward edges.
    // 第一遍：在正向边上求完成次序。
    let mut visited = vec![false; node_count];
    let mut order = Vec::with_capacity(node_count);
    for start in 0..node_count {
        if visited[start] {
            continue;
        }
        // An iterative depth first search, pushing a node again to mark the
        // point at which its children are done.
        // 迭代式深度优先搜索，把某节点再次压栈以标记其子节点完成之处。
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
    // 第二遍：在反向边上按完成次序的逆序取节点。
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
