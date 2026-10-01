//! Routing over time-varying satellite topologies.
//!
//! The topology of a satellite network changes as the satellites move, so a
//! route is only valid over an interval. This crate computes least cost
//! routes through a [`Topology`] and reports the
//! route together with the time it remains valid.
//!
//! Two forms of routing are provided:
//!
//! - [`shortest_path`] finds the least cost route through the topology in
//!   force at one instant.
//! - [`route_over_series`] finds a route that is valid for a whole interval,
//!   by taking the intersection of the topologies across that interval.
//!
//! The second form is the one a store and forward network needs. A route
//! through a topology that changes halfway along it may not exist in the new
//! topology, so a route is only useful if its every link survives the
//! interval it is planned for.
//!
//! The algorithm is Dijkstra's, with a binary heap. Costs must be positive,
//! which the topology layer enforces, so the algorithm is exact: the first
//! time a node is removed from the heap its distance is final.
//!
//! Source: Dijkstra, E. W., *A note on two problems in connexion with
//! graphs*, Numerische Mathematik 1, 1959, and the binary heap formulation
//! as given in Cormen et al., *Introduction to Algorithms*, 3rd edition,
//! MIT Press, 2009, chapter 24.
//!
//! ```
//! use apsis_routing::shortest_path;
//! use apsis_topology::Topology;
//!
//! // A three node line: 0 - 1 - 2, with the direct link 0 - 2 absent.
//! let mut topology = Topology::new(3);
//! topology.add_edge(0, 1, 1.0).expect("the nodes exist");
//! topology.add_edge(1, 2, 1.0).expect("the nodes exist");
//!
//! let route = shortest_path(&topology, 0, 2).expect("a route exists");
//! assert_eq!(route.nodes, vec![0, 1, 2]);
//! assert!((route.cost - 2.0).abs() < 1e-12);
//! ```
//!
//! 时变卫星拓扑上的路由。
//!
//! 卫星网络的拓扑随卫星移动而改变，故一条路由只在一个区间上有效。本 crate 计算
//! 穿过 [`Topology`] 的最小代价路由，并把该路由连同其保持有效的时间一起报告。
//!
//! 提供两种路由形式：
//!
//! - [`shortest_path`] 找出在某一瞬时生效的拓扑中的最小代价路由。
//! - [`route_over_series`] 找出对整个区间都有效的路由，方法是取该区间内各拓扑的
//!   交集。
//!
//! 第二种形式是存储转发网络所需者。穿过一条中途改变的拓扑的路由，在新拓扑中可能
//! 不存在，故一条路由只有在其每条链路都存活于它所规划的那个区间时才有用。
//!
//! 该算法是 Dijkstra 的，配以二叉堆。代价必须为正，这由拓扑层强制，故该算法精确：
//! 某节点首次从堆中取出时其距离即为最终。
//!
//! 出处：Dijkstra, E. W.，*A note on two problems in connexion with graphs*，
//! Numerische Mathematik 1，1959，以及二叉堆的表述，见 Cormen 等，
//! *Introduction to Algorithms*，第 3 版，MIT Press，2009，第 24 章。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod dijkstra;
mod error;
mod route;

pub use dijkstra::{PathTree, route_over_series, shortest_path, shortest_paths_from};
pub use error::{RoutingError, RoutingErrorKind};
pub use route::{Route, RouteLeg, RouteStep};

pub use apsis_topology::{Topology, TopologySample, TopologySeries};
