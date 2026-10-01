//! Tests for shortest path routing.
//!
//! 最短路径路由的测试。

use apsis_routing::{
    PathTree, Route, RouteStep, RoutingErrorKind, route_over_series, shortest_path,
    shortest_paths_from,
};
use apsis_topology::{Topology, TopologySample, TopologySeries};

/// A topology with the given directed edges and unit costs.
///
/// 具有给定有向边与单位代价的拓扑。
fn topology_with(node_count: usize, edges: &[(usize, usize)]) -> Topology {
    let mut topology = Topology::new(node_count);
    for &(from, to) in edges {
        topology.add_edge(from, to, 1.0).expect("the nodes exist");
    }
    topology
}

#[test]
fn a_direct_link_is_the_shortest_route() {
    let mut topology = Topology::new(3);
    topology.add_edge(0, 1, 5.0).expect("valid");
    topology.add_edge(0, 2, 1.0).expect("valid");
    topology.add_edge(2, 1, 1.0).expect("valid");

    // The direct link costs five, the two hop route costs two.
    // 直接链路代价为五，两跳路由代价为二。
    let route = shortest_path(&topology, 0, 1).expect("a route exists");
    assert_eq!(route.nodes, vec![0, 2, 1]);
    assert!((route.cost - 2.0).abs() < 1e-12);
    assert_eq!(route.hop_count(), 2);
    assert_eq!(route.source(), Some(0));
    assert_eq!(route.destination(), Some(1));
}

#[test]
fn a_longer_route_in_hops_can_be_cheaper_in_cost() {
    let mut topology = Topology::new(4);
    // A cheap path of three hops against an expensive direct link.
    // 三条廉价跳的路径对一条昂贵的直接链路。
    topology.add_edge(0, 1, 1.0).expect("valid");
    topology.add_edge(1, 2, 1.0).expect("valid");
    topology.add_edge(2, 3, 1.0).expect("valid");
    topology.add_edge(0, 3, 10.0).expect("valid");

    let route = shortest_path(&topology, 0, 3).expect("a route exists");
    assert_eq!(route.nodes, vec![0, 1, 2, 3]);
    assert!((route.cost - 3.0).abs() < 1e-12);
    assert_eq!(route.hop_count(), 3);
}

#[test]
fn the_cost_is_the_sum_of_the_step_costs() {
    let mut topology = Topology::new(3);
    topology.add_edge(0, 1, 2.5).expect("valid");
    topology.add_edge(1, 2, 1.5).expect("valid");
    let route = shortest_path(&topology, 0, 2).expect("a route exists");
    assert!((route.cost - 4.0).abs() < 1e-12);
    let summed: f64 = route.steps.iter().map(|step| step.cost).sum();
    assert!((route.cost - summed).abs() < 1e-12);
}

#[test]
fn a_route_records_its_steps() {
    let mut topology = Topology::new(3);
    topology.add_edge(0, 1, 2.0).expect("valid");
    topology.add_edge(1, 2, 3.0).expect("valid");
    let route = shortest_path(&topology, 0, 2).expect("a route exists");
    assert_eq!(route.steps.len(), 2);
    assert_eq!(route.steps[0], RouteStep::new(0, 1, 2.0));
    assert_eq!(route.steps[1], RouteStep::new(1, 2, 3.0));
}

#[test]
fn a_directed_topology_may_have_no_route() {
    // The link only goes one way, so the reverse route does not exist.
    // 该链路只有一个方向，故反向路由不存在。
    let mut topology = Topology::new(2);
    topology.add_edge(0, 1, 1.0).expect("valid");
    assert!(shortest_path(&topology, 0, 1).is_ok());
    let error = shortest_path(&topology, 1, 0).expect_err("no route");
    assert_eq!(error.kind(), RoutingErrorKind::NoRoute);
}

#[test]
fn a_disconnected_destination_has_no_route() {
    let mut topology = Topology::new(4);
    topology.add_edge(0, 1, 1.0).expect("valid");
    topology.add_edge(2, 3, 1.0).expect("valid");
    let error = shortest_path(&topology, 0, 3).expect_err("no route");
    assert_eq!(error.kind(), RoutingErrorKind::NoRoute);
}

#[test]
fn the_same_endpoint_is_rejected() {
    let topology = topology_with(2, &[(0, 1)]);
    let error = shortest_path(&topology, 0, 0).expect_err("the endpoints are equal");
    assert_eq!(error.kind(), RoutingErrorKind::SameEndpoint);
}

#[test]
fn an_out_of_range_node_is_rejected() {
    let topology = topology_with(2, &[(0, 1)]);
    assert_eq!(
        shortest_path(&topology, 0, 5)
            .expect_err("no such node")
            .kind(),
        RoutingErrorKind::NoSuchNode
    );
    assert_eq!(
        shortest_paths_from(&topology, 5)
            .expect_err("no such node")
            .kind(),
        RoutingErrorKind::NoSuchNode
    );
}

