//! The route types.

/// One hop of a route.
///
/// A step records the link traversed and the cost of that link, so a caller
/// can attribute a route's total cost to its parts.
///
/// 路由的一跳。
///
/// 一个步骤记录所穿越的链路及其代价，使调用方能把路由的总代价归因于
/// 各部分。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RouteStep {
    /// The node the step leaves.
    ///
    /// 该步骤离开的节点。
    pub from: usize,
    /// The node the step arrives at.
    ///
    /// 该步骤到达的节点。
    pub to: usize,
    /// The cost of the link.
    ///
    /// 该链路的代价。
    pub cost: f64,
}

impl RouteStep {
    /// Creates a step.
    ///
    /// 创建一个步骤。
    pub const fn new(from: usize, to: usize, cost: f64) -> Self {
        Self { from, to, cost }
    }
}

/// A route between two nodes.
///
/// The node list includes both endpoints, so a route of one hop has two
/// nodes and one step, and a route from a node to itself has one node and no
/// steps.
///
/// 两个节点之间的路由。
///
/// 节点列表包含两个端点，故一跳的路由有两个节点和一个步骤，自某节点到
/// 其自身的路由有一个节点且没有步骤。
#[derive(Debug, Clone, PartialEq)]
pub struct Route {
    /// The nodes in order, from the source to the destination.
    ///
    /// 按次序排列的节点，从源到目的地。
    pub nodes: Vec<usize>,
    /// The links traversed, in order.
    ///
    /// 按次序穿越的链路。
    pub steps: Vec<RouteStep>,
    /// The total cost of the route.
    ///
    /// 该路由的总代价。
    pub cost: f64,
}

impl Route {
    /// Creates a route from its nodes and steps.
    ///
    /// The cost is taken as the sum of the step costs, so a caller cannot
    /// make the total disagree with the parts.
    ///
    /// 由其节点与步骤创建路由。
    ///
    /// 代价取各步骤代价之和，故调用方无法使总计与各部分不一致。
    pub fn new(nodes: Vec<usize>, steps: Vec<RouteStep>) -> Self {
        let cost = steps.iter().map(|step| step.cost).sum();
        Self { nodes, steps, cost }
    }

    /// Returns the number of hops.
    ///
    /// 返回跳数。
    pub fn hop_count(&self) -> usize {
        self.steps.len()
    }

    /// Returns the source node.
    ///
    /// Returns `None` when the route has no nodes.
    ///
    /// 返回源节点。
    ///
    /// 路由没有节点时返回 `None`。
    pub fn source(&self) -> Option<usize> {
        self.nodes.first().copied()
    }

    /// Returns the destination node.
    ///
    /// Returns `None` when the route has no nodes.
    ///
    /// 返回目的地节点。
    ///
    /// 路由没有节点时返回 `None`。
    pub fn destination(&self) -> Option<usize> {
        self.nodes.last().copied()
    }

    /// Returns the route traversed in the opposite direction.
    ///
    /// The costs are unchanged, because a route reversed traverses the same
    /// links. This is only correct when every link is present in both
    /// directions with the same cost, which a caller must ensure.
    ///
    /// 返回按相反方向穿越的路由。
    ///
    /// 各代价不变，因为反转后的路由穿越同样的链路。仅当每条链路都以
    /// 相同代价双向存在时才正确，这一点须由调用方保证。
    pub fn reversed(&self) -> Self {
        let nodes = self.nodes.iter().rev().copied().collect();
        let steps = self
            .steps
            .iter()
            .rev()
            .map(|step| RouteStep::new(step.to, step.from, step.cost))
            .collect();
        Self::new(nodes, steps)
    }

    /// Returns `true` when the route passes through a node.
    ///
    /// 当该路由经过某节点时返回 `true`。
    pub fn passes_through(&self, node: usize) -> bool {
        self.nodes.contains(&node)
    }

    /// Returns the position of a node in the route, if it is on it.
    ///
    /// 返回某节点在路由中的位置（若在该路由上）。
    pub fn position_of(&self, node: usize) -> Option<usize> {
        self.nodes.iter().position(|candidate| *candidate == node)
    }
}

/// A route with the interval over which it is valid.
///
/// 一条路由及其有效的时段。
#[derive(Debug, Clone, PartialEq)]
pub struct RouteLeg {
    /// The route.
    ///
    /// 该路由。
    pub route: Route,
    /// The time from which the route is valid, in seconds.
    ///
    /// 该路由开始有效的时间，单位为秒。
    pub start_seconds: f64,
    /// The time until which the route is valid, in seconds.
    ///
    /// This is the time of the topology change that invalidates the route.
    /// It is infinite when the route is valid for all later times.
    ///
    /// 该路由保持有效直到的时间，单位为秒。
    ///
    /// 这是使该路由失效的拓扑变化的时间。当该路由对其后所有时间都有效
    /// 时它为无穷。
    pub end_seconds: f64,
}

impl RouteLeg {
    /// Creates a route leg.
    ///
    /// 创建一段路由。
    pub const fn new(route: Route, start_seconds: f64, end_seconds: f64) -> Self {
        Self {
            route,
            start_seconds,
            end_seconds,
        }
    }

    /// Returns the duration for which the route holds, in seconds.
    ///
    /// Returns infinity when the route is valid for all later times.
    ///
    /// 返回该路由保持的时长，单位为秒。
    ///
    /// 该路由对其后所有时间都有效时返回无穷。
    pub fn duration_seconds(&self) -> f64 {
        self.end_seconds - self.start_seconds
    }
}
