//! The mesh type and the summary of its redundancy.

use crate::connectivity::{edge_connectivity, reachable_from, strongly_connected_components};
use apsis_topology::Topology;

/// A link whose removal would change the connectivity of the mesh.
///
/// 一条移除后会改变网格连通性的链路。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CriticalLink {
    /// The node the link leaves.
    ///
    /// 该链路离开的节点。
    pub from: usize,
    /// The node the link arrives at.
    ///
    /// 该链路到达的节点。
    pub to: usize,
    /// The cost of the link.
    ///
    /// 该链路的代价。
    pub cost: f64,
    /// The number of components the mesh has after the link is removed.
    ///
    /// 移除该链路后网格所具有的分量数。
    pub components_without: usize,
}

impl CriticalLink {
    /// Creates a critical link record.
    ///
    /// 创建关键链路记录。
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
///
/// 网格连通性的摘要。
#[derive(Debug, Clone, PartialEq)]
pub struct MeshSummary {
    /// The number of terminals.
    ///
    /// 终端的数量。
    pub node_count: usize,
    /// The number of directed links.
    ///
    /// 有向链路的数量。
    pub edge_count: usize,
    /// The number of components when the links are treated as undirected.
    ///
    /// 把链路视为无向时的分量数。
    pub components: usize,
    /// The number of strongly connected components.
    ///
    /// 强连通分量的数量。
    pub strongly_connected_components: usize,
    /// The smallest number of independent paths between any two distinct
    /// terminals, or `None` when there are fewer than two terminals.
    ///
    /// This is the worst connection in the mesh. A value of one means some
    /// pair of terminals relies on a single link and the mesh has no
    /// redundancy anywhere.
    ///
    /// 任意两个不同终端之间独立路径条数的最小值；终端少于两个时返回
    /// `None`。
    ///
    /// 这是网格中最差的连接。值为一意味着某对终端依赖单条链路，网格
    /// 处处都无冗余。
    pub minimum_connectivity: Option<usize>,
}

/// A mesh of terminals joined by links.
///
/// The mesh wraps a topology and adds the connectivity measures a mesh
/// application needs. It does not own a schedule or a traffic model; those
/// belong to the caller.
///
/// 由链路连接的终端构成的网格。
///
/// 网格封装一个拓扑并加入网格应用所需的连通性度量。它不拥有调度或业务
/// 模型；那些属于调用方。
#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    topology: Topology,
}

impl Mesh {
    /// Creates a mesh over a topology.
    ///
    /// 在某拓扑之上创建网格。
    pub const fn new(topology: Topology) -> Self {
        Self { topology }
    }

    /// Returns the underlying topology.
    ///
    /// 返回底层拓扑。
    pub const fn topology(&self) -> &Topology {
        &self.topology
    }

    /// Returns the number of terminals.
    ///
    /// 返回终端的数量。
    pub fn node_count(&self) -> usize {
        self.topology.node_count()
    }

    /// Returns the number of links.
    ///
    /// 返回链路的数量。
    pub fn edge_count(&self) -> usize {
        self.topology.edge_count()
    }

    /// Returns `true` when every terminal reaches every other, treating the
    /// links as undirected.
    ///
    /// 把链路视为无向时，当每个终端都能到达其余每个终端，返回 `true`。
    pub fn is_connected(&self) -> bool {
        self.topology.is_connected()
    }

    /// Returns the terminals reachable from a terminal.
    ///
    /// 返回自某终端可达的各终端。
    pub fn reachable_from(&self, source: usize) -> Vec<usize> {
        reachable_from(&self.topology, source)
    }

    /// Returns the number of independent paths between two terminals.
    ///
    /// Returns `None` when either terminal is out of range or the two are
    /// the same.
    ///
    /// 返回两个终端之间独立路径的条数。
    ///
    /// 任一终端越界或两者相同时返回 `None`。
    pub fn edge_connectivity(&self, source: usize, destination: usize) -> Option<usize> {
        edge_connectivity(&self.topology, source, destination)
    }

    /// Returns the smallest number of independent paths between any two
    /// distinct terminals.
    ///
    /// Returns `None` when there are fewer than two terminals.
    ///
    /// 返回任意两个不同终端之间独立路径条数的最小值。
    ///
    /// 终端少于两个时返回 `None`。
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
    ///
    /// 返回移除后会增加连通分量数的那些链路，把每条链路视为无向。
    ///
    /// 它们是网格底层无向图的桥。一条链路只报告一次，即便它有双向，
    /// 因为一次失效是整条链路失效而非其中一个方向。
    ///
    /// 每个候选通过在两方向移除该链路并比较分量数来发现，这精确但每条
    /// 链路要花一次遍历。对卫星网所呈现的图规模而言这是可接受的，且比
    /// 线性时间的桥查找算法更易验证。
    ///
    /// 非桥的链路被省略，故空结果意味着每条链路都在某个环上，网格在
    /// 任何单条链路失效下都不会裂开。
    pub fn critical_links(&self) -> Vec<CriticalLink> {
        let baseline = self.topology.connected_components();
        let mut critical = Vec::new();
        let mut seen: Vec<(usize, usize)> = Vec::new();

        for edge in self.topology.edges() {
            // Canonicalize the pair so that the two directions of one link
            // are considered together and reported once.
            // 规范化该点对，使一条链路的两个方向被一并考虑并只报告一次。
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
            // 在它出现的每个方向上移除该链路，这正是链路失效所做的。
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
    ///
    /// 当网格在任何单条链路失效下都不裂开时返回 `true`。
    ///
    /// 这是网格具备单链路失效容错的条件，而这是不中断地承载业务的网络
    /// 应有的最低冗余。
    pub fn survives_single_link_failure(&self) -> bool {
        self.critical_links().is_empty()
    }

    /// Returns the number of strongly connected components.
    ///
    /// For a mesh of directed links this counts the pieces that can route
    /// internally. A value above one means some terminals cannot be reached
    /// from some others.
    ///
    /// 返回强连通分量的数量。
    ///
    /// 对有向链路的网格而言，它统计可各自内部路由的块数。大于一的值
    /// 意味着有些终端无法从另一些到达。
    pub fn strongly_connected_components(&self) -> usize {
        strongly_connected_components(&self.topology)
    }

    /// Returns a summary of the mesh connectivity.
    ///
    /// 返回网格连通性的摘要。
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
