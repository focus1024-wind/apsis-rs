//! The time series of topologies.

use crate::error::{TopologyError, TopologyErrorKind};
use crate::graph::Topology;

/// One topology and the time from which it applies.
#[derive(Debug, Clone, PartialEq)]
pub struct TopologySample {
    /// The time from which this topology applies, in seconds from the
    /// caller's origin.
    pub seconds: f64,
    /// The topology in force from that time.
    pub topology: Topology,
}

impl TopologySample {
    /// Creates a sample.
    ///
    /// 创建一个样本。
    pub const fn new(seconds: f64, topology: Topology) -> Self {
        Self { seconds, topology }
    }
}

/// A sequence of topologies over time.
///
/// The sequence is a step function. The entry at index `i` applies from its
/// own time until the time of the entry at index `i + 1`, and the last entry
/// applies for all later times. Querying a time before the first entry
/// returns the first topology, so the series behaves as if it extends
/// backwards as well as forwards.
///
/// The times must be strictly increasing. Two entries at the same time would
/// make the topology at that instant ambiguous.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TopologySeries {
    samples: Vec<TopologySample>,
}

impl TopologySeries {
    /// Creates a series from a list of samples.
    ///
    /// Returns an error when the list is empty, when a time is not finite,
    /// or when the times are not strictly increasing.
    pub fn new(samples: Vec<TopologySample>) -> Result<Self, TopologyError> {
        if samples.is_empty() {
            return Err(TopologyError::new(TopologyErrorKind::EmptySeries));
        }
        let mut previous = f64::NEG_INFINITY;
        for (index, sample) in samples.iter().enumerate() {
            if !sample.seconds.is_finite() {
                return Err(TopologyError::new(TopologyErrorKind::InvalidTime)
                    .with_detail(format!("entry {index} has time {}", sample.seconds)));
            }
            if sample.seconds <= previous {
                return Err(
                    TopologyError::new(TopologyErrorKind::UnorderedTimes).with_detail(format!(
                        "entry {index} at {} does not follow {previous}",
                        sample.seconds
                    )),
                );
            }
            previous = sample.seconds;
        }
        Ok(Self { samples })
    }

    /// Creates a series from times and topologies as separate lists.
    ///
    /// The two lists must have the same length.
    ///
    /// Returns an error when the lists differ in length or when the series
    /// itself is not valid.
    pub fn from_parts(times: Vec<f64>, topologies: Vec<Topology>) -> Result<Self, TopologyError> {
        if times.len() != topologies.len() {
            return Err(
                TopologyError::new(TopologyErrorKind::EmptySeries).with_detail(format!(
                    "{} times against {} topologies",
                    times.len(),
                    topologies.len()
                )),
            );
        }
        let samples = times
            .into_iter()
            .zip(topologies)
            .map(|(seconds, topology)| TopologySample::new(seconds, topology))
            .collect();
        Self::new(samples)
    }

    /// Returns the samples.
    pub fn samples(&self) -> &[TopologySample] {
        &self.samples
    }

    /// Returns the number of samples.
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Returns `true` when there are no samples.
    ///
    /// A series built through either constructor is never empty, so this
    /// always returns `false`; it exists because a collection type should
    /// offer it alongside [`TopologySeries::len`].
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Returns the first time in the series.
    pub fn start_seconds(&self) -> f64 {
        self.samples.first().map_or(0.0, |sample| sample.seconds)
    }

    /// Returns the last time in the series.
    pub fn end_seconds(&self) -> f64 {
        self.samples.last().map_or(0.0, |sample| sample.seconds)
    }

    /// Returns the index of the sample that applies at a time.
    ///
    /// The index is the last entry whose time is at or before the query, so
    /// a time before the series returns zero and a time after it returns the
    /// last index.
    ///
    /// Returns `None` when the time is not finite or the series is empty.
    pub fn index_at(&self, seconds: f64) -> Option<usize> {
        if !seconds.is_finite() || self.samples.is_empty() {
            return None;
        }
        // The samples are ordered, so a binary search gives the insertion
        // point of the query, and the entry before it is the one in force.
        match self
            .samples
            .binary_search_by(|sample| sample.seconds.partial_cmp(&seconds).expect("finite times"))
        {
            Ok(exact) => Some(exact),
            Err(0) => Some(0),
            Err(insert) => Some(insert - 1),
        }
    }

    /// Returns the topology in force at a time.
    ///
    /// Returns `None` when the time is not finite or the series is empty.
    pub fn topology_at(&self, seconds: f64) -> Option<&Topology> {
        Some(&self.samples.get(self.index_at(seconds)?)?.topology)
    }

    /// Returns the interval over which the sample at an index applies.
    ///
    /// The upper bound is the time of the next sample, or positive infinity
    /// for the last entry.
    ///
    /// Returns `None` when the index is out of range.
    pub fn interval_of(&self, index: usize) -> Option<(f64, f64)> {
        let sample = self.samples.get(index)?;
        let end = self
            .samples
            .get(index + 1)
            .map_or(f64::INFINITY, |next| next.seconds);
        Some((sample.seconds, end))
    }

    /// Returns the number of times the topology changes.
    ///
    /// This is the number of samples less one, since each change introduces
    /// a new entry. It measures how dynamic the network is: a series with
    /// many changes needs routing that accounts for them.
    pub fn change_count(&self) -> usize {
        self.samples.len().saturating_sub(1)
    }

    /// Returns the mean interval between changes, in seconds.
    ///
    /// The last entry has unbounded duration, so it is excluded from the
    /// mean and the result covers the changes that have a measured interval.
    ///
    /// Returns `None` when there is only one sample.
    pub fn mean_interval_seconds(&self) -> Option<f64> {
        if self.samples.len() < 2 {
            return None;
        }
        let span = self.end_seconds() - self.start_seconds();
        Some(span / self.change_count() as f64)
    }

    /// Returns the number of edge changes across the series.
    ///
    /// Each topology is compared with the one before it, counting the edges
    /// that appear and the edges that disappear. A topology that differs by
    /// one link contributes two, since one link changed.
    ///
    /// Returns zero for a series of one sample.
    pub fn edge_changes(&self) -> usize {
        let mut changes = 0;
        for pair in self.samples.windows(2) {
            let before = &pair[0].topology;
            let after = &pair[1].topology;
            // Edges present before but not after.
            changes += before
                .edges()
                .iter()
                .filter(|edge| !after.has_edge(edge.from, edge.to))
                .count();
            // Edges present after but not before.
            changes += after
                .edges()
                .iter()
                .filter(|edge| !before.has_edge(edge.from, edge.to))
                .count();
        }
        changes
    }

    /// Returns the number of nodes in the series.
    ///
    /// Every entry is expected to describe the same set of nodes, so the
    /// value is taken from the first entry. A caller that changes the node
    /// count between entries is comparing two different networks and should
    /// build two series instead.
    pub fn node_count(&self) -> usize {
        self.samples
            .first()
            .map_or(0, |sample| sample.topology.node_count())
    }
}
