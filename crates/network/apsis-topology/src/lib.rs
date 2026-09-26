//! Time-varying network topology.
//!
//! A satellite network is not a single graph. The set of usable links
//! changes as the satellites move, so the network is a sequence of graphs
//! indexed by time, and this crate provides the data structures for that
//! sequence.
//!
//! The model is deliberately simple. A [`Topology`] is a graph with a fixed
//! set of nodes, which are satellites or ground stations, and a set of edges
//! that may be present or absent. A [`TopologySeries`] is a sequence of
//! topologies at increasing times, together with the interval each one
//! applies to.
//!
//! The series is a step function: a topology holds from its own time until
//! the time of the next entry. That is the standard discrete time model for
//! a time-varying graph, and it is what routing over the series expects.
//!
//! Edges carry a cost as well as their presence, so that a routing algorithm
//! has something to minimise. The cost is a plain number, so a caller may
//! put a delay, a hop count, or anything else in it, as long as the routing
//! algorithm and the cost agree on the meaning.
//!
//! ```
//! use apsis_topology::{Topology, TopologySample, TopologySeries};
//!
//! // Two satellites linked to each other.
//! let mut topology = Topology::new(2);
//! topology.add_edge(0, 1, 1.0).expect("the nodes exist");
//! assert!(topology.has_edge(0, 1));
//!
//! let series = TopologySeries::new(vec![TopologySample::new(0.0, topology)])
//!     .expect("the series is not empty");
//! assert_eq!(series.len(), 1);
//! assert!(series.topology_at(0.0).expect("a topology exists").has_edge(0, 1));
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod error;
mod graph;
mod series;

pub use error::{TopologyError, TopologyErrorKind};
pub use graph::{Edge, Topology};
pub use series::{TopologySample, TopologySeries};

pub use apsis_math::Vec3;
