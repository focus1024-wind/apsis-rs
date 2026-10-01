//! Routing over time-varying satellite topologies.
//!
//! 时变卫星拓扑上的路由。
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

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod dijkstra;
mod error;
mod route;

pub use dijkstra::{PathTree, route_over_series, shortest_path, shortest_paths_from};
pub use error::{RoutingError, RoutingErrorKind};
pub use route::{Route, RouteLeg, RouteStep};

pub use apsis_topology::{Topology, TopologySample, TopologySeries};
