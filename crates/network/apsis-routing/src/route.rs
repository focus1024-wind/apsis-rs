//! The route types.

/// One hop of a route.
///
/// A step records the link traversed and the cost of that link, so a caller
/// can attribute a route's total cost to its parts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RouteStep {
    /// The node the step leaves.
    pub from: usize,
    /// The node the step arrives at.
    pub to: usize,
    /// The cost of the link.
    pub cost: f64,
}

impl RouteStep {
    /// Creates a step.
    pub const fn new(from: usize, to: usize, cost: f64) -> Self {
        Self { from, to, cost }
    }
}

/// A route between two nodes.
///
/// The node list includes both endpoints, so a route of one hop has two
/// nodes and one step, and a route from a node to itself has one node and no
/// steps.
#[derive(Debug, Clone, PartialEq)]
pub struct Route {
    /// The nodes in order, from the source to the destination.
    pub nodes: Vec<usize>,
    /// The links traversed, in order.
    pub steps: Vec<RouteStep>,
    /// The total cost of the route.
    pub cost: f64,
}

impl Route {
    /// Creates a route from its nodes and steps.
    ///
    /// The cost is taken as the sum of the step costs, so a caller cannot
    /// make the total disagree with the parts.
    pub fn new(nodes: Vec<usize>, steps: Vec<RouteStep>) -> Self {
        let cost = steps.iter().map(|step| step.cost).sum();
        Self { nodes, steps, cost }
    }

    /// Returns the number of hops.
    pub fn hop_count(&self) -> usize {
        self.steps.len()
    }

    /// Returns the source node.
    ///
    /// Returns `None` when the route has no nodes.
    pub fn source(&self) -> Option<usize> {
        self.nodes.first().copied()
    }

    /// Returns the destination node.
    ///
    /// Returns `None` when the route has no nodes.
    pub fn destination(&self) -> Option<usize> {
        self.nodes.last().copied()
    }

    /// Returns the route traversed in the opposite direction.
    ///
    /// The costs are unchanged, because a route reversed traverses the same
    /// links. This is only correct when every link is present in both
    /// directions with the same cost, which a caller must ensure.
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
    pub fn passes_through(&self, node: usize) -> bool {
        self.nodes.contains(&node)
    }

    /// Returns the position of a node in the route, if it is on it.
    pub fn position_of(&self, node: usize) -> Option<usize> {
        self.nodes.iter().position(|candidate| *candidate == node)
    }
}

/// A route with the interval over which it is valid.
#[derive(Debug, Clone, PartialEq)]
pub struct RouteLeg {
    /// The route.
    pub route: Route,
    /// The time from which the route is valid, in seconds.
    pub start_seconds: f64,
    /// The time until which the route is valid, in seconds.
    ///
    /// This is the time of the topology change that invalidates the route.
    /// It is infinite when the route is valid for all later times.
    pub end_seconds: f64,
}

impl RouteLeg {
    /// Creates a route leg.
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
    pub fn duration_seconds(&self) -> f64 {
        self.end_seconds - self.start_seconds
    }
}