#[test]
fn the_path_tree_covers_every_reachable_node() {
    let mut topology = Topology::new(4);
    topology.add_edge(0, 1, 1.0).expect("valid");
    topology.add_edge(1, 2, 1.0).expect("valid");
    // Node 3 is isolated.
    // 节点 3 是孤立的。
    let tree = shortest_paths_from(&topology, 0).expect("valid");
    assert_eq!(tree.source, 0);
    assert_eq!(tree.distance(0), Some(0.0));
    assert_eq!(tree.distance(1), Some(1.0));
    assert_eq!(tree.distance(2), Some(2.0));
    assert_eq!(tree.distance(3), None);
    assert!(tree.is_reachable(2));
    assert!(!tree.is_reachable(3));
    assert_eq!(tree.reachable_count(), 3);
    assert_eq!(tree.distance(9), None);
}

#[test]
fn the_path_tree_reconstructs_each_route() {
    let mut topology = Topology::new(4);
    topology.add_edge(0, 1, 1.0).expect("valid");
    topology.add_edge(1, 2, 1.0).expect("valid");
    topology.add_edge(2, 3, 1.0).expect("valid");
    let tree = shortest_paths_from(&topology, 0).expect("valid");
    let route = tree.route_to(3, &topology).expect("a route exists");
    assert_eq!(route.nodes, vec![0, 1, 2, 3]);
    assert!((route.cost - 3.0).abs() < 1e-12);

    // The route to the source is the source alone.
    // 到源的路由只有源本身。
    let self_route = tree.route_to(0, &topology).expect("a route exists");
    assert_eq!(self_route.nodes, vec![0]);
    assert_eq!(self_route.hop_count(), 0);
}

#[test]
fn the_path_tree_rejects_an_unreachable_destination() {
    let topology = topology_with(2, &[(0, 1)]);
    let tree = shortest_paths_from(&topology, 1).expect("valid");
    let error = tree.route_to(0, &topology).expect_err("no route");
    assert_eq!(error.kind(), RoutingErrorKind::NoRoute);
    let error = tree.route_to(9, &topology).expect_err("no such node");
    assert_eq!(error.kind(), RoutingErrorKind::NoSuchNode);
}

#[test]
fn a_cheaper_route_replaces_a_more_expensive_one() {
    // The heap must settle the nearer node first even when it is discovered
    // second, which is the case that a naive breadth first search gets
    // wrong.
    // 即便较近的节点是第二个被发现的，堆也必须先结算它，这正是朴素广度优先搜索会
    // 弄错的情形。
    let mut topology = Topology::new(4);
    topology.add_edge(0, 1, 10.0).expect("valid");
    topology.add_edge(0, 2, 1.0).expect("valid");
    topology.add_edge(2, 3, 1.0).expect("valid");
    topology.add_edge(3, 1, 1.0).expect("valid");

    let route = shortest_path(&topology, 0, 1).expect("a route exists");
    assert_eq!(route.nodes, vec![0, 2, 3, 1]);
    assert!((route.cost - 3.0).abs() < 1e-12);
}

#[test]
fn a_zero_cost_route_to_the_source_is_reported() {
    let topology = topology_with(2, &[(0, 1)]);
    let tree = shortest_paths_from(&topology, 0).expect("valid");
    assert_eq!(tree.distance(0), Some(0.0));
    assert_eq!(tree.predecessors[0], None);
}

#[test]
fn a_route_can_be_reversed() {
    let mut topology = Topology::new(3);
    topology.add_bidirectional_edge(0, 1, 2.0).expect("valid");
    topology.add_bidirectional_edge(1, 2, 3.0).expect("valid");
    let route = shortest_path(&topology, 0, 2).expect("a route exists");
    let reversed = route.reversed();
    assert_eq!(reversed.nodes, vec![2, 1, 0]);
    assert!((reversed.cost - route.cost).abs() < 1e-12);
    assert_eq!(reversed.steps[0], RouteStep::new(2, 1, 3.0));
}

#[test]
fn a_route_reports_whether_it_passes_through_a_node() {
    let mut topology = Topology::new(4);
    topology.add_edge(0, 1, 1.0).expect("valid");
    topology.add_edge(1, 2, 1.0).expect("valid");
    let route = shortest_path(&topology, 0, 2).expect("a route exists");
    assert!(route.passes_through(1));
    assert!(!route.passes_through(3));
    assert_eq!(route.position_of(1), Some(1));
    assert_eq!(route.position_of(3), None);
}

#[test]
fn an_empty_topology_has_no_routes() {
    let topology = Topology::new(3);
    let error = shortest_path(&topology, 0, 1).expect_err("no route");
    assert_eq!(error.kind(), RoutingErrorKind::NoRoute);
    let tree = shortest_paths_from(&topology, 0).expect("valid");
    assert_eq!(tree.reachable_count(), 1);
}

