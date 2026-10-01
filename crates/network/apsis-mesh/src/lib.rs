//! Mesh connectivity abstractions.
//!
//! A satellite mesh is a set of terminals that reach each other through
//! links, some of which are satellite links and some of which are ground
//! links. This crate provides the abstractions that a mesh application needs
//! on top of a topology: which terminals can reach which others, how many
//! independent ways there are to do so, and which links are critical.
//!
//! The three questions this crate answers:
//!
//! - **Reachability**: the set of terminals reachable from a given one.
//! - **Redundancy**: how many independent paths join two terminals, which is
//!   the number of links that can fail independently. This is the
//!   edge-connectivity of the graph, by Menger's theorem.
//! - **Criticality**: the links whose removal would separate the mesh, which
//!   are the ones a network operator must protect.
//!
//! The redundancy computation is exact for the small graphs a satellite mesh
//! presents. It uses the edge version of Menger's theorem, computing the
//! connectivity from a sequence of maximum flow problems, each of unit
//! capacity.
//!
//! Source: Menger's theorem, and the reduction of edge connectivity to
//! maximum flow, as given in Cormen et al., *Introduction to Algorithms*,
//! 3rd edition, MIT Press, 2009, chapter 26.
//!
//! ```
//! use apsis_mesh::Mesh;
//! use apsis_topology::Topology;
//!
//! // Three terminals in a ring, so each pair has two independent paths.
//! let mut topology = Topology::new(3);
//! topology.add_bidirectional_edge(0, 1, 1.0).expect("the nodes exist");
//! topology.add_bidirectional_edge(1, 2, 1.0).expect("the nodes exist");
//! topology.add_bidirectional_edge(2, 0, 1.0).expect("the nodes exist");
//!
//! let mesh = Mesh::new(topology);
//! assert!(mesh.is_connected());
//! assert_eq!(mesh.edge_connectivity(0, 1), Some(2));
//! ```
//!
//! 网格连通性抽象。
//!
//! 卫星网格是一组经链路互相到达的终端，其中一些是卫星链路、一些是地面链路。本
//! crate 提供网格应用在拓扑之上所需的抽象：哪些终端能到达哪些其他终端、有多少条
//! 独立路径可以做到，以及哪些链路是关键。
//!
//! 本 crate 回答的三个问题：
//!
//! - **可达性**：自某给定终端可达的终端集合。
//! - **冗余度**：连接两个终端的独立路径条数，即可独立失效的链路数。依 Menger 定理，
//!   它是该图的边连通度。
//! - **关键性**：移除后会切开网格的那些链路，即网络运营者必须保护的链路。
//!
//! 冗余度计算对卫星网格所呈现的小图是精确的。它使用 Menger 定理的边版本，从一
//! 系列单位容量的最大流问题计算连通度。
//!
//! 出处：Menger 定理，以及把边连通度归约为最大流，见 Cormen 等，
//! *Introduction to Algorithms*，第 3 版，MIT Press，2009，第 26 章。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod connectivity;
mod mesh;

pub use connectivity::{edge_connectivity, reachable_from, strongly_connected_components};
pub use mesh::{CriticalLink, Mesh, MeshSummary};

pub use apsis_topology::{Topology, TopologySeries};
