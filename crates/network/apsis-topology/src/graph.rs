//! The topology graph.
//!
//! 拓扑图。

use crate::error::{TopologyError, TopologyErrorKind};

/// One link in a topology.
///
/// 拓扑中的一条链路。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Edge {
    /// The index of the first endpoint.
    ///
    /// 第一个端点的下标。
    pub from: usize,
    /// The index of the second endpoint.
    ///
    /// 第二个端点的下标。
    pub to: usize,
    /// The cost of traversing the link.
    ///
    /// The meaning is the caller's: a delay in seconds, a hop count, or any
    /// other measure a routing algorithm minimises. It must be finite and
    /// positive, so that adding edges never shortens a path.
    ///
    /// 穿越该链路的代价。
    ///
    /// 其含义由调用方定义：以秒计的时延、跳数，或路由算法所最小化的
    /// 任何其他度量。它必须有限且为正，使添加边绝不缩短路径。
    pub cost: f64,
}

impl Edge {
    /// Creates an edge.
    ///
    /// 创建一条边。
    #[must_use]
    pub const fn new(from: usize, to: usize, cost: f64) -> Self {
        Self { from, to, cost }
    }

    /// Returns the same edge with its endpoints exchanged.
    ///
    /// A link between two satellites carries traffic both ways, so most
    /// topologies add both directions.
    ///
    /// 返回端点互换后的同一条边。
    ///
    /// 两颗卫星之间的链路双向承载业务，故多数拓扑会添加两个方向。
    #[must_use]
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
///
/// 某一瞬时的节点与链路图。
///
/// 节点数在构造时固定，节点以其下标标识。边是有向的：从一节点向另一
/// 节点添加边不会添加反向边，故需要双向链路的调用方要添加两个方向。
/// 这是有意的，因为星间链路每个方向的容量可能不同，而地面链路往往
/// 不对称。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Topology {
    node_count: usize,
    edges: Vec<Edge>,
    /// The positions in `edges` of the edges leaving each node.
    ///
    /// The index is what makes membership a constant time test. Without it,
    /// asking whether an edge already exists scans the whole edge list, and
    /// building a graph of `E` edges costs `O(E^2)`, which turns the
    /// visibility sweep of a constellation, quadratic in the number of
    /// satellites, into cubic work. The index holds positions rather than
    /// destinations so that `remove_edge`, which invalidates positions, can
    /// repair it in one pass.
    ///
    /// 各节点出发的边在 `edges` 中的位置。
    ///
    /// 该索引使成员判断成为常数时间测试。没有它，询问某边是否已存在要
    /// 扫描整个边表，构建含 `E` 条边的图耗费 `O(E^2)`，这把星座的可见性
    /// 遍历（对卫星数二次）变成三次的工作量。该索引保存位置而非目的地，
    /// 使会使位置失效的 `remove_edge` 能一遍修好它。
    outgoing: Vec<Vec<usize>>,
}

impl Topology {
    /// Creates an empty topology with the given number of nodes.
    ///
    /// This is not a `const fn` because it sizes the adjacency index, which
    /// allocates.
    ///
    /// 以给定节点数创建空拓扑。
    ///
    /// 它不是 `const fn`，因为它要为邻接索引确定大小，而这会分配。
    pub fn new(node_count: usize) -> Self {
        let mut outgoing = Vec::new();
        outgoing.resize_with(node_count, Vec::new);
        Self {
            node_count,
            edges: Vec::new(),
            outgoing,
        }
    }

    /// Returns the number of nodes.
    ///
    /// 返回节点的数量。
    #[must_use]
    pub const fn node_count(&self) -> usize {
        self.node_count
    }

    /// Returns the number of edges.
    ///
    /// 返回边的数量。
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Returns `true` when there are no edges.
    ///
    /// 当没有边时返回 `true`。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }

    /// Returns the edges.
    ///
    /// 返回各条边。
    #[must_use]
    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    /// Adds a directed edge.
    ///
    /// Returns an error when either node index is out of range, when the
    /// cost is not finite and positive, or when the edge already exists.
    /// Rejecting a duplicate rather than replacing it keeps a topology free
    /// of silent contradictions about the cost of a link.
    ///
    /// 添加一条有向边。
    ///
    /// 任一端点下标越界、代价非有限且为正，或该边已存在时返回错误。拒绝
    /// 重复边而非替换它，使拓扑不会对某条链路的代价产生无声的矛盾。
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
        self.outgoing[from].push(self.edges.len());
        self.edges.push(Edge::new(from, to, cost));
        Ok(())
    }

    /// Adds an edge in both directions with the same cost.
    ///
    /// If the second direction fails after the first succeeds, the first is
    /// rolled back, so the topology is left unchanged when the call reports
    /// an error.
    ///
    /// 以相同代价在两个方向添加边。
    ///
    /// 若第一个方向成功后第二个方向失败，则回滚第一个，故调用报告错误
    /// 时拓扑保持不变。
    pub fn add_bidirectional_edge(
        &mut self,
        first: usize,
        second: usize,
        cost: f64,
    ) -> Result<(), TopologyError> {
        self.add_edge(first, second, cost)?;
        if let Err(error) = self.add_edge(second, first, cost) {
            // Remove the first direction so the topology is unchanged.
            // 移除第一个方向，使拓扑保持不变。
            self.remove_edge_unchecked(first, second);
            return Err(error);
        }
        Ok(())
    }

    /// Returns `true` when an edge exists from one node to another.
    ///
    /// 当存在从一节点到另一节点的边时返回 `true`。
    #[must_use]
    pub fn has_edge(&self, from: usize, to: usize) -> bool {
        self.outgoing.get(from).is_some_and(|positions| {
            positions
                .iter()
                .any(|&position| self.edges[position].to == to)
        })
    }

    /// Returns the cost of the edge between two nodes.
    ///
    /// Returns `None` when there is no such edge.
    ///
    /// 返回两节点之间边的代价。
    ///
    /// 不存在该边时返回 `None`。
    #[must_use]
    pub fn cost(&self, from: usize, to: usize) -> Option<f64> {
        let positions = self.outgoing.get(from)?;
        positions
            .iter()
            .find(|&&position| self.edges[position].to == to)
            .map(|&position| self.edges[position].cost)
    }

    /// Removes the edge between two nodes.
    ///
    /// Returns an error when there is no such edge.
    ///
    /// 移除两节点之间的边。
    ///
    /// 不存在该边时返回错误。
    pub fn remove_edge(&mut self, from: usize, to: usize) -> Result<(), TopologyError> {
        if !self.has_edge(from, to) {
            return Err(TopologyError::new(TopologyErrorKind::NoSuchEdge)
                .with_detail(format!("edge {from} to {to}")));
        }
        self.remove_edge_unchecked(from, to);
        Ok(())
    }

    /// Removes an edge the caller has already established exists.
    ///
    /// Removing an edge renumbers every position after it, so the index is
    /// rebuilt from the surviving edges in one pass. Removal is rarer than
    /// insertion, so paying for a rebuild here is cheaper than keeping the
    /// index correct incrementally.
    ///
    /// 移除调用方已确认存在的边。
    ///
    /// 移除边会重编号其后的每个位置，故索引由存活的边一遍重建。移除比
    /// 插入少见，故此处付出一次重建的代价比增量地保持索引正确更便宜。
    fn remove_edge_unchecked(&mut self, from: usize, to: usize) {
        self.edges
            .retain(|edge| !(edge.from == from && edge.to == to));
        self.rebuild_index();
    }

    /// Rebuilds the adjacency index from the current edge list.
    ///
    /// 由当前边表重建邻接索引。
    fn rebuild_index(&mut self) {
        for positions in &mut self.outgoing {
            positions.clear();
        }
        for (position, edge) in self.edges.iter().enumerate() {
            self.outgoing[edge.from].push(position);
        }
    }

    /// Returns the indices of the nodes reachable from a node in one hop.
    ///
    /// The result is in the order the edges were added, so it is
    /// deterministic for a given construction sequence.
    ///
    /// 返回自某节点一跳可达的节点下标。
    ///
    /// 结果按边被添加的次序排列，故对给定的构建序列是确定的。
    #[must_use]
    pub fn neighbours(&self, node: usize) -> Vec<usize> {
        self.outgoing
            .get(node)
            .map(|positions| positions.iter().map(|&p| self.edges[p].to).collect())
            .unwrap_or_default()
    }

    /// Returns the number of edges leaving a node.
    ///
    /// 返回离开某节点的边数。
    pub fn out_degree(&self, node: usize) -> usize {
        self.outgoing.get(node).map_or(0, Vec::len)
    }

    /// Returns the number of edges arriving at a node.
    ///
    /// 返回到达某节点的边数。
    #[must_use]
    pub fn in_degree(&self, node: usize) -> usize {
        self.edges.iter().filter(|edge| edge.to == node).count()
    }

    /// Returns the adjacency of every node.
    ///
    /// The result has one entry per node, in node order, each listing the
    /// neighbours of that node.
    ///
    /// 返回每个节点的邻接。
    ///
    /// 结果每个节点一项，按节点次序排列，各自列出该节点的邻居。
    #[must_use]
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
    ///
    /// 把边视为无向时返回连通分量的数量。
    ///
    /// 它回答该拓扑构成多少个独立网络，这是检查星座快照时要看的第一件
    /// 事：已裂成数块的星座仍可在各块内承载业务，但块与块之间不行。
    #[must_use]
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
            // 沿两个方向的边做广度优先遍历。
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
    ///
    /// 把边视为无向时，当每个节点都能到达其余每个节点，返回 `true`。
    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.node_count <= 1 || self.connected_components() == 1
    }

    /// Returns the total cost of every edge.
    ///
    /// This is a crude measure of the network's extent, useful for comparing
    /// snapshots rather than for routing.
    ///
    /// 返回所有边的代价之和。
    ///
    /// 这是对网络规模的一个粗略度量，用于比较快照而非用于路由。
    #[must_use]
    pub fn total_cost(&self) -> f64 {
        self.edges.iter().map(|edge| edge.cost).sum()
    }
}
