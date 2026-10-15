//! Time-varying network topology.
//!
//! A satellite network is not a single graph. The set of usable links
//! changes as the satellites move, so the network is a sequence of graphs
//! indexed by time, and this module provides the data structures for that
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
//! use apsis::apsis_topology::{Topology, TopologySample, TopologySeries};
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
//!
//! 时变网络拓扑。
//!
//! 卫星网络不是单一的图。可用链路的集合随卫星移动而改变，故该网络是按时间索引的
//! 一个图序列，而本模块提供该序列的数据结构。
//!
//! 模型有意简单。一个 [`Topology`] 是节点集合固定、边集合可有可无的图，节点是卫星
//! 或地面站。一个 [`TopologySeries`] 是递增时刻上一串拓扑，连同各自适用的区间。
//!
//! 该序列是阶跃函数：一个拓扑从其自身时间起保持到下一项的时间。这是时变图的标准
//! 离散时间模型，也是对该序列路由所期望的。
//!
//! 边除存在与否外还携带代价，使路由算法有可最小化之物。该代价是一个普通数，故
//! 调用方可以放一个时延、一个跳数，或任何别的东西，只要路由算法与代价在含义上
//! 一致。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod error;
mod graph;
mod series;

pub use error::{TopologyError, TopologyErrorKind};
pub use graph::{Edge, Topology};
pub use series::{TopologySample, TopologySeries};

pub use crate::apsis_math::Vec3;