#[test]
fn the_route_type_is_constructible_directly() {
    let route = Route::new(vec![0, 1], vec![RouteStep::new(0, 1, 2.0)]);
    assert!((route.cost - 2.0).abs() < 1e-12);
    assert_eq!(route.hop_count(), 1);
    // An empty route has no endpoints.
    // 空路由没有端点。
    let empty = Route::new(Vec::new(), Vec::new());
    assert_eq!(empty.source(), None);
    assert_eq!(empty.destination(), None);
}

#[test]
fn a_route_over_a_series_uses_the_links_present_throughout() {
    // The link 1 to 2 exists at both ends of the interval but is absent in
    // the middle, so no route through it survives the interval. The
    // two hop route via node 3 is present throughout.
    // 链路 1 到 2 在区间两端存在但中间缺席，故没有经它的路由能存活整个区间。经
    // 节点 3 的两跳路由则全程存在。
    let series = TopologySeries::new(vec![
        TopologySample::new(0.0, topology_with(4, &[(0, 1), (1, 2), (0, 3), (3, 2)])),
        TopologySample::new(10.0, topology_with(4, &[(0, 1), (0, 3), (3, 2)])),
        TopologySample::new(20.0, topology_with(4, &[(0, 1), (1, 2), (0, 3), (3, 2)])),
    ])
    .expect("the times increase");

    let leg = route_over_series(&series, 0, 2, 0.0, 20.0).expect("a route exists");
    assert_eq!(leg.route.nodes, vec![0, 3, 2]);
    assert!((leg.route.cost - 2.0).abs() < 1e-12);
    // The leg is valid until the next change after the interval.
    // 该路段有效直到区间之后的下一处变化。
    assert_eq!(leg.start_seconds, 0.0);
    assert!(leg.end_seconds.is_infinite());
}

#[test]
fn a_route_over_a_series_reports_its_validity() {
    let series = TopologySeries::new(vec![
        TopologySample::new(0.0, topology_with(2, &[(0, 1)])),
        TopologySample::new(10.0, topology_with(2, &[(0, 1)])),
        TopologySample::new(20.0, topology_with(2, &[])),
    ])
    .expect("the times increase");

    // A route planned over the first two samples is valid until the third.
    // 在前两个样本上规划的路由有效到第三个。
    let leg = route_over_series(&series, 0, 1, 0.0, 10.0).expect("a route exists");
    assert_eq!(leg.end_seconds, 20.0);
    assert!((leg.duration_seconds() - 20.0).abs() < 1e-12);
}

#[test]
fn a_route_over_a_series_fails_when_no_link_survives() {
    // The only link is present at the start and absent at the end, so it
    // does not survive the interval.
    // 唯一的链路在起点存在、终点缺席，故它不能存活整个区间。
    let series = TopologySeries::new(vec![
        TopologySample::new(0.0, topology_with(2, &[(0, 1)])),
        TopologySample::new(10.0, topology_with(2, &[])),
    ])
    .expect("the times increase");
    let error = route_over_series(&series, 0, 1, 0.0, 10.0).expect_err("no route");
    assert_eq!(error.kind(), RoutingErrorKind::NoRoute);
}

#[test]
fn a_route_over_a_series_validates_its_arguments() {
    let series = TopologySeries::new(vec![TopologySample::new(0.0, topology_with(2, &[(0, 1)]))])
        .expect("valid");

    assert_eq!(
        route_over_series(&series, 0, 1, 10.0, 0.0)
            .expect_err("the interval runs backwards")
            .kind(),
        RoutingErrorKind::InvalidInterval
    );
    assert_eq!(
        route_over_series(&series, 0, 5, 0.0, 0.0)
            .expect_err("no such node")
            .kind(),
        RoutingErrorKind::NoSuchNode
    );
    assert_eq!(
        route_over_series(&series, 0, 0, 0.0, 0.0)
            .expect_err("the endpoints are equal")
            .kind(),
        RoutingErrorKind::SameEndpoint
    );
    assert_eq!(
        route_over_series(&series, 0, 1, f64::NAN, 0.0)
            .expect_err("the interval is not finite")
            .kind(),
        RoutingErrorKind::InvalidInterval
    );
}

#[test]
fn the_path_tree_type_is_public() {
    // The type must be nameable by a caller that stores a routing table,
    // and what it hands back must be the tree for the source asked for.
    // 该类型必须能被存储路由表的调用方指名，且它所交回的必须是所请求源的那棵树。
    fn takes_tree(_tree: &PathTree) {}
    let topology = topology_with(2, &[(0, 1)]);
    let tree = shortest_paths_from(&topology, 0).expect("valid");
    takes_tree(&tree);
    assert_eq!(tree.source, 0);
    assert_eq!(tree.distance(0), Some(0.0));
    assert_eq!(tree.distance(1), Some(1.0));
}
